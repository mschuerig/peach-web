use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::KeyboardEvent;

use crate::adapters::audio_context::{AudioContextManager, ensure_audio_ready};
use crate::adapters::audio_soundfont::{SF2Preset, WorkletBridge};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::note_player::create_note_player;
use crate::app::{SoundFontLoadStatus, WorkletAssets, connect_worklet};
use crate::bridge::{DataStoreObserver, ProfileObserver, ProgressTimelineObserver, TimelineObserver, TrendObserver};
use crate::components::audio_gate_overlay::AudioGateOverlay;
use crate::components::help_content::HelpModal;
use crate::components::nav_bar::{NavBar, NavIconButton};
use crate::components::TrainingStats;
use crate::help_sections::COMPARISON_HELP;
use crate::interval_codes::{interval_label, parse_intervals_param};
use domain::ports::{PitchComparisonObserver, NotePlayer, UserSettings};
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
    let sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage> =
        use_context().expect("sf2_presets not provided");
    let worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, LocalStorage> =
        use_context().expect("worklet_assets not provided");
    let sf2_load_status: RwSignal<SoundFontLoadStatus> =
        use_context().expect("SoundFontLoadStatus not provided");
    let worklet_connecting: RwSignal<bool> =
        use_context().expect("worklet_connecting not provided");

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
    let sound_source_clone = sound_source.clone();
    let note_player = Rc::new(RefCell::new(create_note_player(
        &sound_source,
        Rc::clone(&audio_ctx),
        worklet_bridge.get_untracked(),
    )));
    let storage_error: RwSignal<Option<String>> = RwSignal::new(None);
    let audio_error: RwSignal<Option<String>> = RwSignal::new(None);

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

    // Auto-dismiss audio error after 5 seconds
    Effect::new(move || {
        if audio_error.get().is_some() {
            let signal = audio_error;
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
    // Help modal state
    let is_help_open = RwSignal::new(false);
    let on_help_open = {
        let cancelled = Rc::clone(&cancelled);
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let sync = sync_signals.clone();
        move |_| {
            cancelled.set(true);
            session.borrow_mut().stop();
            note_player.borrow().stop_all();
            sync();
            is_help_open.set(true);
        }
    };

    let on_help_close = {
        let navigate = navigate.clone();
        let current_route = {
            let location = web_sys::window().unwrap().location();
            let pathname = location.pathname().unwrap_or_default();
            let search = location.search().unwrap_or_default();
            format!("{pathname}{search}")
        };
        Callback::new(move |()| {
            navigate(&current_route, Default::default());
        })
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

    // AudioContext state change handler — attempts resume on Suspended, interrupts on Closed
    let audiocontext_handler = Closure::<dyn FnMut(web_sys::Event)>::new({
        let interrupt = Rc::clone(&interrupt_and_navigate);
        move |event: web_sys::Event| {
            if let Some(target) = event.target()
                && let Some(base_ctx) = target.dyn_ref::<web_sys::BaseAudioContext>()
            {
                let state = base_ctx.state();
                log::debug!(
                    "[DIAG] PitchComparisonView onstatechange fired — new state: {:?}",
                    state
                );
                match state {
                    web_sys::AudioContextState::Closed => {
                        (*interrupt)();
                    }
                    web_sys::AudioContextState::Suspended => {
                        if let Some(audio_ctx) = target.dyn_ref::<web_sys::AudioContext>() {
                            match audio_ctx.resume() {
                                Ok(promise) => {
                                    let interrupt = Rc::clone(&interrupt);
                                    let target = target.clone();
                                    spawn_local(async move {
                                        let _ = JsFuture::from(promise).await;
                                        TimeoutFuture::new(500).await;
                                        if let Some(ctx) =
                                            target.dyn_ref::<web_sys::BaseAudioContext>()
                                            && ctx.state()
                                                != web_sys::AudioContextState::Running
                                        {
                                            (*interrupt)();
                                        }
                                    });
                                }
                                Err(_) => {
                                    (*interrupt)();
                                }
                            }
                        } else {
                            (*interrupt)();
                        }
                    }
                    _ => {}
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

    let audio_needs_gesture: RwSignal<bool> =
        use_context().expect("audio_needs_gesture context");

    // Start the async training loop
    // SAFETY (RefCell borrows): WASM is single-threaded. Event handlers (on_answer,
    // interruption) can only execute at `.await` yield points. All RefCell borrows
    // in this loop are dropped before each `.await`, so overlapping borrows are impossible.
    {
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let cancelled = Rc::clone(&cancelled);
        let audio_ctx_for_loop = Rc::clone(&audio_ctx);
        let sync = sync_signals.clone();
        spawn_local(async move {
            // Ensure AudioContext is created and running (waits for gesture if needed)
            let ctx_rc = match ensure_audio_ready(
                &audio_ctx_for_loop,
                audio_needs_gesture,
                &cancelled,
            ).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    log::error!("AudioContext failed: {e}");
                    audio_error.set(Some("Audio engine failed to start".into()));
                    return;
                }
            };

            // Wait for SF2 assets if user selected SoundFont
            if sound_source_clone.starts_with("sf2:") {
                while matches!(sf2_load_status.get_untracked(), SoundFontLoadStatus::Fetching) {
                    if cancelled.get() { return; }
                    TimeoutFuture::new(100).await;
                }
                if let SoundFontLoadStatus::Failed(ref msg) = sf2_load_status.get_untracked() {
                    log::warn!("SF2 load failed, falling back to oscillator: {msg}");
                    audio_error.set(Some("Selected sound could not be loaded. Using default sound.".into()));
                }
            }

            // Phase 2: connect worklet if assets are available but bridge isn't.
            // Guard with worklet_connecting flag to prevent parallel connect_worklet calls.
            if worklet_bridge.get_untracked().is_none()
                && !worklet_connecting.get_untracked()
                && let Some(assets) = worklet_assets.get_untracked()
            {
                worklet_connecting.set(true);
                match connect_worklet(&ctx_rc, &assets).await {
                    Ok((bridge, presets)) => {
                        let bridge_rc = Rc::new(RefCell::new(bridge));
                        worklet_bridge.set(Some(bridge_rc.clone()));
                        sf2_presets.set(presets);
                        *note_player.borrow_mut() = create_note_player(
                            &sound_source_clone,
                            Rc::clone(&audio_ctx_for_loop),
                            Some(bridge_rc),
                        );
                    }
                    Err(e) => {
                        log::warn!(
                            "SoundFont worklet connect failed (oscillator fallback): {e}"
                        );
                    }
                }
                worklet_connecting.set(false);
            } else if worklet_bridge.get_untracked().is_some() {
                // Another view already connected — recreate note player with existing bridge
                *note_player.borrow_mut() = create_note_player(
                    &sound_source_clone,
                    Rc::clone(&audio_ctx_for_loop),
                    worklet_bridge.get_untracked(),
                );
            }

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
                    audio_error.set(Some("Audio playback failed".into()));
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
                    audio_error.set(Some("Audio playback failed".into()));
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

    let comparison_title = if is_interval_mode { "Interval Comparison" } else { "Comparison Training" };
    let tuning_label = if is_interval_mode {
        let ts = LocalStorageSettings.tuning_system();
        match ts {
            domain::TuningSystem::EqualTemperament => "Equal Temperament",
            domain::TuningSystem::JustIntonation => "Just Intonation",
        }
    } else {
        ""
    };

    #[allow(clippy::redundant_closure)]
    let on_back_cb = {
        let handler = SendWrapper::new(on_nav_start);
        Callback::new(move |ev| handler(ev))
    };
    #[allow(clippy::redundant_closure)]
    let on_help_cb = {
        let handler = SendWrapper::new(on_help_open);
        Callback::new(move |ev| handler(ev))
    };

    view! {
        <div class="pt-4 pb-12">
            <NavBar title=comparison_title back_href="/" on_back=on_back_cb>
                <NavIconButton label="Help".to_string() icon="?".to_string() on_click=on_help_cb />
            </NavBar>
            <HelpModal title="Comparison Training" sections=COMPARISON_HELP is_open=is_help_open on_close=on_help_close />

            <AudioGateOverlay />

            // SF2 loading indicator for direct navigation (bookmark)
            {move || {
                if matches!(sf2_load_status.get(), SoundFontLoadStatus::Fetching) {
                    view! {
                        <div
                            role="status"
                            aria-live="polite"
                            class="mt-4 rounded-lg bg-indigo-50 border border-indigo-200 px-4 py-3 text-center text-indigo-700 dark:bg-indigo-900/30 dark:border-indigo-700 dark:text-indigo-300"
                        >
                            <span class="inline-block animate-pulse">"Loading sounds\u{2026}"</span>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Interval info display — centered between nav and buttons
            {move || {
                let label = interval_label_text.get();
                if !label.is_empty() {
                    view! {
                        <div class="text-center mt-4 mb-2">
                            <p class="text-lg text-indigo-600 dark:text-indigo-400 font-medium">{label}</p>
                            <p class="text-sm text-gray-500 dark:text-gray-400">{tuning_label}</p>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Compact header: stats left, feedback icon right
            <div class="flex items-start justify-between mb-2">
                // Left: Training stats
                <TrainingStats
                    latest_value=latest_cent_difference.into()
                    session_best=stats_session_best.into()
                    trend=stats_trend.into()
                />
                // Right: checkmark/X feedback indicator
                <div class="text-right" aria-hidden="true">
                    {move || {
                        if show_feedback.get() {
                            if is_last_correct.get() {
                                view! { <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-green-500 dark:bg-green-600 text-white font-bold text-lg">{"\u{2714}"}</span> }.into_any()
                            } else {
                                view! { <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-red-500 dark:bg-red-600 text-white font-bold text-lg">{"\u{2718}"}</span> }.into_any()
                            }
                        } else {
                            view! { <div class="w-8 h-8"></div> }.into_any()
                        }
                    }}
                </div>
            </div>

            // Screen reader live region
            <div aria-live="polite" aria-atomic="true" class="sr-only">
                {move || sr_announcement.get()}
            </div>

            // Higher / Lower buttons — large blue rounded-rectangle cards
            {
                let btn_enabled = "flex flex-1 flex-col items-center justify-center gap-4 w-full rounded-2xl bg-blue-500 px-6 py-12 text-white text-2xl font-semibold shadow-md hover:bg-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:ring-offset-2 dark:bg-blue-600 dark:hover:bg-blue-500 dark:focus:ring-offset-gray-900";
                let btn_disabled = "flex flex-1 flex-col items-center justify-center gap-4 w-full rounded-2xl bg-gray-300 px-6 py-12 text-gray-500 text-2xl font-semibold cursor-not-allowed dark:bg-gray-700 dark:text-gray-500";
                view! {
                    <div class="flex flex-col gap-4 mt-4 landscape:flex-row">
                        <button
                            on:click=on_answer_higher
                            disabled=move || !buttons_enabled.get()
                            class=move || if buttons_enabled.get() { btn_enabled } else { btn_disabled }
                            aria-label="Higher"
                        >
                            <span class="flex items-center justify-center w-16 h-16 rounded-full bg-white/30 text-white text-3xl" aria-hidden="true">{"\u{2191}"}</span>
                            "Higher"
                        </button>
                        <button
                            on:click=on_answer_lower
                            disabled=move || !buttons_enabled.get()
                            class=move || if buttons_enabled.get() { btn_enabled } else { btn_disabled }
                            aria-label="Lower"
                        >
                            <span class="flex items-center justify-center w-16 h-16 rounded-full bg-white/30 text-white text-3xl" aria-hidden="true">{"\u{2193}"}</span>
                            "Lower"
                        </button>
                    </div>
                }
            }

            // Audio error notification — non-blocking, auto-dismissing
            {move || {
                if let Some(msg) = audio_error.get() {
                    view! {
                        <div
                            class="fixed bottom-12 left-1/2 -translate-x-1/2 bg-red-100 border border-red-400 text-red-800 px-4 py-2 rounded-lg shadow-md text-sm dark:bg-red-900 dark:border-red-700 dark:text-red-200"
                            role="alert"
                        >
                            {msg}
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

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
