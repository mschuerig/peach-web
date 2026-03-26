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
use crate::adapters::audio_latency::{bridge_event_to_audio_time, get_output_latency};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::rhythm_scheduler::{
    NORMAL_GAIN, RhythmScheduler, RhythmStep, SchedulerConfig, create_click_buffer,
    play_click_immediate,
};
use crate::app::base_href;
use crate::bridge::{ProfilePort, RecordPort, TimelinePort};
use crate::components::TrainingStats;
use crate::components::audio_gate_overlay::AudioGateOverlay;
use crate::components::help_content::HelpModal;
use crate::components::nav_bar::{NavBar, NavIconButton};
use crate::help_sections::CONTINUOUS_RHYTHM_MATCHING_HELP;
use domain::ports::UserSettings;
use domain::training::CYCLES_PER_TRIAL;
use domain::{
    ContinuousRhythmMatchingSession, ContinuousRhythmMatchingSessionState, PerceptualProfile,
    ProgressTimeline, RandomGapSelector, TrainingDiscipline, Trend,
};
use leptos::reactive::owner::LocalStorage;
use web_sys::KeyboardEvent;

/// Polling interval for the training loop (ms).
const POLL_INTERVAL_MS: u32 = 50;
/// Faster polling for dot animation updates.
const DOT_POLL_INTERVAL_MS: u32 = 20;
/// How long each dot stays lit (seconds).
const DOT_ON_DURATION_SECS: f64 = 0.12;

/// Timing feedback color thresholds (as percentage of sixteenth note duration).
const GREEN_THRESHOLD_PCT: f64 = 5.0;
const YELLOW_THRESHOLD_PCT: f64 = 15.0;

/// Duration to show the trial summary (ms).
const TRIAL_SUMMARY_DURATION_MS: u32 = 2000;

#[component]
pub fn ContinuousRhythmMatchingView() -> impl IntoView {
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
    let enabled_positions = settings.enabled_gap_positions();

    let storage_error: RwSignal<Option<String>> = RwSignal::new(None);
    let audio_error: RwSignal<Option<String>> = RwSignal::new(None);

    let session = Rc::new(RefCell::new(ContinuousRhythmMatchingSession::new(
        Rc::clone(&profile),
        Box::new(ProfilePort::new(Rc::clone(&profile))),
        Box::new(RecordPort::new(db_store, storage_error)),
        Box::new(TimelinePort::new(Rc::clone(&progress_timeline))),
        vec![],
        Box::new(RandomGapSelector),
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

    let training_discipline = TrainingDiscipline::ContinuousRhythmMatching;

    // UI signals bridging domain state to view
    let active_dot: RwSignal<Option<usize>> = RwSignal::new(None);
    let gap_dot: RwSignal<Option<usize>> = RwSignal::new(None);
    let sr_announcement = RwSignal::new(String::new());
    let cycle_counter: RwSignal<u16> = RwSignal::new(0);

    // Tap feedback signals
    let tap_feedback_visible = RwSignal::new(false);
    let tap_offset_ms: RwSignal<Option<f64>> = RwSignal::new(None);

    // Trial summary signals
    let trial_summary_visible = RwSignal::new(false);
    let trial_hit_rate: RwSignal<Option<f64>> = RwSignal::new(None);
    let trial_mean_offset: RwSignal<Option<f64>> = RwSignal::new(None);

    // Training stats
    let latest_difficulty: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_session_best: RwSignal<Option<f64>> = RwSignal::new(None);
    let stats_trend: RwSignal<Option<Trend>> = RwSignal::new(None);

    let i18n: I18n = expect_context();

    // Cancellation flags
    let cancelled = Rc::new(Cell::new(false));
    let terminated = Rc::new(Cell::new(false));
    let help_paused: RwSignal<bool> = RwSignal::new(false);

    // Shared tap result for the current cycle — set by pointerdown, consumed by training loop
    let tap_result: Rc<Cell<Option<f64>>> = Rc::new(Cell::new(None));

    // Shared audio resources populated by the training loop, used by the tap handler
    let shared_ctx: Rc<RefCell<Option<Rc<RefCell<web_sys::AudioContext>>>>> =
        Rc::new(RefCell::new(None));
    let shared_click_buffer: Rc<RefCell<Option<web_sys::AudioBuffer>>> =
        Rc::new(RefCell::new(None));

    let navigate = use_navigate();

    // Navigation away handler
    let on_nav_away = {
        let cancelled = Rc::clone(&cancelled);
        let terminated = Rc::clone(&terminated);
        let session = Rc::clone(&session);
        move || {
            terminated.set(true);
            cancelled.set(true);
            session.borrow_mut().stop();
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

    // Tap handler — called on pointerdown for lowest latency
    // Accepts event_timestamp_ms (PointerEvent/KeyboardEvent.timeStamp in performance.now() ms)
    let on_tap = {
        let session = Rc::clone(&session);
        let shared_ctx = Rc::clone(&shared_ctx);
        let shared_click_buffer = Rc::clone(&shared_click_buffer);
        let tap_result = Rc::clone(&tap_result);
        let cancelled = Rc::clone(&cancelled);
        Rc::new(move |event_timestamp_ms: f64| {
            if cancelled.get() {
                return;
            }
            if session.borrow().state() != ContinuousRhythmMatchingSessionState::Running {
                return;
            }

            // Get audio context from shared state
            let ctx_rc = match shared_ctx.borrow().as_ref() {
                Some(ctx) => Rc::clone(ctx),
                None => return, // Not yet initialized
            };

            // Bridge event timestamp to audio clock time, falling back to currentTime
            let tap_time = bridge_event_to_audio_time(&ctx_rc, event_timestamp_ms)
                .unwrap_or_else(|| ctx_rc.borrow().current_time());

            // Read output latency (re-read per tap — value can change at runtime)
            let output_latency = get_output_latency(&ctx_rc);

            // Evaluate tap against session
            let offset = session.borrow_mut().handle_tap(tap_time, output_latency);

            if let Some(ref rhythm_offset) = offset {
                // Store tap offset for cycle_complete
                tap_result.set(Some(rhythm_offset.ms()));

                // Show timing feedback
                tap_feedback_visible.set(true);
                tap_offset_ms.set(Some(rhythm_offset.ms()));

                // Auto-hide feedback
                spawn_local_scoped_with_cancellation(async move {
                    TimeoutFuture::new(400).await;
                    tap_feedback_visible.set(false);
                });

                // Screen reader announcement
                let ms = rhythm_offset.ms();
                let direction = if ms < 0.0 { "<" } else { ">" };
                sr_announcement.set(format!("{direction} {:.0} ms", ms.abs()));
            }

            // Play click at tap moment for audible fill (reuse shared buffer)
            if let Some(ref buf) = *shared_click_buffer.borrow() {
                let _ = play_click_immediate(&ctx_rc, buf, NORMAL_GAIN);
            }
        })
    };

    // Keyboard event handler — Space/Enter to tap
    let keydown_handler = {
        let on_tap = Rc::clone(&on_tap);
        Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
            let has_modifier = ev.ctrl_key() || ev.meta_key() || ev.alt_key();
            match ev.key().as_str() {
                _ if has_modifier => {}
                " " | "Enter" => {
                    ev.prevent_default();
                    on_tap(ev.time_stamp());
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
        move |event: web_sys::Event| {
            if let Some(target) = event.target()
                && let Some(base_ctx) = target.dyn_ref::<web_sys::BaseAudioContext>()
            {
                let state = base_ctx.state();
                log::debug!(
                    "[DIAG] ContinuousRhythmMatchingView onstatechange — new state: {:?}",
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

    // Start the async training loop
    {
        let session = Rc::clone(&session);
        let cancelled = Rc::clone(&cancelled);
        let terminated = Rc::clone(&terminated);
        let audio_ctx_for_loop = Rc::clone(&audio_ctx);
        let tap_result = Rc::clone(&tap_result);
        let enabled_positions = enabled_positions.clone();
        let shared_ctx_for_loop = Rc::clone(&shared_ctx);
        let shared_click_buffer_for_loop = Rc::clone(&shared_click_buffer);
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

            // Create click buffer once for tap playback
            let click_buffer = match create_click_buffer(&ctx_rc.borrow()) {
                Ok(buf) => buf,
                Err(e) => {
                    log::error!("Failed to create click buffer: {e}");
                    audio_error.set(Some(untrack(|| i18n.tr("audio-playback-failed"))));
                    return;
                }
            };

            // Share audio resources with the tap handler
            *shared_ctx_for_loop.borrow_mut() = Some(Rc::clone(&ctx_rc));
            *shared_click_buffer_for_loop.borrow_mut() = Some(click_buffer.clone());

            let sixteenth_secs = tempo.sixteenth_note_duration_secs();

            // Outer loop enables training restart after help modal close
            'session: loop {
                if terminated.get() {
                    break;
                }

                session.borrow_mut().stop();
                cancelled.set(false);
                sr_announcement.set(untrack(|| i18n.tr("training-started")));

                // Start the session
                session.borrow_mut().start(tempo, enabled_positions.clone());

                'training: loop {
                    if cancelled.get() {
                        break;
                    }

                    // Get gap position for this cycle
                    let gap_position = session.borrow().current_gap_position();
                    let gap_index = gap_position.map(|p| match p {
                        domain::types::StepPosition::First => 0usize,
                        domain::types::StepPosition::Second => 1,
                        domain::types::StepPosition::Third => 2,
                        domain::types::StepPosition::Fourth => 3,
                    });

                    if !terminated.get() {
                        gap_dot.set(gap_index);
                        cycle_counter.set(session.borrow().current_cycle_index());
                    }

                    // Build pattern: 4 steps, gap position is Silent
                    let pattern: Vec<RhythmStep> = (0..4)
                        .map(|i| {
                            if Some(i) == gap_index {
                                RhythmStep::Silent
                            } else {
                                RhythmStep::Play
                            }
                        })
                        .collect();

                    // Create scheduler for one cycle
                    let mut scheduler = RhythmScheduler::new(
                        Rc::clone(&ctx_rc),
                        click_buffer.clone(),
                        SchedulerConfig { pattern, tempo },
                    );

                    // Compute beat times for this cycle
                    let now = ctx_rc.borrow().current_time();
                    let start_time = now + 0.050; // lead-in matching scheduler convention
                    let beat_times: [f64; 4] = [
                        start_time,
                        start_time + sixteenth_secs,
                        start_time + 2.0 * sixteenth_secs,
                        start_time + 3.0 * sixteenth_secs,
                    ];

                    // Tell session the scheduled time of the gap
                    if let Some(gi) = gap_index {
                        session.borrow_mut().set_gap_scheduled_time(beat_times[gi]);
                    }

                    // Reset tap result for this cycle
                    tap_result.set(None);

                    scheduler.start();

                    // Animate dots for this cycle
                    let dot_off_times: [f64; 4] = [
                        beat_times[0] + DOT_ON_DURATION_SECS,
                        beat_times[1] + DOT_ON_DURATION_SECS,
                        beat_times[2] + DOT_ON_DURATION_SECS,
                        beat_times[3] + DOT_ON_DURATION_SECS,
                    ];
                    let cycle_end_time = beat_times[3] + sixteenth_secs;

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

                        if now >= cycle_end_time {
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

                    // Complete the cycle with tap result
                    let tap_offset = tap_result.get().map(domain::types::RhythmOffset::new);
                    let timestamp = js_sys::Date::new_0()
                        .to_iso_string()
                        .as_string()
                        .unwrap_or_default();

                    let completed_trial =
                        session.borrow_mut().cycle_complete(tap_offset, timestamp);

                    if !terminated.get() {
                        cycle_counter.set(session.borrow().current_cycle_index());
                    }

                    // If a trial completed, show summary and update stats
                    if let Some(ref trial) = completed_trial {
                        let metric = trial.metric_value();
                        latest_difficulty.set(Some(metric));

                        // Update session best (smallest metric = best)
                        let current_best = stats_session_best.get_untracked();
                        if current_best.is_none() || metric < current_best.unwrap() {
                            stats_session_best.set(Some(metric));
                        }

                        stats_trend.set(profile.borrow().trend(training_discipline));

                        // Show trial summary
                        trial_hit_rate.set(Some(trial.hit_rate()));
                        trial_mean_offset.set(Some(trial.mean_offset_ms()));
                        trial_summary_visible.set(true);

                        // Auto-hide summary after a delay (non-blocking — sequencer continues)
                        spawn_local_scoped_with_cancellation(async move {
                            TimeoutFuture::new(TRIAL_SUMMARY_DURATION_MS).await;
                            trial_summary_visible.set(false);
                        });
                    }
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
            }
        });
    }

    let fill_the_gap_title = move_tr!("fill-the-gap-title");
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

    // Tap button handler — fires on pointerdown for lowest latency
    #[allow(clippy::redundant_closure)]
    let on_tap_for_button = {
        let on_tap = Rc::clone(&on_tap);
        SendWrapper::new(move |ev: web_sys::PointerEvent| {
            ev.prevent_default();
            on_tap(ev.time_stamp());
        })
    };

    view! {
        <div class="flex flex-col pt-4 pb-12 h-screen">
            <NavBar title=fill_the_gap_title back_href=base_href("/") on_back=on_back_cb pill_group=true>
                <NavIconButton label="Help".to_string() icon="?".to_string() on_click=on_help_cb circled=true />
                <NavIconButton label="Settings".to_string() icon="\u{2699}\u{FE0F}".to_string() href=base_href("/settings") />
                <NavIconButton label="Profile".to_string() icon="\u{1F4CA}".to_string() href=base_href("/profile") />
            </NavBar>
            <HelpModal title=move_tr!("fill-the-gap-help-title") sections=CONTINUOUS_RHYTHM_MATCHING_HELP is_open=is_help_open on_close=on_help_close />

            <AudioGateOverlay />

            // Tempo display
            <div class="text-center text-sm text-gray-500 dark:text-gray-400 mt-2">
                {tempo_label}
            </div>

            // Training stats + timing feedback
            <div class="flex items-start justify-between mb-2 px-4">
                // Left: Training stats
                <TrainingStats
                    latest_value=latest_difficulty.into()
                    session_best=stats_session_best.into()
                    trend=stats_trend.into()
                    discipline=training_discipline
                />
                // Right: timing feedback indicator
                <div class="text-right" aria-hidden="true">
                    {move || {
                        if tap_feedback_visible.get() {
                            if let Some(ms) = tap_offset_ms.get() {
                                let abs_ms = ms.abs();
                                let pct_of_sixteenth = domain::types::RhythmOffset::new(ms).percentage_of_sixteenth(tempo);
                                let color = if pct_of_sixteenth <= GREEN_THRESHOLD_PCT {
                                    "text-green-600 dark:text-green-400"
                                } else if pct_of_sixteenth <= YELLOW_THRESHOLD_PCT {
                                    "text-yellow-600 dark:text-yellow-400"
                                } else {
                                    "text-red-600 dark:text-red-400"
                                };
                                let direction = if ms < 0.0 { "\u{2190}" } else if ms > 0.0 { "\u{2192}" } else { "" };
                                view! {
                                    <div class=format!("flex items-center gap-1 text-lg font-semibold {color}")>
                                        <span>{direction}</span>
                                        <span>{format!("{:.0} ms", abs_ms)}</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div class="w-8 h-8"></div> }.into_any()
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

            // 4-dot visual metronome
            <div class="flex justify-center items-center gap-6 my-8" aria-hidden="true">
                {(0..4usize).map(|i| {
                    let is_accent = i == 0;
                    view! {
                        <div class=move || {
                            let active = active_dot.get() == Some(i);
                            let is_gap = gap_dot.get() == Some(i);
                            let size = if is_accent { "w-10 h-10" } else { "w-7 h-7" };
                            if is_gap {
                                // Gap dot: outline circle (empty)
                                let border_color = if active {
                                    "border-blue-500 dark:border-blue-400 bg-blue-500/20"
                                } else {
                                    "border-gray-400 dark:border-gray-500 bg-transparent"
                                };
                                format!("{size} rounded-full border-3 transition-colors duration-75 {border_color}")
                            } else {
                                // Normal dot: filled
                                let color = if active {
                                    "bg-blue-500 dark:bg-blue-400"
                                } else {
                                    "bg-gray-300 dark:bg-gray-600"
                                };
                                format!("{size} rounded-full transition-colors duration-75 {color}")
                            }
                        } />
                    }
                }).collect_view()}
            </div>

            // Cycle counter (progress within trial)
            <div class="text-center text-xs text-gray-400 dark:text-gray-500 mb-4">
                {move || {
                    let current = cycle_counter.get();
                    let total = CYCLES_PER_TRIAL;
                    format!("{current}/{total}")
                }}
            </div>

            // Trial summary overlay (non-blocking)
            {move || {
                if trial_summary_visible.get() {
                    let hit_rate = trial_hit_rate.get().unwrap_or(0.0);
                    let hits = (hit_rate * CYCLES_PER_TRIAL as f64).round() as u16;
                    let mean_ms = trial_mean_offset.get().unwrap_or(0.0);
                    view! {
                        <div class="text-center py-2 px-4 mx-auto rounded-lg bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800 max-w-xs">
                            <div class="text-sm font-medium text-blue-700 dark:text-blue-300">
                                {tr!("hit-rate-label", {"hits" => hits.to_string(), "total" => CYCLES_PER_TRIAL.to_string()})}
                            </div>
                            <div class="text-xs text-blue-600 dark:text-blue-400">
                                {tr!("mean-offset-label", {"value" => format!("{:.0}", mean_ms.abs())})}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Large tap button — fires on pointerdown for lowest latency
            <div class="flex-1 flex items-center justify-center px-4 mt-4">
                <button
                    on:pointerdown=move |ev: web_sys::PointerEvent| on_tap_for_button(ev)
                    class="w-full h-full min-h-32 max-h-64 rounded-2xl bg-blue-500 text-white text-2xl font-semibold shadow-md hover:bg-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:ring-offset-2 active:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-500 dark:active:bg-blue-700 dark:focus:ring-offset-gray-900 select-none touch-manipulation"
                    aria-label=move || tr!("tap")
                >
                    {move_tr!("tap")}
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
