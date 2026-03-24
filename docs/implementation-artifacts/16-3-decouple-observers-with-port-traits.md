# Story 16.3: Decouple Observers with Port Traits

Status: draft

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

- [ ] Task 1: Define `ProfileUpdating`, `TrainingRecordPersisting`, `ProgressTimelineUpdating` in `ports.rs`
- [ ] Task 2: Define `TrainingRecord` enum wrapping all record types
- [ ] Task 3: Refactor `PitchDiscriminationSession` to use generic ports
- [ ] Task 4: Refactor `PitchMatchingSession` to use generic ports
- [ ] Task 5: Implement generic port traits in web crate (profile, store, timeline)
- [ ] Task 6: Remove old observer traits and bridge observer classes
- [ ] Task 7: Update `app.rs` session wiring
- [ ] Task 8: Update tests
- [ ] Task 9: `cargo test --workspace` and `cargo clippy --workspace` pass

## Dev Notes

- The `TrainingRecord` enum is the Rust equivalent of Swift's `some PersistentModel`. It allows the store to accept any record type without per-discipline methods.
- The IndexedDB store's `save_record()` implementation will pattern-match on `TrainingRecord` to determine which object store to write to. This is a single match in one place, vs. the current approach of per-discipline save methods.
- Consider whether sessions should hold `Rc<RefCell<dyn ProfileUpdating>>` or use a combined trait object. The simplest approach: a single `SessionObserver` struct that holds references to all three and implements a thin dispatch.
- This story eliminates the biggest coupling multiplier: new disciplines no longer need new observer traits or bridge classes.
