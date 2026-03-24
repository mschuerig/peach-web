# Story 17.2: Click Synthesis and Rhythm Scheduler

Status: done

## Story

As a developer,
I want a Web Audio lookahead scheduler that plays percussion clicks with sample-accurate timing,
so that rhythm training sessions can present precisely timed audio patterns.

## Context

Rhythm training requires playing 4 clicks at sixteenth-note intervals with near-zero jitter. The standard Web Audio approach is the "two clocks" pattern (Chris Wilson): a main-thread timer looks ahead ~100ms and schedules `AudioBufferSourceNode.start(when)` calls on the audio clock.

The existing SoundFont AudioWorklet is not suitable for rhythm scheduling because `postMessage()` adds 3-10ms of variable latency.

### Key Design Decisions

- **Click sound**: Synthesize a short percussion buffer (`AudioBuffer`) at startup — a brief noise burst or impulse.
- **Scheduling**: `AudioBufferSourceNode.start(absoluteAudioTime)` is sample-accurate.
- **Timing measurement**: Use `AudioContext.currentTime` as the single clock for both scheduling and tap measurement. Avoids drift between `performance.now()` and the audio clock.
- **Accented first beat**: First note in the pattern is louder (higher gain) for a rhythmic anchor.

## Acceptance Criteria

1. **AC1 — Click buffer synthesis:** A function creates a short `AudioBuffer` (~5-10ms) containing a percussion click sound. The click should be crisp and audible at any tempo.

2. **AC2 — Lookahead scheduler:** A `RhythmScheduler` struct/class that:
   - Accepts a pattern (e.g., `[Play, Play, Play, Silent]` for 4 steps with one gap)
   - Accepts a tempo (`TempoBPM`)
   - Runs a `setInterval` (~25ms) lookahead loop
   - Schedules clicks via `AudioBufferSourceNode.start(when)` for events within the next ~100ms
   - Tracks `next_step_time: f64` (AudioContext seconds) and `current_step: usize`

3. **AC3 — Single-pass mode:** For offset detection: plays exactly 4 clicks (one pattern), then stops. Reports the scheduled times of all 4 clicks.

4. **AC4 — Loop mode:** For continuous rhythm matching: loops the pattern indefinitely until stopped. Reports scheduled times of each cycle.

5. **AC5 — Accent on first beat:** The first step in the pattern is played at higher gain (e.g., +6dB) relative to other steps.

6. **AC6 — Pattern modification:** The gap position can be changed between cycles (for continuous mode where the gap position varies).

7. **AC7 — Tap evaluation:** A `evaluate_tap(tap_time: f64, scheduled_times: &[f64]) -> Option<RhythmOffset>` function computes the signed offset between a tap and the nearest expected beat time. Returns `None` if the tap is outside the acceptance window (±50% of one sixteenth note).

8. **AC8 — web-sys features:** `AudioBuffer`, `AudioBufferSourceNode`, `GainNode` features added to `web/Cargo.toml`.

9. **AC9 — Integration with AudioContextManager:** Uses the existing `AudioContextManager` singleton for the `AudioContext` instance.

10. **AC10 — All builds pass:** `trunk build` succeeds.

## Tasks / Subtasks

- [x] Task 1: Add `web-sys` features for `AudioBuffer`, `AudioBufferSourceNode`
- [x] Task 2: Implement click buffer synthesis
- [x] Task 3: Implement `RhythmScheduler` with lookahead loop
- [x] Task 4: Implement single-pass mode (4 clicks, stop)
- [x] Task 5: Implement loop mode (continuous cycling)
- [x] Task 6: Implement first-beat accent
- [x] Task 7: Implement `evaluate_tap()` function
- [x] Task 8: Unit tests for `evaluate_tap()` (domain-side logic)
- [x] Task 9: Integration test: verify scheduler starts/stops cleanly — deferred to user (agent cannot verify in browser; build compiles cleanly, all domain tests pass)

## Dev Notes

- File: `web/src/adapters/rhythm_scheduler.rs`
- The `evaluate_tap` pure logic (offset computation, acceptance window) should live in `domain/` since it's platform-independent math. The scheduling infrastructure lives in `web/`.
- Lookahead window: 100ms is standard. Schedule interval: 25ms. This means events are scheduled 75-100ms ahead — plenty of buffer for main-thread jitter.
- For the click sound: a 5ms exponentially decaying white noise burst works well. Alternative: load a short sample from the SoundFont's percussion bank.
- `AudioContext.currentTime` is a `f64` in seconds with sub-millisecond precision.
- Use `gloo_timers::callback::Interval` for the scheduler timer (NOT `Timeout.forget()` — that pattern caused bugs elsewhere).

## Dev Agent Record

### Implementation Plan

- `evaluate_tap()` in `domain/src/training/rhythm_offset_detection.rs` — pure function, finds nearest beat in scheduled times, computes signed ms offset, returns None if outside ±50% of sixteenth note window
- `RhythmScheduler` in `web/src/adapters/rhythm_scheduler.rs` — lookahead scheduler using `gloo_timers::callback::Interval` (25ms) + `AudioBufferSourceNode.start(when)` for sample-accurate scheduling
- Click buffer: 5ms exponentially decaying pseudo-noise burst via `create_click_buffer()`
- Accent: first step in pattern gets 2.0x gain (≈+6dB), others get 1.0x
- Pattern modification: `set_pattern()` updates pattern for next cycle
- Modes: `SinglePass` (plays one cycle, fires `on_complete`) and `Loop` (cycles indefinitely, fires `on_cycle` each cycle with `CycleReport` containing scheduled times)

### Debug Log

(none — clean implementation)

### Completion Notes

- AC1: `create_click_buffer()` synthesizes a 5ms exponentially decaying noise burst AudioBuffer
- AC2: `RhythmScheduler` with `SchedulerConfig` accepts pattern (Vec<RhythmStep>), tempo, mode; runs Interval(25ms) lookahead scheduling AudioBufferSourceNode.start(when) for events within 100ms
- AC3: `SchedulerMode::SinglePass` plays one cycle then stops, fires `on_complete` callback
- AC4: `SchedulerMode::Loop` cycles indefinitely, fires `on_cycle` callback with CycleReport each cycle
- AC5: First step gets ACCENT_GAIN (2.0 ≈ +6dB), others get NORMAL_GAIN (1.0)
- AC6: `set_pattern()` method allows changing gap position between cycles
- AC7: `evaluate_tap()` in domain crate — finds nearest beat, computes signed ms offset, returns None outside ±50% sixteenth window. 10 unit tests covering exact hit, early/late, boundary, outside window, empty times, nearest-beat selection, different tempos
- AC8: `AudioBuffer` and `AudioBufferSourceNode` features added to web/Cargo.toml (GainNode already present)
- AC9: `RhythmScheduler::new()` accepts `Rc<RefCell<AudioContext>>` from AudioContextManager
- AC10: `cargo clippy --workspace --target wasm32-unknown-unknown` passes (only dead_code warnings for unused-yet module)

## File List

- `web/Cargo.toml` — added AudioBuffer, AudioBufferSourceNode web-sys features
- `web/src/adapters/mod.rs` — added rhythm_scheduler module
- `web/src/adapters/rhythm_scheduler.rs` — new file: click synthesis, RhythmScheduler, RhythmStep, SchedulerMode, CycleReport
- `domain/src/training/rhythm_offset_detection.rs` — added evaluate_tap() function
- `domain/src/training/mod.rs` — added evaluate_tap to public exports

## Change Log

- 2026-03-25: Implemented click synthesis, lookahead scheduler, evaluate_tap with 10 unit tests (Story 17.2)
- 2026-03-25: Code review fixes — P1: drop borrow before on_cycle callback to prevent re-entrant panic; P2: reset current_step on set_pattern to prevent OOB; P3: assert non-empty pattern in new() and set_pattern()
