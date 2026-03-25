# Story 21.1: AudioContext latencyHint and Tap Click Overlap Suppression

Status: ready-for-dev

## Story

As a user,
I want the audio engine to use minimum-latency settings and not double-play clicks when my tap coincides with a scheduled beat,
so that rhythm training feels responsive and sounds clean.

## Context

Prerequisite: Epic 18 (continuous rhythm matching exists).

Research: `docs/planning-artifacts/research/domain-web-audio-tap-latency-research-2026-03-25.md` — Tier 1 quick wins.

Two independent issues bundled because both are small, high-impact changes:

1. **latencyHint:** The `AudioContext` is currently created with `AudioContext::new()` (no options). Passing `latencyHint: 0` (float) reduces the browser's audio buffer size to the minimum, measurably lowering baseLatency by 5-20ms. Despite the spec saying `"interactive"` should already provide lowest latency, `0` outperforms it in both Chrome and Firefox.

2. **Overlap suppression:** At slow tempos (e.g., 80 BPM, sixteenth = 187.5ms), when a user taps close in time to a non-gap scheduled beat, both the tap click and the scheduler's click play near-simultaneously, creating a "jittery doubling" artifact. Suppressing the tap click when within ±15ms of a non-gap beat eliminates this.

## Acceptance Criteria

1. **AC1 — latencyHint set:** `AudioContext` is created with `AudioContextOptions` and `latency_hint_f64(0.0)`. Verified by logging `ctx.baseLatency` (via JS interop if needed) at creation time.

2. **AC2 — Overlap suppression:** When a user taps within ±15ms of a non-gap scheduled beat, the tap click is suppressed (not played). The scheduled beat's click provides the audible feedback instead.

3. **AC3 — Gap beats not suppressed:** Taps near the gap beat position are never suppressed — the gap beat has no scheduled click, so the tap click must always play for gap positions.

4. **AC4 — Beat times shared to tap handler:** The current cycle's `beat_times` and `gap_index` are accessible to the tap handler via shared state (e.g., `Rc<Cell<>>` following the `shared_click_buffer` pattern).

5. **AC5 — No regression:** Existing rhythm training flow works end-to-end. `cargo test -p domain` passes. `cargo clippy --workspace` clean. `trunk build` succeeds.

## Tasks / Subtasks

- [ ] Task 1: Add `AudioContextOptions` with `latency_hint_f64(0.0)` to `AudioContextManager::get_or_create()` in `web/src/adapters/audio_context.rs`
- [ ] Task 2: Enable `AudioContextOptions` and `AudioContextLatencyCategory` features in `web/Cargo.toml` if not already present
- [ ] Task 3: Share `beat_times` and `gap_index` with the tap handler — add `Rc<Cell<[f64; 4]>>` and `Rc<Cell<Option<usize>>>` following the `shared_click_buffer` pattern in `continuous_rhythm_matching_view.rs`
- [ ] Task 4: Add overlap check in tap handler before `play_click_at` — suppress if any non-gap beat is within 15ms of current time
- [ ] Task 5: Verify doubling is eliminated via manual test at 80 BPM
- [ ] Task 6: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `trunk build`

## Dev Notes

- The 15ms threshold is well below the smallest sixteenth note at max tempo (125ms at 120 BPM) — no risk of suppressing valid tap clicks
- `AudioContextOptions::new()` + `opts.set_latency_hint_f64(0.0)` + `AudioContext::new_with_context_options(&opts)` — all stable in `web_sys`
- Two sounds within 15ms are perceptually fused; suppressing one does not change the user's experience except by eliminating the doubling artifact
- The `shared_click_buffer` pattern at `continuous_rhythm_matching_view.rs:130-137` is the model for sharing beat_times/gap_index
