# Story 22.1: MIDI Adapter Module with Note-On Detection

Status: ready-for-dev

## Story

As a developer,
I want a `midi_input.rs` adapter that handles Web MIDI API access, feature detection, and note-on event listening,
so that MIDI input is encapsulated in a single module following the existing adapter pattern.

## Context

Epic 22 adds MIDI controller support as progressive enhancement for rhythm training. This is story 1 of 2 — it builds the adapter module. Story 22.2 wires it into the training views.

Prerequisites: Epic 18 (continuous rhythm matching exists), Epic 21 (tap latency pipeline with `bridge_event_to_audio_time`).

Technical research: `docs/planning-artifacts/research/technical-web-midi-input-research-2026-03-26.md` — covers browser support, timestamp compatibility, web-sys feature flags, and MIDI message format.

Key insight: `MIDIMessageEvent.timeStamp` uses the same `performance.now()` coordinate space as `PointerEvent`/`KeyboardEvent`. The existing `bridge_event_to_audio_time()` in `audio_latency.rs` works unchanged for MIDI timestamps.

## Acceptance Criteria

1. **AC1 — web-sys feature flags:** `web/Cargo.toml` enables `MidiAccess`, `MidiInput`, `MidiInputMap`, `MidiMessageEvent`, `MidiOptions`, `MidiPort`, `MidiConnectionEvent`, and `Navigator` in the `web-sys` features list.

2. **AC2 — Feature detection (supported browser):** `is_midi_available()` returns `true` when the browser supports Web MIDI API.

3. **AC3 — Feature detection (unsupported browser):** `is_midi_available()` returns `false` when the browser does not support Web MIDI API (e.g., Safari).

4. **AC4 — Note-on detection (positive):** `is_note_on(data)` returns `true` for a 3-byte MIDI message with status byte `0x90`-`0x9F` and velocity > 0.

5. **AC5 — Note-on detection (negative):** `is_note_on(data)` returns `false` for: velocity-0 note-on, note-off (`0x80`-`0x8F`), control change, and messages with fewer than 3 bytes.

6. **AC6 — Listener setup:** `setup_midi_listeners(on_note_on)` attaches a `midimessage` event listener to each connected MIDI input that calls `on_note_on(timestamp_ms)` for note-on events. Returns a `MidiCleanupHandle`.

7. **AC7 — Listener cleanup:** Calling `cleanup()` on `MidiCleanupHandle` removes all `midimessage` listeners from their respective `MidiInput` targets.

8. **AC8 — Unit tests:** `is_note_on` has unit tests covering: note-on channel 1, note-on channel 16, velocity-zero note-off, explicit note-off, control change ignored, truncated message ignored. Tests pass with `cargo test -p web`.

9. **AC9 — Clean build:** `cargo clippy --workspace` produces no warnings.

## Tasks / Subtasks

- [ ] Task 1: Add web-sys MIDI feature flags to `web/Cargo.toml` (AC: 1)
  - [ ] Add all 8 features: `Navigator`, `MidiOptions`, `MidiAccess`, `MidiInputMap`, `MidiInput`, `MidiMessageEvent`, `MidiPort`, `MidiConnectionEvent`
  - [ ] Keep features sorted alphabetically with existing entries
- [ ] Task 2: Create `web/src/adapters/midi_input.rs` and register in `web/src/adapters/mod.rs` (AC: 2-7)
  - [ ] Add `pub mod midi_input;` to `web/src/adapters/mod.rs`
  - [ ] Implement `is_midi_available()` — check `Navigator` for `requestMIDIAccess` presence via JS reflection
  - [ ] Implement `is_note_on(data: &[u8]) -> bool` — pure function, no web-sys needed
  - [ ] Implement `setup_midi_listeners(on_note_on: impl Fn(f64) + 'static) -> Result<MidiCleanupHandle, JsValue>` — async function using `requestMIDIAccess({ sysex: false })`
  - [ ] Implement `MidiCleanupHandle` struct that stores listener closures and removes them on `cleanup()`
- [ ] Task 3: Write unit tests for `is_note_on` (AC: 8)
  - [ ] `test_note_on_channel_1` — status `0x90`, note 60, velocity 100
  - [ ] `test_note_on_channel_16` — status `0x9F`, note 60, velocity 100
  - [ ] `test_velocity_zero_is_not_note_on` — status `0x90`, note 60, velocity 0
  - [ ] `test_note_off_is_not_note_on` — status `0x80`, note 60, velocity 64
  - [ ] `test_control_change_is_not_note_on` — status `0xB0`, control 64, value 127
  - [ ] `test_truncated_message_is_not_note_on` — 2-byte and 1-byte messages
- [ ] Task 4: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p web` (AC: 8, 9)

## Dev Notes

### Architecture Compliance

- **Module location:** `web/src/adapters/midi_input.rs` — follows the existing adapter pattern alongside `audio_context.rs`, `audio_latency.rs`, `rhythm_scheduler.rs`
- **No domain changes:** The domain crate is NOT modified. MIDI is a web-only input adapter. The domain types `MIDINote` and `MIDIVelocity` in `domain/src/types/midi.rs` are unrelated to this adapter (they are value types for pitch training)
- **Crate boundary:** All `web-sys` MIDI bindings stay in the web crate

### Technical Implementation Details

**Feature detection (`is_midi_available`):**
- Check if `window().navigator()` has `requestMIDIAccess` via JS reflection (`js_sys::Reflect::get`)
- Do NOT call `requestMIDIAccess` — that triggers a permission prompt. Only check for its existence.
- Returns `bool`, not `Result` — this is a synchronous capability check

**MIDI access (`setup_midi_listeners`):**
- Call `navigator.request_midi_access_with_options(options)` where options has `sysex: false`
- `requestMIDIAccess` returns a `Promise<MIDIAccess>` — use `wasm_bindgen_futures::JsFuture` to await it
- Iterate `MIDIAccess.inputs()` (a `MidiInputMap`) — use `js_sys::try_iter` or `entries()` to iterate the map
- For each `MidiInput`, add a `midimessage` event listener via `add_event_listener_with_callback`
- The callback reads `MidiMessageEvent.data()` (Uint8Array), checks `is_note_on`, and if true calls `on_note_on(event.time_stamp())`
- `event.time_stamp()` returns `DOMHighResTimeStamp` in `performance.now()` coordinates — same as pointer/keyboard events

**Note-on detection (`is_note_on`):**
- MIDI note-on: 3 bytes, status `0x90`-`0x9F` (channel 1-16), velocity > 0
- Velocity 0 with note-on status is conventionally treated as note-off — return `false`
- Note-off: status `0x80`-`0x8F` — return `false`
- Any message < 3 bytes — return `false`

**Cleanup handle (`MidiCleanupHandle`):**
- Store `Vec<(MidiInput, Closure<dyn FnMut(MidiMessageEvent)>)>` — each input paired with its closure
- `cleanup()` calls `remove_event_listener_with_callback("midimessage", closure.as_ref().unchecked_ref())` on each
- Must keep `Closure` alive until cleanup — dropping a `wasm_bindgen::Closure` before removing the listener causes it to become a dangling reference

**MidiInputMap iteration:**
- `MidiInputMap` implements the JS `MapLike` interface but web-sys doesn't provide a Rust iterator
- Use `js_sys::try_iter(&midi_access.inputs())` or call `.values()` and iterate with `js_sys::Iterator`
- Each entry value is a `JsValue` that must be cast to `MidiInput` via `.dyn_into::<MidiInput>()`

### Existing Adapter Patterns to Follow

- **`audio_context.rs`:** Singleton pattern, user-gesture-gated initialization, `web_sys` direct bindings
- **`audio_latency.rs`:** `bridge_event_to_audio_time()` — the function MIDI timestamps will feed into in Story 22.2
- **`rhythm_scheduler.rs`:** `Closure` ownership for event callbacks, cleanup via stored references

### web-sys Feature Flags Reference

Add these 8 features to the `web-sys` `features` list in `web/Cargo.toml`:
```toml
"MidiAccess",
"MidiConnectionEvent",
"MidiInput",
"MidiInputMap",
"MidiMessageEvent",
"MidiOptions",
"MidiPort",
"Navigator",
```

Note: `Navigator` may already be transitively available but must be explicitly listed for `web_sys::Navigator` type access.

### Testing Notes

- `is_note_on` is a pure function on `&[u8]` — fully testable with `#[cfg(test)] mod tests` inline, no browser needed
- `is_midi_available` and `setup_midi_listeners` require browser APIs — defer to manual/integration testing
- Run `cargo test -p web` (not `-p domain`) since this module is in the web crate
- Existing domain tests must remain unaffected: `cargo test -p domain`

### Project Structure Notes

- New file: `web/src/adapters/midi_input.rs`
- Modified: `web/src/adapters/mod.rs` (add `pub mod midi_input;`)
- Modified: `web/Cargo.toml` (add 8 web-sys features)
- No other files modified in this story

### References

- [Source: docs/planning-artifacts/research/technical-web-midi-input-research-2026-03-26.md] — Web MIDI API research, browser support, latency analysis, Rust binding approach
- [Source: docs/planning-artifacts/epics.md#Epic 22] — Epic definition and acceptance criteria
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture] — Adapter pattern, web-sys direct bindings, Closure ownership
- [Source: web/src/adapters/audio_latency.rs] — `bridge_event_to_audio_time()` — timestamp conversion reused in Story 22.2
- [Source: web/src/adapters/rhythm_scheduler.rs] — Closure storage and cleanup pattern reference
- [Source: docs/project-context.md] — Naming conventions, error handling, testing standards

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
