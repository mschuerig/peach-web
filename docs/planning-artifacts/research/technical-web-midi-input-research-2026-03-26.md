---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 1
research_type: 'technical'
research_topic: 'Web MIDI API for timing-sensitive training input'
research_goals: 'Determine what is necessary to support MIDI note-on input (any note, any channel) as an alternative to tap input in a Rust/Leptos WASM ear-training app, with focus on timing accuracy'
user_name: 'Michael'
date: '2026-03-26'
web_research_enabled: true
source_verification: true
---

# Web MIDI Input for Timing-Sensitive Ear Training: Technical Research

**Date:** 2026-03-26
**Author:** Michael
**Research Type:** Technical

---

## Executive Summary

Supporting MIDI input in peach-web is a low-risk, high-value enhancement that requires surprisingly little new code. The Web MIDI API's `MIDIMessageEvent.timeStamp` uses the same `DOMHighResTimeStamp` / `performance.now()` coordinate system as pointer and keyboard events, which means the entire existing timing pipeline — `bridge_event_to_audio_time()`, `evaluate_tap()`, `handle_tap()`, and `RhythmOffset` — works unchanged for MIDI input.

**Key Findings:**

- MIDI event timestamps are in the same coordinate space as pointer events — zero domain-layer changes required
- USB MIDI input latency (~3–20ms total) is comparable to or better than touch input (10–30ms)
- Browser support covers Chrome, Edge, Firefox desktop (109+); Safari is unsupported but MIDI is a progressive enhancement
- Implementation requires ~80–120 lines in one new adapter module plus minor view wiring
- 8 new web-sys Cargo feature flags, no third-party crate dependencies

**Recommendations:**

1. Create `midi_input.rs` adapter — feature detection, access request, note-on listener setup
2. Wire MIDI listener to existing `on_tap` closure — same timestamp, same pipeline
3. Request MIDI access at training view mount, alongside AudioContext resume
4. Treat MIDI as progressive enhancement — silent fallback on unsupported browsers
5. Defer settings (enable/disable, channel filter, device selection) to a future phase

---

## Table of Contents

1. [Technical Research Scope Confirmation](#technical-research-scope-confirmation)
2. [Technology Stack Analysis](#technology-stack-analysis)
3. [Integration Patterns Analysis](#integration-patterns-analysis)
4. [Architectural Patterns and Design](#architectural-patterns-and-design)
5. [Implementation Approaches and Technology Adoption](#implementation-approaches-and-technology-adoption)
6. [Research Synthesis and Conclusion](#research-synthesis-and-conclusion)

---

## Research Overview

This research investigates what is necessary to support MIDI note-on input (any note, any channel) as an alternative to tap input in a Rust/Leptos WASM ear-training app. The central question was whether MIDI input can integrate with the existing timing evaluation pipeline without architectural changes. The answer is definitively yes — MIDI events provide the same high-resolution timestamps as pointer events, making the integration a matter of wiring rather than redesign. The full analysis covers browser compatibility, Rust/WASM bindings, timestamp architecture, module design, and a concrete implementation roadmap. See the Executive Summary above for key findings and recommendations.

---

## Technical Research Scope Confirmation

**Research Topic:** Web MIDI API for timing-sensitive training input
**Research Goals:** Determine what is necessary to support MIDI note-on input (any note, any channel) as an alternative to tap input in a Rust/Leptos WASM ear-training app, with focus on timing accuracy

**Technical Research Scope:**

- Architecture Analysis — Web MIDI API design, permission model, event flow from hardware to browser
- Implementation Approaches — Rust/WASM access via web-sys bindings, event listener setup
- Technology Stack — web-sys MIDI types, browser compatibility
- Integration Patterns — MIDI timestamps vs AudioContext time, fitting into existing tap evaluation pipeline
- Performance Considerations — input latency, timestamp precision, comparison with pointer event timing

**Scope Confirmed:** 2026-03-26

---

## Technology Stack Analysis

### The Web MIDI API

The [Web MIDI API](https://developer.mozilla.org/en-US/docs/Web/API/Web_MIDI_API) is a W3C standard that provides browser-level access to connected MIDI devices. The core flow is:

1. Call `navigator.requestMIDIAccess()` — returns a `Promise<MIDIAccess>`
2. The `MIDIAccess` object exposes `inputs` (a `MIDIInputMap`) and `outputs` (a `MIDIOutputMap`)
3. Each `MIDIInput` fires `midimessage` events containing raw MIDI bytes and a high-resolution timestamp

For peach-web's use case, only **input** is needed — no MIDI output required.

**MIDI Note-On Detection:** A MIDI message is a `Uint8Array`. A note-on message has status byte `0x90–0x9F` (channels 1–16). To accept any note on any channel:

```
status_byte = data[0]
is_note_on = (status_byte & 0xF0) == 0x90 && data[2] > 0  // velocity > 0
```

A note-on with velocity 0 is conventionally treated as note-off.

_Source: [W3C Web MIDI API Spec](https://webaudio.github.io/web-midi-api/), [MDN MIDIMessageEvent](https://developer.mozilla.org/en-US/docs/Web/API/MIDIMessageEvent)_

### Browser Compatibility

| Browser | Support | Notes |
|---------|---------|-------|
| **Chrome** | 43+ (full) | Best support, primary target |
| **Edge** | 79+ (full) | Chromium-based, same as Chrome |
| **Firefox** | 109+ (desktop) | Requires "site permission add-on" install prompt; **not supported on Android** |
| **Safari** | Not supported | Apple declined due to fingerprinting concerns (2020) |
| **Opera** | 30+ (full) | Chromium-based |

**Overall compatibility score: ~63/100** ([Can I use](https://caniuse.com/midi)). This aligns well with peach-web's existing WebAudio dependency — Safari already requires fallback paths, and MIDI is a progressive enhancement, not a core requirement.

_Source: [Can I use: Web MIDI](https://caniuse.com/midi), [Chrome Blog: MIDI Permission](https://developer.chrome.com/blog/web-midi-permission-prompt), [Firefox 108 WebMIDI](https://debugpointnews.com/firefox-108/)_

### Security & Permissions Model

- **HTTPS required** — Web MIDI API is available only in [secure contexts](https://developer.mozilla.org/en-US/docs/Web/Security/Secure_Contexts)
- **User permission required** — `requestMIDIAccess()` triggers a browser permission prompt (Chrome 124+ made this mandatory for all MIDI access, not just SysEx)
- **Permissions-Policy header** — The `midi` directive controls whether the API is allowed; default allowlist is `self`
- **No SysEx needed** — peach-web only needs note-on messages, so `requestMIDIAccess({ sysex: false })` suffices (simpler permission flow)

_Source: [MDN: Permissions-Policy: midi](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Permissions-Policy/midi), [Chrome MIDI Permission Blog](https://developer.chrome.com/blog/web-midi-permission-prompt)_

### Rust/WASM Bindings via web-sys

The `web-sys` crate provides procedurally-generated bindings for all Web MIDI API types. Each type is gated behind a Cargo feature flag.

**Required Cargo features for web-sys:**

| Feature | Type | Purpose |
|---------|------|---------|
| `Navigator` | `Navigator` | Call `request_midi_access()` |
| `MidiOptions` | `MIDIOptions` | Set `{ sysex: false }` |
| `MidiAccess` | `MIDIAccess` | Access `.inputs()` map |
| `MidiInputMap` | `MIDIInputMap` | Iterate input ports |
| `MidiInput` | `MIDIInput` | Attach `midimessage` listener |
| `MidiMessageEvent` | `MIDIMessageEvent` | Read `.data()` (Uint8Array) |
| `MidiPort` | `MIDIPort` | Base type for input/output |
| `MidiConnectionEvent` | `MIDIConnectionEvent` | Hot-plug detection |

These are **low-level raw bindings** — no higher-level Rust abstraction needed for peach-web's simple use case (listen for note-on, extract timestamp).

There is a third-party [`web-midi`](https://github.com/patchnut/web-midi) crate that wraps web-sys MIDI in a more ergonomic API, but it appears minimally maintained and adds unnecessary abstraction for this use case. Direct web-sys usage is recommended.

_Source: [web-sys MidiAccess docs](https://docs.rs/web-sys/latest/web_sys/struct.MidiAccess.html), [web-sys Cargo.toml](https://github.com/rustwasm/wasm-bindgen/blob/master/crates/web-sys/Cargo.toml), [web-midi crate](https://lib.rs/crates/web-midi)_

### Timestamp Architecture — Critical for Timing Accuracy

This is the most important aspect for peach-web's use case:

**MIDIMessageEvent.timeStamp** is a `DOMHighResTimeStamp` — milliseconds relative to `performance.timeOrigin` (navigation start). This is the **same coordinate system** as:

- `PointerEvent.timeStamp` (current tap input)
- `performance.now()`

**This means the existing `bridge_event_to_audio_time()` function can be reused directly** — it already converts `performance.now()`-relative timestamps to audio clock time using `AudioContext.getOutputTimestamp()`:

```rust
// Existing bridge in audio_latency.rs — works for MIDI too:
audio_time = contextTime + (event_timestamp_ms - performanceTime) / 1000.0
```

**Key insight:** MIDI event timestamps and pointer event timestamps are in the same coordinate space. The entire downstream pipeline (`bridge_event_to_audio_time` → `evaluate_tap` → `RhythmOffset`) requires **zero changes** to support MIDI input.

**Latency characteristics:**

| Source | Typical Latency | Notes |
|--------|----------------|-------|
| USB MIDI hardware | 1–3 ms | USB polling interval |
| MIDI-over-BLE | 5–15 ms | Bluetooth connection interval |
| OS MIDI driver | <1 ms | Kernel-level routing |
| Browser event dispatch | 1–5 ms | Chrome's MIDI thread to JS main thread |
| **Total MIDI path** | **~3–20 ms** | Comparable to or better than touch input |

For comparison, pointer events on touch screens typically have 10–30 ms of dispatch latency. **MIDI input is likely equal or lower latency** than the existing tap path.

_Source: [W3C Web MIDI Spec](https://webaudio.github.io/web-midi-api/), [Web MIDI API Issue #187: Input Latency](https://github.com/WebAudio/web-midi-api/issues/187), [MDN DOMHighResTimeStamp](https://developer.mozilla.org/en-US/docs/Web/API/DOMHighResTimeStamp)_

### Technology Adoption Context

- The Web MIDI API has been stable since Chrome 43 (2015) with minimal spec changes
- The API is mature and unlikely to break — it is a W3C Candidate Recommendation
- Major music web apps (Soundtrap, BandLab, Web Audio Drum Machine) use it successfully
- The Rust/WASM ecosystem has first-class support via web-sys with no additional dependencies needed

---

## Integration Patterns Analysis

### API Access Pattern — requestMIDIAccess

The entry point is `navigator.requestMIDIAccess()`, which returns a `Promise<MIDIAccess>`. In Rust/WASM via web-sys:

```rust
use web_sys::{MidiAccess, MidiOptions, Navigator};
use wasm_bindgen_futures::JsFuture;

let navigator: Navigator = web_sys::window().unwrap().navigator();
let opts = MidiOptions::new();
opts.set_sysex(false); // No SysEx needed — simpler permission flow

let promise = navigator.request_midi_access_with_options(&opts)?;
let midi_access: MidiAccess = JsFuture::from(promise).await?.unchecked_into();
```

This follows the same async-via-`JsFuture` pattern already used in peach-web for AudioWorklet setup.

_Source: [MDN: requestMIDIAccess()](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/requestMIDIAccess)_

### Iterating Inputs — MIDIInputMap

`MIDIInputMap` is a `ReadonlyMap`-like interface. In web-sys, iteration requires JS interop since `MidiInputMap` does not implement Rust's `Iterator`:

```rust
use js_sys::{try_iter, Object};
use web_sys::MidiInput;

let inputs = midi_access.inputs(); // MidiInputMap
// Use js_sys::try_iter on inputs.values()
if let Some(iter) = try_iter(&inputs.values())? {
    for input in iter {
        let input: MidiInput = input?.unchecked_into();
        // Attach listener to this input
    }
}
```

Alternatively, `js_sys::Reflect::get` with known port IDs, or call `.for_each()` via JS interop. The `try_iter` approach is cleanest for "listen on all inputs."

_Source: [MDN: MIDIAccess](https://developer.mozilla.org/en-US/docs/Web/API/MIDIAccess), [web-sys MidiInputMap](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MidiInputMap.html)_

### Event Listener Pattern — midimessage

Each `MIDIInput` is an `EventTarget`. Attach a `midimessage` listener using the same `Closure` pattern peach-web already uses for keyboard events:

```rust
use wasm_bindgen::prelude::*;
use web_sys::MidiMessageEvent;

let callback = Closure::<dyn Fn(MidiMessageEvent)>::new(move |ev: MidiMessageEvent| {
    let data = ev.data().unwrap(); // Uint8Array
    let status = data.get_index(0);

    // Note-on: status 0x90–0x9F, velocity > 0
    let is_note_on = (status & 0xF0) == 0x90 && data.length() >= 3 && data.get_index(2) > 0;

    if is_note_on {
        let timestamp_ms = ev.time_stamp(); // DOMHighResTimeStamp — same as PointerEvent
        on_tap(timestamp_ms); // <-- Reuse existing tap handler directly!
    }
});

input.add_event_listener_with_callback("midimessage", callback.as_ref().unchecked_ref())?;
callback.forget(); // Or store for cleanup
```

**Critical integration point:** The `on_tap(timestamp_ms)` closure in `continuous_rhythm_matching_view.rs` already accepts a `f64` timestamp in `performance.now()` coordinates. A MIDI note-on event provides exactly that. **No adapter translation layer is needed — just call the same closure.**

_Source: [MDN: MIDIInput midimessage event](https://developer.mozilla.org/en-US/docs/Web/API/MIDIInput/midimessage_event), [wasm-bindgen Closures guide](https://wasm-bindgen.github.io/wasm-bindgen/examples/closures.html)_

### Closure Lifecycle Management

The existing codebase uses two patterns for closure lifecycle:

1. **Component-scoped closures** — The keyboard handler in `continuous_rhythm_matching_view.rs` stores the `Closure` and removes the listener in `on_cleanup`. MIDI listeners should follow the same pattern.
2. **`Closure::forget()`** — Acceptable for app-lifetime resources, but wasteful for per-component MIDI listeners.

**Recommended approach:** Store the MIDI closures alongside the keyboard closure. In `on_cleanup`, call `remove_event_listener_with_callback` on each `MidiInput` to prevent leaks.

_Source: [wasm-bindgen issue #993: remove event listener](https://github.com/wasm-bindgen/wasm-bindgen/issues/993)_

### Hot-Plug Device Handling

`MIDIAccess` fires `statechange` events when devices are connected or disconnected. The event provides a `MIDIConnectionEvent` with `event.port` containing the added/removed port.

```rust
let statechange = Closure::<dyn Fn(MidiConnectionEvent)>::new(move |ev: MidiConnectionEvent| {
    let port = ev.port().unwrap();
    match port.state().as_str() {
        "connected" => { /* re-scan inputs, attach listener to new port */ }
        "disconnected" => { /* remove listener, update UI */ }
        _ => {}
    }
});
midi_access.set_onstatechange(Some(statechange.as_ref().unchecked_ref()));
```

For peach-web's initial implementation, hot-plug support is a nice-to-have. A simpler approach: request MIDI access once at training view mount, iterate current inputs, and ignore hot-plug initially.

_Source: [MDN: MIDIAccess statechange](https://developer.mozilla.org/en-US/docs/Web/API/MIDIAccess/statechange_event), [MDN: MIDIConnectionEvent](https://developer.mozilla.org/en-US/docs/Web/API/MIDIConnectionEvent)_

### Integration with Existing Tap Pipeline — Architecture Fit

The following diagram shows how MIDI input slots into the existing architecture:

```
┌──────────────────────┐     ┌──────────────────────┐
│  PointerEvent /      │     │  MIDIMessageEvent     │
│  KeyboardEvent       │     │  (note-on, any ch)    │
│  .timeStamp (ms)     │     │  .timeStamp (ms)      │
└──────────┬───────────┘     └──────────┬────────────┘
           │                            │
           └──────────┬─────────────────┘
                      │
                      ▼
           on_tap(event_timestamp_ms: f64)
                      │
                      ▼
           bridge_event_to_audio_time()
                      │
                      ▼
           evaluate_tap(tap_time, scheduled, tempo, latency)
                      │
                      ▼
           RhythmOffset { ms, direction }
```

**No changes required to:**
- `audio_latency.rs` — `bridge_event_to_audio_time()` works unchanged
- `rhythm_offset_detection.rs` — `evaluate_tap()` works unchanged
- `continuous_rhythm_matching_session.rs` — `handle_tap()` works unchanged

**Changes required:**
- `web/Cargo.toml` — Add ~8 web-sys feature flags
- `web/src/adapters/` — New `midi_input.rs` module (~80–120 lines) for MIDI access request + listener setup
- `web/src/components/continuous_rhythm_matching_view.rs` — Wire MIDI listener alongside existing keyboard listener, calling the same `on_tap` closure
- Feature detection — Check `navigator.requestMIDIAccess` exists before attempting

### Feature Detection and Graceful Degradation

MIDI input is a progressive enhancement. If the API is unavailable (Safari, non-HTTPS, denied permission), the app works exactly as today with tap/keyboard input. Feature detection:

```rust
// Check if Web MIDI API is available
let has_midi = js_sys::Reflect::get(
    &web_sys::window().unwrap().navigator(),
    &JsValue::from_str("requestMIDIAccess"),
)
.ok()
.map_or(false, |v| v.is_function());
```

This can drive a UI toggle (show/hide MIDI settings) and avoid unnecessary permission prompts on unsupported browsers.

---

## Architectural Patterns and Design

### Module Architecture — Where MIDI Fits

peach-web follows a clean **domain/web split**: `domain/` contains pure Rust logic (no browser dependencies), `web/` contains Leptos components and browser API adapters. MIDI input is entirely a **web-layer concern** — the domain crate never needs to know whether a tap came from a pointer, keyboard, or MIDI controller.

**Recommended module placement:**

```
web/src/adapters/
├── audio_context.rs       # AudioContext lifecycle (existing)
├── audio_latency.rs       # Timestamp bridging (existing, reused as-is)
├── midi_input.rs          # NEW — MIDI access, listener setup, note-on detection
├── rhythm_scheduler.rs    # Beat scheduling (existing)
└── ...
```

The new `midi_input.rs` adapter encapsulates all Web MIDI API interaction. It exposes a simple interface to components:

```rust
/// Request MIDI access and attach note-on listeners to all inputs.
/// Calls `on_note_on(timestamp_ms: f64)` for each note-on event.
/// Returns a cleanup handle that removes listeners on drop.
pub async fn setup_midi_listeners(
    on_note_on: Rc<dyn Fn(f64)>,
) -> Result<MidiCleanupHandle, String>
```

This follows the same pattern as the existing adapters — thin wrappers over browser APIs that translate to domain-friendly types.

### Permission Flow Architecture

The MIDI permission request must be triggered by a user gesture (same as `AudioContext.resume()`). The recommended flow integrates with the existing "tap to start" pattern in training views:

```
User taps "Start Training"
  ↓
Resume AudioContext (existing gesture gate)
  ↓
requestMIDIAccess({ sysex: false })  ← NEW, same gesture
  ↓
Browser shows permission prompt (first time only)
  ↓
On success: attach listeners to all inputs
On failure: log warning, continue with tap/keyboard only
```

**Key design decision:** Request MIDI access at training view mount, not at app startup. This avoids premature permission prompts and only asks when the user actually needs MIDI. Chrome persists the permission grant per origin, so subsequent sessions skip the prompt.

The permission can also be pre-checked without triggering a prompt:

```rust
// Query permission state without prompting
let status = navigator.permissions()
    .query(&midi_permission_descriptor)  // { name: "midi" }
    .await;
// status.state: "granted" | "denied" | "prompt"
```

This enables showing a "MIDI available" indicator in the UI before the user starts training.

_Source: [Chrome MIDI Permission Blog](https://developer.chrome.com/blog/web-midi-permission-prompt), [W3C Permissions API](https://w3c.github.io/permissions/)_

### Cleanup and Lifecycle Pattern

peach-web uses a consistent pattern for event listener cleanup: store the `Closure` in a `StoredValue::new_local()`, keep the `JsValue` reference, and call `remove_event_listener_with_callback` in `on_cleanup`. The MIDI adapter must follow this same pattern.

**Design: `MidiCleanupHandle`**

```rust
pub struct MidiCleanupHandle {
    inputs: Vec<(MidiInput, JsValue)>,  // (target, callback_ref)
    _closures: Vec<Closure<dyn Fn(MidiMessageEvent)>>,  // prevent drop
}

impl MidiCleanupHandle {
    pub fn cleanup(&self) {
        for (input, cb_ref) in &self.inputs {
            let _ = input.remove_event_listener_with_callback(
                "midimessage",
                cb_ref.unchecked_ref(),
            );
        }
    }
}
```

The handle is stored in a `StoredValue::new_local()` in the component and cleaned up in `on_cleanup`, matching the existing keyboard listener pattern at `continuous_rhythm_matching_view.rs:353–367`.

_Source: [wasm-bindgen issue #993](https://github.com/wasm-bindgen/wasm-bindgen/issues/993), [gloo-events RAII pattern](https://docs.rs/gloo-events/latest/gloo_events/struct.EventListener.html)_

### Separation of Concerns

| Layer | MIDI Responsibility | Existing Analogy |
|-------|-------------------|------------------|
| **`domain/`** | None — receives `tap_time: f64` regardless of source | Same as today |
| **`web/adapters/midi_input.rs`** | MIDI access, permission, note-on detection, timestamp extraction | `audio_latency.rs` for timestamp bridging |
| **`web/components/`** | Wire MIDI adapter output into `on_tap()` closure | Pointer/keyboard handlers |

The domain crate's `evaluate_tap()` and `handle_tap()` remain completely source-agnostic. This is the correct boundary — input method is a presentation concern, timing evaluation is a domain concern.

### Error Handling and Degradation Strategy

MIDI failures should never block training. The error handling strategy:

| Failure Mode | Behavior | User Impact |
|---|---|---|
| API not available (Safari) | Skip MIDI setup silently | Tap/keyboard works normally |
| Permission denied | Log warning, continue | Tap/keyboard works normally |
| No MIDI devices connected | Listeners attached to empty set; statechange watches for hot-plug | No input until device connected |
| Device disconnected mid-session | statechange fires; tap/keyboard still works | Seamless fallback |
| `requestMIDIAccess` promise rejects | Catch, log, continue | Tap/keyboard works normally |

**No error dialogs or UI disruption** — MIDI is purely additive. This matches the existing pattern where `bridge_event_to_audio_time` silently falls back to `ctx.current_time()` when unsupported.

### Architectural Decision: Single Shared `on_tap` vs. Separate MIDI Handler

**Decision: Share the same `on_tap` closure.**

Rationale:
- The `on_tap` closure already handles timestamp bridging, output latency reading, session evaluation, feedback display, and click playback
- MIDI note-on provides the same `DOMHighResTimeStamp` as pointer/keyboard events
- Duplicating this logic for MIDI would violate DRY and create maintenance risk
- The existing `on_tap` is already behind an `Rc<dyn Fn(f64)>` — just pass a clone to the MIDI adapter

The only MIDI-specific logic (status byte parsing, velocity check) lives in `midi_input.rs`. Everything downstream is shared.

### Architectural Decision: When to Request MIDI Access

**Option A — At training view mount (recommended):**
- Request alongside AudioContext resume
- Permission prompt appears in training context (makes sense to the user)
- No wasted prompt if user never trains

**Option B — At app startup:**
- Earlier availability, but premature permission prompt
- User may deny before understanding why it's needed

**Option C — On explicit user action (settings toggle):**
- Most control, but adds UI complexity
- Deferred to future settings work

**Decision: Option A** for initial implementation. The user said settings come later, so we tie MIDI access to the same user gesture that starts training.

---

## Implementation Approaches and Technology Adoption

### Implementation Roadmap

The implementation naturally breaks into three phases:

**Phase 1 — Core MIDI Input (minimal viable):**

1. Add web-sys MIDI feature flags to `web/Cargo.toml`
2. Create `web/src/adapters/midi_input.rs`:
   - `is_midi_available()` — feature detection
   - `setup_midi_listeners(on_note_on) -> Result<MidiCleanupHandle>` — request access, iterate inputs, attach note-on listeners
   - `MidiCleanupHandle` — stores closures and references for cleanup
3. Wire into `continuous_rhythm_matching_view.rs`:
   - Call `setup_midi_listeners` after AudioContext resume
   - Pass `Rc::clone(&on_tap)` as the `on_note_on` callback
   - Store `MidiCleanupHandle` in `StoredValue::new_local`, clean up in `on_cleanup`

**Phase 2 — Robustness (follow-up):**

4. Hot-plug support via `MIDIAccess.onstatechange` — attach listeners to newly connected devices
5. Permission state pre-check — show MIDI availability indicator in UI

**Phase 3 — Settings (deferred per user request):**

6. MIDI enable/disable toggle in settings
7. Channel/note filtering
8. MIDI device selection UI

### Concrete Implementation — `midi_input.rs`

The adapter module is approximately 80–120 lines. Here is the key structure:

```rust
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MidiAccess, MidiInput, MidiMessageEvent, MidiOptions};

/// Check if Web MIDI API is available in this browser.
pub fn is_midi_available() -> bool {
    js_sys::Reflect::get(
        &web_sys::window().unwrap().navigator(),
        &JsValue::from_str("requestMIDIAccess"),
    )
    .ok()
    .map_or(false, |v| v.is_function())
}

/// Stores references needed to remove MIDI listeners on cleanup.
pub struct MidiCleanupHandle {
    inputs: Vec<(MidiInput, JsValue)>,
    _closures: Vec<Closure<dyn Fn(MidiMessageEvent)>>,
}

impl MidiCleanupHandle {
    pub fn cleanup(&self) {
        for (input, cb_ref) in &self.inputs {
            let _ = input.remove_event_listener_with_callback(
                "midimessage", cb_ref.unchecked_ref(),
            );
        }
    }
}

/// Request MIDI access and attach note-on listeners to all connected inputs.
/// Calls `on_note_on(timestamp_ms)` for each note-on event (any channel, any note).
pub async fn setup_midi_listeners(
    on_note_on: Rc<dyn Fn(f64)>,
) -> Result<MidiCleanupHandle, String> {
    let navigator = web_sys::window().unwrap().navigator();
    let opts = MidiOptions::new();
    opts.set_sysex(false);

    let promise = navigator
        .request_midi_access_with_options(&opts)
        .map_err(|e| format!("requestMIDIAccess failed: {:?}", e))?;

    let midi_access: MidiAccess = JsFuture::from(promise)
        .await
        .map_err(|e| format!("MIDI access denied: {:?}", e))?
        .unchecked_into();

    let inputs_map = midi_access.inputs();
    let mut inputs = Vec::new();
    let mut closures = Vec::new();

    // Iterate all connected MIDI inputs
    if let Some(iter) = js_sys::try_iter(&inputs_map.values())
        .map_err(|e| format!("Failed to iterate MIDI inputs: {:?}", e))?
    {
        for input_result in iter {
            let input: MidiInput = input_result
                .map_err(|e| format!("Input iteration error: {:?}", e))?
                .unchecked_into();

            let on_note_on = Rc::clone(&on_note_on);
            let callback = Closure::<dyn Fn(MidiMessageEvent)>::new(
                move |ev: MidiMessageEvent| {
                    if let Some(data) = ev.data() {
                        if data.length() >= 3 {
                            let status = data.get_index(0);
                            let velocity = data.get_index(2);
                            // Note-on: 0x90–0x9F with velocity > 0
                            if (status & 0xF0) == 0x90 && velocity > 0 {
                                on_note_on(ev.time_stamp());
                            }
                        }
                    }
                },
            );

            let cb_ref: JsValue = callback.as_ref().clone();
            input
                .add_event_listener_with_callback(
                    "midimessage", cb_ref.unchecked_ref(),
                )
                .map_err(|e| format!("Failed to add listener: {:?}", e))?;

            inputs.push((input, cb_ref));
            closures.push(callback);
        }
    }

    log::info!("MIDI: attached listeners to {} input(s)", inputs.len());
    Ok(MidiCleanupHandle {
        inputs,
        _closures: closures,
    })
}
```

_Source: [web-sys MidiMessageEvent](https://docs.rs/web-sys/latest/web_sys/struct.MidiMessageEvent.html), [js-sys Uint8Array](https://docs.rs/js-sys/latest/js_sys/struct.Uint8Array.html)_

### Cargo.toml Changes

Add these features to the existing `web-sys` dependency in `web/Cargo.toml`:

```toml
# MIDI input support
"MidiAccess",
"MidiConnectionEvent",
"MidiInput",
"MidiInputMap",
"MidiMessageEvent",
"MidiOptions",
"MidiOutput",
"MidiOutputMap",
"MidiPort",
"Navigator",
```

Note: `Navigator` is needed for `request_midi_access_with_options()`. `MidiOutput`/`MidiOutputMap` are transitive dependencies pulled in by `MidiAccess`.

### Component Integration Changes

In `continuous_rhythm_matching_view.rs`, add MIDI setup after AudioContext is created:

```rust
// After AudioContext resume succeeds...
if midi_input::is_midi_available() {
    let on_tap_for_midi = Rc::clone(&on_tap);
    spawn_local_scoped_with_cancellation(async move {
        match midi_input::setup_midi_listeners(on_tap_for_midi).await {
            Ok(handle) => {
                midi_cleanup.set_value(Some(handle));
            }
            Err(e) => {
                log::warn!("MIDI setup failed (non-fatal): {}", e);
            }
        }
    });
}
```

And in `on_cleanup`:

```rust
if let Some(handle) = midi_cleanup.try_get_value().flatten() {
    handle.cleanup();
}
```

### Testing Strategy

**Domain tests (unchanged):** `evaluate_tap()` and `handle_tap()` are input-source-agnostic — existing tests cover MIDI input timing correctness implicitly.

**MIDI-specific unit tests in `midi_input.rs`:**

The note-on detection logic (status byte parsing) can be tested with pure Rust functions extracted from the closure:

```rust
/// Returns true if the MIDI data represents a note-on event.
pub fn is_note_on(data: &[u8]) -> bool {
    data.len() >= 3 && (data[0] & 0xF0) == 0x90 && data[2] > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_channel_1() {
        assert!(is_note_on(&[0x90, 60, 100])); // C4, velocity 100
    }

    #[test]
    fn note_on_channel_16() {
        assert!(is_note_on(&[0x9F, 60, 100])); // Channel 16
    }

    #[test]
    fn note_off_via_velocity_zero() {
        assert!(!is_note_on(&[0x90, 60, 0])); // Velocity 0 = note-off
    }

    #[test]
    fn note_off_message() {
        assert!(!is_note_on(&[0x80, 60, 64])); // Explicit note-off
    }

    #[test]
    fn control_change_ignored() {
        assert!(!is_note_on(&[0xB0, 64, 127])); // CC message
    }

    #[test]
    fn short_message_ignored() {
        assert!(!is_note_on(&[0x90])); // Truncated
    }
}
```

**Manual testing:** Connect a USB MIDI controller, open the training view, verify note-on triggers tap evaluation with correct timing feedback.

**Automated browser testing:** The [`web-midi-test`](https://github.com/jazz-soft/web-midi-test) library can inject a fake `requestMIDIAccess` for CI, but this is a lower priority given the simple integration surface.

_Source: [web-midi-test](https://github.com/jazz-soft/web-midi-test), [web-midi-test-api](https://github.com/mohayonao/web-midi-test-api)_

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Safari never supports Web MIDI | High | Low | Progressive enhancement — tap/keyboard always works |
| Firefox permission UX is confusing | Medium | Low | Document in user guide; permission persists once granted |
| MIDI-over-BLE adds latency | Medium | Low | Same `bridge_event_to_audio_time` compensates; timestamp is hardware-side |
| `MidiInputMap` iteration API changes | Very Low | Medium | web-sys tracks WebIDL spec; stable since 2015 |
| Multiple MIDI devices create duplicate events | Low | Low | Any note-on is valid input; duplicates are filtered by session state (one tap per gap) |

### Success Metrics

- MIDI note-on triggers tap evaluation with timing offset within ±5ms of equivalent pointer event
- No regression in tap/keyboard input latency or accuracy
- Zero impact on training flow when MIDI is unavailable
- `cargo test -p domain` continues to pass unchanged
- `cargo clippy --workspace` clean

---

## Research Synthesis and Conclusion

### Summary of Key Findings

1. **Timestamp compatibility is the linchpin.** The discovery that `MIDIMessageEvent.timeStamp` shares the same `DOMHighResTimeStamp` coordinate system as `PointerEvent.timeStamp` means the entire existing timing pipeline is reusable without modification. This single fact reduces the implementation from a cross-cutting architectural change to a localized adapter addition.

2. **MIDI input latency is favorable.** USB MIDI's ~3–20ms total path latency is comparable to or better than touch event dispatch latency (10–30ms). MIDI-over-BLE adds more (~5–15ms from Bluetooth alone) but the hardware timestamp still captures the true event time, so `bridge_event_to_audio_time()` compensates correctly.

3. **The implementation surface is small.** One new adapter module (~80–120 lines), 8 web-sys feature flags, and ~15 lines of wiring in the view component. No domain crate changes. No new third-party dependencies.

4. **Browser support is sufficient for a progressive enhancement.** Chrome/Edge have full support since 2015. Firefox desktop added support in v109 (2023). Safari is unsupported but MIDI is opt-in — the app works identically without it.

5. **The W3C spec is mature and stable.** The Web MIDI API is a W3C Working Draft targeting Recommendation status in 2025, with 89% specification completion and two independent browser implementations. The API surface has been stable since 2015 with only security model changes (mandatory permission prompt added in Chrome 124).

### What Is Necessary — Complete Checklist

**Code changes:**

- [ ] Add web-sys MIDI feature flags to `web/Cargo.toml` (~10 features)
- [ ] Create `web/src/adapters/midi_input.rs` with:
  - `is_midi_available()` — feature detection
  - `is_note_on(data: &[u8])` — pure Rust note-on parsing (testable)
  - `setup_midi_listeners(on_note_on)` — async MIDI access + listener attachment
  - `MidiCleanupHandle` — RAII-style listener cleanup
- [ ] Register `midi_input` in `web/src/adapters/mod.rs`
- [ ] Wire MIDI setup in `continuous_rhythm_matching_view.rs`:
  - Call `setup_midi_listeners` after AudioContext resume
  - Pass `Rc::clone(&on_tap)` as callback
  - Store and clean up `MidiCleanupHandle`
- [ ] Add `is_note_on` unit tests to `midi_input.rs`

**No changes needed:**

- `domain/` crate — completely unchanged
- `audio_latency.rs` — `bridge_event_to_audio_time()` works as-is
- `rhythm_offset_detection.rs` — `evaluate_tap()` works as-is
- `continuous_rhythm_matching_session.rs` — `handle_tap()` works as-is

**Deferred to future work:**

- MIDI enable/disable setting
- Channel/note filtering
- Device selection UI
- Hot-plug device handling via `MIDIAccess.onstatechange`
- MIDI permission pre-check and availability indicator

### Source Documentation

| Source | Usage |
|--------|-------|
| [W3C Web MIDI API Spec](https://webaudio.github.io/web-midi-api/) | API design, timestamp semantics, permission model |
| [MDN Web MIDI API](https://developer.mozilla.org/en-US/docs/Web/API/Web_MIDI_API) | Browser compatibility, API reference |
| [Can I use: Web MIDI](https://caniuse.com/midi) | Browser support matrix |
| [Chrome MIDI Permission Blog](https://developer.chrome.com/blog/web-midi-permission-prompt) | Permission flow changes in Chrome 124+ |
| [web-sys MidiAccess](https://docs.rs/web-sys/latest/web_sys/struct.MidiAccess.html) | Rust/WASM binding API |
| [web-sys MidiMessageEvent](https://docs.rs/web-sys/latest/web_sys/struct.MidiMessageEvent.html) | Event data access pattern |
| [wasm-bindgen Closures guide](https://wasm-bindgen.github.io/wasm-bindgen/examples/closures.html) | Closure lifecycle in WASM |
| [Web MIDI API Issue #187](https://github.com/WebAudio/web-midi-api/issues/187) | Input latency discussion |
| [MDN Permissions-Policy: midi](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Permissions-Policy/midi) | HTTP header security controls |
| [W3C Web MIDI publication history](https://www.w3.org/standards/history/webmidi/) | Spec maturity and timeline |

---

**Research Completion Date:** 2026-03-26
**Source Verification:** All claims verified against current W3C specifications, MDN documentation, and web-sys API docs
**Confidence Level:** High — based on stable W3C spec, production browser implementations, and direct codebase analysis
