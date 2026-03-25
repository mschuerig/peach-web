use std::cell::RefCell;
use std::rc::Rc;

use domain::types::TempoBPM;
use gloo_timers::callback::Interval;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode};

/// A step in the rhythm pattern: either a click or silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RhythmStep {
    Play,
    Silent,
}

/// Mode of scheduler operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerMode {
    /// Play exactly one cycle of the pattern, then stop.
    SinglePass,
    /// Loop the pattern indefinitely until stopped.
    #[allow(dead_code)]
    Loop,
}

/// Scheduler configuration.
pub struct SchedulerConfig {
    pub pattern: Vec<RhythmStep>,
    pub tempo: TempoBPM,
    pub mode: SchedulerMode,
}

/// Lookahead interval in milliseconds (how often the scheduling loop runs).
const SCHEDULE_INTERVAL_MS: u32 = 25;

/// How far ahead to schedule events, in seconds.
const LOOKAHEAD_SECS: f64 = 0.100;

/// Accent gain for the first beat: +6dB ≈ 2.0x amplitude.
pub const ACCENT_GAIN: f32 = 2.0;

/// Normal (non-accented) gain.
pub const NORMAL_GAIN: f32 = 1.0;

/// Duration of the click buffer in seconds.
const CLICK_DURATION_SECS: f64 = 0.005;

/// Synthesize a short percussion click buffer (~5ms exponentially decaying noise burst).
pub fn create_click_buffer(ctx: &AudioContext) -> Result<AudioBuffer, String> {
    let sample_rate = ctx.sample_rate();
    let length = (sample_rate as f64 * CLICK_DURATION_SECS).ceil() as u32;

    let buffer = ctx
        .create_buffer(1, length, sample_rate)
        .map_err(|e| format!("Failed to create AudioBuffer: {:?}", e))?;

    // Fill with exponentially decaying white noise
    let mut data = vec![0f32; length as usize];
    // Use a simple deterministic pseudo-noise (alternating sign pattern mixed with decay)
    // For a crisp click, we use a noise-like burst that decays exponentially.
    let decay_rate = 5.0 / CLICK_DURATION_SECS; // decay to ~e^-5 ≈ 0.7% by end
    for (i, sample) in data.iter_mut().enumerate() {
        let t = i as f64 / sample_rate as f64;
        let envelope = (-decay_rate * t).exp() as f32;
        // Pseudo-random noise using a simple hash-like pattern
        let noise = pseudo_noise(i);
        *sample = noise * envelope;
    }

    buffer
        .copy_to_channel(&data, 0)
        .map_err(|e| format!("Failed to write click buffer data: {:?}", e))?;

    Ok(buffer)
}

/// Simple deterministic pseudo-noise in range [-1.0, 1.0].
fn pseudo_noise(index: usize) -> f32 {
    // Use a simple hash to get deterministic "noise"
    let mut x = index as u32;
    x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    x = (x >> 16) ^ x;
    x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    // Map to [-1.0, 1.0]
    (x as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// Internal mutable state for the scheduler.
struct SchedulerState {
    next_step_time: f64,
    current_step: usize,
    pattern: Vec<RhythmStep>,
    mode: SchedulerMode,
    sixteenth_secs: f64,
    cycle_times: Vec<f64>,
    stopped: bool,
}

/// A lookahead rhythm scheduler that uses the Web Audio clock for sample-accurate timing.
///
/// Uses the "two clocks" pattern: a main-thread `setInterval` (~25ms) looks ahead ~100ms
/// and schedules `AudioBufferSourceNode.start(when)` calls on the audio clock.
pub struct RhythmScheduler {
    state: Rc<RefCell<SchedulerState>>,
    ctx: Rc<RefCell<AudioContext>>,
    click_buffer: AudioBuffer,
    _interval: Option<Interval>,
}

impl RhythmScheduler {
    /// Create a new scheduler. Call `start()` to begin playback.
    ///
    /// # Panics
    /// Panics if the pattern is empty.
    pub fn new(
        ctx: Rc<RefCell<AudioContext>>,
        click_buffer: AudioBuffer,
        config: SchedulerConfig,
    ) -> Self {
        assert!(!config.pattern.is_empty(), "pattern must not be empty");
        let sixteenth_secs = config.tempo.sixteenth_note_duration_secs();

        let state = Rc::new(RefCell::new(SchedulerState {
            next_step_time: 0.0,
            current_step: 0,
            pattern: config.pattern,
            mode: config.mode,
            sixteenth_secs,
            cycle_times: Vec::new(),
            stopped: false,
        }));

        Self {
            state,
            ctx,
            click_buffer,
            _interval: None,
        }
    }

    /// Start the scheduler. Begins the lookahead loop.
    pub fn start(&mut self) {
        let current_time = self.ctx.borrow().current_time();
        {
            let mut state = self.state.borrow_mut();
            state.next_step_time = current_time + 0.050; // small lead-in
            state.current_step = 0;
            state.cycle_times.clear();
            state.stopped = false;
        }

        let state = Rc::clone(&self.state);
        let ctx = Rc::clone(&self.ctx);
        let buffer = self.click_buffer.clone();

        let interval = Interval::new(SCHEDULE_INTERVAL_MS, move || {
            schedule_ahead(&ctx, &buffer, &state);
        });

        self._interval = Some(interval);
    }
}

/// The core scheduling function called by the interval timer.
fn schedule_ahead(
    ctx: &Rc<RefCell<AudioContext>>,
    click_buffer: &AudioBuffer,
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
            let gain = if is_first_step {
                ACCENT_GAIN
            } else {
                NORMAL_GAIN
            };
            let _ = schedule_click(ctx, click_buffer, next_time, gain);
            state.borrow_mut().cycle_times.push(next_time);
        }

        // Advance to next step
        let next_step = step_index + 1;

        if next_step >= pattern_len {
            // End of cycle
            let mode = state.borrow().mode;
            match mode {
                SchedulerMode::SinglePass => {
                    state.borrow_mut().stopped = true;
                    return;
                }
                SchedulerMode::Loop => {
                    let mut s = state.borrow_mut();
                    s.current_step = 0;
                    s.next_step_time = next_time + sixteenth;
                    s.cycle_times.clear();
                }
            }
        } else {
            let mut s = state.borrow_mut();
            s.current_step = next_step;
            s.next_step_time = next_time + sixteenth;
        }
    }
}

/// Schedule a single click at the given audio-clock time with the specified gain.
///
/// Public so that training views can schedule individual clicks outside the scheduler's
/// pattern (e.g. the offset click in rhythm offset detection).
pub fn play_click_at(
    ctx: &Rc<RefCell<AudioContext>>,
    buffer: &AudioBuffer,
    when: f64,
    gain_value: f32,
) -> Result<(), String> {
    schedule_click(ctx, buffer, when, gain_value)
}

/// Internal click-scheduling used by the lookahead loop.
fn schedule_click(
    ctx: &Rc<RefCell<AudioContext>>,
    buffer: &AudioBuffer,
    when: f64,
    gain_value: f32,
) -> Result<(), String> {
    let ctx_ref = ctx.borrow();

    let source: AudioBufferSourceNode = ctx_ref
        .create_buffer_source()
        .map_err(|e| format!("create_buffer_source failed: {:?}", e))?;
    source.set_buffer(Some(buffer));

    let gain_node: GainNode = ctx_ref
        .create_gain()
        .map_err(|e| format!("create_gain failed: {:?}", e))?;
    gain_node.gain().set_value(gain_value);

    source
        .connect_with_audio_node(&gain_node)
        .map_err(|e| format!("source→gain connect failed: {:?}", e))?;
    gain_node
        .connect_with_audio_node(&ctx_ref.destination())
        .map_err(|e| format!("gain→destination connect failed: {:?}", e))?;

    source
        .start_with_when(when)
        .map_err(|e| format!("start_with_when failed: {:?}", e))?;

    Ok(())
}
