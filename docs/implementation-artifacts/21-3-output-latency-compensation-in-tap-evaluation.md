# Story 21.3: Output Latency Compensation in Tap Evaluation

Status: done

## Story

As a user,
I want my tap accuracy to be evaluated relative to when I heard the beat, not when it was scheduled,
so that my timing scores are accurate even when using external speakers or headphones.

## Context

Prerequisite: Story 21.1 (latencyHint). Story 21.2 is independent but the FFI module from 21.2 can be shared.

Research: `docs/planning-artifacts/research/domain-web-audio-tap-latency-research-2026-03-25.md` — Tier 2b.

When the scheduler plays a beat at `scheduled_time` on the audio clock, the user hears it `outputLatency` seconds later. On wired output this is 10-25ms (negligible), but on USB audio interfaces or wired headphones with DAC processing it can be 30-50ms — enough to skew offset measurement.

The fix is simple: when evaluating whether a tap was early or late, compare the tap time against `scheduled_time + outputLatency` rather than `scheduled_time` alone.

`AudioContext.outputLatency` is not in `web_sys` — read via `js_sys::Reflect` (same pattern as story 21.2).

## Acceptance Criteria

1. **AC1 — FFI access:** `outputLatency` is read from `AudioContext` via `js_sys::Reflect` (not in `web_sys`), returning `f64`.

2. **AC2 — Helper function:** A `get_output_latency(ctx) -> f64` function returns `outputLatency` or `0.0` if unsupported (NaN check).

3. **AC3 — Domain function updated:** `evaluate_tap()` in `domain/src/training/rhythm_offset_detection.rs` accepts an additional `output_latency_secs: f64` parameter. The offset calculation uses `nearest_time + output_latency_secs` as the comparison point.

4. **AC4 — Callers updated:** All callers of `evaluate_tap()` pass the output latency value. The web layer reads `AudioContext.outputLatency` and passes it in. Domain tests pass `0.0` for the parameter.

5. **AC5 — Domain stays pure:** The domain crate has no `web_sys` dependency. The output latency is passed as a plain `f64`.

6. **AC6 — Existing tests updated:** All existing `evaluate_tap` tests continue to pass (with `0.0` for the new parameter). New test cases verify that a non-zero `output_latency_secs` shifts the offset by the expected amount.

7. **AC7 — No regression:** `cargo test -p domain` passes. `cargo clippy --workspace` clean. `trunk build` succeeds.

## Tasks / Subtasks

- [x] Task 1: Add `output_latency` FFI binding to `web/src/adapters/audio_latency.rs` (create module if 21.2 not done yet, or extend it)
- [x] Task 2: Add `output_latency_secs: f64` parameter to `evaluate_tap()` in `domain/src/training/rhythm_offset_detection.rs`
- [x] Task 3: Update offset calculation: `let offset_ms = (tap_time - (nearest_time + output_latency_secs)) * 1000.0;`
- [x] Task 4: Update all existing test calls to pass `0.0` for the new parameter
- [x] Task 5: Add new test: `evaluate_tap` with `output_latency_secs = 0.050` — verify offset shifts by -50ms
- [x] Task 6: Update callers in `continuous_rhythm_matching_view.rs` to read and pass `outputLatency` (`rhythm_offset_detection_view.rs` does not call `evaluate_tap`)
- [x] Task 7: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `trunk build`

## Dev Notes

- `outputLatency` is available in Chrome 64+ and Firefox 70+. Not available in Safari — feature-detect via NaN check, fall back to 0.0
- Some audio drivers report incorrect values (known Windows `GetStreamLatency` issue). On macOS/iOS this is reliable.
- `outputLatency` can change over time (e.g., if the OS resizes audio buffers) — re-read it per evaluation, don't cache
- The domain function signature change is a breaking API change — update callers (only `continuous_rhythm_matching_view.rs`; `rhythm_offset_detection_view.rs` does not call `evaluate_tap`)
- The `handle_tap()` method in `ContinuousRhythmMatchingSession` calls `evaluate_tap` — it also needs the output latency parameter threaded through

## Dev Agent Record

### Implementation Plan

- Add `get_output_latency()` helper to `audio_latency.rs` using JS reflection (same pattern as `bridge_event_to_audio_time`)
- Add `output_latency_secs: f64` parameter to `evaluate_tap()` and thread through `handle_tap()` in `ContinuousRhythmMatchingSession`
- Update offset calculation: `heard_time = nearest_time + output_latency_secs`
- Update all existing test calls with `0.0`, add 3 new tests with 50ms latency
- Read `outputLatency` per-tap in the continuous rhythm matching view

### Debug Log

No issues encountered.

### Completion Notes

- AC1/AC2: `get_output_latency()` added to `web/src/adapters/audio_latency.rs` using `js_sys::Reflect` (consistent with existing `bridge_event_to_audio_time` pattern). Returns `0.0` for unsupported/NaN/negative values.
- AC3: `evaluate_tap()` now takes `output_latency_secs: f64`. Offset computed as `tap_time - (nearest_time + output_latency_secs)`.
- AC4: `handle_tap()` in `ContinuousRhythmMatchingSession` threads the parameter through. View reads `get_output_latency(&ctx_rc)` per tap. All domain tests pass `0.0`.
- AC5: Domain crate remains pure Rust — latency passed as plain `f64`.
- AC6: All 10 existing `evaluate_tap` tests updated with `0.0` and still pass. 3 new tests verify output latency shifts (perfect hit, early, late with 50ms latency).
- AC7: `cargo test -p domain` — 483 tests pass. `cargo clippy --workspace` — clean. `trunk build` — success.
- Note: `rhythm_offset_detection_view.rs` does not call `evaluate_tap` (different training mode — user identifies offset beats, no tap timing). Only `continuous_rhythm_matching_view.rs` was updated.

## File List

- `web/src/adapters/audio_latency.rs` — added `get_output_latency()` function
- `domain/src/training/rhythm_offset_detection.rs` — added `output_latency_secs` parameter to `evaluate_tap()`, updated offset calculation, updated all test calls, added 3 new tests
- `domain/src/session/continuous_rhythm_matching_session.rs` — added `output_latency_secs` parameter to `handle_tap()`, updated 3 test calls
- `web/src/components/continuous_rhythm_matching_view.rs` — imported `get_output_latency`, reads output latency per tap and passes to `handle_tap()`
- `docs/implementation-artifacts/21-3-output-latency-compensation-in-tap-evaluation.md` — updated status and tasks
- `docs/implementation-artifacts/sprint-status.yaml` — updated story status

## Change Log

- 2026-03-26: Implemented output latency compensation — `evaluate_tap()` and `handle_tap()` accept `output_latency_secs`, web layer reads `AudioContext.outputLatency` per tap, 3 new tests added.
