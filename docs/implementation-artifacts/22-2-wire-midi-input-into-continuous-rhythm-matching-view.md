# Story 22.2: Wire MIDI Input into Continuous Rhythm Matching View

Status: ready-for-dev

## Story

As a user with a MIDI controller,
I want my MIDI note-on events to trigger tap evaluation in rhythm training,
so that I can practice rhythm with a pad, keyboard, or any MIDI device instead of (or alongside) tapping the screen.

## Context

Epic 22 adds MIDI controller support as progressive enhancement for rhythm training. This is story 2 of 2 — it wires the adapter module (built in Story 22.1) into the continuous rhythm matching view. No domain changes required.

Prerequisites: Story 22.1 (MIDI adapter module, done), Epic 18 (continuous rhythm matching), Epic 21 (tap latency pipeline).

Key insight: `MIDIMessageEvent.timeStamp` is a `DOMHighResTimeStamp` in the same `performance.now()` coordinate space as `PointerEvent`/`KeyboardEvent`. The existing `on_tap` closure already accepts `f64` event timestamps and calls `bridge_event_to_audio_time()` — MIDI events feed into the same path with zero changes to the tap evaluation pipeline.

Technical research: `docs/planning-artifacts/research/technical-web-midi-input-research-2026-03-26.md`

## Acceptance Criteria

1. **AC1 — MIDI setup on supported browser:** When the training view mounts and AudioContext is resumed, `setup_midi_listeners` is called with a clone of the existing `on_tap` closure, if `is_midi_available()` returns `true`.

2. **AC2 — MIDI tap evaluation:** A MIDI note-on event during training is processed identically to a pointer or keyboard tap — same `bridge_event_to_audio_time` -> `handle_tap` -> `RhythmOffset` path, same feedback display, same click playback.

3. **AC3 — Graceful failure:** If MIDI setup fails (permission denied, API error), a warning is logged via `log::warn!` and training continues with pointer/keyboard input only. No error dialog or UI disruption.

4. **AC4 — Unsupported browser skip:** If `is_midi_available()` returns `false`, MIDI setup is skipped entirely and training works as before.

5. **AC5 — Cleanup on unmount:** When the training view unmounts (`on_cleanup`), the `MidiCleanupHandle` is cleaned up, removing all MIDI event listeners.

6. **AC6 — Remove dead_code annotation:** The `#![allow(dead_code)]` at the top of `midi_input.rs` is removed since the module now has callers.

7. **AC7 — Domain unchanged:** `cargo test -p domain` passes unchanged — the domain crate is not modified.

8. **AC8 — Clean build:** `cargo clippy --workspace` produces no warnings.

## Tasks / Subtasks

- [ ] Task 1: Remove `#![allow(dead_code)]` from `web/src/adapters/midi_input.rs` (AC: 6)
- [ ] Task 2: Wire MIDI into `web/src/components/continuous_rhythm_matching_view.rs` (AC: 1-5)
  - [ ] Add `use crate::adapters::midi_input;` import
  - [ ] Create `StoredValue<Option<SendWrapper<midi_input::MidiCleanupHandle>>>` for cleanup handle storage, before the `on_cleanup` block
  - [ ] In the async training loop, after `ensure_audio_ready` succeeds and shared resources are set (after line ~414), add MIDI setup block: check `is_midi_available()`, spawn async task to call `setup_midi_listeners(on_tap_clone)`, store handle on success, log warning on failure
  - [ ] In the `on_cleanup` closure, add MIDI cleanup: take the handle from `StoredValue` and call `.cleanup()` if `Some`
- [ ] Task 3: Run `cargo fmt`, `cargo clippy --workspace`, `cargo test -p domain`, `cargo test -p web` (AC: 7, 8)

## Dev Notes

### Architecture Compliance

- **No domain changes.** The domain crate is not modified. MIDI is a web-only input source. The `on_tap` closure already abstracts away the input source — it accepts `f64` timestamps regardless of origin.
- **No audio_latency changes.** `bridge_event_to_audio_time()` works unchanged for MIDI timestamps (same `performance.now()` coordinate space).
- **Single integration point:** Only `continuous_rhythm_matching_view.rs` is modified (plus the dead_code annotation removal in `midi_input.rs`).

### Technical Implementation Details

**MIDI setup location:**
- Inside the async training loop in `continuous_rhythm_matching_view.rs`, after `ensure_audio_ready` succeeds and shared audio resources are stored (after the `*shared_ctx_for_loop.borrow_mut() = ...` line around line 414).
- This timing ensures: (1) AudioContext is running (user gesture completed), (2) the `on_tap` closure's shared resources are populated, (3) training is about to start.

**MIDI setup pattern:**
```rust
// After shared audio resources are set up...
if midi_input::is_midi_available() {
    let on_tap_for_midi = Rc::clone(&on_tap);
    match midi_input::setup_midi_listeners(move |ts| on_tap_for_midi(ts)).await {
        Ok(handle) => {
            midi_cleanup_handle.set_value(Some(SendWrapper::new(handle)));
        }
        Err(e) => {
            log::warn!("MIDI setup failed (non-fatal): {:?}", e);
        }
    }
}
```

**Cleanup handle storage:**
- Use `StoredValue::new_local(None::<SendWrapper<midi_input::MidiCleanupHandle>>)` — matches the existing pattern used for `_keydown_closure`, `_visibility_closure`, `_audiocontext_closure` in this file.
- `SendWrapper` is required because `MidiCleanupHandle` contains `Closure` and `MidiInput` which are not `Send`. Leptos `StoredValue` requires `Send + Sync` — `SendWrapper` is the established pattern (safe in single-threaded WASM).
- Declare the `StoredValue` BEFORE the `on_cleanup` block so it can be captured by both the cleanup closure and the training loop.

**MIDI cleanup in `on_cleanup`:**
```rust
// Inside the on_cleanup closure, after existing cleanup...
if let Some(handle) = midi_cleanup_handle.try_get_value().flatten() {
    handle.cleanup();
}
```

**Important: `setup_midi_listeners` is async.** It must be awaited. Since the training loop is already inside a `spawn_local` async block, the MIDI setup can be directly `.await`ed inline — no additional spawn needed. This is simpler and ensures MIDI is ready before the training loop begins.

**The `on_tap` closure is `Rc<dyn Fn(f64)>`** (line 201). Clone it with `Rc::clone(&on_tap)` and pass it to `setup_midi_listeners`. The MIDI adapter calls `on_note_on(event.time_stamp())` which feeds directly into this closure.

### Existing Patterns to Follow

- **StoredValue for closures:** Lines 273, 289, 341 — `StoredValue::new_local(closure)` keeps closures alive and prevents them from being dropped while listeners are active.
- **SendWrapper for non-Send types:** Used throughout this file and `app.rs` for `Rc<RefCell<T>>` and closures that must be stored in Leptos reactive primitives.
- **on_cleanup tuple pattern:** Lines 344-370 — all cleanup actions grouped in a single `on_cleanup` closure that captures a `SendWrapper` tuple. The MIDI cleanup should be added to this existing closure, not as a separate `on_cleanup`.
- **Error logging without UI disruption:** `log::warn!` / `log::error!` for non-fatal issues, only `audio_error.set(...)` for fatal audio failures.

### What NOT to Do

- Do NOT create a separate `on_midi_tap` closure — reuse the existing `on_tap` closure. MIDI timestamps are in the same coordinate space.
- Do NOT add MIDI status indicators to the UI — this is progressive enhancement, invisible to the user.
- Do NOT modify the domain crate — MIDI is a web-only input concern.
- Do NOT use `spawn_local` or `spawn_local_scoped_with_cancellation` for MIDI setup — the training loop is already async, so `.await` the setup directly inline.
- Do NOT add a second `on_cleanup` block — add MIDI cleanup to the existing one by including the `StoredValue` handle in the cleanup tuple.

### Project Structure Notes

- Modified: `web/src/adapters/midi_input.rs` (remove `#![allow(dead_code)]` line only)
- Modified: `web/src/components/continuous_rhythm_matching_view.rs` (add MIDI setup, cleanup, and import)
- No other files modified

### References

- [Source: docs/planning-artifacts/epics.md#Story 22.2] — Acceptance criteria and user story
- [Source: docs/planning-artifacts/research/technical-web-midi-input-research-2026-03-26.md] — Timestamp compatibility, integration pattern, cleanup lifecycle
- [Source: docs/implementation-artifacts/22-1-midi-adapter-module-with-note-on-detection.md] — Previous story: adapter API, cleanup handle pattern, debug learnings
- [Source: web/src/adapters/midi_input.rs] — `is_midi_available()`, `setup_midi_listeners()`, `MidiCleanupHandle`
- [Source: web/src/adapters/audio_latency.rs] — `bridge_event_to_audio_time()` — reused unchanged by MIDI timestamps
- [Source: web/src/components/continuous_rhythm_matching_view.rs] — Target file: `on_tap` closure (line 195-250), `on_cleanup` (line 343-370), async training loop (line 375+)
- [Source: docs/project-context.md] — SendWrapper pattern, no-domain-in-web rule, cleanup conventions

### Previous Story Intelligence (22.1)

- **`MidiMessageEvent.data()` returns `Result<Vec<u8>, JsValue>`** (not `Option`) — already handled in the adapter.
- **`MidiOptions::set_sysex()`** is the correct API (not deprecated `.sysex()`).
- **`MidiInputMap` iteration** uses the JS iterator protocol via `.values().next()` with `done`/`value` property checks — all encapsulated in `setup_midi_listeners`.
- **`MidiCleanupHandle` implements `Drop`** — dropping it also removes listeners, so even if explicit `.cleanup()` is skipped, the listeners are cleaned up. But prefer explicit cleanup for clarity.
- **`#![allow(dead_code)]`** was added because nothing called the module yet — must be removed in this story.

### Git Intelligence

Recent commits show Story 22.1 was implemented and reviewed (3 commits: create story, implement, two review fixes). The adapter API is stable. Story 21 series established the `bridge_event_to_audio_time` and `get_output_latency` pipeline that MIDI timestamps feed into.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
