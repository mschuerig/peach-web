use std::cell::{Cell, RefCell};
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
use crate::adapters::audio_soundfont::WorkletBridge;
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::note_player::create_note_player;
use crate::bridge::{DataStoreObserver, ProfileObserver, ProgressTimelineObserver, TimelineObserver, TrendObserver};
use crate::components::TrainingStats;
use crate::interval_codes::{interval_label, parse_intervals_param};
use domain::ports::{PitchComparisonObserver, NotePlayer};
use domain::types::{AmplitudeDB, MIDIVelocity};
use domain::{
    PitchComparisonSession, PitchComparisonSessionState, Interval, PerceptualProfile,
    ProgressTimeline, ThresholdTimeline, TrainingMode, Trend, TrendAnalyzer, FEEDBACK_DURATION_SECS,
};
use leptos::reactive::owner::LocalStorage;
use leptos_router::hooks::use_query_map;

const POLL_INTERVAL_MS: u32 = 50;

#[component]
pub fn PitchComparisonView() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile not provided");
    let audio_ctx: SendWrapper<Rc<RefCell<AudioContextManager>>> =
        use_context().expect("AudioContextManager not provided");
    let trend_analyzer: SendWrapper<Rc<RefCell<TrendAnalyzer>>> =
        use_context().expect("TrendAnalyzer not provided");
    let timeline: SendWrapper<Rc<RefCell<ThresholdTimeline>>> =
        use_context().expect("ThresholdTimeline not provided");
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline not provided");
    let db_store: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage> =
        use_context().expect("db_store not provided");
    let worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>, LocalStorage> =
        use_context().expect("worklet_bridge not provided");

    // Eagerly create AudioContext in synchronous render path.
    // This ensures creation happens within the user gesture call stack (click on Start Page),
    // satisfying Safari/iOS autoplay policies that reject async AudioContext creation.
    if let Err(e) = audio_ctx.borrow_mut().get_or_create() {
        log::error!("Failed to create AudioContext: {e}");
    }

    // Parse interval mode from query params
    let query = use_query_map();
    let intervals_from_query = {
        let param = query.read_untracked().get("intervals").unwrap_or_default();
        parse_intervals_param(&param)
    };
    let is_interval_mode = intervals_from_query
        .iter()
        .any(|di| di.interval != Interval::Prime);
    let interval_label_text: RwSignal<String> = RwSignal::new(String::new());

    let settings = LocalStorageSettings;
    let sound_source = LocalStorageSettings::get_string("peach.sound_source")
        .unwrap_or_else(|| "oscillator:sine".to_string());
    let note_player = Rc::new(RefCell::new(create_note_player(
        &sound_source,
        Rc::clone(&audio_ctx),
        worklet_bridge.get_untracked(),
    )));
    let storage_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Build observers list — DataStoreObserver holds the signal and checks
    // store availability on each call, so it works even if IndexedDB
    // opens after PitchComparisonView mounts.
    let observers: Vec<Box<dyn PitchComparisonObserver>> = vec![
        Box::new(ProfileObserver::new(Rc::clone(&profile))),
        Box::new(TrendObserver::new(Rc::clone(&trend_analyzer))),
        Box::new(TimelineObserver::new(Rc::clone(&timeline))),
        Box::new(ProgressTimelineObserver::new(Rc::clone(&progress_timeline))),
        Box::new(DataStoreObserver::new(db_store, storage_error)),
    ];

    let session = Rc::new(RefCell::new(PitchComparisonSession::new(
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

    // Determine TrainingMode from intervals
    let training_mode = if is_interval_mode {
        TrainingMode::IntervalPitchComparison
    } else {
        TrainingMode::UnisonPitchComparison
    };

    // Training stats signals
    let latest_cent_difference: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_session_best: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_trend: RwSignal<Option<Trend>> = RwSignal::new(None);

    // Signals bridging domain state to UI
    let show_feedback = RwSignal::new(false);
    let is_last_correct = RwSignal::new(false);
    let buttons_enabled = RwSignal::new(false);
    let sr_announcement = RwSignal::new(String::new());

    // Sync all UI signals from session state — extracted for readability
    fn sync_session_to_signals(
        session: &RefCell<PitchComparisonSession>,
        show_feedback: RwSignal<bool>,
        is_last_correct: RwSignal<bool>,
        buttons_enabled: RwSignal<bool>,
        sr_announcement: RwSignal<String>,
        interval_label_text: RwSignal<String>,
        is_interval_mode: bool,
    ) {
        let s = session.borrow();
        show_feedback.set(s.show_feedback());
        is_last_correct.set(s.is_last_answer_correct());
        let state = s.state();
        buttons_enabled.set(
            state == PitchComparisonSessionState::PlayingTargetNote
                || state == PitchComparisonSessionState::AwaitingAnswer,
        );
        if s.show_feedback() {
            sr_announcement.set(if s.is_last_answer_correct() {
                "Correct".into()
            } else {
                "Incorrect".into()
            });
        } else {
            sr_announcement.set(String::new());
        }
        if is_interval_mode
            && let Some(di) = s.current_interval()
        {
            if di.interval != Interval::Prime {
                let label = interval_label(di.interval, di.direction);
                sr_announcement.set(label.clone());
                interval_label_text.set(label);
            } else {
                interval_label_text.set(String::new());
            }
        }
    }

    let sync_signals = {
        let session = Rc::clone(&session);
        move || {
            sync_session_to_signals(
                &session,
                show_feedback,
                is_last_correct,
                buttons_enabled,
                sr_announcement,
                interval_label_text,
                is_interval_mode,
            );
        }
    };

    // Cancellation flag shared between loop and event handlers
    let cancelled = Rc::new(Cell::new(false));

    // Answer handler — used by both buttons and keyboard
    // No feedback timer here — the main loop controls feedback timing.
    let on_answer = {
        let session = Rc::clone(&session);
        let progress_timeline = Rc::clone(&progress_timeline);
        let sync = sync_signals.clone();
        let cancelled = Rc::clone(&cancelled);
        Rc::new(move |is_higher: bool| {
            let answer_str = if is_higher { "higher" } else { "lower" };
            if cancelled.get() {
                log::info!("User pressed {answer_str} — ignored (cancelled)");
                return;
            }
            let state = session.borrow().state();
            if state != PitchComparisonSessionState::PlayingTargetNote
                && state != PitchComparisonSessionState::AwaitingAnswer
            {
                log::info!("User pressed {answer_str} — ignored (state: {state:?})");
                return;
            }

            log::info!("User pressed {answer_str}");
            let timestamp = js_sys::Date::new_0()
                .to_iso_string()
                .as_string()
                .unwrap_or_default();
            session.borrow_mut().handle_answer(is_higher, timestamp);

            // Update training stats signals
            {
                let s = session.borrow();
                latest_cent_difference.set(s.last_cent_difference());
                stats_session_best.set(s.session_best_cent_difference());
            }
            stats_trend.set(progress_timeline.borrow().trend(training_mode));

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
            sr_announcement.set("Training stopped".into());
        }
    };
    let on_nav_start = {
        let on_nav_away = on_nav_away.clone();
        let navigate = navigate.clone();
        move |ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
            on_nav_away();
            navigate("/", Default::default());
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
    let on_nav_info = {
        let on_nav_away = on_nav_away.clone();
        let navigate = navigate.clone();
        move |ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
            on_nav_away();
            navigate("/info", Default::default());
        }
    };

    // Shared interruption closure — stops training and navigates to start page
    let interrupt_and_navigate = {
        let cancelled = Rc::clone(&cancelled);
        let navigate = navigate.clone();
        Rc::new(move || {
            if cancelled.get() {
                return;
            }
            on_nav_away();
            navigate("/", Default::default());
        })
    };

    let keydown_handler = {
        let on_answer = Rc::clone(&on_answer);
        let interrupt = Rc::clone(&interrupt_and_navigate);
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
                    (*interrupt)();
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

    // Visibility change handler — interrupts training when tab is hidden
    let visibility_handler = Closure::<dyn FnMut(web_sys::Event)>::new({
        let interrupt = Rc::clone(&interrupt_and_navigate);
        let document = document.clone();
        move |_event: web_sys::Event| {
            if document.hidden() {
                (*interrupt)();
            }
        }
    });

    let visibility_fn: JsValue = visibility_handler.as_ref().clone();
    document
        .add_event_listener_with_callback("visibilitychange", visibility_fn.unchecked_ref())
        .unwrap();
    let _visibility_closure = StoredValue::new_local(visibility_handler);

    // AudioContext state change handler — interrupts on context suspension
    let audiocontext_handler = Closure::<dyn FnMut(web_sys::Event)>::new({
        let interrupt = Rc::clone(&interrupt_and_navigate);
        move |event: web_sys::Event| {
            if let Some(target) = event.target()
                && let Some(ctx) = target.dyn_ref::<web_sys::BaseAudioContext>()
            {
                let state = ctx.state();
                if state == web_sys::AudioContextState::Suspended
                    || state == web_sys::AudioContextState::Closed
                {
                    (*interrupt)();
                }
            }
        }
    });

    audio_ctx
        .borrow()
        .set_state_change_handler(audiocontext_handler.as_ref().unchecked_ref());
    let _audiocontext_closure = StoredValue::new_local(audiocontext_handler);

    // Cleanup on component unmount — stop training AND remove all event listeners
    {
        let cleanup_state = SendWrapper::new((
            Rc::clone(&cancelled),
            Rc::clone(&session),
            Rc::clone(&note_player),
            Rc::clone(&audio_ctx),
            keydown_fn,
            visibility_fn,
        ));
        on_cleanup(move || {
            let (cancelled, session, note_player, audio_ctx, keydown_fn, visibility_fn) =
                &*cleanup_state;
            cancelled.set(true);
            session.borrow_mut().stop();
            note_player.borrow().stop_all();
            audio_ctx.borrow().clear_state_change_handler();
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                let _ = document.remove_event_listener_with_callback(
                    "keydown",
                    keydown_fn.unchecked_ref(),
                );
                let _ = document.remove_event_listener_with_callback(
                    "visibilitychange",
                    visibility_fn.unchecked_ref(),
                );
            }
        });
    }

    // Start the async training loop
    // SAFETY (RefCell borrows): WASM is single-threaded. Event handlers (on_answer,
    // interruption) can only execute at `.await` yield points. All RefCell borrows
    // in this loop are dropped before each `.await`, so overlapping borrows are impossible.
    {
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let cancelled = Rc::clone(&cancelled);
        let sync = sync_signals.clone();
        spawn_local(async move {
            session.borrow_mut().start(intervals_from_query, &settings);
            sync();
            sr_announcement.set("Training started".into());

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

                // === PlayingReferenceNote phase (buttons disabled) ===
                note_player.borrow().stop_all(); // Stop any lingering audio
                if let Err(e) = note_player.borrow().play_for_duration(
                    data.reference_frequency,
                    data.duration,
                    MIDIVelocity::new(63),
                    AmplitudeDB::new(0.0),
                ) {
                    log::error!("Reference note playback failed: {e}");
                }
                // Wait for reference note duration with responsive cancellation
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

                // Transition: PlayingReferenceNote → PlayingTargetNote
                session.borrow_mut().on_reference_note_finished();
                sync();

                // === PlayingTargetNote phase (buttons enabled — early answer possible) ===
                if let Err(e) = note_player.borrow().play_for_duration(
                    data.target_frequency,
                    data.duration,
                    MIDIVelocity::new(63),
                    data.target_amplitude_db,
                ) {
                    log::error!("Target note playback failed: {e}");
                }
                // Wait for target note duration OR early answer
                elapsed = 0;
                while elapsed < duration_ms {
                    if cancelled.get() {
                        break 'training;
                    }
                    // Detect early answer: answer handler transitions to ShowingFeedback
                    if session.borrow().state() == PitchComparisonSessionState::ShowingFeedback {
                        break;
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                    elapsed += POLL_INTERVAL_MS;
                }
                if cancelled.get() {
                    break;
                }

                // On early answer, stop target note audio immediately
                if session.borrow().state() == PitchComparisonSessionState::ShowingFeedback {
                    note_player.borrow().stop_all();
                }

                // Transition to AwaitingAnswer if no early answer was given
                if session.borrow().state() == PitchComparisonSessionState::PlayingTargetNote {
                    session.borrow_mut().on_target_note_finished();
                    sync();
                }

                // === Wait for answer if not already given ===
                while session.borrow().state() != PitchComparisonSessionState::ShowingFeedback {
                    if cancelled.get() {
                        break 'training;
                    }
                    if session.borrow().state() == PitchComparisonSessionState::Idle {
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
                if session.borrow().state() == PitchComparisonSessionState::ShowingFeedback {
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
            <nav aria-label="Page navigation" class="flex gap-6 text-sm mb-6">
                <a
                    href="/"
                    on:click=on_nav_start
                    class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Start"
                </a>
                <a
                    href="/settings"
                    on:click=on_nav_settings
                    class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Settings"
                </a>
                <a
                    href="/profile"
                    on:click=on_nav_profile
                    class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Profile"
                </a>
                <a
                    href="/info"
                    on:click=on_nav_info
                    class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Info"
                </a>
            </nav>
            <h1 class="text-2xl font-bold dark:text-white">
                {if is_interval_mode { "Interval Comparison" } else { "Comparison Training" }}
            </h1>

            // Interval label — only visible in interval mode
            {move || {
                let label = interval_label_text.get();
                if !label.is_empty() {
                    view! { <p class="mt-2 text-lg text-indigo-600 dark:text-indigo-400 font-medium">{label}</p> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Training stats
            <TrainingStats
                latest_value=latest_cent_difference.into()
                session_best=stats_session_best.into()
                trend=stats_trend.into()
            />

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
            <div aria-live="polite" aria-atomic="true" class="sr-only">
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
