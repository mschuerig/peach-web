# Story 1.6: Comparison Session State Machine

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the ComparisonSession state machine implemented with its full training loop and observer pattern,
so that the domain logic drives the comparison training experience correctly.

## Acceptance Criteria

1. **AC1 — Idle state:** Given ComparisonSession is constructed with injected dependencies, when it is in `idle` state, then no training loop is running and all observable state is at defaults.

2. **AC2 — Start:** Given ComparisonSession is idle, when `start(intervals)` is called with at least one interval, then state transitions to `playingNote1`, the tuning system is snapshot from settings for the session, and the first comparison is generated.

3. **AC3 — Note playback data:** Given the training loop is running, when a comparison is generated, then the session provides: reference frequency at velocity 63 and amplitudeDB 0.0, target frequency at velocity 63 with calculated amplitude variation, and the configured noteDuration (FR3).

4. **AC4 — Note1 to Note2 transition:** Given state is `playingNote1`, when `on_note1_finished()` is called, then state transitions to `playingNote2`.

5. **AC5 — Note2 to AwaitingAnswer:** Given state is `playingNote2`, when `on_note2_finished()` is called and no answer was given, then state transitions to `awaitingAnswer`.

6. **AC6 — Handle answer:** Given state is `playingNote2` or `awaitingAnswer`, when `handle_answer(is_higher, timestamp)` is called, then a `CompletedComparison` is created with the provided timestamp and session's snapshot tuning system, all observers receive `comparison_completed(&completed)`, and state transitions to `showingFeedback`.

7. **AC7 — Feedback state:** Given state is `showingFeedback`, when `on_feedback_finished()` is called, then `show_feedback` becomes false and the next comparison is generated with state returning to `playingNote1`.

8. **AC8 — No loudness variation:** Given `vary_loudness` is 0.0, when target amplitude is calculated, then amplitudeDB is 0.0.

9. **AC9 — Loudness variation:** Given `vary_loudness` is > 0.0, when target amplitude is calculated, then amplitudeDB is random in `[-vary_loudness * 5.0, +vary_loudness * 5.0]`.

10. **AC10 — Stop:** Given session is running, when `stop()` is called, then state returns to `idle`, current comparison is discarded, and all session-level state resets (FR7).

11. **AC11 — Observer error isolation:** Given the observer pattern, when an observer panics, then the panic is caught internally and logged — never propagated to the session.

## Tasks / Subtasks

- [ ] Task 1: Add observer and settings port traits to domain (AC: 6,11)
  - [ ] 1.1 Add `ComparisonObserver` trait to `domain/src/ports.rs`: `fn comparison_completed(&mut self, completed: &CompletedComparison)`
  - [ ] 1.2 Add `Resettable` trait to `domain/src/ports.rs`: `fn reset(&mut self)`
  - [ ] 1.3 Add `UserSettings` trait to `domain/src/ports.rs` with getters: `note_range_min`, `note_range_max`, `note_duration`, `reference_pitch`, `tuning_system`, `vary_loudness`
  - [ ] 1.4 Update `domain/src/lib.rs` to re-export new traits

- [ ] Task 2: Create ComparisonSessionState and ComparisonSession (AC: 1,2)
  - [ ] 2.1 Create `domain/src/session/mod.rs` with module declarations
  - [ ] 2.2 Create `domain/src/session/comparison_session.rs` with `ComparisonSessionState` enum (Idle, PlayingNote1, PlayingNote2, AwaitingAnswer, ShowingFeedback)
  - [ ] 2.3 Create `ComparisonSession` struct with: `Rc<RefCell<PerceptualProfile>>` for profile, `Vec<Box<dyn ComparisonObserver>>` for observers, `Vec<Box<dyn Resettable>>` for resettables, session-level state fields
  - [ ] 2.4 Implement `new()` constructor accepting all dependencies
  - [ ] 2.5 Implement observable state accessors: `state()`, `show_feedback()`, `is_last_answer_correct()`, `session_best_cent_difference()`, `current_interval()`
  - [ ] 2.6 Implement `current_playback_data() -> Option<ComparisonPlaybackData>` for the web layer to read
  - [ ] 2.7 Add `pub mod session;` to `domain/src/lib.rs` and re-export session types

- [ ] Task 3: Implement start and state transitions (AC: 2,3,4,5,7,8,9)
  - [ ] 3.1 Implement `start(intervals, settings)` — validate non-empty intervals, snapshot tuning_system + reference_pitch + note_duration + vary_loudness from settings, generate first comparison via `next_comparison()`, calculate target amplitude, set state to `PlayingNote1`
  - [ ] 3.2 Implement `on_note1_finished()` — guard: must be PlayingNote1, transition to PlayingNote2
  - [ ] 3.3 Implement `on_note2_finished()` — guard: must be PlayingNote2, transition to AwaitingAnswer
  - [ ] 3.4 Implement `on_feedback_finished()` — guard: must be ShowingFeedback, generate next comparison, set state to PlayingNote1
  - [ ] 3.5 Implement `calculate_target_amplitude(vary_loudness: f64) -> AmplitudeDB` private method

- [ ] Task 4: Implement handle_answer with observer notification (AC: 6,11)
  - [ ] 4.1 Implement `handle_answer(is_higher, timestamp)` — guard on PlayingNote2 or AwaitingAnswer, create CompletedComparison with session tuning system, update session_best_cent_difference, set show_feedback + is_last_answer_correct, transition to ShowingFeedback
  - [ ] 4.2 Implement observer notification loop with `std::panic::catch_unwind` for error isolation — log caught panics via `eprintln!` (no browser deps in domain)

- [ ] Task 5: Implement stop and reset (AC: 10)
  - [ ] 5.1 Implement `stop()` — guard: not idle, reset state to Idle, clear current_comparison, clear session-level transient state
  - [ ] 5.2 Implement `reset_training_data()` — stop if running, clear last_completed + session_best, call `profile.borrow_mut().reset()`, call `reset()` on each resettable

- [ ] Task 6: Write comprehensive unit tests (AC: all)
  - [ ] 6.1 Create mock types: `MockObserver` (records calls), `MockResettable` (tracks reset), helper `default_settings() -> impl UserSettings`
  - [ ] 6.2 Test full lifecycle: idle → start → playingNote1 → on_note1_finished → playingNote2 → on_note2_finished → awaitingAnswer → handle_answer → showingFeedback → on_feedback_finished → playingNote1
  - [ ] 6.3 Test early answer: playingNote2 → handle_answer (skip awaitingAnswer)
  - [ ] 6.4 Test start guards: must be idle, must have ≥1 interval
  - [ ] 6.5 Test handle_answer guards: invalid from Idle, PlayingNote1, ShowingFeedback
  - [ ] 6.6 Test CompletedComparison: correct timestamp, snapshot tuning system, correct is_correct derivation
  - [ ] 6.7 Test observer notification: MockObserver receives comparison_completed
  - [ ] 6.8 Test observer error isolation: panicking observer + normal observer, normal still receives event
  - [ ] 6.9 Test amplitude: vary_loudness=0 → AmplitudeDB(0.0), vary_loudness=0.5 → within ±2.5
  - [ ] 6.10 Test session_best tracking: only updates on correct, tracks smallest cent difference
  - [ ] 6.11 Test stop: returns to idle, clears state
  - [ ] 6.12 Test reset_training_data: stops, resets profile, calls resettables
  - [ ] 6.13 `cargo test -p domain` — all existing + new tests pass
  - [ ] 6.14 `cargo clippy -p domain` — zero warnings

## Dev Notes

### Session Architecture: Event-Driven State Machine (Domain-Pure)

The ComparisonSession is a **pure domain state machine** in the `domain` crate with no browser dependencies. The blueprint's async training loop is split across two layers:

**Domain crate (this story):** State management, comparison generation, answer processing, observer notification, amplitude calculation — all synchronous.

**Web crate (story 1.7):** Async training loop driver using `spawn_local` + `gloo-timers`, NotePlayer audio calls, Leptos signal bridge.

The session exposes event-driven transition methods:
```
start(intervals, &dyn UserSettings)
  → generates comparison, state = PlayingNote1

on_note1_finished()
  → state = PlayingNote2

on_note2_finished()
  → state = AwaitingAnswer (if no early answer)

handle_answer(is_higher, timestamp)
  → creates CompletedComparison, notifies observers, state = ShowingFeedback

on_feedback_finished()
  → generates next comparison, state = PlayingNote1

stop()
  → state = Idle, clears transient state
```

The web crate (story 1.7) will drive the loop:
```rust
spawn_local(async move {
    session.borrow_mut().start(intervals, &settings);
    loop {
        let data = session.borrow().current_playback_data().unwrap();
        note_player.play_for_duration(data.reference_frequency, data.duration, ...)?;
        sleep(data.duration.raw_value()).await;
        session.borrow_mut().on_note1_finished();
        note_player.play_for_duration(data.target_frequency, data.duration, ...)?;
        sleep(data.duration.raw_value()).await;
        if session.borrow().state() == ComparisonSessionState::PlayingNote2 {
            session.borrow_mut().on_note2_finished();
        }
        // wait for user answer via signal/flag...
        session.borrow_mut().handle_answer(is_higher, timestamp);
        sleep(0.4).await; // feedback duration
        session.borrow_mut().on_feedback_finished();
    }
});
```

### Why No NotePlayer Injection

The blueprint injects NotePlayer into the session. In the Rust implementation, NotePlayer has an associated type (`type Handle: PlaybackHandle`) which prevents usage as `dyn NotePlayer`. Options considered:

1. **Generic session** `ComparisonSession<P: NotePlayer>` — works but forces generics through the entire call chain
2. **Type-erased wrapper** — adds complexity for no domain-logic benefit
3. **Data-only session** (chosen) — session provides `ComparisonPlaybackData`; web layer drives audio

Option 3 preserves clean domain/web separation. The session calculates frequencies using `TuningSystem.frequency()` (pure domain logic) and provides them via `current_playback_data()`. The web layer reads this data and calls NotePlayer directly.

### ComparisonPlaybackData Struct

```rust
pub struct ComparisonPlaybackData {
    pub reference_frequency: Frequency,
    pub target_frequency: Frequency,
    pub duration: NoteDuration,
    pub target_amplitude_db: AmplitudeDB,
}
```

Constants (not configurable):
- Reference velocity: `MIDIVelocity::new(63)`
- Reference amplitude: `AmplitudeDB::new(0.0)` (unity)
- Target velocity: `MIDIVelocity::new(63)`
- Feedback duration: 0.4 seconds (defined as constant `FEEDBACK_DURATION_SECS: f64 = 0.4`)

### UserSettings Trait Design

```rust
pub trait UserSettings {
    fn note_range_min(&self) -> MIDINote;
    fn note_range_max(&self) -> MIDINote;
    fn note_duration(&self) -> NoteDuration;
    fn reference_pitch(&self) -> Frequency;
    fn tuning_system(&self) -> TuningSystem;
    fn vary_loudness(&self) -> f64; // 0.0-1.0 (UnitInterval range)
}
```

The session reads settings at `start()` and snapshots `tuning_system`, `reference_pitch`, `note_duration`, and `vary_loudness` for the session. Settings are read via `&dyn UserSettings` (trait object — no associated types, straightforward dynamic dispatch).

For `next_comparison()`, the session constructs a `TrainingSettings` from the snapshot:
```rust
let training_settings = TrainingSettings::new(
    self.session_reference_pitch,
    self.note_range_min,  // from settings snapshot
    self.note_range_max,
);
```

Note: `min_cent_difference` and `max_cent_difference` use TrainingSettings defaults (0.1 and 100.0 cents). These are algorithm parameters, not user-facing settings.

### Observer Pattern Implementation

```rust
pub trait ComparisonObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison);
}
```

Observers are stored as `Vec<Box<dyn ComparisonObserver>>`. On `handle_answer()`:
```rust
for observer in &mut self.observers {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observer.comparison_completed(&completed);
    }));
    if let Err(e) = result {
        eprintln!("Observer panicked: {:?}", e);
    }
}
```

**Note on `catch_unwind`:** Requires `ComparisonObserver` implementations to be `UnwindSafe`. In practice, `RefCell`-based observers are not `UnwindSafe` by default. Use `AssertUnwindSafe` wrapper since observer panics represent bugs that should be logged, not propagated.

**Profile as observer:** The PerceptualProfile is shared via `Rc<RefCell<PerceptualProfile>>`. A `ProfileObserver` wrapper can implement `ComparisonObserver`:
```rust
struct ProfileObserver(Rc<RefCell<PerceptualProfile>>);
impl ComparisonObserver for ProfileObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let mut profile = self.0.borrow_mut();
        let cent_offset = completed.comparison().target_note().offset.raw_value.abs();
        profile.update(
            completed.comparison().reference_note(),
            cent_offset,
            completed.is_correct(),
        );
    }
}
```

This is composition root wiring (story 1.7+), but the pattern is documented here for the dev agent.

### Shared PerceptualProfile (Rc<RefCell>)

The session needs the profile for two purposes:
1. **Read** during comparison generation: `next_comparison(&profile, ...)` needs `&PerceptualProfile`
2. **Write** via observer notification: `profile.update(...)` needs `&mut PerceptualProfile`

Solution: `Rc<RefCell<PerceptualProfile>>` shared between session (read) and ProfileObserver (write). The session holds its own `Rc` clone. Borrow rules:
- Session borrows `Ref` (immutable) when calling `next_comparison`
- Observer borrows `RefMut` (mutable) when processing `comparison_completed`
- These never overlap because `handle_answer` first creates the `CompletedComparison` (no profile borrow), then notifies observers (profile write), then the loop generates the next comparison later (profile read).

### Amplitude Calculation (from blueprint)

```rust
fn calculate_target_amplitude(vary_loudness: f64) -> AmplitudeDB {
    if vary_loudness <= 0.0 {
        return AmplitudeDB::new(0.0);
    }
    let range = vary_loudness * 5.0; // maxLoudnessOffsetDB = 5.0
    let offset = rand::random::<f64>() * 2.0 * range - range; // uniform [-range, +range]
    AmplitudeDB::new(offset as f32)
}
```

Note: `AmplitudeDB` uses `f32` (matching Web Audio GainNode's f32 interface, per story 1.5).

### Interval Selection Per Comparison

Each comparison randomly selects one interval from the session's interval set:
```rust
let intervals_vec: Vec<_> = self.session_intervals.iter().collect();
let interval = *intervals_vec[rand::random::<usize>() % intervals_vec.len()];
```

Use `rand::seq::SliceRandom::choose()` with `rand::rng()` for proper random selection.

### Comparison Frequency Calculation

The session uses `TuningSystem::frequency()` to convert MIDI notes to Hz:
```rust
let ref_freq = self.session_tuning_system.frequency(
    comparison.reference_note().into(), // MIDINote → DetunedMIDINote (zero offset)
    self.session_reference_pitch,
);
let target_freq = self.session_tuning_system.frequency(
    comparison.target_note(), // DetunedMIDINote (with cent offset)
    self.session_reference_pitch,
);
```

Both `TuningSystem::frequency()` and `TuningSystem::frequency_for_note()` exist. Use `frequency()` which accepts `DetunedMIDINote` — handles both reference (zero offset) and target (with offset).

### Existing Code Dependencies

**Used directly by ComparisonSession:**
- `domain::strategy::next_comparison(profile, settings, last_comparison, interval) -> Comparison`
- `domain::strategy::TrainingSettings::new(reference_pitch, note_range_min, note_range_max)` (check constructor signature)
- `domain::profile::PerceptualProfile` (via `Rc<RefCell<>>`, read-only for comparison generation)
- `domain::training::Comparison`, `domain::training::CompletedComparison`
- `domain::tuning::TuningSystem` — `frequency(DetunedMIDINote, Frequency) -> Frequency`
- `domain::types::*` — MIDINote, MIDIVelocity, Frequency, AmplitudeDB, NoteDuration, Cents, DirectedInterval, DetunedMIDINote

**Verify before implementing:**
- Check `TrainingSettings::new()` signature — may need to match min/max cent difference defaults
- Check `AmplitudeDB::new()` accepts f32 or f64 — story 1.5 uses f32
- Check `From<MIDINote> for DetunedMIDINote` exists (zero offset conversion)
- Check `CompletedComparison::new()` parameter order

### Project Structure Notes

```
domain/src/
├── session/
│   ├── mod.rs                    (NEW — module declarations and re-exports)
│   └── comparison_session.rs     (NEW — ComparisonSession, ComparisonSessionState, ComparisonPlaybackData)
├── ports.rs                      (MODIFIED — add ComparisonObserver, Resettable, UserSettings traits)
├── lib.rs                        (MODIFIED — add pub mod session, re-export session types)
└── ... (unchanged)
```

Do NOT create `session/pitch_matching_session.rs` yet — that belongs to story 4.1.

### Testing Strategy

**All tests in `domain/src/session/comparison_session.rs` as `#[cfg(test)] mod tests`.**

Mock types needed:
```rust
struct MockObserver {
    calls: Vec<CompletedComparison>,  // Records received events
}

struct MockResettable {
    reset_count: u32,
}

struct DefaultTestSettings; // Implements UserSettings with sensible defaults
```

Note: `CompletedComparison` needs `Clone` to store in MockObserver. Check if it's derived — if not, the mock can store derived data instead.

**Test naming convention:** `test_` prefix + descriptive name (per project-context.md).

**What NOT to test:**
- Don't test NotePlayer calls — session doesn't own NotePlayer
- Don't test async timing — no async in domain crate
- Don't test web-sys AudioContext — pure domain tests only
- Don't test `next_comparison` internals — already tested in strategy.rs

### Previous Story Intelligence (Story 1.5)

Patterns established in story 1.5 that apply:
- **Module structure:** `mod.rs` + individual files pattern — apply for `session/mod.rs` + `comparison_session.rs`
- **Trait design:** Associated type pattern (`type Handle: PlaybackHandle`) used for NotePlayer — this is why the session can't use `dyn NotePlayer`
- **Error pattern:** `AudioError::PlaybackFailed(String)` variant added during code review — session should handle AudioError if any observer wraps audio
- **Interior mutability:** `Rc<RefCell<>>` pattern established for AudioContextManager — same pattern for shared PerceptualProfile
- **Clippy compliance:** Run `cargo clippy -p domain` early and fix warnings
- **Current domain test count:** 187 tests passing — do not regress

### Git History Context

Recent commits follow the pattern: create story → implement → code review → done:
```
83af585 Apply code review fixes for story 1.5 and mark as done
2b4395e Implement story 1.5 Audio Engine (Oscillator)
d86e6e1 Add story 1.5 Audio Engine Oscillator and mark as ready-for-dev
5e2b06f Add .idea to .gitignore.
1c27a20 Apply code review fixes for story 1.4 and mark as done
```

### References

- [Source: docs/planning-artifacts/epics.md#Story 1.6: Comparison Session State Machine]
- [Source: docs/ios-reference/domain-blueprint.md#§7 Session State Machines]
- [Source: docs/ios-reference/domain-blueprint.md#§7.1 ComparisonSession]
- [Source: docs/ios-reference/domain-blueprint.md#§4 Training Entities]
- [Source: docs/ios-reference/domain-blueprint.md#§9 Port Interfaces]
- [Source: docs/ios-reference/domain-blueprint.md#§9.8 Observer Protocols]
- [Source: docs/ios-reference/domain-blueprint.md#§11 Composition Rules]
- [Source: docs/planning-artifacts/architecture.md#Async Model]
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: docs/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: docs/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/planning-artifacts/ux-design-specification.md#Comparison Loop Mechanics]
- [Source: docs/planning-artifacts/ux-design-specification.md#Interruption Handling]
- [Source: docs/project-context.md#Critical Implementation Rules]
- [Source: docs/project-context.md#Rust Language Rules]
- [Source: docs/implementation-artifacts/1-5-audio-engine-oscillator.md#Dev Notes]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
