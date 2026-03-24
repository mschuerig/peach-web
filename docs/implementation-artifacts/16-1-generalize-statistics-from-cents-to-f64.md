# Story 16.1: Generalize TrainingDisciplineStatistics from Cents to f64

Status: draft

## Story

As a developer,
I want statistics tracking to work with any f64 metric value (not just Cents),
so that rhythm disciplines can use percentage-of-sixteenth-note as their metric unit.

## Context

`TrainingDisciplineStatistics` currently uses `WelfordAccumulator<Cents>` and `Vec<MetricPoint<Cents>>`. Rhythm disciplines measure accuracy as "% of sixteenth note" — a different unit but the same mathematical operations (mean, stddev, EWMA, trend).

The generalization is straightforward: replace `Cents` with `f64` in the statistics engine. The `WelfordAccumulator` is already generic via the `WelfordMeasurement` trait; we just need `MetricPoint<f64>` instead of `MetricPoint<Cents>`.

### iOS Reference

iOS `TrainingDisciplineStatistics` stores `[MetricPoint]` with `Double` values. The `StatisticalSummary.continuous(stats)` variant wraps the generic statistics. The unit interpretation (cents vs. percentage) lives in the discipline config, not in the statistics engine.

## Acceptance Criteria

1. **AC1 — Statistics use f64:** `TrainingDisciplineStatistics` stores `WelfordAccumulator<f64>` and `Vec<MetricPoint<f64>>` instead of `WelfordAccumulator<Cents>` / `MetricPoint<Cents>`.

2. **AC2 — MetricPoint simplified:** If `MetricPoint<Cents>` is the only parameterization used, simplify to `MetricPoint` with an `f64` value field. If `MetricPoint` is already generic and used with other types, keep it generic but ensure `f64` works.

3. **AC3 — WelfordAccumulator works with f64:** Either `f64` implements `WelfordMeasurement`, or the accumulator is simplified to always use `f64` (since `Cents` just delegates to its inner `f64` anyway).

4. **AC4 — Profile API updated:** `PerceptualProfile::add_point()` takes `MetricPoint<f64>` (or simplified `MetricPoint`). The `is_correct` filtering for comparison modes remains.

5. **AC5 — ProgressTimeline updated:** Display buckets work with `f64` metric values.

6. **AC6 — All existing tests pass:** The change is a type simplification, not a behavioral change. All existing tests pass with minimal mechanical updates (removing `.into()` calls or `Cents(...)` wrappers).

7. **AC7 — Unit label on config:** The `TrainingDisciplineConfig::unit_label` field already distinguishes "cents" vs. "% of 16th" — no behavioral change needed, just ensure rhythm configs (added in 15.1) use the correct label.

## Tasks / Subtasks

- [ ] Task 1: Simplify `WelfordAccumulator` to use `f64` directly (or add `f64: WelfordMeasurement`)
- [ ] Task 2: Update `TrainingDisciplineStatistics` to use `f64` instead of `Cents`
- [ ] Task 3: Update `MetricPoint` type parameter (simplify if appropriate)
- [ ] Task 4: Update `PerceptualProfile` API signatures
- [ ] Task 5: Update `ProgressTimeline` to use f64 metrics
- [ ] Task 6: Update all call sites (profile observer bridge code, hydration, etc.)
- [ ] Task 7: Update tests — remove `Cents(...)` wrappers, use raw `f64`
- [ ] Task 8: `cargo test --workspace` and `cargo clippy --workspace` pass

## Dev Notes

- This is a simplification, not an abstraction increase. `Cents` was a newtype around `f64` that added no behavior beyond `WelfordMeasurement::statistical_value(&self) -> f64` (which just returned the inner value). Removing the indirection makes the code clearer.
- The `Cents` type itself (in `domain/src/types/`) remains unchanged — it's still used for pitch intervals and detuning. Only the statistics engine drops it.
- Check whether `comparison_mean()` / `matching_mean()` return `Option<Cents>` or `Option<f64>` — update to `Option<f64>`.
