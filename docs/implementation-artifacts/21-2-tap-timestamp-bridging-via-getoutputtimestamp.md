# Story 21.2: Tap Timestamp Bridging via getOutputTimestamp

Status: ready-for-dev

## Story

As a developer,
I want the tap handler to convert `PointerEvent.timeStamp` to audio clock time using `getOutputTimestamp()`,
so that tap offset measurement is accurate regardless of main-thread delivery delay.

## Context

Prerequisite: Story 21.1 (latencyHint and overlap suppression).

Research: `docs/planning-artifacts/research/domain-web-audio-tap-latency-research-2026-03-25.md` — Tier 2a.

The current tap handler reads `ctx.current_time()` at the moment the `pointerdown` handler executes. But the physical touch happened 10-30ms earlier (touch digitizer + OS pipeline + browser event dispatch). This introduces a systematic late bias in offset measurement.

`AudioContext.getOutputTimestamp()` returns `{contextTime, performanceTime}` — a mapping between the audio clock and `performance.now()`. Since `PointerEvent.timeStamp` is in `performance.now()` coordinates, we can bridge:

```
tap_audio_time = ots.contextTime + (event.timeStamp - ots.performanceTime) / 1000.0
```

This recovers the audio-clock-equivalent time of the physical touch.

`getOutputTimestamp()` is not in `web_sys` — manual `#[wasm_bindgen]` FFI is required.

## Acceptance Criteria

1. **AC1 — FFI binding:** A `#[wasm_bindgen]` extern block provides `get_output_timestamp()` on `AudioContext`, returning a `JsValue`.

2. **AC2 — Bridge function:** A `bridge_event_to_audio_time(ctx, event_timestamp_ms) -> Option<f64>` function converts a `PointerEvent.timeStamp` (ms) to audio clock time (seconds). Returns `None` if `getOutputTimestamp()` is unsupported.

3. **AC3 — Tap handler uses bridge:** The `on_tap` closure receives the event's `time_stamp()` and uses the bridge function. Falls back to `ctx.current_time()` when the bridge returns `None`.

4. **AC4 — Keyboard handler too:** The keyboard tap handler (`Space`/`Enter`) also passes `KeyboardEvent.time_stamp()` through the same bridge.

5. **AC5 — Feature detection:** Graceful fallback on browsers without `getOutputTimestamp()` (Safari). No panics, no errors logged.

6. **AC6 — No regression:** Existing flow works. `cargo test -p domain` passes. `cargo clippy --workspace` clean. `trunk build` succeeds.

## Tasks / Subtasks

- [ ] Task 1: Create `web/src/adapters/audio_latency.rs` module with `#[wasm_bindgen]` FFI for `getOutputTimestamp()` and the `bridge_event_to_audio_time()` function
- [ ] Task 2: Update `on_tap` closure in `continuous_rhythm_matching_view.rs` to accept an `event_timestamp_ms: f64` parameter
- [ ] Task 3: Update `pointerdown` handler to pass `event.time_stamp()` to `on_tap`
- [ ] Task 4: Update keyboard handler to pass `event.time_stamp()` to `on_tap`
- [ ] Task 5: Pass bridged tap time to `session.handle_tap()` instead of raw `current_time()`
- [ ] Task 6: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `trunk build`

## Dev Notes

- `PointerEvent.time_stamp()` and `KeyboardEvent.time_stamp()` are both `f64` in milliseconds (DOMHighResTimeStamp), available via `web_sys::Event::time_stamp()`
- The `on_tap` closure is currently `Rc<dyn Fn()>` — it needs to become `Rc<dyn Fn(f64)>` to accept the timestamp
- `getOutputTimestamp()` returns a JS object `{contextTime: number, performanceTime: number}` — extract with `js_sys::Reflect::get()`
- Browser support: Chrome 57+, Firefox 70+. Not available in Safari. Feature-detect by checking if `contextTime` is present and non-zero.
- The bridge math: `audio_time = contextTime + (eventTimestamp - performanceTime) / 1000.0` — note the ms→seconds conversion
- Consider also applying this bridge in the rhythm offset detection view (`rhythm_offset_detection_view.rs`) for consistency
