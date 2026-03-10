---
title: 'Vary Loudness — Fix, Extend Range, Add to Pitch Matching'
slug: 'vary-loudness-fix-extend-add-matching'
created: '2026-03-10'
status: 'ready-for-dev'
stepsCompleted: [1, 2, 3, 4]
tech_stack: [Rust, Leptos, WASM, OxiSynth worklet, Web Audio API]
files_to_modify:
  - domain/src/session/pitch_comparison_session.rs
  - domain/src/session/pitch_matching_session.rs
  - web/src/adapters/audio_soundfont.rs
  - web/src/components/pitch_matching_view.rs
  - web/src/app.rs
code_patterns:
  - calculate_target_amplitude function pattern from pitch_comparison_session
  - session_vary_loudness snapshot-at-start pattern
  - GainNode insertion between audio source and destination (oscillator pattern)
test_patterns:
  - test_amplitude_zero_vary_loudness / test_amplitude_with_vary_loudness pattern
  - LoudnessTestSettings mock struct for parameterized vary_loudness tests
---

# Tech-Spec: Vary Loudness — Fix, Extend Range, Add to Pitch Matching

**Created:** 2026-03-10

## Overview

### Problem Statement

The varyLoudness feature is incomplete in three ways: (1) the soundfont backend ignores the `amplitude_db` parameter entirely (`_amplitude_db` with underscore), so loudness variation only works with the oscillator; (2) the maximum range is ±5 dB which is too narrow; (3) pitch matching training doesn't use loudness variation at all.

### Solution

Fix the soundfont backend to apply amplitude_db as a gain adjustment, double the scaling constant from 5.0 to 10.0 for a ±10 dB range at max slider, and add vary_loudness support to PitchMatchingSession following the same pattern as PitchComparisonSession (target note only).

### Scope

**In Scope:**

- Fix `SoundFontNotePlayer::play()` to apply `amplitude_db` gain
- Change `AMPLITUDE_VARY_SCALING` from 5.0 to 10.0
- Add `session_vary_loudness` and amplitude calculation to `PitchMatchingSession`
- Add `target_amplitude_db` field to `PitchMatchingPlaybackData`
- Pass amplitude to target note playback in `pitch_matching_view.rs`
- Update existing tests, add new tests for pitch matching amplitude

**Out of Scope:**

- Varying the reference note loudness (stays at 0 dB)
- Changing the UI slider (0–100% stays the same, maps to wider range)
- Interval training mode loudness variation

## Context for Development

### Codebase Patterns

- Sessions snapshot settings at `start()` into `session_*` fields
- `calculate_target_amplitude(vary_loudness)` is a private function in `pitch_comparison_session.rs` — will need to be shared or duplicated
- `PitchComparisonPlaybackData` has `target_amplitude_db: AmplitudeDB`; `PitchMatchingPlaybackData` currently lacks it
- The web layer passes `AmplitudeDB` to `NotePlayer::play()` and `play_for_duration()`
- Oscillator backend applies amplitude via `10^(amplitude_db / 20)` gain calculation
- Soundfont backend currently ignores amplitude_db in `play()` (parameter prefixed with `_`)

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `domain/src/session/pitch_comparison_session.rs` | Reference pattern for vary_loudness + calculate_target_amplitude |
| `domain/src/session/pitch_matching_session.rs` | Add vary_loudness support here |
| `web/src/adapters/audio_soundfont.rs` | Fix amplitude_db application |
| `web/src/adapters/audio_oscillator.rs` | Reference for how amplitude gain is applied |
| `web/src/components/pitch_matching_view.rs` | Pass amplitude to target note playback |
| `web/src/components/pitch_comparison_view.rs` | Reference for how amplitude is passed in playback |
| `domain/src/types/amplitude.rs` | AmplitudeDB type (clamped -90..=12 dB) |
| `domain/src/ports.rs` | UserSettings trait with vary_loudness() |
| `web/src/app.rs` | Worklet node creation and audio graph wiring |

### Technical Decisions

- The UI slider (0–100%) and localStorage storage (0.0–1.0 float) remain unchanged; only the domain scaling constant changes.
- Loudness variation applies to the target note only, in both training modes.
- **SoundFont gain approach**: Insert a `web_sys::GainNode` between the worklet node and `ctx.destination()` in `app.rs`. Pass it to `SoundFontNotePlayer` to set gain before each noteOn. Notes are sequential (non-overlapping), so a single shared GainNode is sufficient. Uses same formula as oscillator: `gain_linear = 10^(amplitude_db / 20)`.
- **Function duplication over extraction**: Duplicate `calculate_target_amplitude()` (5 lines) into pitch_matching_session.rs rather than extracting to a shared module — keeps sessions self-contained.

## Implementation Plan

### Tasks

- [ ] Task 1: Extend amplitude range
  - File: `domain/src/session/pitch_comparison_session.rs`
  - Action: Change `AMPLITUDE_VARY_SCALING` from `5.0` to `10.0`
  - Notes: This is line 22. Update the doc comment to say "±10 dB at max".

- [ ] Task 2: Update existing pitch comparison amplitude tests
  - File: `domain/src/session/pitch_comparison_session.rs`
  - Action: Update `test_amplitude_with_vary_loudness` — with vary_loudness=0.5, max range changes from ±2.5 dB to ±5.0 dB. Update the `test_playback_data_amplitude_varies_when` test — the `LoudnessTestSettings { vary_loudness: 0.5 }` assertion bounds change from 2.5 to 5.0.
  - Notes: Search for all test assertions referencing the old 2.5 dB bound and update to 5.0.

- [ ] Task 3: Add `session_vary_loudness` to `PitchMatchingSession`
  - File: `domain/src/session/pitch_matching_session.rs`
  - Action: (a) Add `use crate::types::AmplitudeDB;` to imports. (b) Add `pub const AMPLITUDE_VARY_SCALING: f64 = 10.0;` constant. (c) Add `session_vary_loudness: f64` field to struct (after `session_note_range`). (d) Initialize to `0.0` in `new()`. (e) Snapshot in `start()`: `self.session_vary_loudness = settings.vary_loudness();`.

- [ ] Task 4: Add `target_amplitude_db` to `PitchMatchingPlaybackData`
  - File: `domain/src/session/pitch_matching_session.rs`
  - Action: Add `pub target_amplitude_db: AmplitudeDB` field to `PitchMatchingPlaybackData` struct.

- [ ] Task 5: Add `calculate_target_amplitude` and wire into challenge generation
  - File: `domain/src/session/pitch_matching_session.rs`
  - Action: (a) Add private function `calculate_target_amplitude(vary_loudness: f64) -> AmplitudeDB` — identical to the one in pitch_comparison_session.rs but using the local `AMPLITUDE_VARY_SCALING` constant. (b) In `generate_next_challenge()`, call `let target_amplitude_db = calculate_target_amplitude(self.session_vary_loudness);` and add it to the `PitchMatchingPlaybackData` struct literal.

- [ ] Task 6: Pass amplitude to tunable note playback in pitch matching view
  - File: `web/src/components/pitch_matching_view.rs`
  - Action: (a) In the slider `on_input` handler (~line 312-315), where the tunable note is first played with `AmplitudeDB::new(0.0)`, change to use `data.target_amplitude_db` from the current playback data. The `data` variable needs to be captured — read the playback data from the session at the point of first slider touch. (b) The reference note playback (~line 624) stays at `AmplitudeDB::new(0.0)`.
  - Notes: The tunable note is played via `note_player.borrow().play()` (not `play_for_duration`), so only the initial play call needs the amplitude. The continuous frequency adjustments don't change amplitude.

- [ ] Task 7: Insert GainNode into soundfont audio graph
  - File: `web/src/app.rs`
  - Action: In the worklet setup (~line 400-406), after creating the `AudioWorkletNode`, create a `GainNode` with initial gain 1.0. Connect worklet → gainNode → destination (instead of worklet → destination). Pass the `GainNode` alongside the `WorkletBridge` to `SoundFontNotePlayer`.
  - Notes: The GainNode needs to be wrapped in `Rc<RefCell<web_sys::GainNode>>` to be shared. Update `SoundFontNotePlayer::new()` to accept it.

- [ ] Task 8: Apply amplitude_db in SoundFontNotePlayer
  - File: `web/src/adapters/audio_soundfont.rs`
  - Action: (a) Add `gain_node: Rc<RefCell<web_sys::GainNode>>` field to `SoundFontNotePlayer`. (b) Update `new()` to accept and store the gain node. (c) In `play()`, rename `_amplitude_db` to `amplitude_db` and add `self.gain_node.borrow().gain().set_value(10_f32.powf(amplitude_db.raw_value() / 20.0));` before the `send_note_on` call.
  - Notes: Import `web_sys::GainNode` at top of file. The gain formula is identical to `audio_oscillator.rs:108`.

- [ ] Task 9: Add pitch matching amplitude tests
  - File: `domain/src/session/pitch_matching_session.rs`
  - Action: Add tests mirroring pitch comparison's amplitude tests: (a) `test_amplitude_zero_vary_loudness` — with default settings (vary_loudness=0.0), `target_amplitude_db` should be `AmplitudeDB::new(0.0)`. (b) `test_amplitude_with_vary_loudness` — with `vary_loudness=0.5`, `target_amplitude_db` should be within ±5.0 dB. (c) `test_playback_data_amplitude_varies_when_loudness_set` — start session with `LoudnessTestSettings { vary_loudness: 0.5 }`, verify playback data has non-trivial amplitude. Add `LoudnessTestSettings` mock struct (same pattern as pitch_comparison_session.rs tests).

- [ ] Task 10: Verify compilation and run domain tests
  - Action: Run `cargo clippy --workspace` and `cargo test -p domain` to verify all changes compile and tests pass.

## Acceptance Criteria

- [ ] AC1: Given `vary_loudness` is 0.0 in settings, when a pitch comparison target note is generated, then `target_amplitude_db` is `AmplitudeDB(0.0)`.

- [ ] AC2: Given `vary_loudness` is 1.0 in settings, when a pitch comparison target note is generated, then `target_amplitude_db` is within [-10.0, +10.0] dB.

- [ ] AC3: Given `vary_loudness` is 0.5 in settings, when a pitch comparison target note is generated, then `target_amplitude_db` is within [-5.0, +5.0] dB.

- [ ] AC4: Given `vary_loudness` is 0.0 in settings, when a pitch matching challenge is generated, then `target_amplitude_db` is `AmplitudeDB(0.0)`.

- [ ] AC5: Given `vary_loudness` is 0.5 in settings, when a pitch matching challenge is generated, then `target_amplitude_db` is within [-5.0, +5.0] dB.

- [ ] AC6: Given the soundfont backend is active and `amplitude_db` is non-zero, when a note is played, then the GainNode gain is set to `10^(amplitude_db / 20)` before the noteOn message.

- [ ] AC7: Given the soundfont backend is active and `amplitude_db` is 0.0, when a note is played, then the GainNode gain is set to 1.0 (unity gain).

- [ ] AC8: Given either training mode is active, when the reference note plays, then it always plays at `AmplitudeDB(0.0)` (unity gain) regardless of `vary_loudness` setting.

## Additional Context

### Dependencies

None — all changes are internal to existing crates. The `web_sys::GainNode` is already available (used by the oscillator backend).

### Testing Strategy

**Unit tests (domain crate):**
- Update existing pitch comparison amplitude tests for new ±10 dB range (Task 2)
- Add new pitch matching amplitude tests mirroring the comparison pattern (Task 9)
- Both use `LoudnessTestSettings` mock struct to parameterize `vary_loudness`

**Manual testing:**
- With soundfont active: set loudness variation to 100%, run pitch comparison training, verify audible volume differences on target notes
- With soundfont active: set loudness variation to 100%, run pitch matching training, verify audible volume differences on tunable notes
- With oscillator active: same tests to verify no regression
- With loudness variation at 0%: verify both modes play at consistent volume

**Compilation check:**
- `cargo clippy --workspace` — no new warnings
- `cargo test -p domain` — all tests pass

### Notes

- The `SoundFontNotePlayer` gain reset: since gain is set before each noteOn and notes are sequential, there's no need to explicitly reset gain after noteOff. The next play call sets the appropriate gain.
- At ±10 dB max, the loudest target is ~3.16x the reference volume and the quietest is ~0.316x. This is a meaningful but not extreme range for ear training.
- Future consideration: if interval training mode is added, it should also use this pattern for consistency.
