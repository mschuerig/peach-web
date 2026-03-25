# Story 21.3: Output Latency Compensation in Tap Evaluation

Status: ready-for-dev

## Story

As a user,
I want my tap accuracy to be evaluated relative to when I heard the beat, not when it was scheduled,
so that my timing scores are accurate even when using external speakers or headphones.

## Context

Prerequisite: Story 21.1 (latencyHint). Story 21.2 is independent but the FFI module from 21.2 can be shared.

Research: `docs/planning-artifacts/research/domain-web-audio-tap-latency-research-2026-03-25.md` — Tier 2b.

When the scheduler plays a beat at `scheduled_time` on the audio clock, the user hears it `outputLatency` seconds later. On wired output this is 10-25ms (negligible), but on USB audio interfaces or wired headphones with DAC processing it can be 30-50ms — enough to skew offset measurement.

The fix is simple: when evaluating whether a tap was early or late, compare the tap time against `scheduled_time + outputLatency` rather than `scheduled_time` alone.

`AudioContext.outputLatency` is not in `web_sys` — manual `#[wasm_bindgen]` FFI is required (can share the module created in story 21.2).

## Acceptance Criteria

1. **AC1 — FFI binding:** A `#[wasm_bindgen]` extern block provides `output_latency` getter on `AudioContext`, returning `f64`.

2. **AC2 — Helper function:** A `get_output_latency(ctx) -> f64` function returns `outputLatency` or `0.0` if unsupported (NaN check).

3. **AC3 — Domain function updated:** `evaluate_tap()` in `domain/src/training/rhythm_offset_detection.rs` accepts an additional `output_latency_secs: f64` parameter. The offset calculation uses `nearest_time + output_latency_secs` as the comparison point.

4. **AC4 — Callers updated:** All callers of `evaluate_tap()` pass the output latency value. The web layer reads `AudioContext.outputLatency` and passes it in. Domain tests pass `0.0` for the parameter.

5. **AC5 — Domain stays pure:** The domain crate has no `web_sys` dependency. The output latency is passed as a plain `f64`.

6. **AC6 — Existing tests updated:** All existing `evaluate_tap` tests continue to pass (with `0.0` for the new parameter). New test cases verify that a non-zero `output_latency_secs` shifts the offset by the expected amount.

7. **AC7 — No regression:** `cargo test -p domain` passes. `cargo clippy --workspace` clean. `trunk build` succeeds.

## Tasks / Subtasks

- [ ] Task 1: Add `output_latency` FFI binding to `web/src/adapters/audio_latency.rs` (create module if 21.2 not done yet, or extend it)
- [ ] Task 2: Add `output_latency_secs: f64` parameter to `evaluate_tap()` in `domain/src/training/rhythm_offset_detection.rs`
- [ ] Task 3: Update offset calculation: `let offset_ms = (tap_time - (nearest_time + output_latency_secs)) * 1000.0;`
- [ ] Task 4: Update all existing test calls to pass `0.0` for the new parameter
- [ ] Task 5: Add new test: `evaluate_tap` with `output_latency_secs = 0.050` — verify offset shifts by -50ms
- [ ] Task 6: Update callers in `continuous_rhythm_matching_view.rs` and `rhythm_offset_detection_view.rs` to read and pass `outputLatency`
- [ ] Task 7: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `trunk build`

## Dev Notes

- `outputLatency` is available in Chrome 64+ and Firefox 70+. Not available in Safari — feature-detect via NaN check, fall back to 0.0
- Some audio drivers report incorrect values (known Windows `GetStreamLatency` issue). On macOS/iOS this is reliable.
- `outputLatency` can change over time (e.g., if the OS resizes audio buffers) — re-read it per evaluation, don't cache
- The domain function signature change is a breaking API change — update both training views (continuous rhythm matching and rhythm offset detection)
- The `handle_tap()` method in `ContinuousRhythmMatchingSession` calls `evaluate_tap` — it also needs the output latency parameter threaded through
