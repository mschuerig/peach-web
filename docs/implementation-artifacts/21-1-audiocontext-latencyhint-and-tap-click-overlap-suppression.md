# Story 21.1: AudioContext latencyHint and Tap Click Overlap Suppression

Status: done

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

2. **AC2 — Overlap suppression:** ~~When a user taps within ±15ms of a non-gap scheduled beat, the tap click is suppressed (not played).~~ **Descoped** — event dispatch latency (30-90ms) makes `currentTime`-based suppression unreliable. Requires bridged timestamps from Story 21.2 to work correctly.

3. **AC3 — Gap beats not suppressed:** Descoped with AC2.

4. **AC4 — Beat times shared to tap handler:** Descoped with AC2.

5. **AC5 — No regression:** Existing rhythm training flow works end-to-end. `cargo test -p domain` passes. `cargo clippy --workspace` clean. `trunk build` succeeds.

## Tasks / Subtasks

- [x] Task 1: Add `AudioContextOptions` with `latency_hint_f64(0.0)` to `AudioContextManager::get_or_create()` in `web/src/adapters/audio_context.rs`
- [x] Task 2: Enable `AudioContextOptions` feature in `web/Cargo.toml`
- [N/A] Task 3: Share `beat_times` and `gap_index` — descoped (overlap suppression removed)
- [N/A] Task 4: Add overlap check in tap handler — descoped (overlap suppression removed)
- [N/A] Task 5: Verify doubling is eliminated — descoped (overlap suppression removed)
- [x] Task 6: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `trunk build`

## Dev Notes

- `AudioContextOptions::new()` + `opts.set_latency_hint(&JsValue::from(0.0))` + `AudioContext::new_with_context_options(&opts)` — stable in `web_sys`
- Overlap suppression was implemented and tested but removed: event dispatch latency causes `currentTime` in the tap handler to lag the physical tap by 30-90ms, making a `currentTime`-vs-beat-time comparison unreliable. Proper suppression requires the bridged `PointerEvent.timeStamp` from Story 21.2.

## Dev Agent Record

### Implementation Plan

1. **latencyHint (AC1):** Replace `AudioContext::new()` with `AudioContext::new_with_context_options(&opts)` where opts has `latency_hint` set to `0.0` via `set_latency_hint(&JsValue::from(0.0))`. Log `baseLatency` via `js_sys::Reflect` since web-sys doesn't expose it natively.

### Debug Log

- `web_sys::AudioContextOptions::latency_hint()` is deprecated — used `set_latency_hint()` instead
- `web_sys::AudioContext` does not expose `base_latency()` — used `js_sys::Reflect::get()` with `"baseLatency"` key for logging
- Only `AudioContextOptions` feature needed in Cargo.toml (not `AudioContextLatencyCategory` since we use the f64 variant)
- Overlap suppression implemented then removed: on-device testing showed min distance of 30-90ms between `currentTime` and scheduled beat times, far exceeding the 15ms threshold. Root cause: event dispatch pipeline adds 30+ ms before JS handler runs.

### Completion Notes

- AC1: AudioContext now created with `latencyHint: 0.0` — baseLatency logged at creation via JS interop
- AC2-AC4: Descoped — overlap suppression requires bridged timestamps (Story 21.2)
- AC5: `cargo clippy --workspace` clean, `cargo test -p domain` all pass, `trunk build` succeeds

## File List

- `web/Cargo.toml` — added `AudioContextOptions` web-sys feature
- `web/src/adapters/audio_context.rs` — AudioContext created with latencyHint 0.0, baseLatency logged

## Change Log

- 2026-03-25: Implemented latencyHint (Story 21.1); overlap suppression descoped after on-device testing
