# Story 17.3: Rhythm Offset Detection Session State Machine

Status: review

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

- [x] Task 1: Define `RhythmOffsetDetectionSessionState` enum
- [x] Task 2: Implement adaptive strategy (per-direction difficulty tracking)
- [x] Task 3: Implement `RhythmOffsetDetectionSession` struct with state transitions
- [x] Task 4: Wire port trait calls in `submit_answer`
- [x] Task 5: Unit tests for adaptive strategy
- [x] Task 6: Unit tests for session state transitions
- [x] Task 7: `cargo test -p domain` passes

## Dev Notes

- The session lives in `domain/src/session/rhythm_offset_detection_session.rs`
- The adaptive strategy can be a simple struct with `early_difficulty_pct: f64` and `late_difficulty_pct: f64`, narrowing by a fixed factor (e.g., ×0.85 on correct, ×1.2 on incorrect), clamped to [min_pct, max_pct]
- The 3rd note is the offset one (not 4th) per iOS Epic 56 refinement — "user has reference clicks on both sides"
- Pattern for the scheduler: `[Play, Play, OffsetPlay, Play]` where `OffsetPlay` is scheduled at `beat_time + offset_ms`

## Dev Agent Record

### Implementation Plan

- Followed existing session pattern from `PitchDiscriminationSession`
- `AdaptiveRhythmOffsetStrategy` tracks independent `early_difficulty_pct` / `late_difficulty_pct`
- Narrowing factor ×0.85 on correct, widening factor ×1.2 on incorrect, clamped to [1%, 20%]
- Session uses generic port traits (`ProfileUpdating`, `TrainingRecordPersisting`, `ProgressTimelineUpdating`)
- `StatisticsKey::Rhythm(RhythmOffsetDetection, tempo_range, direction)` used for profile updates
- `RhythmOffsetDetectionTrialParams` struct exposes trial data to web layer

### Debug Log

No issues encountered during implementation.

### Completion Notes

- Implemented `RhythmOffsetDetectionSessionState` enum with 4 states: Idle, Playing, AwaitingAnswer, ShowingFeedback
- Implemented `AdaptiveRhythmOffsetStrategy` with per-direction difficulty tracking
- Implemented `RhythmOffsetDetectionSession` with full state machine and port trait integration
- 31 unit tests covering: adaptive strategy (10 tests), session state transitions (13 tests), port call verification (2 tests), invalid state transitions (4 tests), trial parameter generation (2 tests)
- All 439 domain tests pass, clippy clean, cargo fmt applied

## File List

- `domain/src/session/rhythm_offset_detection_session.rs` (new) — Session state machine, adaptive strategy, and tests
- `domain/src/session/mod.rs` (modified) — Added module declaration and re-exports
- `domain/src/lib.rs` (modified) — Added public re-exports for new types

## Change Log

- 2026-03-25: Implemented rhythm offset detection session state machine with adaptive per-direction difficulty strategy (Story 17.3)
