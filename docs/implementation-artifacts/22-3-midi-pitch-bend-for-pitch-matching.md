# Story 22.3: MIDI Pitch Bend for Pitch Matching

Status: ready-for-dev

## Story

As a musician with a MIDI controller,
I want to use the pitch bend wheel for pitch matching training,
so that I can use a familiar physical control instead of dragging an on-screen slider.

## Acceptance Criteria

1. **Pitch bend detection**: The MIDI adapter recognises pitch bend messages (status `0xE0`–`0xEF`, 2 data bytes LSB+MSB forming a 14-bit value 0–16383, center 8192) from any connected MIDI input.
2. **Full-range mapping**: The complete pitch bend range maps linearly to the slider's `[-1.0, +1.0]` domain: 0 → −1.0, 8192 → 0.0, 16383 → +1.0.
3. **Visual slider sync**: The on-screen slider thumb moves in real-time to reflect the current pitch bend position.
4. **Note auto-start**: When in the `AwaitingSliderTouch` state and the pitch bend leaves the neutral zone (center ± small dead-zone, e.g. ±256 out of 8192 ≈ ±3%), the tunable note starts playing — identical to the first slider touch today.
5. **Real-time pitch adjustment**: Continuous pitch bend changes drive `adjust_pitch()` on the session exactly like slider drag, producing real-time frequency changes.
6. **Pitch commit on return to center**: When the pitch bend returns to the neutral zone after being deflected (while in `PlayingTunable` state), the pitch is committed — equivalent to slider release.
7. **Progressive enhancement**: MIDI unavailable → pitch bend listeners skipped, training works with slider/keyboard only. MIDI setup failure → warning log, no error dialog.
8. **Cleanup**: Pitch bend listeners are removed on view unmount via `MidiCleanupHandle`, same pattern as Story 22.2.
9. **No domain crate changes**: All pitch bend logic is web-layer only; the session's existing `adjust_pitch(value)` and `commit_pitch(value, timestamp)` APIs are reused as-is.
10. **Coexistence**: Slider, keyboard, and MIDI pitch bend inputs all remain active simultaneously. The user can switch freely between them.

## Tasks / Subtasks

- [ ] Task 1: Extend MIDI adapter with pitch bend parsing (AC: 1, 2)
  - [ ] 1.1 Add `is_pitch_bend(data: &[u8]) -> bool` — status `0xE0`–`0xEF`, requires 3 bytes
  - [ ] 1.2 Add `parse_pitch_bend(data: &[u8]) -> f64` — combine LSB (data[1]) + MSB (data[2]) into 14-bit value, normalize to `[-1.0, +1.0]`
  - [ ] 1.3 Add unit tests: center (0x00, 0x40) → 0.0, full down (0x00, 0x00) → −1.0, full up (0x7F, 0x7F) → +1.0, channels 1-16, truncated messages, non-pitch-bend status bytes
- [ ] Task 2: Add pitch bend listener setup (AC: 1, 7, 8)
  - [ ] 2.1 Add `setup_midi_pitch_bend_listeners(on_pitch_bend: impl Fn(f64) + 'static) -> Result<MidiCleanupHandle, JsValue>` that attaches `midimessage` listeners filtering for pitch bend messages and calling `on_pitch_bend` with the normalized value
  - [ ] 2.2 Reuse existing `MidiCleanupHandle` pattern — share the single MIDI access request or call `request_midi_access` once for both note-on and pitch bend
- [ ] Task 3: Make slider externally drivable (AC: 3)
  - [ ] 3.1 Add optional `external_value: Option<Signal<f64>>` prop to `VerticalPitchSlider`
  - [ ] 3.2 When `external_value` changes, update internal `value` signal and thumb position (but do NOT call `on_change` — the MIDI handler calls session methods directly)
  - [ ] 3.3 Ensure pointer drag still works when external_value is also set (pointer takes priority during active drag)
- [ ] Task 4: Wire pitch bend into pitch matching view (AC: 4, 5, 6, 7, 8, 10)
  - [ ] 4.1 After AudioContext resume, call `setup_midi_pitch_bend_listeners` (same timing as Story 22.2's note-on setup)
  - [ ] 4.2 In the pitch bend callback: call `slider_on_change` callback with normalized value to drive session + start note
  - [ ] 4.3 Update slider external_value signal so thumb tracks pitch bend position
  - [ ] 4.4 Detect return-to-center: if in `PlayingTunable` state and value enters dead-zone → call `on_commit` with last deflected value
  - [ ] 4.5 Store `MidiCleanupHandle` in `StoredValue` for cleanup on unmount
  - [ ] 4.6 Guard with `is_midi_available()` check; log warning on setup failure, do not block training

## Dev Notes

### MIDI Pitch Bend Protocol

- Status byte: `0xE0` (channel 1) through `0xEF` (channel 16)
- Data byte 1 (index 1): LSB — 7 bits (0–127)
- Data byte 2 (index 2): MSB — 7 bits (0–127)
- Combined 14-bit value: `(MSB << 7) | LSB` → range 0–16383
- Center (no bend): 8192 (`LSB=0x00, MSB=0x40`)
- Normalization: `(combined - 8192) as f64 / 8192.0` → `[-1.0, +1.0]`

### Dead-Zone for Neutral Detection

Use a dead-zone of ±256 (out of 8192) for center detection, i.e. values in `[-0.03125, +0.03125]` normalized. This prevents accidental commits from mechanical centering noise on physical pitch bend wheels.

### Integration with Existing Code

**`midi_input.rs`** — Add `is_pitch_bend()`, `parse_pitch_bend()`, `setup_midi_pitch_bend_listeners()`. Consider refactoring `setup_midi_listeners` and the new function to share a single `request_midi_access` call if feasible (DRY), but a second independent call is acceptable if the refactor is complex.

**`pitch_slider.rs`** — Add `external_value: Option<Signal<f64>>` prop. When present, an `Effect` syncs the internal `value` RwSignal to match, but only when the user is NOT actively dragging (check `dragging` signal).

**`pitch_matching_view.rs`** — Wire pitch bend alongside the existing training loop. The pitch bend callback reuses `slider_on_change` (which calls `session.adjust_pitch()` and handles the `AwaitingSliderTouch → PlayingTunable` transition + note start). For commit-on-center-return, the callback checks session state and calls `on_commit`.

### Key Reuse Points — DO NOT Reinvent

- `slider_on_change` Callback already handles: adjust_pitch → frequency update → note start on first touch
- `on_commit` Rc closure already handles: commit_pitch → stop note → update stats → sync signals
- `MidiCleanupHandle` already handles: listener removal on drop
- `is_midi_available()` already handles: browser feature detection
- `StoredValue::new_local()` pattern from Story 22.2 for cleanup handle storage

### Patterns from Story 22.2

- MIDI setup is called in the async training loop, after AudioContext resume
- Setup failures are logged with `log::warn!`, never shown to user
- `is_midi_available()` check before `setup_midi_pitch_bend_listeners`
- Cleanup handle stored in `StoredValue::new_local(SendWrapper::new(handle))`
- No domain crate changes needed

### Project Structure Notes

- `web/src/adapters/midi_input.rs` — pitch bend parsing + listener setup
- `web/src/components/pitch_slider.rs` — external value prop
- `web/src/components/pitch_matching_view.rs` — wiring
- Domain crate unchanged — `session.adjust_pitch(value)` and `session.commit_pitch(value, ts)` used as-is

### Anti-Patterns to Avoid

- Do NOT add MIDI pitch bend support to the domain crate — this is a web-layer input concern
- Do NOT use `f32` for any pitch/frequency values — all `f64`
- Do NOT create a new `MidiAccess` request if you can share with note-on setup
- Do NOT remove or modify existing slider pointer/keyboard handling
- Do NOT add gamification or visual proximity feedback
- Do NOT use `spawn_local` for cleanup — use `on_cleanup` + `StoredValue`

### References

- [Source: docs/planning-artifacts/epics.md#Epic 22] — MIDI progressive enhancement model
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture] — `.detune` for pitch adjustment
- [Source: docs/planning-artifacts/ux-design-specification.md#Pitch Matching] — No visual proximity feedback, ear-only training
- [Source: docs/project-context.md] — Crate separation rules, numeric precision, error handling
- [Source: docs/implementation-artifacts/22-2-wire-midi-input-into-continuous-rhythm-matching-view.md] — MIDI wiring patterns, cleanup handle storage

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
