use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;

use crate::adapters::audio_context::AudioContextManager;
use crate::adapters::audio_oscillator::OscillatorNotePlayer;
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::bridge::{DataStoreObserver, ProfileObserver, TimelineObserver, TrendObserver};
use domain::ports::{ComparisonObserver, NotePlayer};
use domain::types::{AmplitudeDB, MIDIVelocity};
use domain::{
    ComparisonSession, ComparisonSessionState, DirectedInterval, Direction, Interval,
    PerceptualProfile, ThresholdTimeline, TrendAnalyzer, FEEDBACK_DURATION_SECS,
};
use leptos::reactive::owner::LocalStorage;

const POLL_INTERVAL_MS: u32 = 50;

#[component]
pub fn ComparisonView() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile not provided");
    let audio_ctx: SendWrapper<Rc<RefCell<AudioContextManager>>> =
        use_context().expect("AudioContextManager not provided");
    let trend_analyzer: SendWrapper<Rc<RefCell<TrendAnalyzer>>> =
        use_context().expect("TrendAnalyzer not provided");
    let timeline: SendWrapper<Rc<RefCell<ThresholdTimeline>>> =
        use_context().expect("ThresholdTimeline not provided");
    let db_store: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage> =
        use_context().expect("db_store not provided");

    // Eagerly create AudioContext in synchronous render path.
    // This ensures creation happens within the user gesture call stack (click on Start Page),
    // satisfying Safari/iOS autoplay policies that reject async AudioContext creation.
    if let Err(e) = audio_ctx.borrow_mut().get_or_create() {
        log::error!("Failed to create AudioContext: {e}");
    }

    let settings = LocalStorageSettings;
    let note_player = Rc::new(RefCell::new(OscillatorNotePlayer::new(Rc::clone(&audio_ctx))));
    let storage_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Build observers list — DataStoreObserver holds the signal and checks
    // store availability on each call, so it works even if IndexedDB
    // opens after ComparisonView mounts.
    let observers: Vec<Box<dyn ComparisonObserver>> = vec![
        Box::new(ProfileObserver(Rc::clone(&profile))),
        Box::new(TrendObserver(Rc::clone(&trend_analyzer))),
        Box::new(TimelineObserver(Rc::clone(&timeline))),
        Box::new(DataStoreObserver::new(db_store, storage_error)),
    ];

    let session = Rc::new(RefCell::new(ComparisonSession::new(
        Rc::clone(&profile),
        observers,
        vec![],
    )));

    // Auto-dismiss storage error after 5 seconds
    Effect::new(move || {
        if storage_error.get().is_some() {
            let signal = storage_error;
            gloo_timers::callback::Timeout::new(5000, move || {
                signal.set(None);
            })
            .forget();
        }
    });

    // Signals bridging domain state to UI
    let show_feedback = RwSignal::new(false);
    let is_last_correct = RwSignal::new(false);
    let buttons_enabled = RwSignal::new(false);
    let sr_announcement = RwSignal::new(String::new());

    // Sync all signals from session state
    let sync_signals = {
        let session = Rc::clone(&session);
        move || {
            let s = session.borrow();
            show_feedback.set(s.show_feedback());
            is_last_correct.set(s.is_last_answer_correct());
            let state = s.state();
            buttons_enabled.set(
                state == ComparisonSessionState::PlayingNote2
                    || state == ComparisonSessionState::AwaitingAnswer,
            );
            if s.show_feedback() {
                sr_announcement.set(if s.is_last_answer_correct() {
                    "Correct".into()
                } else {
                    "Incorrect".into()
                });
            }
        }
    };

    // Cancellation flag shared between loop and event handlers
    let cancelled = Rc::new(Cell::new(false));

    // Answer handler — used by both buttons and keyboard
    // No feedback timer here — the main loop controls feedback timing.
    let on_answer = {
        let session = Rc::clone(&session);
        let sync = sync_signals.clone();
        let cancelled = Rc::clone(&cancelled);
        Rc::new(move |is_higher: bool| {
            if cancelled.get() {
                return;
            }
            let state = session.borrow().state();
            if state != ComparisonSessionState::PlayingNote2
                && state != ComparisonSessionState::AwaitingAnswer
            {
                return;
            }

            let timestamp = js_sys::Date::new_0()
                .to_iso_string()
                .as_string()
                .unwrap_or_default();
            session.borrow_mut().handle_answer(is_higher, timestamp);
            sync();
        })
    };

    // Button click handlers
    let on_answer_higher = {
        let on_answer = Rc::clone(&on_answer);
        move |_| on_answer(true)
    };
    let on_answer_lower = {
        let on_answer = Rc::clone(&on_answer);
        move |_| on_answer(false)
    };

    // Keyboard event handler
    let navigate = use_navigate();
    let keydown_handler = {
        let on_answer = Rc::clone(&on_answer);
        let cancelled = Rc::clone(&cancelled);
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let sync = sync_signals.clone();
        let navigate = navigate.clone();
        Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
            match ev.key().as_str() {
                "ArrowUp" | "h" | "H" => {
                    ev.prevent_default();
                    on_answer(true);
                }
                "ArrowDown" | "l" | "L" => {
                    ev.prevent_default();
                    on_answer(false);
                }
                "Escape" => {
                    ev.prevent_default();
                    cancelled.set(true);
                    session.borrow_mut().stop();
                    note_player.borrow().stop_all();
                    sync();
                    navigate("/", Default::default());
                }
                _ => {}
            }
        })
    };

    // Register keydown listener on document and save JS function ref for cleanup
    let document = web_sys::window().unwrap().document().unwrap();
    let keydown_fn: JsValue = keydown_handler.as_ref().clone();
    document
        .add_event_listener_with_callback("keydown", keydown_fn.unchecked_ref())
        .unwrap();

    // Keep closure alive for component lifetime
    let _keydown_closure = StoredValue::new_local(keydown_handler);

    // Navigation away handler — stops training before nav
    let on_nav_away = {
        let cancelled = Rc::clone(&cancelled);
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let sync = sync_signals.clone();
        move || {
            cancelled.set(true);
            session.borrow_mut().stop();
            note_player.borrow().stop_all();
            sync();
        }
    };
    let on_nav_settings = {
        let on_nav_away = on_nav_away.clone();
        let navigate = navigate.clone();
        move |ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
            on_nav_away();
            navigate("/settings", Default::default());
        }
    };
    let on_nav_profile = {
        let on_nav_away = on_nav_away.clone();
        let navigate = navigate.clone();
        move |ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
            on_nav_away();
            navigate("/profile", Default::default());
        }
    };

    // Cleanup on component unmount — stop training AND remove keydown listener
    {
        let cleanup_state = SendWrapper::new((
            Rc::clone(&cancelled),
            Rc::clone(&session),
            Rc::clone(&note_player),
            keydown_fn,
        ));
        on_cleanup(move || {
            let (cancelled, session, note_player, keydown_fn) = &*cleanup_state;
            cancelled.set(true);
            session.borrow_mut().stop();
            note_player.borrow().stop_all();
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                let _ = document.remove_event_listener_with_callback(
                    "keydown",
                    keydown_fn.unchecked_ref(),
                );
            }
        });
    }

    // Start the async training loop
    {
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let cancelled = Rc::clone(&cancelled);
        let sync = sync_signals.clone();
        spawn_local(async move {
            // Start session with unison mode intervals
            let mut intervals = HashSet::new();
            intervals.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
            session.borrow_mut().start(intervals, &settings);
            sync();

            let feedback_ms = (FEEDBACK_DURATION_SECS * 1000.0) as u32;

            'training: loop {
                if cancelled.get() {
                    break;
                }

                let data = match session.borrow().current_playback_data() {
                    Some(data) => data,
                    None => break,
                };

                let duration_ms = (data.duration.raw_value() * 1000.0) as u32;

                // === PlayingNote1 phase (buttons disabled) ===
                note_player.borrow().stop_all(); // Stop any lingering audio
                if let Err(e) = note_player.borrow().play_for_duration(
                    data.reference_frequency,
                    data.duration,
                    MIDIVelocity::new(63),
                    AmplitudeDB::new(0.0),
                ) {
                    log::error!("Note1 playback failed: {e}");
                }
                // Wait for note1 duration with responsive cancellation
                let mut elapsed = 0u32;
                while elapsed < duration_ms {
                    if cancelled.get() {
                        break 'training;
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                    elapsed += POLL_INTERVAL_MS;
                }
                if cancelled.get() {
                    break;
                }

                // Transition: PlayingNote1 → PlayingNote2
                session.borrow_mut().on_note1_finished();
                sync();

                // === PlayingNote2 phase (buttons enabled — early answer possible) ===
                if let Err(e) = note_player.borrow().play_for_duration(
                    data.target_frequency,
                    data.duration,
                    MIDIVelocity::new(63),
                    data.target_amplitude_db,
                ) {
                    log::error!("Note2 playback failed: {e}");
                }
                // Wait for note2 duration OR early answer
                elapsed = 0;
                while elapsed < duration_ms {
                    if cancelled.get() {
                        break 'training;
                    }
                    // Detect early answer: answer handler transitions to ShowingFeedback
                    if session.borrow().state() == ComparisonSessionState::ShowingFeedback {
                        break;
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                    elapsed += POLL_INTERVAL_MS;
                }
                if cancelled.get() {
                    break;
                }

                // On early answer, stop note2 audio immediately
                if session.borrow().state() == ComparisonSessionState::ShowingFeedback {
                    note_player.borrow().stop_all();
                }

                // Transition to AwaitingAnswer if no early answer was given
                if session.borrow().state() == ComparisonSessionState::PlayingNote2 {
                    session.borrow_mut().on_note2_finished();
                    sync();
                }

                // === Wait for answer if not already given ===
                while session.borrow().state() != ComparisonSessionState::ShowingFeedback {
                    if cancelled.get() {
                        break 'training;
                    }
                    if session.borrow().state() == ComparisonSessionState::Idle {
                        break 'training;
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                }

                // === ShowingFeedback phase — main loop controls timing ===
                sync(); // Ensure feedback indicator is visible
                TimeoutFuture::new(feedback_ms).await;
                if cancelled.get() {
                    break;
                }

                // End feedback, generate next comparison
                if session.borrow().state() == ComparisonSessionState::ShowingFeedback {
                    session.borrow_mut().on_feedback_finished();
                    sync();
                }
            }

            // Final cleanup
            session.borrow_mut().stop();
            note_player.borrow().stop_all();
            sync();
        });
    }

    view! {
        <div class="py-12">
            <h1 class="text-2xl font-bold dark:text-white">"Comparison Training"</h1>

            // Feedback indicator
            <div class="flex items-center justify-center h-16" aria-hidden="true">
                {move || {
                    if show_feedback.get() {
                        if is_last_correct.get() {
                            view! { <span class="text-4xl text-green-600 dark:text-green-400">{"\u{1F44D}"}</span> }.into_any()
                        } else {
                            view! { <span class="text-4xl text-red-600 dark:text-red-400">{"\u{1F44E}"}</span> }.into_any()
                        }
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>

            // Screen reader live region
            <div aria-live="polite" class="sr-only">
                {move || sr_announcement.get()}
            </div>

            // Higher / Lower buttons
            <div class="flex gap-4 justify-center mt-8">
                <button
                    on:click=on_answer_higher
                    disabled=move || !buttons_enabled.get()
                    class=move || if buttons_enabled.get() {
                        "min-h-11 min-w-[120px] rounded-lg bg-indigo-600 px-6 py-4 text-lg font-semibold text-white shadow-md hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-400"
                    } else {
                        "min-h-11 min-w-[120px] rounded-lg bg-gray-300 px-6 py-4 text-lg font-semibold text-gray-500 cursor-not-allowed dark:bg-gray-700 dark:text-gray-500"
                    }
                    aria-label="Higher"
                >
                    "Higher"
                </button>
                <button
                    on:click=on_answer_lower
                    disabled=move || !buttons_enabled.get()
                    class=move || if buttons_enabled.get() {
                        "min-h-11 min-w-[120px] rounded-lg bg-indigo-600 px-6 py-4 text-lg font-semibold text-white shadow-md hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-400"
                    } else {
                        "min-h-11 min-w-[120px] rounded-lg bg-gray-300 px-6 py-4 text-lg font-semibold text-gray-500 cursor-not-allowed dark:bg-gray-700 dark:text-gray-500"
                    }
                    aria-label="Lower"
                >
                    "Lower"
                </button>
            </div>

            // Navigation links
            <nav class="mt-8 flex justify-center gap-6">
                <a
                    href="/settings"
                    on:click=on_nav_settings
                    class="min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                >
                    "Settings"
                </a>
                <a
                    href="/profile"
                    on:click=on_nav_profile
                    class="min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                >
                    "Profile"
                </a>
            </nav>

            // Storage error notification — non-blocking, auto-dismissing
            {move || {
                if let Some(msg) = storage_error.get() {
                    view! {
                        <div
                            class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-amber-100 border border-amber-400 text-amber-800 px-4 py-2 rounded-lg shadow-md text-sm dark:bg-amber-900 dark:border-amber-700 dark:text-amber-200"
                            role="alert"
                        >
                            {msg}
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </div>
    }
}
