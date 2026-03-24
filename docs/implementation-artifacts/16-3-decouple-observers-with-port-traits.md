# Story 16.3: Decouple Observers with Port Traits

Status: done

## Story

As a developer,
I want discipline-specific observer traits replaced with generic port traits,
so that adding a new discipline doesn't require new observer traits, new bridge classes, or changes to the profile/store.

## Context

Currently, each discipline has its own observer trait (`PitchDiscriminationObserver`, `PitchMatchingObserver`) and the web crate implements 5 observer classes (2 profile adapters, 2 store adapters, 1 timeline adapter). Adding rhythm would require 4+ more observer classes and 2+ more trait definitions.

The iOS solution (Epic 55): two generic port traits that all disciplines use.

### iOS Reference

```swift
protocol ProfileUpdating {
    func update(_ key: StatisticsKey, timestamp: Date, value: Double)
}

protocol TrainingRecordPersisting {
    func save(_ record: some PersistentModel) throws
}
```

Each discipline has thin adapter structs that map trial results to these generic port operations.

## Acceptance Criteria

1. **AC1 — ProfileUpdating port trait:**
   ```rust
   pub trait ProfileUpdating {
       fn update_profile(&mut self, key: StatisticsKey, timestamp: &str, value: f64);
   }
   ```

2. **AC2 — TrainingRecordPersisting port trait:**
   ```rust
   pub trait TrainingRecordPersisting {
       fn save_record(&self, record: TrainingRecord) -> Result<(), StorageError>;
   }
   ```
   Where `TrainingRecord` is an enum wrapping all record types (pitch discrimination, pitch matching, and later rhythm records).

3. **AC3 — ProgressTimelineUpdating port trait:**
   ```rust
   pub trait ProgressTimelineUpdating {
       fn add_metric(&mut self, discipline: TrainingDiscipline, timestamp: &str, value: f64);
   }
   ```

4. **AC4 — Old observer traits removed:** `PitchDiscriminationObserver` and `PitchMatchingObserver` traits deleted from `ports.rs`.

5. **AC5 — Sessions use generic ports:** `PitchDiscriminationSession` and `PitchMatchingSession` accept `impl ProfileUpdating + TrainingRecordPersisting + ProgressTimelineUpdating` instead of discipline-specific observers.

6. **AC6 — Bridge simplified:** The 5 observer classes in `bridge.rs` collapse to 3 generic implementations:
   - `PerceptualProfile` implements `ProfileUpdating`
   - `IndexedDbStore` (or a wrapper) implements `TrainingRecordPersisting`
   - `ProgressTimeline` (or a wrapper) implements `ProgressTimelineUpdating`

7. **AC7 — Adapter structs in sessions:** Each session maps its domain-specific trial result to the generic port calls. The mapping logic (e.g., "compute StatisticsKey from interval, call update_profile") lives in the session, not in the bridge.

8. **AC8 — All tests pass:** `cargo test --workspace` passes. Session tests may need mock trait implementations updated.

## Tasks / Subtasks

- [x] Task 1: Define `ProfileUpdating`, `TrainingRecordPersisting`, `ProgressTimelineUpdating` in `ports.rs`
- [x] Task 2: Define `TrainingRecord` enum wrapping all record types
- [x] Task 3: Refactor `PitchDiscriminationSession` to use generic ports
- [x] Task 4: Refactor `PitchMatchingSession` to use generic ports
- [x] Task 5: Implement generic port traits in web crate (profile, store, timeline)
- [x] Task 6: Remove old observer traits and bridge observer classes
- [x] Task 7: Update view session wiring (pitch_discrimination_view.rs, pitch_matching_view.rs)
- [x] Task 8: Update tests
- [x] Task 9: `cargo test --workspace` and `cargo clippy --workspace` pass

## Dev Notes

- The `TrainingRecord` enum is the Rust equivalent of Swift's `some PersistentModel`. It allows the store to accept any record type without per-discipline methods.
- The IndexedDB store's `save_record()` implementation will pattern-match on `TrainingRecord` to determine which object store to write to. This is a single match in one place, vs. the current approach of per-discipline save methods.
- Consider whether sessions should hold `Rc<RefCell<dyn ProfileUpdating>>` or use a combined trait object. The simplest approach: a single `SessionObserver` struct that holds references to all three and implements a thin dispatch.
- This story eliminates the biggest coupling multiplier: new disciplines no longer need new observer traits or bridge classes.

## Dev Agent Record

### Implementation Plan

Sessions hold three `Box<dyn Trait>` port fields instead of `Vec<Box<dyn Observer>>`. Each session's
handle_answer/commit_pitch method computes the discipline, StatisticsKey, and metric internally,
then calls the three ports directly. The bridge collapses from 5 observer structs to 3 generic port
structs (ProfilePort, RecordPort, TimelinePort).

### Debug Log

No issues encountered. Clean compilation and all tests pass.

### Completion Notes

- Defined three generic port traits in `domain/src/ports.rs`: `ProfileUpdating`, `TrainingRecordPersisting`, `ProgressTimelineUpdating`
- Defined `TrainingRecord` enum in `domain/src/records.rs` wrapping `PitchDiscriminationRecord` and `PitchMatchingRecord`
- Refactored both session state machines to hold `Box<dyn ProfileUpdating>`, `Box<dyn TrainingRecordPersisting>`, `Box<dyn ProgressTimelineUpdating>` instead of observer Vecs
- Moved mapping logic (discipline determination, StatisticsKey computation, metric extraction) from bridge into session handle_answer/commit_pitch methods
- Collapsed 5 bridge observer classes (ProfileObserver, PitchMatchingProfileObserver, DataStoreObserver, PitchMatchingDataStoreObserver, ProgressTimelineObserver) into 3 generic port structs (ProfilePort, RecordPort, TimelinePort)
- Added `ProgressTimeline::add_metric_for_discipline()` to support the generic timeline port
- Updated `TrainingDataStore` trait to use `TrainingRecord` instead of per-type methods
- Updated all session tests with mock port implementations (MockProfilePort, MockRecordPort, MockTimelinePort, NoOp variants)
- Removed observer panic isolation tests (no longer applicable — single port per concern, no multi-observer dispatch)
- Added new port verification tests (test_handle_answer_calls_all_ports, test_incorrect_answer_skips_profile_port, test_commit_pitch_calls_all_ports)
- 375 tests pass, zero clippy warnings

## File List

- domain/src/ports.rs (modified — removed old observer traits, added 3 port traits, updated TrainingDataStore)
- domain/src/records.rs (modified — added TrainingRecord enum)
- domain/src/lib.rs (modified — updated exports)
- domain/src/session/pitch_discrimination_session.rs (modified — refactored to use port traits)
- domain/src/session/pitch_matching_session.rs (modified — refactored to use port traits)
- domain/src/progress_timeline.rs (modified — added add_metric_for_discipline method)
- web/src/bridge.rs (modified — collapsed 5 observers to 3 generic port structs)
- web/src/components/pitch_discrimination_view.rs (modified — updated session wiring)
- web/src/components/pitch_matching_view.rs (modified — updated session wiring)

## Change Log

- 2026-03-24: Implemented Story 16.3 — replaced discipline-specific observer traits with 3 generic port traits, collapsed 5 bridge observer classes to 3 generic implementations, moved mapping logic into sessions
