# Story 16.2: StatisticsKey and Profile Redesign

Status: draft

## Story

As a developer,
I want the perceptual profile to use typed `StatisticsKey`s instead of `TrainingDiscipline` as map keys,
so that rhythm disciplines can expand to multiple keys (per tempo range × direction) while pitch disciplines keep 1:1 key mapping.

## Context

Pitch disciplines have a simple 1:1 mapping: one discipline → one statistics entry. Rhythm disciplines need **multi-key expansion**: each rhythm discipline tracks statistics across 3 tempo ranges (slow/medium/fast) × 2 directions (early/late) = 6 entries per discipline.

The iOS solution: `StatisticsKey` enum with discipline-specific key expansion.

### iOS Reference

```swift
enum StatisticsKey: Hashable {
    case pitch(TrainingDisciplineID)
    case rhythm(TrainingDisciplineID, TempoRange, RhythmDirection)
}
```

Profile: `[StatisticsKey: TrainingDisciplineStatistics]` with `mergedStatistics(for: [StatisticsKey])` to aggregate across keys.

## Acceptance Criteria

1. **AC1 — StatisticsKey enum:**
   ```rust
   pub enum StatisticsKey {
       Pitch(TrainingDiscipline),
       Rhythm(TrainingDiscipline, TempoRange, RhythmDirection),
   }
   ```
   Derives `Clone, Copy, Hash, Eq, Debug`.

2. **AC2 — TempoRange type:** Enum with `Slow` (40–79 BPM), `Medium` (80–119), `Fast` (120–200). Method `from_bpm(TempoBPM) -> TempoRange`.

3. **AC3 — RhythmDirection type:** Enum `Early` / `Late`. Method `from_offset_ms(f64) -> RhythmDirection` (negative → Early, zero/positive → Late).

4. **AC4 — Profile uses StatisticsKey:** `PerceptualProfile` stores `HashMap<StatisticsKey, TrainingDisciplineStatistics>`.

5. **AC5 — Key expansion on TrainingDiscipline:** Each discipline provides its key set:
   - Pitch disciplines: `vec![StatisticsKey::Pitch(self)]` (1 key)
   - Rhythm disciplines: expand across `TempoRange::ALL × RhythmDirection::ALL` (6 keys)

6. **AC6 — Merged statistics:** `PerceptualProfile::merged_statistics(keys: &[StatisticsKey]) -> Option<TrainingDisciplineStatistics>` aggregates metrics from multiple keys chronologically and recomputes Welford/EWMA/trend.

7. **AC7 — Per-discipline convenience:** `PerceptualProfile::discipline_statistics(discipline) -> Option<TrainingDisciplineStatistics>` calls `merged_statistics()` with the discipline's key expansion. This replaces the old `statistics(mode)` method.

8. **AC8 — Initialization:** `PerceptualProfile::new()` initializes all keys for all disciplines (4 pitch keys + 12 rhythm keys = 16 total for 6 disciplines).

9. **AC9 — Backward-compat APIs removed:** `comparison_mean()`, `matching_mean()`, `matching_std_dev()`, `matching_sample_count()` are deleted. They were strategy-facing helpers that can be replaced with `discipline_statistics(mode).map(|s| s.welford.mean())`.

10. **AC10 — add_point updated:** `PerceptualProfile::add_point(key: StatisticsKey, point: MetricPoint, config: &TrainingDisciplineConfig)` takes a specific key, not a discipline.

11. **AC11 — All tests updated and pass:** Existing pitch tests use `StatisticsKey::Pitch(discipline)`. New tests verify rhythm key expansion and merging.

## Tasks / Subtasks

- [ ] Task 1: Add `TempoRange` type to `domain/src/types/`
- [ ] Task 2: Add `RhythmDirection` type to `domain/src/types/`
- [ ] Task 3: Create `StatisticsKey` enum in `domain/src/`
- [ ] Task 4: Add `statistics_keys()` method to `TrainingDiscipline`
- [ ] Task 5: Refactor `PerceptualProfile` to use `HashMap<StatisticsKey, ...>`
- [ ] Task 6: Implement `merged_statistics()` and `discipline_statistics()`
- [ ] Task 7: Update `add_point()` to take `StatisticsKey`
- [ ] Task 8: Remove backward-compat APIs
- [ ] Task 9: Update all call sites in domain crate (strategy, etc.)
- [ ] Task 10: Update web crate call sites (bridge observers, hydration, profile view)
- [ ] Task 11: Update and add tests
- [ ] Task 12: `cargo test --workspace` and `cargo clippy --workspace` pass

## Dev Notes

- `TempoRange` and `RhythmDirection` are domain types, not settings. They live alongside `TempoBPM` in `domain/src/types/`.
- The strategy module currently calls `profile.comparison_mean(interval)` — replace with `profile.discipline_statistics(discipline).and_then(|s| s.welford.mean())`.
- `ProgressTimeline` must also be updated to use `StatisticsKey` — it currently iterates `TrainingDiscipline::ALL` and extracts metrics. Consider whether `ProgressTimeline` should show merged per-discipline timelines or per-key timelines. For now, per-discipline (merged) is simplest and matches the current UI.
