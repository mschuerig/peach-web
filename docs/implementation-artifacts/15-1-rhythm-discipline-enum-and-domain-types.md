# Story 15.1: Rhythm Discipline Enum Cases and Domain Types

Status: draft

## Story

As a developer,
I want the `TrainingDiscipline` enum extended with rhythm cases and basic rhythm domain types added,
so that the start screen and routing infrastructure can reference rhythm disciplines.

## Context

Prerequisite: Epic 14 (terminology rename) is complete. `TrainingDiscipline` exists with 4 pitch variants.

This story adds the two new rhythm variants and the minimal domain types they need for configuration and settings. Full session state machines, adaptive strategies, and profile integration come in later epics.

### iOS Reference

- `TrainingDisciplineID`: 6 cases total (4 pitch + 2 rhythm)
- `TempoBPM`: Int newtype, range 40–200, default 80
- `StepPosition`: enum with 4 cases (first/second/third/fourth)
- Rhythm configs use `"% of 16th"` as unit label

## Acceptance Criteria

1. **AC1 — New enum variants:**
   ```rust
   TrainingDiscipline::RhythmOffsetDetection
   TrainingDiscipline::ContinuousRhythmMatching
   ```

2. **AC2 — `ALL` constant updated:** `TrainingDiscipline::ALL` contains all 6 variants.

3. **AC3 — Configs for rhythm disciplines:**
   - `RhythmOffsetDetection`: display_name = `"training-mode-compare-timing"`, unit_label = `"% of 16th"`, optimal_baseline = 5.0, standard EWMA/session gap
   - `ContinuousRhythmMatching`: display_name = `"training-mode-fill-the-gap"`, unit_label = `"% of 16th"`, optimal_baseline = 5.0, standard EWMA/session gap

4. **AC4 — `TempoBPM` type:** Newtype wrapping `u16`. Range 40–200 BPM. Default 80. Constructor validates range. Provides `sixteenth_note_duration_secs() -> f64` (= 60.0 / (bpm * 4)).

5. **AC5 — `StepPosition` type:** Enum with variants `First`, `Second`, `Third`, `Fourth`. Derives `Clone, Copy, Hash, Eq`. Display labels: "Beat", "E", "And", "A".

6. **AC6 — Metric extraction returns `None` for rhythm:** `extract_comparison_metric()` and `extract_matching_metric()` return `None` for rhythm variants (rhythm has its own metric extraction, added later).

7. **AC7 — Slug values:**
   - `RhythmOffsetDetection` → `"rhythm-offset-detection"`
   - `ContinuousRhythmMatching` → `"continuous-rhythm-matching"`

8. **AC8 — No profile breakage:** `PerceptualProfile` initializes rhythm disciplines with `NoData` state. No rhythm-specific statistics logic yet.

9. **AC9 — All tests pass:** `cargo test -p domain` and `cargo clippy --workspace` pass. Existing tests unaffected.

## Tasks / Subtasks

- [ ] Task 1: Add `TempoBPM` type to `domain/src/types/`
- [ ] Task 2: Add `StepPosition` type to `domain/src/types/`
- [ ] Task 3: Extend `TrainingDiscipline` enum with rhythm variants + configs
- [ ] Task 4: Update `ALL` constant and `matches_interval()` (return `false` for rhythm)
- [ ] Task 5: Ensure `extract_comparison_metric` / `extract_matching_metric` return `None` for rhythm
- [ ] Task 6: Update `PerceptualProfile::new()` to initialize rhythm disciplines
- [ ] Task 7: Add unit tests for new types and configs
- [ ] Task 8: `cargo test -p domain` and `cargo clippy --workspace` pass

## Dev Notes

- `TempoBPM` and `StepPosition` go in `domain/src/types/` alongside existing value types
- The rhythm configs use placeholder optimal_baseline values (5.0) — these will be tuned during rhythm implementation
- `PerceptualProfile` will need the rhythm variants in its map, but with no special statistics handling yet
- Do NOT add rhythm records, sessions, or observers — those come in Epics 17–18
