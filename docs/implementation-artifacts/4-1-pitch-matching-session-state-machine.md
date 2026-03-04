# Story 4.1: Pitch Matching Session State Machine

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the PitchMatchingSession state machine implemented with challenge generation, pitch adjustment, and commit logic,
so that the domain logic correctly drives the pitch matching training experience.

## Acceptance Criteria

1. **AC1 — Idle state:** Given PitchMatchingSession is constructed with injected dependencies (profile, observers, resettables), when it is in `idle` state, then no training loop is running and observable state shows idle defaults.

2. **AC2 — Start transitions:** Given PitchMatchingSession is idle, when `start(intervals, settings)` is called, then state transitions to `PlayingReference`, a challenge is generated with a random reference note in the configured range, and playback data is available for the web layer.

3. **AC3 — Challenge generation:** Given `generate_challenge` is called, when generating the challenge, then the reference note is random within the note range (adjusted for interval transposition), the target note is the reference transposed by the selected interval, and `initial_cent_offset` is random in [-20.0, +20.0] cents.

4. **AC4 — Reference finished transition:** Given state is `PlayingReference`, when `on_reference_finished()` is called, then state transitions to `AwaitingSliderTouch` and tunable playback data is available (target frequency with initial cent offset applied).

5. **AC5 — First slider interaction:** Given state is `AwaitingSliderTouch`, when `adjust_pitch(value)` is called (first slider interaction), then state transitions to `PlayingTunable`.

6. **AC6 — Pitch adjustment:** Given state is `PlayingTunable`, when `adjust_pitch(value)` is called with value in [-1.0, +1.0], then the method returns the adjusted frequency calculated as `target_frequency * 2^((value * 20.0) / 1200.0)` for the web layer to apply to the PlaybackHandle.

7. **AC7 — Commit pitch:** Given state is `PlayingTunable`, when `commit_pitch(value, timestamp)` is called (slider released), then `user_cent_error` is calculated as `value * 20.0` cents, a `CompletedPitchMatching` is created with all fields, all observers receive `pitch_matching_completed(result)`, and state transitions to `ShowingFeedback`.

8. **AC8 — Feedback to next challenge:** Given state is `ShowingFeedback`, when `on_feedback_finished()` is called (after 400ms), then the next challenge is generated and state transitions to `PlayingReference` (loop continues).

9. **AC9 — Stop:** Given session is running, when `stop()` is called, then state returns to idle, all session state is cleared.

10. **AC10 — Observer panic isolation:** Given an observer panics during `pitch_matching_completed`, when the notification is broadcast, then the panic does not propagate, remaining observers still receive the event, and the session continues normally.

11. **AC11 — PitchMatchingObserver trait:** Given the domain ports, when a PitchMatchingObserver trait is defined, then it has a single method `pitch_matching_completed(&mut self, completed: &CompletedPitchMatching)`.

12. **AC12 — PitchMatchingRecord:** Given a completed pitch matching attempt, when `PitchMatchingRecord::from_completed()` is called, then a flat persistence record is created with fields: `reference_note`, `target_note`, `initial_cent_offset`, `user_cent_error`, `interval`, `tuning_system`, `timestamp`.

13. **AC13 — TrainingDataStore extension:** Given the `TrainingDataStore` trait, when pitch matching methods are added, then `save_pitch_matching()` and `fetch_all_pitch_matchings()` are available.

14. **AC14 — Reset training data:** Given `reset_training_data()` is called, when it executes, then the session stops, the profile's matching accumulators are reset via `reset_matching()`, and all resettables are called.

## Tasks / Subtasks

- [ ] Task 1: Add PitchMatchingObserver trait to ports.rs (AC: 11)
  - [ ] 1.1 Add `use crate::training::CompletedPitchMatching;` import to ports.rs
  - [ ] 1.2 Define `PitchMatchingObserver` trait with `fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching)`
  - [ ] 1.3 Export `PitchMatchingObserver` from `lib.rs`

- [ ] Task 2: Add PitchMatchingRecord to records.rs (AC: 12)
  - [ ] 2.1 Add `use crate::training::CompletedPitchMatching;` import
  - [ ] 2.2 Define `PitchMatchingRecord` struct with fields: `reference_note: u8`, `target_note: u8`, `initial_cent_offset: f64`, `user_cent_error: f64`, `interval: u8`, `tuning_system: String`, `timestamp: String`
  - [ ] 2.3 Derive `Clone, Debug, PartialEq, Serialize, Deserialize` (same as `ComparisonRecord`)
  - [ ] 2.4 Implement `from_completed(&CompletedPitchMatching) -> Self` following the same pattern as `ComparisonRecord::from_completed`
  - [ ] 2.5 Export `PitchMatchingRecord` from `lib.rs`
  - [ ] 2.6 Write unit tests: field extraction, interval calculation, serde roundtrip, interval-exceeds-octave default

- [ ] Task 3: Extend TrainingDataStore trait (AC: 13)
  - [ ] 3.1 Add `use crate::records::PitchMatchingRecord;` import to ports.rs
  - [ ] 3.2 Add `fn save_pitch_matching(&self, record: PitchMatchingRecord) -> Result<(), StorageError>` to TrainingDataStore
  - [ ] 3.3 Add `fn fetch_all_pitch_matchings(&self) -> Result<Vec<PitchMatchingRecord>, StorageError>` to TrainingDataStore
  - [ ] 3.4 Update any existing TrainingDataStore implementations (currently none in domain crate — the web crate's `IndexedDbStore` does not implement this trait directly, so no breaking changes)

- [ ] Task 4: Create PitchMatchingSession state machine (AC: 1,2,3,4,5,6,7,8,9,10,14)
  - [ ] 4.1 Create `domain/src/session/pitch_matching_session.rs`
  - [ ] 4.2 Define `PitchMatchingSessionState` enum: `Idle`, `PlayingReference`, `AwaitingSliderTouch`, `PlayingTunable`, `ShowingFeedback`
  - [ ] 4.3 Define `PitchMatchingPlaybackData` struct: `reference_frequency: Frequency`, `tunable_frequency: Frequency`, `duration: NoteDuration`
  - [ ] 4.4 Define `PitchMatchingSession` struct with fields (see Dev Notes)
  - [ ] 4.5 Implement `new(profile, observers, resettables)` constructor
  - [ ] 4.6 Implement observable state accessors: `state()`, `show_feedback()`, `last_completed()`, `current_challenge()`, `current_playback_data()`, `current_interval()`
  - [ ] 4.7 Implement `start(intervals, settings)` — guard Idle state, snapshot settings, generate first challenge, transition to PlayingReference
  - [ ] 4.8 Implement `on_reference_finished()` — guard PlayingReference, compute tunable frequency with initial cent offset, transition to AwaitingSliderTouch
  - [ ] 4.9 Implement `adjust_pitch(value: f64) -> Option<Frequency>` — if AwaitingSliderTouch transition to PlayingTunable, if PlayingTunable calculate and return adjusted frequency
  - [ ] 4.10 Implement `commit_pitch(value: f64, timestamp: String)` — guard PlayingTunable, calculate user_cent_error, create CompletedPitchMatching, notify observers, update profile, transition to ShowingFeedback
  - [ ] 4.11 Implement `on_feedback_finished()` — guard ShowingFeedback, generate next challenge, transition to PlayingReference
  - [ ] 4.12 Implement `stop()` — return to Idle, clear all session state (noop if already Idle)
  - [ ] 4.13 Implement `reset_training_data()` — stop, reset matching accumulators, call resettables
  - [ ] 4.14 Implement private `generate_challenge(interval)` helper
  - [ ] 4.15 Implement private `random_interval()` helper (same pattern as ComparisonSession)
  - [ ] 4.16 Implement private `notify_observers(completed)` with panic isolation via `catch_unwind`

- [ ] Task 5: Update session/mod.rs and lib.rs exports (AC: all)
  - [ ] 5.1 Add `pub mod pitch_matching_session;` to `session/mod.rs`
  - [ ] 5.2 Add `pub use pitch_matching_session::{PitchMatchingPlaybackData, PitchMatchingSession, PitchMatchingSessionState};` to `session/mod.rs`
  - [ ] 5.3 Add re-exports in `lib.rs` for `PitchMatchingSession`, `PitchMatchingSessionState`, `PitchMatchingPlaybackData`, `PitchMatchingObserver`, `PitchMatchingRecord`

- [ ] Task 6: Write comprehensive tests (AC: all)
  - [ ] 6.1 Mock types: `MockPitchMatchingObserver`, `PanickingPitchMatchingObserver`, `MockResettable`, `DefaultTestSettings` (reuse pattern from comparison_session tests)
  - [ ] 6.2 Test idle state defaults (AC1)
  - [ ] 6.3 Test start transitions to PlayingReference (AC2)
  - [ ] 6.4 Test start generates challenge with valid playback data (AC2, AC3)
  - [ ] 6.5 Test start panics when not idle (guard)
  - [ ] 6.6 Test start panics with empty intervals (guard)
  - [ ] 6.7 Test on_reference_finished transitions to AwaitingSliderTouch (AC4)
  - [ ] 6.8 Test on_reference_finished provides tunable frequency with offset (AC4)
  - [ ] 6.9 Test adjust_pitch from AwaitingSliderTouch transitions to PlayingTunable (AC5)
  - [ ] 6.10 Test adjust_pitch returns correct frequency calculation (AC6)
  - [ ] 6.11 Test adjust_pitch at value=0.0 returns target frequency (AC6)
  - [ ] 6.12 Test adjust_pitch at boundaries ±1.0 (AC6)
  - [ ] 6.13 Test commit_pitch creates CompletedPitchMatching and notifies observers (AC7)
  - [ ] 6.14 Test commit_pitch transitions to ShowingFeedback (AC7)
  - [ ] 6.15 Test commit_pitch user_cent_error calculation (AC7)
  - [ ] 6.16 Test on_feedback_finished generates next challenge and loops (AC8)
  - [ ] 6.17 Test full lifecycle: Idle → PlayingReference → AwaitingSliderTouch → PlayingTunable → ShowingFeedback → PlayingReference
  - [ ] 6.18 Test stop returns to idle and clears state (AC9)
  - [ ] 6.19 Test stop from idle is noop (AC9)
  - [ ] 6.20 Test observer panic isolation (AC10)
  - [ ] 6.21 Test reset_training_data stops session, resets matching, calls resettables (AC14)
  - [ ] 6.22 Test challenge generation respects note range with interval transposition (AC3)
  - [ ] 6.23 Test initial_cent_offset is within [-20.0, +20.0] range (AC3)
  - [ ] 6.24 State guard tests: invalid state transitions panic with descriptive messages

- [ ] Task 7: Verify and validate (AC: all)
  - [ ] 7.1 `cargo clippy -p domain` — zero warnings
  - [ ] 7.2 `cargo clippy -p web` — zero warnings (no web changes expected, but verify no breakage from TrainingDataStore extension)
  - [ ] 7.3 `cargo test -p domain` — all tests pass (existing 254 + new tests)
  - [ ] 7.4 `trunk build` — successful WASM compilation

## Dev Notes

### Core Approach: Domain-Only State Machine Following ComparisonSession Pattern

This story creates the `PitchMatchingSession` state machine in the domain crate. It follows the **exact same architectural pattern** as the existing `ComparisonSession` — a pure domain state machine with injected dependencies, observable state, observer notifications with panic isolation, and session-level settings snapshot.

**Key difference from ComparisonSession:** The pitch matching loop has a different shape. Instead of "play note 1 → play note 2 → await answer (higher/lower)", it's "play reference → await slider touch → play tunable (continuously) → commit pitch → show feedback". The tunable note plays **indefinitely** until the user releases the slider, and the pitch is adjusted **in real time** during playback.

**Critical design decision:** The domain state machine does NOT hold a `PlaybackHandle` or call `adjust_frequency()` directly. It cannot — the domain crate has no access to `web-sys` or the NotePlayer trait's concrete implementation. Instead, the session calculates and returns frequencies, and the **web layer** (the async training loop in the web crate) is responsible for calling `NotePlayer.play()`, `handle.adjust_frequency()`, and `handle.stop()`. This is the same pattern as ComparisonSession, which provides `ComparisonPlaybackData` for the web layer to use.

### PitchMatchingSession Struct Design

```rust
pub struct PitchMatchingSession {
    state: PitchMatchingSessionState,
    profile: Rc<RefCell<PerceptualProfile>>,
    observers: Vec<Box<dyn PitchMatchingObserver>>,
    resettables: Vec<Box<dyn Resettable>>,

    // Session-level state (snapshot from settings at start)
    session_intervals: HashSet<DirectedInterval>,
    session_tuning_system: TuningSystem,
    session_reference_pitch: Frequency,
    session_note_duration: NoteDuration,
    session_note_range: NoteRange,

    // Current challenge state
    current_challenge: Option<PitchMatchingChallenge>,
    current_playback_data: Option<PitchMatchingPlaybackData>,
    last_completed: Option<CompletedPitchMatching>,

    // Target frequency for pitch calculation (the "correct answer" frequency)
    target_frequency: Option<Frequency>,

    // Observable feedback state
    show_feedback: bool,
}
```

**Note:** No `session_vary_loudness` — pitch matching does not vary loudness (velocity is always 63, amplitude 0 dB). No `session_best_cent_difference` or `is_last_answer_correct` — pitch matching feedback shows cent error magnitude/direction, not correct/incorrect.

### State Machine Flow

```
Idle
  → start(intervals, settings)
  → generate challenge, compute reference frequency
  → PlayingReference
      [web layer: play reference note for duration]
  → on_reference_finished()
  → compute tunable frequency (target + initial cent offset)
  → AwaitingSliderTouch
      [web layer: play tunable note indefinitely]
  → adjust_pitch(value) [first touch]
  → PlayingTunable
      [web layer: update frequency via handle.adjust_frequency()]
  → adjust_pitch(value) [continuous]
      → returns adjusted frequency for web layer
  → commit_pitch(value, timestamp)
  → calculate error, notify observers, update profile
  → ShowingFeedback
      [web layer: stop tunable note, show feedback for 400ms]
  → on_feedback_finished()
  → generate next challenge
  → PlayingReference (loop)
```

### Frequency Calculations (Blueprint §7.2)

**Reference frequency:** Standard tuning system calculation:
```rust
let ref_frequency = tuning_system.frequency_for_note(challenge.reference_note(), reference_pitch);
```

**Target frequency (the "correct answer"):**
```rust
let target_frequency = tuning_system.frequency_for_note(challenge.target_note(), reference_pitch);
```

**Initial tunable frequency** (target + random cent offset):
```rust
let initial_offset = challenge.initial_cent_offset(); // [-20.0, +20.0]
let tunable_frequency = Frequency::new(
    target_frequency.raw_value() * 2.0_f64.powf(initial_offset / 1200.0)
);
```

**Adjusted frequency** (during slider drag, value in [-1.0, +1.0]):
```rust
let cent_offset = value * 20.0; // Maps slider range to ±20 cents
let adjusted = Frequency::new(
    target_frequency.raw_value() * 2.0_f64.powf(cent_offset / 1200.0)
);
```

**User cent error** (on commit):
```rust
let user_cent_error = value * 20.0; // Signed: positive = sharp, negative = flat
```

**Important:** The `user_cent_error` is directly `value * 20.0` because the slider value IS the cent offset from the target frequency. The blueprint's `1200.0 * log2(userFrequency / targetFrequency)` formula would yield the same result — it's just the inverse of the frequency calculation. Using `value * 20.0` directly is simpler and avoids floating point drift.

### Challenge Generation (Blueprint §7.2)

```rust
fn generate_challenge(&self, interval: DirectedInterval) -> PitchMatchingChallenge {
    // Ensure transposed note stays in range
    let (min_raw, max_raw) = match interval.direction {
        Direction::Up => (
            self.session_note_range.min().raw_value(),
            self.session_note_range.max().raw_value()
                .saturating_sub(interval.interval.semitones()),
        ),
        Direction::Down => (
            self.session_note_range.min().raw_value()
                .saturating_add(interval.interval.semitones()),
            self.session_note_range.max().raw_value(),
        ),
    };

    let reference_note = MIDINote::random(min_raw..=max_raw);
    let target_note = reference_note.transposed(interval)
        .expect("range-adjusted reference ensures valid transposition");
    let initial_cent_offset = rand::random::<f64>() * 40.0 - 20.0; // [-20.0, +20.0]

    PitchMatchingChallenge::new(reference_note, target_note, initial_cent_offset)
}
```

### adjust_pitch Return Value Design

`adjust_pitch(value: f64) -> Option<Frequency>` returns `Some(frequency)` when the web layer should update the playing note's frequency, and `None` when the call is invalid (wrong state). This avoids the web layer needing to compute frequencies — the domain owns all pitch math.

For the `AwaitingSliderTouch → PlayingTunable` transition, the method returns `Some(adjusted_frequency)` — the web layer uses this as the initial frequency to start playing the tunable note, then continues calling `adjust_pitch()` for subsequent slider movements.

### PitchMatchingPlaybackData

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchMatchingPlaybackData {
    pub reference_frequency: Frequency,
    pub tunable_frequency: Frequency,  // Initial tunable frequency (with offset)
    pub duration: NoteDuration,        // Reference note duration only
}
```

The `tunable_frequency` is the initial frequency including the random cent offset. The web layer plays the reference at `reference_frequency` for `duration`, then starts the tunable at `tunable_frequency` indefinitely.

### Observer Notification

Same pattern as ComparisonSession — panic isolation via `catch_unwind`:

```rust
fn notify_observers(&mut self, completed: &CompletedPitchMatching) {
    for observer in &mut self.observers {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observer.pitch_matching_completed(completed);
        }));
        if let Err(e) = result {
            eprintln!("Observer panicked: {:?}", e);
        }
    }
}
```

### Profile Update in commit_pitch

After creating `CompletedPitchMatching` and notifying observers, update the profile:

```rust
self.profile.borrow_mut().update_matching(
    completed.target_note(),
    completed.user_cent_error(),
);
```

`PerceptualProfile::update_matching()` already exists (implemented in Epic 3 stories) — uses Welford's algorithm on `abs(cent_error)`.

### PitchMatchingRecord Design

Flat persistence record following the same pattern as `ComparisonRecord`:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PitchMatchingRecord {
    pub reference_note: u8,
    pub target_note: u8,
    pub initial_cent_offset: f64,
    pub user_cent_error: f64,
    pub interval: u8,
    pub tuning_system: String,
    pub timestamp: String,
}

impl PitchMatchingRecord {
    pub fn from_completed(completed: &CompletedPitchMatching) -> Self {
        let interval = Interval::between(
            completed.reference_note(),
            completed.target_note(),
        )
        .map(|i| i.semitones())
        .unwrap_or(0);

        let tuning_system = match completed.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        };

        Self {
            reference_note: completed.reference_note().raw_value(),
            target_note: completed.target_note().raw_value(),
            initial_cent_offset: completed.initial_cent_offset(),
            user_cent_error: completed.user_cent_error(),
            interval,
            tuning_system: tuning_system.to_string(),
            timestamp: completed.timestamp().to_string(),
        }
    }
}
```

### Velocity and Amplitude Constants

Per the blueprint §7.2:
- Velocity: 63 (constant for pitch matching — not configurable)
- Amplitude: 0.0 dB (no loudness variation in pitch matching)

These are NOT stored in the session — they are constants the web layer uses when calling `NotePlayer.play()`. Define as:

```rust
/// MIDI velocity for pitch matching playback (fixed at 63).
pub const PITCH_MATCHING_VELOCITY: u8 = 63;
```

The web layer will use `MIDIVelocity::new(PITCH_MATCHING_VELOCITY)` and `AmplitudeDB::new(0.0)`.

### What NOT to Implement

- **No web crate changes** — this is domain-only. The web adapter (audio playback, IndexedDB persistence, UIObserver bridge) is for stories 4.2-4.4.
- **No actual audio playback** — the session provides data; the web layer plays audio.
- **No UI components** — stories 4.2 (slider) and 4.3 (training UI) handle this.
- **No IndexedDB persistence** — story 4.4 handles the web adapter for saving records.
- **No session best tracking** — unlike comparison training, pitch matching doesn't have a "correct/incorrect" binary; it has cent error magnitude. Session-level "best" tracking is not specified in the blueprint for pitch matching.

### Project Structure Notes

**New files:**
- `domain/src/session/pitch_matching_session.rs` — PitchMatchingSession state machine, PitchMatchingSessionState enum, PitchMatchingPlaybackData struct, all tests

**Modified files:**
- `domain/src/ports.rs` — Add `PitchMatchingObserver` trait, extend `TrainingDataStore` with pitch matching methods
- `domain/src/records.rs` — Add `PitchMatchingRecord` struct with `from_completed`
- `domain/src/session/mod.rs` — Add pitch_matching_session module and re-exports
- `domain/src/lib.rs` — Add re-exports for new public types

**No changes to:**
- Any web crate files (this is domain-only)
- `domain/src/profile.rs` — `update_matching()` already exists
- `domain/src/training/pitch_matching.rs` — `PitchMatchingChallenge` and `CompletedPitchMatching` already exist
- `domain/src/types/` — All needed types exist
- `domain/src/tuning.rs` — frequency calculation methods exist

### References

- [Source: docs/planning-artifacts/epics.md#Story 4.1] — Full acceptance criteria with BDD scenarios
- [Source: docs/ios-reference/domain-blueprint.md#§7.2] — PitchMatchingSession state machine, states, methods, formulas
- [Source: docs/ios-reference/domain-blueprint.md#§4.3-4.4] — PitchMatchingChallenge, CompletedPitchMatching entities
- [Source: docs/ios-reference/domain-blueprint.md#§9.8] — PitchMatchingObserver interface definition
- [Source: docs/ios-reference/domain-blueprint.md#§10.2] — PitchMatchingRecord persistence schema
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture] — NotePlayer trait, PlaybackHandle, hybrid audio approach
- [Source: docs/planning-artifacts/architecture.md#Project Structure] — session/ directory, ports.rs, records.rs locations
- [Source: docs/planning-artifacts/architecture.md#Implementation Patterns] — Observer signatures, error handling, naming conventions
- [Source: docs/planning-artifacts/ux-design-specification.md#Pitch Matching] — Training flow, feedback behavior, slider interaction
- [Source: docs/project-context.md] — Coding conventions, anti-patterns, type naming rules
- [Source: domain/src/session/comparison_session.rs] — Reference implementation for session pattern

### Previous Story Intelligence (from Story 3.3)

**Patterns to follow:**
- Context extraction: `Rc<RefCell<PerceptualProfile>>` shared ownership — same pattern used in session constructors
- Observer notification with `catch_unwind` for panic isolation — established in ComparisonSession
- Session-level settings snapshot on `start()` — snapshot intervals, tuning system, reference pitch, note duration, note range from UserSettings
- State guard assertions with descriptive messages: `assert_eq!(self.state, ..., "method() requires X state")`

**Code review learnings from Epic 3:**
- Collapse `else { if }` blocks per clippy `collapsible_else_if` lint
- Prefer `let _ = ...` pattern ONLY for deliberate ignoring — never for error results
- String ownership: clone before use if needed in multiple places

**Commit pattern:**
- Recent commits: "Implement story X.Y ..." → "Apply code review fixes for story X.Y and mark as done"
- Follow the same pattern

### Git Intelligence

Recent commits (last 5):
- `48e4117` Implement story 3.3 Profile Preview on Start Page and mark epic 3 as done
- `0d0fa6a` Add story 3.3 Profile Preview on Start Page and mark as ready-for-dev
- `1e4850b` Apply code review fixes for story 3.2 and mark as done
- `3740a05` Implement story 3.2 Perceptual Profile Visualization
- `7190ada` Add story 3.2 Perceptual Profile Visualization and mark as ready-for-dev

**Patterns observed:**
- Story creation commit: "Add story X.Y <title> and mark as ready-for-dev"
- Implementation commit: "Implement story X.Y <title>"
- Code review commit: "Apply code review fixes for story X.Y and mark as done"
- Epic completion noted in story implementation commit when last story in epic

**Files modified in recent commits:**
- `domain/src/` — profile.rs, types, session code
- `web/src/components/` — UI components
- `docs/implementation-artifacts/` — story files, sprint status

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
