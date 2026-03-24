# Story 17.1: Rhythm Domain Types, Records, and Observer Ports

Status: draft

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
   - Method to extract metric: `metric_value(tempo: TempoBPM) -> f64` returns `offset.percentage_of_sixteenth(tempo)`

3. **AC3 — RhythmOffsetDetectionRecord:** Flat persistence record:
   - `tempo_bpm: u16`, `offset_ms: f64`, `is_correct: bool`, `timestamp: String`
   - `from_completed(trial: &CompletedRhythmOffsetDetectionTrial) -> Self`

4. **AC4 — TrainingRecord enum extended:** `TrainingRecord::RhythmOffsetDetection(RhythmOffsetDetectionRecord)` variant added.

5. **AC5 — Metric extraction on TrainingDiscipline:** `RhythmOffsetDetection` discipline can extract metrics from its record type (maps to `StatisticsKey::Rhythm(discipline, tempo_range, direction)`).

6. **AC6 — All tests pass.**

## Tasks / Subtasks

- [ ] Task 1: Add `RhythmOffset` to `domain/src/types/`
- [ ] Task 2: Add `CompletedRhythmOffsetDetectionTrial` to `domain/src/training/`
- [ ] Task 3: Add `RhythmOffsetDetectionRecord` to `domain/src/records.rs`
- [ ] Task 4: Extend `TrainingRecord` enum
- [ ] Task 5: Add metric extraction for rhythm offset detection
- [ ] Task 6: Unit tests
- [ ] Task 7: `cargo test -p domain` passes

## Dev Notes

- `RhythmOffset` is a value type in `types/`, `CompletedRhythmOffsetDetectionTrial` is in `training/`
- Percentage formula: `abs(offset_ms) / sixteenth_note_duration_ms * 100.0`
- At 80 BPM, 1 sixteenth = 187.5ms; an offset of 9.375ms = 5%
