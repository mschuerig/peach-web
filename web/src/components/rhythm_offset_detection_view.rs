use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped_with_cancellation;
use leptos_router::hooks::use_navigate;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};

use leptos_fluent::{I18n, move_tr, tr};

use crate::adapters::audio_context::{AudioContextManager, ensure_audio_ready};
use crate::adapters::audio_soundfont::{SF2Preset, WorkletBridge};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::rhythm_scheduler::{
    RhythmScheduler, RhythmStep, SchedulerConfig, schedule_click_at, select_percussion_program,
};
use crate::app::{WorkletAssets, base_href, ensure_worklet_connected};
use crate::bridge::{ProfilePort, RecordPort, TimelinePort};
use crate::components::TrainingStats;
use crate::components::audio_gate_overlay::AudioGateOverlay;
use crate::components::help_content::HelpModal;
use crate::components::nav_bar::{NavBar, NavIconButton};
use crate::help_sections::RHYTHM_OFFSET_DETECTION_HELP;
use domain::ports::UserSettings;
use domain::{
    PerceptualProfile, ProgressTimeline, RHYTHM_FEEDBACK_DURATION_SECS,
    RhythmOffsetDetectionSession, RhythmOffsetDetectionSessionState, TrainingDiscipline, Trend,
};
use leptos::reactive::owner::LocalStorage;
use web_sys::KeyboardEvent;

const POLL_INTERVAL_MS: u32 = 50;
/// Faster polling during click pattern playback for smoother dot animation.
const DOT_POLL_INTERVAL_MS: u32 = 20;
/// How long each dot stays lit (seconds).
const DOT_ON_DURATION_SECS: f64 = 0.12;
/// Pause after last click before enabling answer buttons.
const POST_PATTERN_DELAY_MS: u32 = 200;
/// Pause between feedback end and next trial start.
const INTER_TRIAL_DELAY_MS: u32 = 300;

#[component]
pub fn RhythmOffsetDetectionView() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile not provided");
    let audio_ctx: SendWrapper<Rc<RefCell<AudioContextManager>>> =
        use_context().expect("AudioContextManager not provided");
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline not provided");
    let db_store: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage> =
        use_context().expect("db_store not provided");

    // Eagerly create AudioContext within user gesture call stack
    if let Err(e) = audio_ctx.borrow_mut().get_or_create() {
        log::error!("Failed to create AudioContext: {e}");
    }

    let settings = LocalStorageSettings;
    let tempo = settings.tempo_bpm();

    let storage_error: RwSignal<Option<String>> = RwSignal::new(None);
    let audio_error: RwSignal<Option<String>> = RwSignal::new(None);

    let session = Rc::new(RefCell::new(RhythmOffsetDetectionSession::new(
        Rc::clone(&profile),
        Box::new(ProfilePort::new(Rc::clone(&profile))),
        Box::new(RecordPort::new(db_store, storage_error)),
        Box::new(TimelinePort::new(Rc::clone(&progress_timeline))),
        vec![],
    )));

    // Auto-dismiss error notifications after 5 seconds
    Effect::new(move || {
        if storage_error.get().is_some() {
            spawn_local_scoped_with_cancellation(async move {
                TimeoutFuture::new(5000).await;
                storage_error.set(None);
            });
        }
    });
    Effect::new(move || {
        if audio_error.get().is_some() {
            spawn_local_scoped_with_cancellation(async move {
                TimeoutFuture::new(5000).await;
                audio_error.set(None);
            });
        }
    });

    let training_discipline = TrainingDiscipline::RhythmOffsetDetection;

    // UI signals bridging domain state to view
    let show_feedback = RwSignal::new(false);
    let is_last_correct = RwSignal::new(false);
    let buttons_enabled = RwSignal::new(false);
    let active_dot: RwSignal<Option<usize>> = RwSignal::new(None);
    let difficulty_pct_signal: RwSignal<Option<f64>> = RwSignal::new(None);
    let sr_announcement = RwSignal::new(String::new());

    // Training stats
    let latest_difficulty: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_session_best: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_trend: RwSignal<Option<Trend>> = RwSignal::new(None);

    let i18n: I18n = expect_context();

    // Sync all UI signals from session state
    fn sync_session_to_signals(
        i18n: &I18n,
        session: &RefCell<RhythmOffsetDetectionSession>,
        show_feedback: RwSignal<bool>,
        is_last_correct: RwSignal<bool>,
        buttons_enabled: RwSignal<bool>,
        difficulty_pct_signal: RwSignal<Option<f64>>,
        sr_announcement: RwSignal<String>,
    ) {
        untrack(|| {
            let s = session.borrow();
            show_feedback.set(s.show_feedback());
            is_last_correct.set(s.is_last_answer_correct());
            buttons_enabled.set(s.state() == RhythmOffsetDetectionSessionState::AwaitingAnswer);
            difficulty_pct_signal.set(s.last_difficulty_pct());
            if s.show_feedback() {
                let result_text = if s.is_last_answer_correct() {
                    i18n.tr("correct")
                } else {
                    i18n.tr("incorrect")
                };
                let announcement = if let Some(pct) = s.last_difficulty_pct() {
                    format!("{result_text} — {pct:.0}%")
                } else {
                    result_text
                };
                sr_announcement.set(announcement);
            } else {
                sr_announcement.set(String::new());
            }
        });
    }

    let sync_signals = {
        let session = Rc::clone(&session);
        move || {
            sync_session_to_signals(
                &i18n,
                &session,
                show_feedback,
                is_last_correct,
                buttons_enabled,
                difficulty_pct_signal,
                sr_announcement,
            );
        }
    };

    // Cancellation flags
    let cancelled = Rc::new(Cell::new(false));
    let terminated = Rc::new(Cell::new(false));
    let help_paused: RwSignal<bool> = RwSignal::new(false);

    // Answer handler — used by buttons and keyboard
    let on_answer = {
        let session = Rc::clone(&session);
        let sync = sync_signals.clone();
        let cancelled = Rc::clone(&cancelled);
        Rc::new(move |is_early: bool| {
            let answer_str = if is_early { "early" } else { "late" };
            if cancelled.get() {
                log::info!("User pressed {answer_str} — ignored (cancelled)");
                return;
            }
            let state = session.borrow().state();
            if state != RhythmOffsetDetectionSessionState::AwaitingAnswer {
                log::info!("User pressed {answer_str} — ignored (state: {state:?})");
                return;
            }

            log::info!("User pressed {answer_str}");
            let timestamp = js_sys::Date::new_0()
                .to_iso_string()
                .as_string()
                .unwrap_or_default();
            session.borrow_mut().submit_answer(is_early, timestamp);

            // Update training stats
            {
                let s = session.borrow();
                latest_difficulty.set(s.last_difficulty_pct());
                // Session best = smallest difficulty % answered correctly
                if s.is_last_answer_correct()
                    && let Some(pct) = s.last_difficulty_pct()
                {
                    let current_best = stats_session_best.get_untracked();
                    if current_best.is_none() || pct < current_best.unwrap() {
                        stats_session_best.set(Some(pct));
                    }
                }
            }
            stats_trend.set(profile.borrow().trend(training_discipline));

            sync();
        })
    };

    // Button click handlers
    let on_answer_early = {
        let on_answer = Rc::clone(&on_answer);
        move |_| on_answer(true)
    };
    let on_answer_late = {
        let on_answer = Rc::clone(&on_answer);
        move |_| on_answer(false)
    };

    let navigate = use_navigate();

    // Navigation away handler
    let on_nav_away = {
        let cancelled = Rc::clone(&cancelled);
        let terminated = Rc::clone(&terminated);
        let session = Rc::clone(&session);
        let sync = sync_signals.clone();
        move || {
            terminated.set(true);
            cancelled.set(true);
            session.borrow_mut().stop();
            sync();
            sr_announcement.set(untrack(|| i18n.tr("training-stopped")));
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

    // Help modal
    let is_help_open = RwSignal::new(false);
    let on_help_open = {
        let cancelled = Rc::clone(&cancelled);
        move |_| {
            help_paused.set(true);
            cancelled.set(true);
            is_help_open.set(true);
        }
    };
    let on_help_close = Callback::new(move |()| {
        is_help_open.set(false);
        help_paused.set(false);
    });

    // Shared interruption closure
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

    // Capture locale-driven shortcut keys once at mount
    let early_key = untrack(|| tr!("rhythm-offset-early-key")).to_lowercase();
    let late_key = untrack(|| tr!("rhythm-offset-late-key")).to_lowercase();
    if early_key.chars().count() != 1 {
        log::warn!("rhythm-offset-early-key locale value is not a single character: {early_key:?}");
    }
    if late_key.chars().count() != 1 {
        log::warn!("rhythm-offset-late-key locale value is not a single character: {late_key:?}");
    }

    // Keyboard event handler
    let keydown_handler = {
        let on_answer = Rc::clone(&on_answer);
        Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
            let has_modifier = ev.ctrl_key() || ev.meta_key() || ev.alt_key();
            let key = ev.key().to_lowercase();
            match ev.key().as_str() {
                _ if has_modifier => {}
                "ArrowLeft" => {
                    ev.prevent_default();
                    on_answer(true); // early
                }
                "ArrowRight" => {
                    ev.prevent_default();
                    on_answer(false); // late
                }
                _ if key == early_key => {
                    ev.prevent_default();
                    on_answer(true); // early
                }
                _ if key == late_key => {
                    ev.prevent_default();
                    on_answer(false); // late
                }
                _ => {}
            }
        })
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let keydown_fn: JsValue = keydown_handler.as_ref().clone();
    document
        .add_event_listener_with_callback("keydown", keydown_fn.unchecked_ref())
        .unwrap();
    let _keydown_closure = StoredValue::new_local(keydown_handler);

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

    // AudioContext state change handler
    let audiocontext_handler = Closure::<dyn FnMut(web_sys::Event)>::new({
        let interrupt = Rc::clone(&interrupt_and_navigate);
        let terminated = Rc::clone(&terminated);
        move |event: web_sys::Event| {
            if let Some(target) = event.target()
                && let Some(base_ctx) = target.dyn_ref::<web_sys::BaseAudioContext>()
            {
                let state = base_ctx.state();
                log::debug!(
                    "[DIAG] RhythmOffsetDetectionView onstatechange — new state: {:?}",
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
                                    let terminated = Rc::clone(&terminated);
                                    let target = target.clone();
                                    spawn_local(async move {
                                        let _ = JsFuture::from(promise).await;
                                        if terminated.get() {
                                            return;
                                        }
                                        TimeoutFuture::new(500).await;
                                        if terminated.get() {
                                            return;
                                        }
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
            Rc::clone(&audio_ctx),
            keydown_fn,
            visibility_fn,
        ));
        on_cleanup(move || {
            let (cancelled, terminated, session, audio_ctx, keydown_fn, visibility_fn) =
                &*cleanup_state;
            terminated.set(true);
            cancelled.set(true);
            help_paused.set(false);
            session.borrow_mut().stop();
            audio_ctx.borrow().clear_state_change_handler();
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                let _ = document
                    .remove_event_listener_with_callback("keydown", keydown_fn.unchecked_ref());
                let _ = document.remove_event_listener_with_callback(
                    "visibilitychange",
                    visibility_fn.unchecked_ref(),
                );
            }
        });
    }

    let crate::app::AudioNeedsGesture(audio_needs_gesture) =
        use_context().expect("audio_needs_gesture context");

    // WorkletBridge and related context signals (provided in app.rs)
    let worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>, LocalStorage> =
        use_context().expect("WorkletBridge signal not provided");
    let worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, LocalStorage> =
        use_context().expect("worklet_assets not provided");
    let crate::app::WorkletConnecting(worklet_connecting) =
        use_context().expect("worklet_connecting not provided");
    let sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage> =
        use_context().expect("sf2_presets not provided");
    let sf_gain_node: RwSignal<Option<Rc<web_sys::GainNode>>, LocalStorage> =
        use_context().expect("sf_gain_node not provided");

    // Start the async training loop
    {
        let session = Rc::clone(&session);
        let cancelled = Rc::clone(&cancelled);
        let terminated = Rc::clone(&terminated);
        let audio_ctx_for_loop = Rc::clone(&audio_ctx);
        let sync = sync_signals.clone();
        spawn_local(async move {
            // Ensure AudioContext is created and running
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
                    audio_error.set(Some(untrack(|| i18n.tr("audio-engine-failed"))));
                    return;
                }
            };

            // Ensure worklet is connected before using the bridge
            ensure_worklet_connected(
                &ctx_rc,
                worklet_bridge,
                worklet_assets,
                worklet_connecting,
                sf2_presets,
                sf_gain_node,
            )
            .await;

            // Get the WorkletBridge for percussion playback
            let bridge = match worklet_bridge.get() {
                Some(b) => b,
                None => {
                    log::error!("WorkletBridge not available after connection attempt");
                    audio_error.set(Some(untrack(|| i18n.tr("audio-engine-failed"))));
                    return;
                }
            };

            // Select percussion program once for this training session
            select_percussion_program(&bridge);

            let feedback_ms = (RHYTHM_FEEDBACK_DURATION_SECS * 1000.0) as u32;
            let sixteenth_secs = tempo.sixteenth_note_duration_secs();

            // Outer loop enables training restart after help modal close
            'session: loop {
                if terminated.get() {
                    break;
                }

                session.borrow_mut().stop();
                cancelled.set(false);
                sync();
                sr_announcement.set(untrack(|| i18n.tr("training-started")));

                'training: loop {
                    if cancelled.get() {
                        break;
                    }

                    // Start a new trial
                    session.borrow_mut().start_trial(tempo);
                    sync();

                    let offset_secs = {
                        let s = session.borrow();
                        let params = s.current_trial_params().expect("trial params after start");
                        params.offset.ms() / 1000.0
                    };

                    // Compute all 4 beat times upfront
                    let now = ctx_rc.borrow().current_time();
                    let start_time = now + 0.050; // lead-in matching scheduler convention
                    let beat_times = [
                        start_time,
                        start_time + sixteenth_secs,
                        start_time + 2.0 * sixteenth_secs + offset_secs, // offset beat
                        start_time + 3.0 * sixteenth_secs,
                    ];

                    // Schedule clicks: use scheduler for beats 1, 2, 4 and manual for beat 3
                    let mut scheduler = RhythmScheduler::new(
                        Rc::clone(&ctx_rc),
                        Rc::clone(&bridge),
                        SchedulerConfig {
                            pattern: vec![
                                RhythmStep::Play,
                                RhythmStep::Play,
                                RhythmStep::Silent,
                                RhythmStep::Play,
                            ],
                            tempo,
                        },
                    );
                    scheduler.start();

                    // Schedule the offset click (beat 3) manually
                    schedule_click_at(&ctx_rc, &bridge, beat_times[2], false);

                    // Animate dots based on precomputed beat times
                    let dot_off_times: [f64; 4] = [
                        beat_times[0] + DOT_ON_DURATION_SECS,
                        beat_times[1] + DOT_ON_DURATION_SECS,
                        beat_times[2] + DOT_ON_DURATION_SECS,
                        beat_times[3] + DOT_ON_DURATION_SECS,
                    ];

                    // Wait for pattern to finish while animating dots
                    let pattern_end_time = beat_times[3] + DOT_ON_DURATION_SECS;
                    loop {
                        if cancelled.get() {
                            active_dot.set(None);
                            break 'training;
                        }

                        let now = ctx_rc.borrow().current_time();

                        // Determine which dot should be lit
                        let mut new_active = None;
                        for (i, (&on_time, &off_time)) in
                            beat_times.iter().zip(dot_off_times.iter()).enumerate()
                        {
                            if now >= on_time && now < off_time {
                                new_active = Some(i);
                                break;
                            }
                        }
                        if !terminated.get() {
                            active_dot.set(new_active);
                        }

                        if now >= pattern_end_time {
                            break;
                        }

                        TimeoutFuture::new(DOT_POLL_INTERVAL_MS).await;
                    }
                    // Drop scheduler to stop its interval timer
                    drop(scheduler);

                    if cancelled.get() {
                        break;
                    }

                    active_dot.set(None);

                    // Brief pause after pattern before enabling buttons
                    TimeoutFuture::new(POST_PATTERN_DELAY_MS).await;
                    if cancelled.get() {
                        break;
                    }

                    // Transition to AwaitingAnswer
                    session.borrow_mut().pattern_finished();
                    sync();

                    // Wait for user's answer
                    while session.borrow().state()
                        != RhythmOffsetDetectionSessionState::ShowingFeedback
                    {
                        if cancelled.get() {
                            break 'training;
                        }
                        if session.borrow().state() == RhythmOffsetDetectionSessionState::Idle {
                            break 'training;
                        }
                        TimeoutFuture::new(POLL_INTERVAL_MS).await;
                    }

                    // ShowingFeedback phase
                    sync();
                    TimeoutFuture::new(feedback_ms).await;
                    if cancelled.get() {
                        break;
                    }

                    // End feedback, prepare next trial
                    if session.borrow().state()
                        == RhythmOffsetDetectionSessionState::ShowingFeedback
                    {
                        session.borrow_mut().feedback_complete();
                        sync();
                    }

                    // Inter-trial pause
                    TimeoutFuture::new(INTER_TRIAL_DELAY_MS).await;
                }

                // After 'training loop exits, decide: restart or exit
                if terminated.get() || !help_paused.get_untracked() {
                    break 'session;
                }

                // Help modal is open — wait for close, then restart
                while help_paused.get_untracked() {
                    if terminated.get() {
                        break 'session;
                    }
                    TimeoutFuture::new(POLL_INTERVAL_MS).await;
                }
            }

            // Final cleanup
            if !terminated.get() {
                session.borrow_mut().stop();
                sync();
            }
        });
    }

    let rhythm_offset_title = move_tr!("rhythm-offset-title");
    let tempo_label =
        Signal::derive(move || tr!("bpm-label", {"value" => tempo.bpm().to_string()}));

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

    let btn_enabled = "flex flex-1 flex-col items-center justify-center gap-4 w-full rounded-2xl bg-blue-500 px-6 py-12 text-white text-2xl font-semibold shadow-md hover:bg-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:ring-offset-2 dark:bg-blue-600 dark:hover:bg-blue-500 dark:focus:ring-offset-gray-900";
    let btn_disabled = "flex flex-1 flex-col items-center justify-center gap-4 w-full rounded-2xl bg-gray-300 px-6 py-12 text-gray-500 text-2xl font-semibold cursor-not-allowed dark:bg-gray-700 dark:text-gray-500";

    view! {
        <div class="flex flex-col pt-4 pb-12 h-screen">
            <NavBar title=rhythm_offset_title back_href=base_href("/") on_back=on_back_cb pill_group=true>
                <NavIconButton label="Help".to_string() icon="?".to_string() on_click=on_help_cb circled=true />
                <NavIconButton label="Settings".to_string() icon="\u{2699}\u{FE0F}".to_string() href=base_href("/settings") />
                <NavIconButton label="Profile".to_string() icon="\u{1F4CA}".to_string() href=base_href("/profile") />
            </NavBar>
            <HelpModal title=move_tr!("rhythm-offset-help-title") sections=RHYTHM_OFFSET_DETECTION_HELP is_open=is_help_open on_close=on_help_close />

            <AudioGateOverlay />

            // Tempo display
            <div class="text-center text-sm text-gray-500 dark:text-gray-400 mt-2">
                {tempo_label}
            </div>

            // Training stats + feedback indicator
            <div class="flex items-start justify-between mb-2 px-4">
                // Left: Training stats
                <TrainingStats
                    latest_value=latest_difficulty.into()
                    session_best=stats_session_best.into()
                    trend=stats_trend.into()
                    discipline=training_discipline
                />
                // Right: feedback icon + difficulty percentage
                <div class="text-right" aria-hidden="true">
                    {move || {
                        if show_feedback.get() {
                            let icon = if is_last_correct.get() {
                                view! { <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-green-500 dark:bg-green-600 text-white font-bold text-lg">{"\u{2714}"}</span> }.into_any()
                            } else {
                                view! { <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-red-500 dark:bg-red-600 text-white font-bold text-lg">{"\u{2718}"}</span> }.into_any()
                            };
                            let pct_text = difficulty_pct_signal.get().map(|pct| tr!("difficulty-pct", {"value" => format!("{:.0}", pct)})).unwrap_or_default();
                            view! {
                                <div class="flex items-center gap-2">
                                    {icon}
                                    <span class="text-lg font-semibold text-gray-700 dark:text-gray-300">{pct_text}</span>
                                </div>
                            }.into_any()
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

            // 4-dot visual metronome
            <div class="flex justify-center items-center gap-6 my-8" aria-hidden="true">
                {(0..4usize).map(|i| {
                    let is_accent = i == 0;
                    let is_offset = i == 2;
                    view! {
                        <div class=move || {
                            let active = active_dot.get() == Some(i);
                            let size = if is_accent { "w-10 h-10" } else { "w-7 h-7" };
                            let color = if active {
                                "bg-blue-500 dark:bg-blue-400"
                            } else {
                                "bg-gray-300 dark:bg-gray-600"
                            };
                            let border = if is_offset {
                                " ring-2 ring-gray-400 dark:ring-gray-500 ring-offset-2 dark:ring-offset-gray-900"
                            } else {
                                ""
                            };
                            format!("{size} rounded-full transition-colors duration-75 {color}{border}")
                        } />
                    }
                }).collect_view()}
            </div>

            // Early / Late answer buttons
            <div class="flex gap-4 mt-auto px-4 landscape:flex-row">
                <button
                    on:click=on_answer_early
                    disabled=move || !buttons_enabled.get()
                    class=move || if buttons_enabled.get() { btn_enabled } else { btn_disabled }
                    aria-label=move || tr!("early")
                >
                    <span class="flex items-center justify-center w-16 h-16 rounded-full bg-white/30 text-white text-3xl" aria-hidden="true">{"\u{2190}"}</span>
                    {move || tr!("early")}
                </button>
                <button
                    on:click=on_answer_late
                    disabled=move || !buttons_enabled.get()
                    class=move || if buttons_enabled.get() { btn_enabled } else { btn_disabled }
                    aria-label=move || tr!("late")
                >
                    <span class="flex items-center justify-center w-16 h-16 rounded-full bg-white/30 text-white text-3xl" aria-hidden="true">{"\u{2192}"}</span>
                    {move || tr!("late")}
                </button>
            </div>

            // Audio error notification
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
