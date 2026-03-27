use std::cell::RefCell;
use std::rc::Rc;

use domain::types::TempoBPM;
use gloo_timers::callback::{Interval, Timeout};
use web_sys::AudioContext;

use crate::adapters::audio_soundfont::WorkletBridge;

/// A step in the rhythm pattern: either a click or silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RhythmStep {
    Play,
    Silent,
}

/// Scheduler configuration.
pub struct SchedulerConfig {
    pub pattern: Vec<RhythmStep>,
    pub tempo: TempoBPM,
}

/// Lookahead interval in milliseconds (how often the scheduling loop runs).
const SCHEDULE_INTERVAL_MS: u32 = 25;

/// How far ahead to schedule events, in seconds.
const LOOKAHEAD_SECS: f64 = 0.100;

/// MIDI percussion channel (General MIDI channel 10, zero-indexed = 9).
const PERCUSSION_CHANNEL: u8 = 9;

/// MIDI bank number for percussion (bank 128).
const PERCUSSION_BANK: u32 = 128;

/// First percussion program.
const PERCUSSION_PROGRAM: u8 = 0;

/// General MIDI high woodblock note number.
const WOODBLOCK_NOTE: u8 = 76;

/// Velocity for accented (first) beat.
const ACCENT_VELOCITY: u8 = 127;

/// Velocity for normal (non-accented) beats.
const NORMAL_VELOCITY: u8 = 80;

/// Delay in milliseconds before sending noteOff after noteOn.
const NOTE_OFF_DELAY_MS: u32 = 50;

/// Select the percussion program on the percussion channel.
///
/// Call once per training session before creating schedulers. Sends a
/// `selectProgram` message to the OxiSynth worklet for bank 128 / program 0
/// on MIDI channel 9.
pub fn select_percussion_program(bridge: &Rc<RefCell<WorkletBridge>>) {
    let _ = bridge.borrow().send_select_program_ch(
        PERCUSSION_CHANNEL,
        PERCUSSION_BANK,
        PERCUSSION_PROGRAM,
    );
}

/// Internal mutable state for the scheduler.
struct SchedulerState {
    next_step_time: f64,
    current_step: usize,
    pattern: Vec<RhythmStep>,
    sixteenth_secs: f64,
    cycle_times: Vec<f64>,
    stopped: bool,
}

/// A lookahead rhythm scheduler that uses the Web Audio clock for sample-accurate timing.
///
/// Uses the "two clocks" pattern: a main-thread `setInterval` (~25ms) looks ahead ~100ms
/// and schedules percussion noteOn/noteOff events via the OxiSynth worklet bridge.
pub struct RhythmScheduler {
    state: Rc<RefCell<SchedulerState>>,
    ctx: Rc<RefCell<AudioContext>>,
    bridge: Rc<RefCell<WorkletBridge>>,
    _interval: Option<Interval>,
}

impl RhythmScheduler {
    /// Create a new scheduler. Call `start()` to begin playback.
    ///
    /// # Panics
    /// Panics if the pattern is empty.
    pub fn new(
        ctx: Rc<RefCell<AudioContext>>,
        bridge: Rc<RefCell<WorkletBridge>>,
        config: SchedulerConfig,
    ) -> Self {
        assert!(!config.pattern.is_empty(), "pattern must not be empty");
        let sixteenth_secs = config.tempo.sixteenth_note_duration_secs();

        let state = Rc::new(RefCell::new(SchedulerState {
            next_step_time: 0.0,
            current_step: 0,
            pattern: config.pattern,
            sixteenth_secs,
            cycle_times: Vec::new(),
            stopped: false,
        }));

        Self {
            state,
            ctx,
            bridge,
            _interval: None,
        }
    }

    /// Start the scheduler. Begins the lookahead loop.
    ///
    /// # Panics
    /// Panics if the `AudioContext` is not in the `running` state. Callers must
    /// resume the context before starting the scheduler — a suspended context
    /// has `currentTime` frozen at 0, which causes all steps to be scheduled
    /// in a single burst.
    pub fn start(&mut self) {
        let ctx_ref = self.ctx.borrow();
        assert!(
            ctx_ref.state() == web_sys::AudioContextState::Running,
            "RhythmScheduler::start() called with AudioContext in {:?} state — \
             resume the context first to avoid click burst",
            ctx_ref.state()
        );
        let current_time = ctx_ref.current_time();
        drop(ctx_ref);
        {
            let mut state = self.state.borrow_mut();
            state.next_step_time = current_time + 0.050; // small lead-in
            state.current_step = 0;
            state.cycle_times.clear();
            state.stopped = false;
        }

        let state = Rc::clone(&self.state);
        let ctx = Rc::clone(&self.ctx);
        let bridge = Rc::clone(&self.bridge);

        let interval = Interval::new(SCHEDULE_INTERVAL_MS, move || {
            schedule_ahead(&ctx, &bridge, &state);
        });

        self._interval = Some(interval);
    }
}

/// The core scheduling function called by the interval timer.
fn schedule_ahead(
    ctx: &Rc<RefCell<AudioContext>>,
    bridge: &Rc<RefCell<WorkletBridge>>,
    state: &Rc<RefCell<SchedulerState>>,
) {
    let current_time = ctx.borrow().current_time();
    let deadline = current_time + LOOKAHEAD_SECS;

    loop {
        let (next_time, step_index, step, pattern_len, sixteenth, is_first_step) = {
            let s = state.borrow();
            if s.stopped {
                return;
            }
            (
                s.next_step_time,
                s.current_step,
                s.pattern[s.current_step],
                s.pattern.len(),
                s.sixteenth_secs,
                s.current_step == 0,
            )
        };

        if next_time > deadline {
            break;
        }

        // Schedule the click if this step is Play
        if step == RhythmStep::Play {
            let accent = is_first_step;
            schedule_click_at(ctx, bridge, next_time, accent);
            state.borrow_mut().cycle_times.push(next_time);
        }

        // Advance to next step
        let next_step = step_index + 1;

        if next_step >= pattern_len {
            // End of cycle — single pass, stop.
            state.borrow_mut().stopped = true;
            return;
        } else {
            let mut s = state.borrow_mut();
            s.current_step = next_step;
            s.next_step_time = next_time + sixteenth;
        }
    }
}

/// Play a click as soon as possible (no delay).
///
/// Used for reactive tap clicks where minimal latency matters.
/// Sends noteOn immediately and schedules noteOff after `NOTE_OFF_DELAY_MS`.
pub fn play_click_immediate(bridge: &Rc<RefCell<WorkletBridge>>, accent: bool) {
    let velocity = if accent {
        ACCENT_VELOCITY
    } else {
        NORMAL_VELOCITY
    };
    let _ = bridge
        .borrow()
        .send_note_on_ch(PERCUSSION_CHANNEL, WOODBLOCK_NOTE, velocity);

    // Schedule noteOff
    let bridge_clone = Rc::clone(bridge);
    Timeout::new(NOTE_OFF_DELAY_MS, move || {
        let _ = bridge_clone
            .borrow()
            .send_note_off_ch(PERCUSSION_CHANNEL, WOODBLOCK_NOTE);
    })
    .forget();
}

/// Schedule a single click at the given audio-clock time with the specified accent.
///
/// Public so that training views can schedule individual clicks outside the scheduler's
/// pattern (e.g. the offset click in rhythm offset detection).
pub fn schedule_click_at(
    ctx: &Rc<RefCell<AudioContext>>,
    bridge: &Rc<RefCell<WorkletBridge>>,
    when: f64,
    accent: bool,
) {
    let current_time = ctx.borrow().current_time();
    let delay_secs = (when - current_time).max(0.0);
    let delay_ms = (delay_secs * 1000.0) as u32;

    let velocity = if accent {
        ACCENT_VELOCITY
    } else {
        NORMAL_VELOCITY
    };

    let bridge_clone = Rc::clone(bridge);
    Timeout::new(delay_ms, move || {
        let _ = bridge_clone
            .borrow()
            .send_note_on_ch(PERCUSSION_CHANNEL, WOODBLOCK_NOTE, velocity);

        // Schedule noteOff
        let bridge_off = Rc::clone(&bridge_clone);
        Timeout::new(NOTE_OFF_DELAY_MS, move || {
            let _ = bridge_off
                .borrow()
                .send_note_off_ch(PERCUSSION_CHANNEL, WOODBLOCK_NOTE);
        })
        .forget();
    })
    .forget();
}
