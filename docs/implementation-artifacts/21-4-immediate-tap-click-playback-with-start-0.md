# Story 21.4: Immediate Tap Click Playback with start(0)

Status: review

## Story

As a user,
I want the tap click to play at the earliest possible moment,
so that I get the most immediate audible feedback when I tap.

## Context

Prerequisite: Story 21.1 (overlap suppression modifies the same code path).

Research: `docs/planning-artifacts/research/domain-web-audio-tap-latency-research-2026-03-25.md` — Implementation section.

The current `play_click_at()` reads `ctx.current_time()` and passes it to `source.start_with_when(when)`. If `currentTime` advances between the read and the `start()` call (even by one render quantum ~2.67ms), the browser may defer playback to the next quantum. Calling `source.start()` with no argument (equivalent to `start(0)`) tells the browser to play at the earliest possible moment.

However, the scheduler's lookahead-scheduled beats must continue using `start_with_when(when)` for sample-accurate timing. This requires splitting the function into two variants.

## Acceptance Criteria

1. **AC1 — Immediate playback function:** A new `play_click_immediate(ctx, buffer, gain_value)` function calls `source.start()` (no `when` argument) for reactive tap clicks.

2. **AC2 — Scheduled playback preserved:** The existing `schedule_click(ctx, buffer, when, gain_value)` function (used by the scheduler lookahead loop) continues to use `start_with_when(when)`.

3. **AC3 — Tap handler updated:** The tap click in `continuous_rhythm_matching_view.rs` uses `play_click_immediate` instead of `play_click_at`.

4. **AC4 — Public API clean:** `play_click_at` is either removed (if no external callers need the `when` parameter for tap clicks) or renamed to `schedule_click_at` for clarity. The rhythm offset detection view's tap click also uses the immediate variant.

5. **AC5 — No regression:** Scheduler beat clicks still play with sample-accurate timing. `cargo test -p domain` passes. `cargo clippy --workspace` clean. `trunk build` succeeds.

## Tasks / Subtasks

- [x] Task 1: Add `play_click_immediate()` to `web/src/adapters/rhythm_scheduler.rs` — same as `schedule_click` but calls `source.start()` instead of `source.start_with_when(when)`
- [x] Task 2: Update tap click call in `continuous_rhythm_matching_view.rs` to use `play_click_immediate`
- [x] Task 3: Update tap click call in `rhythm_offset_detection_view.rs` (if applicable) to use `play_click_immediate`
- [x] Task 4: Rename `play_click_at` to `schedule_click_at` for clarity (or remove if unused after changes)
- [x] Task 5: Verify scheduler's lookahead beats still use `start_with_when(when)`
- [x] Task 6: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `trunk build`

## Dev Notes

- `source.start()` in `web_sys` is the zero-argument version. `source.start_with_when(when)` is the one-argument version. Both are available.
- The latency difference is marginal (at most one render quantum, ~2.67ms at 48kHz) but it's a trivial change that eliminates an edge case
- This story is the smallest in the epic — consider bundling with 21.1 if the developer prefers, but it's cleanly separable
- The rhythm offset detection view (`rhythm_offset_detection_view.rs`) may also play a tap click — check if it uses `play_click_at` and update accordingly

## Dev Agent Record

### Implementation Plan

Split `play_click_at` into two distinct public functions:
- `play_click_immediate()` — uses `source.start()` for reactive tap clicks (no `when` arg)
- `schedule_click_at()` — uses `source.start_with_when(when)` for pre-scheduled clicks

### Debug Log

No issues encountered. Straightforward API split.

### Completion Notes

- Added `play_click_immediate()` to `rhythm_scheduler.rs` — creates source/gain nodes and calls `source.start()` with no argument for earliest-possible playback
- Renamed `play_click_at` to `schedule_click_at` — delegates to internal `schedule_click` with `start_with_when(when)`
- Updated `continuous_rhythm_matching_view.rs` tap handler to use `play_click_immediate` (removed unnecessary `ctx.current_time()` read)
- Updated `rhythm_offset_detection_view.rs` offset click to use `schedule_click_at` (this is a pre-scheduled click at `beat_times[2]`, not a reactive tap)
- Scheduler lookahead loop unchanged — still uses internal `schedule_click` with `start_with_when(when)`
- All checks pass: `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain` (4 tests), `trunk build`

## File List

- `web/src/adapters/rhythm_scheduler.rs` — Added `play_click_immediate()`, renamed `play_click_at` to `schedule_click_at`
- `web/src/components/continuous_rhythm_matching_view.rs` — Import and call site updated to `play_click_immediate`
- `web/src/components/rhythm_offset_detection_view.rs` — Import and call site updated to `schedule_click_at`

## Change Log

- 2026-03-26: Implemented story 21.4 — split click playback API into immediate (tap) and scheduled (lookahead/offset) variants
