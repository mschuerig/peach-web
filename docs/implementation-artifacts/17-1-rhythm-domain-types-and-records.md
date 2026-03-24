# Story 17.1: Rhythm Domain Types, Records, and Observer Ports

Status: done

## Story

As a developer,
I want rhythm-specific domain types, record structs, and trial types defined,
so that the rhythm offset detection session can be implemented.

## Context

Prerequisite: Epic 16 (architecture refactoring). Generic port traits and StatisticsKey exist.

This story adds the rhythm-specific domain types that the offset detection session needs. The types mirror the iOS domain model.

### iOS Reference

- `RhythmOffset`: Signed duration (negative = early, positive = late), computes percentage of sixteenth note
- `CompletedRhythmOffsetDetectionTrial`: tempo, offset, isCorrect, timestamp
- `RhythmOffsetDetectionRecord`: flat persistence record

## Acceptance Criteria

1. **AC1 — RhythmOffset type:** Stores offset in milliseconds (f64, signed). Methods:
   - `direction() -> RhythmDirection` (negative → Early, non-negative → Late)
   - `percentage_of_sixteenth(tempo: TempoBPM) -> f64` (absolute offset as % of one 16th note)
   - `abs_ms() -> f64`

2. **AC2 — CompletedRhythmOffsetDetectionTrial:**
   - Fields: `tempo: TempoBPM`, `offset: RhythmOffset`, `is_correct: bool`, `timestamp: String`
   - Method to extract metric: `metric_value() -> f64` returns `offset.percentage_of_sixteenth(self.tempo)` (uses stored tempo)

3. **AC3 — RhythmOffsetDetectionRecord:** Flat persistence record:
   - `tempo_bpm: u16`, `offset_ms: f64`, `is_correct: bool`, `timestamp: String`
   - `from_completed(trial: &CompletedRhythmOffsetDetectionTrial) -> Self`

4. **AC4 — TrainingRecord enum extended:** `TrainingRecord::RhythmOffsetDetection(RhythmOffsetDetectionRecord)` variant added.

5. **AC5 — Metric extraction on TrainingDiscipline:** `RhythmOffsetDetection` discipline can extract metrics from its record type (maps to `StatisticsKey::Rhythm(discipline, tempo_range, direction)`).

6. **AC6 — All tests pass.**

## Tasks / Subtasks

- [x] Task 1: Add `RhythmOffset` to `domain/src/types/`
- [x] Task 2: Add `CompletedRhythmOffsetDetectionTrial` to `domain/src/training/`
- [x] Task 3: Add `RhythmOffsetDetectionRecord` to `domain/src/records.rs`
- [x] Task 4: Extend `TrainingRecord` enum
- [x] Task 5: Add metric extraction for rhythm offset detection
- [x] Task 6: Unit tests
- [x] Task 7: `cargo test -p domain` passes

## Dev Notes

- `RhythmOffset` is a value type in `types/`, `CompletedRhythmOffsetDetectionTrial` is in `training/`
- Percentage formula: `abs(offset_ms) / sixteenth_note_duration_ms * 100.0`
- At 80 BPM, 1 sixteenth = 187.5ms; an offset of 9.375ms = 5%

## Dev Agent Record

### Implementation Plan

- Added `RhythmOffset` value type with `direction()`, `abs_ms()`, `percentage_of_sixteenth()` methods
- Added `CompletedRhythmOffsetDetectionTrial` with `metric_value()` that computes % of sixteenth note
- Added `RhythmOffsetDetectionRecord` flat persistence struct with `from_completed()` constructor
- Extended `TrainingRecord` enum with `RhythmOffsetDetection` variant
- Added `extract_rhythm_offset_metric()` and `rhythm_offset_statistics_key()` to `TrainingDiscipline`
- Updated all exhaustive matches in domain (progress_timeline) and web (app.rs, indexeddb_store, csv_export_import)
- Updated `STORE_NAMES` registry to use `RHYTHM_OFFSET_DETECTION_STORE` constant
- Added fetch logic in `IndexedDbStore::fetch_all_records()` for rhythm records

### Completion Notes

All 6 ACs satisfied. 407 tests pass (392 unit + 15 integration). Zero clippy warnings. `cargo fmt` clean. Web crate compiles with all new match arms handled.

## File List

- domain/src/types/rhythm_offset.rs (new)
- domain/src/types/mod.rs (modified)
- domain/src/training/rhythm_offset_detection.rs (new)
- domain/src/training/mod.rs (modified)
- domain/src/records.rs (modified)
- domain/src/training_discipline.rs (modified)
- domain/src/progress_timeline.rs (modified)
- domain/src/lib.rs (modified)
- web/src/app.rs (modified)
- web/src/adapters/indexeddb_store.rs (modified)
- web/src/adapters/csv_export_import.rs (modified)

## Change Log

- 2026-03-24: Implemented rhythm domain types, records, and metric extraction (Story 17.1)
