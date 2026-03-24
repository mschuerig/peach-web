# Story 17.3: Rhythm Offset Detection Session State Machine

Status: draft

## Story

As a developer,
I want a session state machine for rhythm offset detection training,
so that the exercise flow (play pattern → await answer → show feedback → repeat) is managed cleanly.

## Context

Prerequisite: Stories 17.1 (domain types) and 17.2 (scheduler).

The rhythm offset detection exercise:
1. Play 4 clicks at sixteenth-note intervals at the user's tempo
2. The 3rd click is offset early or late by the current difficulty
3. User answers "Early" or "Late"
4. Show feedback (correct/incorrect + current difficulty %)
5. Repeat with adapted difficulty

### iOS Reference

- `RhythmOffsetDetectionSession` with states: `.idle`, `.playing`, `.awaitingAnswer`, `.showingFeedback`
- Difficulty adapts independently for early and late (asymmetric)
- `AdaptiveRhythmOffsetStrategy`: narrows offset on correct, widens on incorrect, per direction

## Acceptance Criteria

1. **AC1 — Session state enum:**
   ```rust
   pub enum RhythmOffsetDetectionSessionState {
       Idle,
       Playing,
       AwaitingAnswer,
       ShowingFeedback,
   }
   ```

2. **AC2 — Session struct:** Holds state, current trial parameters (tempo, offset, direction), last result, difficulty tracker per direction.

3. **AC3 — Adaptive strategy:** Offset narrows on correct answer, widens on incorrect. Independent tracking for early vs. late. Configurable min (1%) and max (20%) offset percentages.

4. **AC4 — Session API:**
   - `start_trial()` → generates next offset (choosing early/late, computing ms from % and tempo), transitions to `Playing`
   - `pattern_finished()` → transitions to `AwaitingAnswer`
   - `submit_answer(is_early: bool)` → evaluates correctness, updates difficulty, calls port traits (profile + store + timeline), transitions to `ShowingFeedback`
   - `feedback_complete()` → transitions to `Idle` (ready for next trial)

5. **AC5 — Uses generic port traits:** Session holds references to `ProfileUpdating`, `TrainingRecordPersisting`, `ProgressTimelineUpdating` (from story 16.3), not discipline-specific observers.

6. **AC6 — Generates StatisticsKey:** When updating profile, uses `StatisticsKey::Rhythm(RhythmOffsetDetection, tempo_range, direction)`.

7. **AC7 — All tests pass:** Session logic unit-tested with mock port implementations. `cargo test -p domain` passes.

## Tasks / Subtasks

- [ ] Task 1: Define `RhythmOffsetDetectionSessionState` enum
- [ ] Task 2: Implement adaptive strategy (per-direction difficulty tracking)
- [ ] Task 3: Implement `RhythmOffsetDetectionSession` struct with state transitions
- [ ] Task 4: Wire port trait calls in `submit_answer`
- [ ] Task 5: Unit tests for adaptive strategy
- [ ] Task 6: Unit tests for session state transitions
- [ ] Task 7: `cargo test -p domain` passes

## Dev Notes

- The session lives in `domain/src/session/rhythm_offset_detection_session.rs`
- The adaptive strategy can be a simple struct with `early_difficulty_pct: f64` and `late_difficulty_pct: f64`, narrowing by a fixed factor (e.g., ×0.85 on correct, ×1.2 on incorrect), clamped to [min_pct, max_pct]
- The 3rd note is the offset one (not 4th) per iOS Epic 56 refinement — "user has reference clicks on both sides"
- Pattern for the scheduler: `[Play, Play, OffsetPlay, Play]` where `OffsetPlay` is scheduled at `beat_time + offset_ms`
