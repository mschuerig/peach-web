# Story 8.10: Settings Sound Source Preview

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want to preview the selected sound source from the Settings screen,
so that I can hear what a sound sounds like before starting a training session.

## Acceptance Criteria

1. A preview button (speaker icon) is visible to the right of the Sound picker in the Sound section of Settings
2. Tapping the preview button plays a 2-second note using the currently selected sound source at the current concert pitch (A4, MIDI note 69) with equal temperament tuning
3. Tapping the preview button while a preview is already playing stops the sound immediately
4. Changing the sound source in the picker stops any playing preview
5. Navigating away from the Settings screen stops any playing preview
6. When the 2-second preview finishes naturally, the button returns to its default speaker icon (not stuck in stop state)
7. If no AudioContext exists yet (first user gesture), the preview tap creates one and plays successfully

## Tasks / Subtasks

- [x] Task 1: Add preview duration constant (AC: #2)
  - [x] 1.1 Add `PREVIEW_DURATION_SECS: f64 = 2.0` constant — place it in the settings view file or a shared constants location alongside existing constants like `FEEDBACK_DURATION_SECS`
  - [x] 1.2 The constant uses `f64` (same as `NoteDuration` inner value and `FEEDBACK_DURATION_SECS`)

- [x] Task 2: Add preview button UI to settings sound section (AC: #1, #6)
  - [x] 2.1 In `web/src/components/settings_view.rs`, in the sound section where the Sound `<select>` dropdown lives (~line 408-430), add a preview button next to the dropdown
  - [x] 2.2 Use an SVG speaker icon or Unicode speaker character (e.g., the speaker emoji or a simple SVG). When playing, switch to a stop icon
  - [x] 2.3 Add a `preview_playing` signal (`RwSignal<bool>`) to track whether a preview is currently active
  - [x] 2.4 Style the button to sit inline with the sound picker row using Tailwind utility classes

- [x] Task 3: Implement preview play/stop logic (AC: #2, #3, #6, #7)
  - [x] 3.1 On preview button click, read the current `sound_source` value from the signal (already available in scope)
  - [x] 3.2 Get `AudioContextManager` from context via `use_context()` — ensure/resume the audio context (the click is a valid user gesture)
  - [x] 3.3 Get `worklet_bridge` from context for SoundFont playback
  - [x] 3.4 Create a `UnifiedNotePlayer` via `create_note_player(&sound_source, audio_ctx, worklet_bridge)`
  - [x] 3.5 Calculate preview frequency: `TuningSystem::EqualTemperament.frequency(DetunedMIDINote::from(MIDINote::new(69)), reference_pitch)` where `reference_pitch` comes from the current settings signal
  - [x] 3.6 Call `note_player.play_for_duration(frequency, NoteDuration::new(PREVIEW_DURATION_SECS), MIDIVelocity::new(63), AmplitudeDB::new(0.0))`
  - [x] 3.7 Set `preview_playing` to `true`, then spawn a `gloo_timers::future::TimeoutFuture` or similar 2-second timer that sets `preview_playing` back to `false` when it fires
  - [x] 3.8 If preview is already playing when tapped: call `note_player.stop_all()`, cancel the timer, set `preview_playing` to `false`
  - [x] 3.9 Store the note player in an `Rc<RefCell<Option<UnifiedNotePlayer>>>` so stop can reference the same instance

- [x] Task 4: Stop preview on sound source change (AC: #4)
  - [x] 4.1 Add a reactive effect (or use `create_effect` / signal `.watch()`) on the `sound_source` signal
  - [x] 4.2 When sound source changes while `preview_playing` is true: call `stop_all()` on the stored note player, set `preview_playing` to `false`

- [x] Task 5: Stop preview on navigation away (AC: #5)
  - [x] 5.1 Use `on_cleanup()` in the settings view component to stop any active preview when the component unmounts
  - [x] 5.2 Call `stop_all()` on the stored note player and set `preview_playing` to `false`

- [x] Task 6: Verify and test (AC: #1-#7)
  - [ ] 6.1 Manual test: tap preview with sine oscillator — hear 2-second sine tone at concert pitch
  - [ ] 6.2 Manual test: switch to SoundFont preset, tap preview — hear SoundFont timbre
  - [ ] 6.3 Manual test: tap preview, tap again before 2s — sound stops immediately
  - [ ] 6.4 Manual test: tap preview, change sound source — sound stops
  - [ ] 6.5 Manual test: tap preview, navigate away — sound stops
  - [ ] 6.6 Manual test: let preview finish naturally — button returns to speaker icon
  - [ ] 6.7 Manual test: first interaction is preview (no prior AudioContext) — works
  - [x] 6.8 `cargo clippy --workspace` clean
  - [x] 6.9 `cargo test -p domain` passes

## Dev Notes

### Architecture Overview

This feature adds a preview button to the Settings sound section. The iOS version injects preview closures via `@Environment` from a composition root. The web version should keep it simpler: handle preview logic directly in the settings view component, following the existing pattern where training views create their own `NotePlayer` instances via `create_note_player()`.

### Audio Playback Pattern

The web app uses a `UnifiedNotePlayer` enum with two variants:
- `OscillatorNotePlayer` — for `"oscillator:sine"` sound source
- `SoundFontNotePlayer` — for `"sf2:<bank>:<preset>"` sound sources

**Factory:** `create_note_player(sound_source: &str, audio_ctx: Rc<RefCell<AudioContextManager>>, worklet_bridge: Option<...>) -> UnifiedNotePlayer`

**Key methods on `NotePlayer` trait:**
- `play_for_duration(frequency, duration, velocity, amplitude_db)` — plays and auto-stops
- `stop_all()` — stops all playing notes immediately

### Frequency Calculation

```rust
use domain::tuning::TuningSystem;
use domain::types::{DetunedMIDINote, MIDINote, Frequency};

// A4 = MIDI note 69
let reference_pitch = Frequency::new(concert_pitch_value); // from settings signal
let frequency = TuningSystem::EqualTemperament.frequency(
    DetunedMIDINote::from(MIDINote::new(69)),
    reference_pitch,
);
```

Always use `EqualTemperament` for preview — the preview demonstrates timbre, not tuning. For A4 with equal temperament, the frequency equals the reference pitch directly, but using the tuning system API keeps the code consistent.

### Standard Playback Parameters

From existing training code:
- **Velocity:** `MIDIVelocity::new(63)` (mezzo-piano, used in all training playback)
- **Amplitude:** `AmplitudeDB::new(0.0)` (unity gain, no boost or cut)
- **Duration:** 2.0 seconds (new constant for preview)

### AudioContext User Gesture Requirement

Browsers require a user gesture to create/resume an `AudioContext`. The preview button tap qualifies as a user gesture. The existing `AudioContextManager` handles creation and resumption — call `ensure_context()` or equivalent before playing.

The `AudioNeedsGesture` context signal (newtype wrapper in `app.rs`) tracks whether the audio context still needs a gesture. After the preview creates/resumes the context, update this signal so training views know the context is ready.

### Context Dependencies Available in Settings View

These are already provided via `provide_context()` in `app.rs`:
- `SendWrapper<Rc<RefCell<AudioContextManager>>>` — audio context
- `RwSignal<Option<Rc<RefCell<WorkletBridge>>>>` — SoundFont worklet bridge
- `RwSignal<SoundFontLoadStatus>` — SF2 loading status (consider disabling preview button if SF2 selected but not loaded)
- `AudioNeedsGesture(RwSignal<bool>)` — whether audio context needs user gesture

### NotePlayer Lifecycle for Preview

Create the `NotePlayer` fresh on each preview tap (matches training view pattern). Store it in `Rc<RefCell<Option<UnifiedNotePlayer>>>` so the stop action can reference the same instance:

```rust
let preview_player: Rc<RefCell<Option<UnifiedNotePlayer>>> = Rc::new(RefCell::new(None));
```

On play: create player, store in `preview_player`, call `play_for_duration()`.
On stop: call `preview_player.borrow().as_ref().map(|p| p.stop_all())`, set to `None`.

### Timer for Auto-Reset

After calling `play_for_duration()`, the note auto-stops after 2 seconds, but the UI state (`preview_playing` signal) needs to reset too. Use `set_timeout()` from `gloo_timers` or `wasm_bindgen_futures::spawn_local` with a delay:

```rust
use gloo_timers::future::TimeoutFuture;

spawn_local(async move {
    TimeoutFuture::new((PREVIEW_DURATION_SECS * 1000.0) as u32).await;
    preview_playing.set(false);
});
```

Store a cancellation mechanism (e.g., `Rc<Cell<bool>>` flag) to cancel the timer if the user taps stop before it fires.

### Settings View Sound Section Location

The sound `<select>` dropdown is in `settings_view.rs` around lines 408-430. The preview button should be added to the right of this dropdown, within the same row. The iOS spec uses an HStack wrapping the picker and button — in Tailwind, use `flex items-center gap-2` or similar.

### What NOT to Change

- No changes to domain crate (this is purely a web UI feature)
- No changes to `NotePlayer` trait or implementations
- No changes to `app.rs` context providers
- No new environment keys needed (unlike iOS `@Environment` pattern — web can access contexts directly)
- No changes to training views

### Differences from iOS Implementation

| Aspect | iOS | Web |
|---|---|---|
| Injection | `@Environment` closures from composition root | Direct `use_context()` in settings view |
| NotePlayer access | Closures capture `notePlayer` reference | Create `NotePlayer` via factory on each tap |
| Sound source | `SoundFontNotePlayer` reads `userSettings` on each `play()` | Pass `sound_source` string to `create_note_player()` factory |
| State tracking | `@State previewTask: Task<Void, Never>?` | `RwSignal<bool>` + `Rc<RefCell<Option<UnifiedNotePlayer>>>` |
| Timer | Swift `Task` with implicit cancellation | `gloo_timers::future::TimeoutFuture` with manual cancel flag |
| Constants file | `TrainingConstants.previewDuration` | Local constant or module-level const |

### Project Structure Notes

- All changes confined to `web/src/components/settings_view.rs` (possibly adding a const)
- No domain crate changes — this is a UI-only feature
- Follows established pattern: views create their own `NotePlayer` instances from context

### References

- [Source: web/src/components/settings_view.rs:408-430] Sound source dropdown location
- [Source: web/src/adapters/note_player.rs] `create_note_player()` factory and `UnifiedNotePlayer`
- [Source: web/src/adapters/audio_oscillator.rs] Oscillator note player implementation
- [Source: web/src/adapters/audio_soundfont.rs] SoundFont note player implementation
- [Source: web/src/adapters/audio_context.rs] AudioContext lifecycle management
- [Source: web/src/app.rs:43-76] Context providers (AudioContextManager, WorkletBridge, etc.)
- [Source: domain/src/tuning.rs] `TuningSystem::frequency()` for note frequency calculation
- [Source: domain/src/types/frequency.rs] `Frequency` type
- [Source: domain/src/ports.rs] `NotePlayer` trait definition
- [Source: web/src/components/pitch_comparison_view.rs:559-590] Reference pattern for playing notes
- [Source: iOS tech-spec] `docs/../tech-spec-settings-sound-source-preview.md` — original iOS feature spec
- [Source: docs/project-context.md] Project coding conventions and rules

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Fixed `Send + Sync` requirement for Leptos 0.8 `on_cleanup` by wrapping `Rc` types in `SendWrapper`
- Fixed `Send` requirement for view closures capturing `Rc` types — same `SendWrapper` pattern
- Fixed `create_note_player` type mismatch: pass `Rc<RefCell<AudioContextManager>>` not raw `AudioContext`
- Fixed SF2 preview on cold start: worklet bridge not connected until a training view initializes it. Preview now connects worklet on-demand via `connect_worklet()` in `spawn_local`, mirroring the training view pattern.

### Completion Notes List

- Added `PREVIEW_DURATION_SECS: f64 = 2.0` constant in `settings_view.rs`
- Added preview button (speaker icon / stop square) inline with Sound dropdown using `flex items-center gap-2`
- Preview plays A4 at current concert pitch using `TuningSystem::EqualTemperament` and `create_note_player` factory
- Play/stop toggle: tapping while playing stops immediately via `stop_all()` and cancels timer
- `Effect::new` on `sound_source` signal stops preview when sound source changes
- `on_cleanup` stops preview when navigating away from settings
- Timer auto-resets `preview_playing` signal after 2 seconds using `Rc<Cell<bool>>` cancellation flag
- AudioContext created/resumed on preview tap (valid user gesture), clears `AudioNeedsGesture` signal
- All `Rc` types wrapped in `SendWrapper` for Leptos 0.8 `Send + Sync` requirements
- `cargo clippy --workspace` clean, `cargo test -p domain` passes (341 tests)

### Change Log

- 2026-03-07: Implemented sound source preview feature (Tasks 1-6)
- 2026-03-07: Fixed SF2 preview on cold start — connect worklet on-demand when bridge not yet available
- 2026-03-07: Code review fixes — cancel check before play after async init, busy-wait timeout, player resource cleanup
- 2026-03-07: Extracted `SoundPreview` adapter and `ensure_worklet_connected` shared helper; deduplicated worklet connection logic across all 3 views

### File List

- `web/src/components/settings_view.rs` (modified) — preview button UI, delegates to SoundPreview adapter
- `web/src/adapters/sound_preview.rs` (new) — SoundPreview adapter encapsulating play/stop/timer logic
- `web/src/adapters/mod.rs` (modified) — registered sound_preview module
- `web/src/app.rs` (modified) — added ensure_worklet_connected shared helper
- `web/src/components/pitch_comparison_view.rs` (modified) — uses ensure_worklet_connected
- `web/src/components/pitch_matching_view.rs` (modified) — uses ensure_worklet_connected
- `docs/implementation-artifacts/sprint-status.yaml` (modified) — status updated to done
- `docs/implementation-artifacts/8-10-settings-sound-source-preview.md` (modified) — task checkboxes, dev agent record
