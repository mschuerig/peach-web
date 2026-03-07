use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};

use crate::adapters::audio_context::{AudioContextManager, ensure_audio_ready};
use crate::adapters::audio_soundfont::{SF2Preset, WorkletBridge};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::note_player::{UnifiedPlaybackHandle, create_note_player};
use crate::app::{SoundFontLoadStatus, WorkletAssets, ensure_worklet_connected};
use crate::bridge::{PitchMatchingDataStoreObserver, ProgressTimelineObserver};
use crate::components::TrainingStats;
use crate::components::audio_gate_overlay::AudioGateOverlay;
use crate::components::help_content::HelpModal;
use crate::components::nav_bar::{NavBar, NavIconButton};
use crate::components::pitch_slider::VerticalPitchSlider;
use crate::help_sections::PITCH_MATCHING_HELP;
use crate::interval_codes::{interval_label, parse_intervals_param};
use domain::ports::{NotePlayer, PitchMatchingObserver, PlaybackHandle};
use domain::types::{AmplitudeDB, MIDIVelocity};
use domain::{
    FEEDBACK_DURATION_SECS, Interval, PITCH_MATCHING_VELOCITY, PerceptualProfile,
    PitchMatchingSession, PitchMatchingSessionState, ProgressTimeline, TrainingMode, Trend,
};
use leptos::reactive::owner::LocalStorage;
use leptos_router::hooks::use_query_map;

const POLL_INTERVAL_MS: u32 = 50;

#[component]
pub fn PitchMatchingView() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile not provided");
    let audio_ctx: SendWrapper<Rc<RefCell<AudioContextManager>>> =
        use_context().expect("AudioContextManager not provided");
    let db_store: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage> =
        use_context().expect("db_store not provided");
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline not provided");
    let worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>, LocalStorage> =
        use_context().expect("worklet_bridge not provided");
    let sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage> =
        use_context().expect("sf2_presets not provided");
    let worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, LocalStorage> =
        use_context().expect("worklet_assets not provided");
    let sf2_load_status: RwSignal<SoundFontLoadStatus> =
        use_context().expect("SoundFontLoadStatus not provided");
    let crate::app::WorkletConnecting(worklet_connecting) =
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

    // Build observers — PitchMatchingSession already updates profile directly,
    // so only the data store observer is needed here.
    let observers: Vec<Box<dyn PitchMatchingObserver>> = vec![
        Box::new(ProgressTimelineObserver::new(Rc::clone(&progress_timeline))),
        Box::new(PitchMatchingDataStoreObserver::new(db_store, storage_error)),
    ];

    let session = Rc::new(RefCell::new(PitchMatchingSession::new(
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
        TrainingMode::IntervalMatching
    } else {
        TrainingMode::UnisonMatching
    };

    // Training stats signals
    let latest_cent_error: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_session_best: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_trend: RwSignal<Option<Trend>> = RwSignal::new(None);

    // Signals bridging domain state to UI
    let slider_enabled = RwSignal::new(false);
    let show_feedback = RwSignal::new(false);
    let feedback_text = RwSignal::new(String::new());
    let feedback_color = RwSignal::new(String::new());
    let feedback_arrow = RwSignal::new(String::new());
    let sr_announcement = RwSignal::new(String::new());
    let reset_trigger = RwSignal::new(0u32);

    // Tunable note handle for real-time frequency adjustment
    let tunable_handle: Rc<RefCell<Option<UnifiedPlaybackHandle>>> = Rc::new(RefCell::new(None));

    // Sync UI signals from session state
    fn sync_session_to_signals(
        session: &RefCell<PitchMatchingSession>,
        slider_enabled: RwSignal<bool>,
        show_feedback: RwSignal<bool>,
        feedback_text: RwSignal<String>,
        feedback_color: RwSignal<String>,
        feedback_arrow: RwSignal<String>,
        sr_announcement: RwSignal<String>,
        interval_label_text: RwSignal<String>,
        is_interval_mode: bool,
    ) {
        let s = session.borrow();
        let state = s.state();
        slider_enabled.set(
            state == PitchMatchingSessionState::AwaitingSliderTouch
                || state == PitchMatchingSessionState::PlayingTunable,
        );
        show_feedback.set(s.show_feedback());

        if s.show_feedback()
            && let Some(completed) = s.last_completed()
        {
            let error = completed.user_cent_error();
            let abs_error = error.abs();

            // Arrow: up for sharp, down for flat, dot for dead center
            let arrow = if abs_error < 1.0 {
                "\u{00B7}".to_string() // · (dead center)
            } else if error > 0.0 {
                "\u{2191}".to_string() // ↑ (sharp)
            } else {
                "\u{2193}".to_string() // ↓ (flat)
            };

            // Text: signed cent offset or "Dead center"
            let text = if abs_error < 1.0 {
                "Dead center".to_string()
            } else {
                format!("{:+.0} cents", error)
            };

            // Color: green (<10), yellow (10-30), red (>30)
            let color = if abs_error < 10.0 {
                "text-green-600 dark:text-green-400".to_string()
            } else if abs_error <= 30.0 {
                "text-yellow-600 dark:text-yellow-400".to_string()
            } else {
                "text-red-600 dark:text-red-400".to_string()
            };

            // Screen reader announcement
            let sr = if abs_error < 1.0 {
                "Dead center".to_string()
            } else if error > 0.0 {
                format!("{:.0} cents sharp", abs_error)
            } else {
                format!("{:.0} cents flat", abs_error)
            };

            feedback_arrow.set(arrow);
            feedback_text.set(text);
            feedback_color.set(color);
            sr_announcement.set(sr);
        } else {
            sr_announcement.set(String::new());
        }

        if is_interval_mode && let Some(di) = s.current_interval() {
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
                slider_enabled,
                show_feedback,
                feedback_text,
                feedback_color,
                feedback_arrow,
                sr_announcement,
                interval_label_text,
                is_interval_mode,
            );
        }
    };

    // Cancellation flag shared between loop and event handlers
    let cancelled = Rc::new(Cell::new(false));
    // Permanent exit flag — distinguishes real cancellation from help-pause
    let terminated = Rc::new(Cell::new(false));
    // Help modal pause — when true, training loop waits instead of exiting
    let help_paused: RwSignal<bool> = RwSignal::new(false);

    // Commit handler — used by slider on_commit and keyboard Enter/Space
    let on_commit = {
        let session = Rc::clone(&session);
        let progress_timeline = Rc::clone(&progress_timeline);
        let tunable_handle = Rc::clone(&tunable_handle);
        let sync = sync_signals.clone();
        let cancelled = Rc::clone(&cancelled);
        Rc::new(move |value: f64| {
            if cancelled.get() {
                return;
            }
            let state = session.borrow().state();
            if state != PitchMatchingSessionState::PlayingTunable {
                return;
            }

            let timestamp = js_sys::Date::new_0()
                .to_iso_string()
                .as_string()
                .unwrap_or_default();
            session.borrow_mut().commit_pitch(value, timestamp);

            // Update training stats signals
            {
                let s = session.borrow();
                if let Some(completed) = s.last_completed() {
                    let abs_error = completed.user_cent_error().abs();
                    latest_cent_error.set(Some(abs_error));
                    let new_best = match stats_session_best.get_untracked() {
                        Some(best) if abs_error < best => abs_error,
                        None => abs_error,
                        Some(best) => best,
                    };
                    stats_session_best.set(Some(new_best));
                }
            }
            stats_trend.set(progress_timeline.borrow().trend(training_mode));

            // Stop tunable note
            if let Some(ref mut h) = *tunable_handle.borrow_mut() {
                h.stop();
            }

            sync();
        })
    };

    // Slider on_change handler — SendWrapper bridges Rc<RefCell> (non-Send) into
    // Callback's Send+Sync requirement. Safe because WASM is single-threaded.
    let slider_on_change = {
        let handler = SendWrapper::new({
            let session = Rc::clone(&session);
            let tunable_handle = Rc::clone(&tunable_handle);
            let note_player = Rc::clone(&note_player);
            move |value: f64| {
                let was_awaiting =
                    session.borrow().state() == PitchMatchingSessionState::AwaitingSliderTouch;
                if let Some(freq) = session.borrow_mut().adjust_pitch(value) {
                    if was_awaiting {
                        // First touch: start the tunable note
                        match note_player.borrow().play(
                            freq,
                            MIDIVelocity::new(PITCH_MATCHING_VELOCITY),
                            AmplitudeDB::new(0.0),
                        ) {
                            Ok(handle) => {
                                tunable_handle.borrow_mut().replace(handle);
                            }
                            Err(e) => {
                                log::error!("Tunable note playback failed: {e}");
                            }
                        }
                    } else if let Some(ref mut h) = *tunable_handle.borrow_mut()
                        && let Err(e) = h.adjust_frequency(freq)
                    {
                        log::warn!("Failed to adjust frequency: {e}");
                    }
                }
            }
        });
        Callback::new(move |value: f64| {
            handler(value);
        })
    };

    // Slider on_commit handler
    let slider_on_commit = {
        let handler = SendWrapper::new({
            let on_commit = Rc::clone(&on_commit);
            move |value: f64| {
                on_commit(value);
            }
        });
        Callback::new(move |value: f64| {
            handler(value);
        })
    };

    // Navigation
    let navigate = use_navigate();

    // Navigation away handler — stops training before nav
    let on_nav_away = {
        let cancelled = Rc::clone(&cancelled);
        let terminated = Rc::clone(&terminated);
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let tunable_handle = Rc::clone(&tunable_handle);
        let sync = sync_signals.clone();
        move || {
            terminated.set(true);
            cancelled.set(true);
            session.borrow_mut().stop();
            if let Some(ref mut h) = *tunable_handle.borrow_mut() {
                h.stop();
            }
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
        let note_player = Rc::clone(&note_player);
        let tunable_handle = Rc::clone(&tunable_handle);
        move |_| {
            help_paused.set(true);
            cancelled.set(true);
            if let Some(ref mut h) = *tunable_handle.borrow_mut() {
                h.stop();
            }
            note_player.borrow().stop_all();
            is_help_open.set(true);
        }
    };

    let on_help_close = Callback::new(move |()| {
        is_help_open.set(false);
        help_paused.set(false);
    });

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

    let document = web_sys::window().unwrap().document().unwrap();

    // Visibility change handler
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
                    "[DIAG] PitchMatchingView onstatechange fired — new state: {:?}",
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
                                            && ctx.state() != web_sys::AudioContextState::Running
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

    // Cleanup on component unmount
    {
        let cleanup_state = SendWrapper::new((
            Rc::clone(&cancelled),
            Rc::clone(&terminated),
            Rc::clone(&session),
            Rc::clone(&note_player),
            Rc::clone(&tunable_handle),
            Rc::clone(&audio_ctx),
            visibility_fn,
        ));
        on_cleanup(move || {
            let (
                cancelled,
                terminated,
                session,
                note_player,
                tunable_handle,
                audio_ctx,
                visibility_fn,
            ) = &*cleanup_state;
            terminated.set(true);
            cancelled.set(true);
            help_paused.set(false);
            session.borrow_mut().stop();
            if let Some(ref mut h) = *tunable_handle.borrow_mut() {
                h.stop();
            }
            note_player.borrow().stop_all();
            audio_ctx.borrow().clear_state_change_handler();
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                let _ = document.remove_event_listener_with_callback(
                    "visibilitychange",
                    visibility_fn.unchecked_ref(),
                );
            }
        });
    }

    let crate::app::AudioNeedsGesture(audio_needs_gesture) =
        use_context().expect("audio_needs_gesture context");

    // Start the async training loop
    {
        let session = Rc::clone(&session);
        let note_player = Rc::clone(&note_player);
        let tunable_handle = Rc::clone(&tunable_handle);
        let cancelled = Rc::clone(&cancelled);
        let terminated = Rc::clone(&terminated);
        let audio_ctx_for_loop = Rc::clone(&audio_ctx);
        let sync = sync_signals.clone();
        spawn_local(async move {
            // Ensure AudioContext is created and running (waits for gesture if needed)
            let ctx_rc = match ensure_audio_ready(
                &audio_ctx_for_loop,
                audio_needs_gesture,
                &cancelled,
            )
            .await
            {
                Ok(ctx) => ctx,
                Err(e) => {
                    log::error!("AudioContext failed: {e}");
                    audio_error.set(Some("Audio engine failed to start".into()));
                    return;
                }
            };

            // Wait for SF2 assets if user selected SoundFont
            if sound_source_clone.starts_with("sf2:") {
                while matches!(
                    sf2_load_status.get_untracked(),
                    SoundFontLoadStatus::Fetching
                ) {
                    if cancelled.get() {
                        return;
                    }
                    TimeoutFuture::new(100).await;
                }
                if let SoundFontLoadStatus::Failed(ref msg) = sf2_load_status.get_untracked() {
                    log::warn!("SF2 load failed, falling back to oscillator: {msg}");
                    audio_error.set(Some(
                        "Selected sound could not be loaded. Using default sound.".into(),
                    ));
                }
            }

            // Phase 2: connect worklet if assets are available but bridge isn't.
            ensure_worklet_connected(
                &ctx_rc,
                worklet_bridge,
                worklet_assets,
                worklet_connecting,
                sf2_presets,
            )
            .await;
            *note_player.borrow_mut() = create_note_player(
                &sound_source_clone,
                Rc::clone(&audio_ctx_for_loop),
                worklet_bridge.get_untracked(),
            );

            let feedback_ms = (FEEDBACK_DURATION_SECS * 1000.0) as u32;

            // Outer loop enables training restart after help modal close.
            // Inner 'training loop breaks on cancelled; outer loop checks whether
            // to restart (help_paused) or exit permanently (terminated).
            'session: loop {
                if terminated.get() {
                    break;
                }

                session.borrow_mut().stop();
                session
                    .borrow_mut()
                    .start(intervals_from_query.clone(), &settings);
                cancelled.set(false);
                sync();
                sr_announcement.set("Training started".into());

                'training: loop {
                    if cancelled.get() {
                        break;
                    }

                    let data = match session.borrow().current_playback_data() {
                        Some(data) => data,
                        None => break,
                    };

                    let duration_ms = (data.duration.raw_value() * 1000.0) as u32;

                    // === PlayingReference phase (slider disabled) ===
                    note_player.borrow().stop_all();
                    if let Err(e) = note_player.borrow().play_for_duration(
                        data.reference_frequency,
                        data.duration,
                        MIDIVelocity::new(PITCH_MATCHING_VELOCITY),
                        AmplitudeDB::new(0.0),
                    ) {
                        log::error!("Reference note playback failed: {e}");
                        audio_error.set(Some("Audio playback failed".into()));
                    }

                    // Wait for reference note duration
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

                    // Transition: PlayingReference → AwaitingSliderTouch
                    session.borrow_mut().on_reference_finished();
                    sync();

                    // === AwaitingSliderTouch phase ===
                    // Tunable note will start when user touches the slider (see slider_on_change)

                    // Enable slider and reset to center for new challenge
                    reset_trigger.set(reset_trigger.get_untracked() + 1);

                    // Wait for commit (slider release or Enter/Space)
                    while session.borrow().state() != PitchMatchingSessionState::ShowingFeedback {
                        if cancelled.get() {
                            break 'training;
                        }
                        if session.borrow().state() == PitchMatchingSessionState::Idle {
                            break 'training;
                        }
                        TimeoutFuture::new(POLL_INTERVAL_MS).await;
                    }

                    // === ShowingFeedback phase ===
                    sync();
                    TimeoutFuture::new(feedback_ms).await;
                    if cancelled.get() {
                        break;
                    }

                    // End feedback, generate next challenge
                    if session.borrow().state() == PitchMatchingSessionState::ShowingFeedback {
                        session.borrow_mut().on_feedback_finished();
                        sync();
                    }
                }

                // After 'training loop exits, decide: restart or exit
                if !help_paused.get_untracked() {
                    break 'session; // Real cancellation (nav, visibility, etc.)
                }

                // Help modal is open — wait for it to close, then restart
                while help_paused.get_untracked() {
                    if terminated.get() {
                        break 'session;
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                }
            }

            // Final cleanup
            session.borrow_mut().stop();
            if let Some(ref mut h) = *tunable_handle.borrow_mut() {
                h.stop();
            }
            note_player.borrow().stop_all();
            sync();
        });
    }

    let matching_title = if is_interval_mode {
        "Interval Pitch Matching"
    } else {
        "Pitch Matching Training"
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
        <div class="flex flex-col pt-4 pb-12 h-screen">
            <NavBar title=matching_title back_href="/" on_back=on_back_cb>
                <NavIconButton label="Help".to_string() icon="?".to_string() on_click=on_help_cb />
            </NavBar>
            <HelpModal title="Pitch Matching Training" sections=PITCH_MATCHING_HELP is_open=is_help_open on_close=on_help_close />

            <AudioGateOverlay />

            // SF2 loading indicator for direct navigation (bookmark)
            {move || {
                if matches!(sf2_load_status.get(), SoundFontLoadStatus::Fetching) {
                    view! {
                        <div
                            role="status"
                            aria-live="polite"
                            class="rounded-lg bg-indigo-50 border border-indigo-200 px-4 py-3 text-center text-indigo-700 dark:bg-indigo-900/30 dark:border-indigo-700 dark:text-indigo-300"
                        >
                            <span class="inline-block animate-pulse">"Loading sounds\u{2026}"</span>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Compact header: stats left, deviation right, interval center
            <div class="flex items-start justify-between mb-2">
                // Left: Latest + Best stats
                <TrainingStats
                    latest_value=latest_cent_error.into()
                    session_best=stats_session_best.into()
                    trend=stats_trend.into()
                />
                // Right: signed cent deviation feedback
                <div class="text-right" aria-hidden="true">
                    {move || {
                        if show_feedback.get() {
                            let arrow = feedback_arrow.get();
                            let text = feedback_text.get();
                            let color = feedback_color.get();
                            view! {
                                <div class=format!("flex items-center justify-end gap-1 {color}")>
                                    <span class="text-2xl font-bold">{text}</span>
                                    <span class="text-2xl">{arrow}</span>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="h-8"></div> }.into_any()
                        }
                    }}
                </div>
            </div>

            // Interval label — centered below stats
            {move || {
                let label = interval_label_text.get();
                if !label.is_empty() {
                    view! {
                        <p class="text-center text-lg text-indigo-600 dark:text-indigo-400 font-medium mb-2">{label}</p>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            // Screen reader live region
            <div aria-live="polite" aria-atomic="true" class="sr-only">
                {move || sr_announcement.get()}
            </div>

            // Pitch slider — fills all remaining vertical space
            <div class="flex-1 flex justify-center items-stretch min-h-0">
                <VerticalPitchSlider
                    enabled=slider_enabled.into()
                    on_change=slider_on_change
                    on_commit=slider_on_commit
                    reset_trigger=reset_trigger.into()
                />
            </div>

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

            // Storage error notification
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
