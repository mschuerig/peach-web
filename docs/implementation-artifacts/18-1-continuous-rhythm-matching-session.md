# Story 18.1: Continuous Rhythm Matching Session and Step Sequencer

Status: done

## Story

As a developer,
I want a session state machine and step sequencer for continuous rhythm matching ("Fill the Gap"),
so that the exercise loop (play pattern with gap → user taps to fill → aggregate → repeat) is managed cleanly.

## Context

Prerequisite: Story 17.2 (rhythm scheduler with loop mode).

Continuous rhythm matching differs fundamentally from offset detection:
- The beat never stops — a 4-step loop plays continuously
- One step per cycle is a silent gap; the user taps to fill it
- Gap position can change each cycle (randomly selected from enabled positions)
- 16 cycles aggregate into one "trial" — the statistical unit
- Taps outside the acceptance window (±50% of one 16th note) are ignored
- Missed gaps (no tap in window) are recorded as misses

### iOS Reference (Epic 54)

- `ContinuousRhythmMatchingSession` with states: `.idle`, `.running`, `.paused`
- `StepSequencer` loops indefinitely, calls `StepProvider` at cycle boundaries
- `CompletedContinuousRhythmMatchingTrial`: meanOffsetMs, hitRate, per-position breakdown, cycleCount
- 16 cycles per trial; incomplete trials discarded on exit

## Acceptance Criteria

1. **AC1 — Session state enum:**
   ```rust
   pub enum ContinuousRhythmMatchingSessionState {
       Idle,
       Running,
   }
   ```

2. **AC2 — Cycle tracking:** Session tracks: current cycle index (0-15), gap position per cycle, tap result per cycle (hit with offset, or miss).

3. **AC3 — Gap position selection:** Each cycle, randomly select a gap position from the user's enabled set. If only one position enabled, always use that one.

4. **AC4 — Tap evaluation:** When user taps during a running cycle:
   - Read `audioContext.currentTime`
   - Compute offset relative to the gap's scheduled time
   - If within ±50% of one 16th note duration: record as hit with signed offset
   - If outside window: ignore (not a miss — the user tapped at the wrong time)
   - If no tap occurs before the next cycle: record as miss

5. **AC5 — Trial aggregation:** After 16 cycles, aggregate:
   - `mean_offset_ms: f64` (mean of hit offsets)
   - `hit_rate: f64` (hits / 16)
   - Per-position breakdown: mean offset per `StepPosition`
   - `cycle_count: u16` (always 16 for a complete trial)

6. **AC6 — CompletedContinuousRhythmMatchingTrial:**
   ```rust
   pub struct CompletedContinuousRhythmMatchingTrial {
       pub tempo: TempoBPM,
       pub mean_offset_ms: f64,
       pub hit_rate: f64,
       pub per_position_mean_ms: [Option<f64>; 4],
       pub cycle_count: u16,
       pub timestamp: String,
   }
   ```

7. **AC7 — ContinuousRhythmMatchingRecord:** Flat persistence record with all trial fields.

8. **AC8 — TrainingRecord extended:** `TrainingRecord::ContinuousRhythmMatching(ContinuousRhythmMatchingRecord)` variant added.

9. **AC9 — Port trait integration:** On trial completion, session calls `ProfileUpdating`, `TrainingRecordPersisting`, `ProgressTimelineUpdating` via generic ports.

10. **AC10 — Incomplete trial handling:** If the session stops mid-trial (user navigates away), the incomplete trial is discarded.

11. **AC11 — All tests pass.**

## Tasks / Subtasks

- [x] Task 1: Add `CompletedContinuousRhythmMatchingTrial` and record type
- [x] Task 2: Extend `TrainingRecord` enum
- [x] Task 3: Implement cycle tracking and gap position selection
- [x] Task 4: Implement tap evaluation with acceptance window
- [x] Task 5: Implement 16-cycle trial aggregation
- [x] Task 6: Implement session state machine
- [x] Task 7: Wire port trait calls on trial completion
- [x] Task 8: Unit tests for cycle tracking, tap evaluation, aggregation
- [x] Task 9: `cargo test -p domain` passes

## Dev Notes

- The step sequencer (loop mode from story 17.2) drives the audio. The session observes cycle transitions and evaluates taps.
- Gap position randomization: use `js_sys::Math::random()` in web, or accept a `rand`-like trait for testability in domain.
- The metric for profile/timeline is `mean_offset_ms` converted to percentage of 16th note.
- The "Fill the Gap" metaphor: silence on the gap position, user's tap fills it. If the tap plays a click sound (Epic 57 refinement), the user literally fills the gap audibly.

## Dev Agent Record

### Implementation Plan

Implemented the continuous rhythm matching ("Fill the Gap") session state machine following the same patterns as `RhythmOffsetDetectionSession`:

1. **CompletedContinuousRhythmMatchingTrial** — immutable trial result with tempo, mean_offset_ms, hit_rate, per-position breakdown, cycle_count, timestamp. Includes `metric_value()` for profile metric extraction.
2. **CycleResult** enum — `Hit(RhythmOffset)` or `Miss` for per-cycle tracking.
3. **aggregate_trial()** — pure function aggregating 16 `(StepPosition, CycleResult)` pairs into a completed trial. Handles all-misses case (0% hit rate, 0 mean offset).
4. **ContinuousRhythmMatchingRecord** — flat persistence record with serde support.
5. **TrainingRecord::ContinuousRhythmMatching** variant — added to enum with timestamp(), store_name(), and all match arms updated across the workspace.
6. **ContinuousRhythmMatchingSession** — state machine with Idle/Running states, cycle tracking, gap position selection (via injectable `GapPositionSelector` trait for testability), tap evaluation reusing `evaluate_tap()`, and port trait calls on trial completion.
7. **TrainingDiscipline** — added `extract_continuous_rhythm_metric()` and `continuous_rhythm_statistics_key()` methods for hydration/replay.
8. **IndexedDB** — `CONTINUOUS_RHYTHM_MATCHING_STORE` constant, fetch/save support in `indexeddb_store.rs`.

### Completion Notes

- 24 new tests added (480 total domain lib tests pass)
- Clippy clean, no new warnings
- Gap position randomization uses injectable `GapPositionSelector` trait (production: `RandomGapSelector` using `rand` crate; tests: `FixedGapSelector` for determinism)
- Incomplete trials discarded on stop (AC10)
- Web crate CSV export/import gracefully logs unsupported rhythm record types

### Debug Log

No issues encountered during implementation.

## File List

- `domain/src/training/continuous_rhythm_matching.rs` (new)
- `domain/src/training/mod.rs` (modified)
- `domain/src/session/continuous_rhythm_matching_session.rs` (new)
- `domain/src/session/mod.rs` (modified)
- `domain/src/records.rs` (modified)
- `domain/src/training_discipline.rs` (modified)
- `domain/src/lib.rs` (modified)
- `web/src/adapters/indexeddb_store.rs` (modified)
- `web/src/adapters/csv_export_import.rs` (modified)

## Change Log

- 2026-03-25: Implemented continuous rhythm matching session state machine, trial types, records, and persistence integration (Story 18.1)
