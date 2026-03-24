# Story 18.1: Continuous Rhythm Matching Session and Step Sequencer

Status: draft

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

- [ ] Task 1: Add `CompletedContinuousRhythmMatchingTrial` and record type
- [ ] Task 2: Extend `TrainingRecord` enum
- [ ] Task 3: Implement cycle tracking and gap position selection
- [ ] Task 4: Implement tap evaluation with acceptance window
- [ ] Task 5: Implement 16-cycle trial aggregation
- [ ] Task 6: Implement session state machine
- [ ] Task 7: Wire port trait calls on trial completion
- [ ] Task 8: Unit tests for cycle tracking, tap evaluation, aggregation
- [ ] Task 9: `cargo test -p domain` passes

## Dev Notes

- The step sequencer (loop mode from story 17.2) drives the audio. The session observes cycle transitions and evaluates taps.
- Gap position randomization: use `js_sys::Math::random()` in web, or accept a `rand`-like trait for testability in domain.
- The metric for profile/timeline is `mean_offset_ms` converted to percentage of 16th note.
- The "Fill the Gap" metaphor: silence on the gap position, user's tap fills it. If the tap plays a click sound (Epic 57 refinement), the user literally fills the gap audibly.
