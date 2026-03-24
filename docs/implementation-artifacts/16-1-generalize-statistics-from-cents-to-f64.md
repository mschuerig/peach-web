# Story 16.1: Generalize TrainingDisciplineStatistics from Cents to f64

Status: done

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

- [x] Task 1: Simplify `WelfordAccumulator` to use `f64` directly (or add `f64: WelfordMeasurement`)
- [x] Task 2: Update `TrainingDisciplineStatistics` to use `f64` instead of `Cents`
- [x] Task 3: Update `MetricPoint` type parameter (simplify if appropriate)
- [x] Task 4: Update `PerceptualProfile` API signatures
- [x] Task 5: Update `ProgressTimeline` to use f64 metrics
- [x] Task 6: Update all call sites (profile observer bridge code, hydration, etc.)
- [x] Task 7: Update tests — remove `Cents(...)` wrappers, use raw `f64`
- [x] Task 8: `cargo test --workspace` and `cargo clippy --workspace` pass

## Dev Notes

- This is a simplification, not an abstraction increase. `Cents` was a newtype around `f64` that added no behavior beyond `WelfordMeasurement::statistical_value(&self) -> f64` (which just returned the inner value). Removing the indirection makes the code clearer.
- The `Cents` type itself (in `domain/src/types/`) remains unchanged — it's still used for pitch intervals and detuning. Only the statistics engine drops it.
- Check whether `comparison_mean()` / `matching_mean()` return `Option<Cents>` or `Option<f64>` — update to `Option<f64>`.

## Dev Agent Record

### Implementation Plan

Simplified the statistics engine by removing the `WelfordMeasurement` trait abstraction entirely:
- `WelfordAccumulator` is now a concrete struct operating on `f64` (no generics, no PhantomData)
- `MetricPoint` is now a concrete struct with `f64` value field (no generics)
- Removed `typed_mean()` and `typed_std_dev()` methods (redundant when operating on `f64`)
- All call sites updated to pass raw `f64` values instead of `Cents::new(...)` wrappers
- `discrimination_mean()` and `matching_mean()` still return `Option<Cents>` for the strategy API

### Debug Log

No issues encountered. Straightforward mechanical refactoring.

### Completion Notes

- ✅ Removed `WelfordMeasurement` trait and its `Cents` implementation
- ✅ Simplified `WelfordAccumulator` from generic `WelfordAccumulator<M>` to concrete `WelfordAccumulator` using `f64`
- ✅ Simplified `MetricPoint` from generic `MetricPoint<M>` to concrete `MetricPoint` with `f64` value
- ✅ Updated `TrainingDisciplineStatistics` fields and methods to use simplified types
- ✅ Updated `PerceptualProfile` API (`add_point`, `rebuild_all`) to take `MetricPoint` (f64-based)
- ✅ `ProgressTimeline` already used `f64` directly — no changes needed (AC5 satisfied)
- ✅ Updated web crate call sites: `bridge.rs` observers and `app.rs` hydration
- ✅ Updated domain tests: unit tests, integration tests (profile_hydration, strategy_convergence), session tests
- ✅ All 356 domain tests pass, 0 clippy warnings, code formatted

## File List

- domain/src/welford.rs — Removed `WelfordMeasurement` trait, simplified `WelfordAccumulator` to use `f64`
- domain/src/metric_point.rs — Simplified from `MetricPoint<M>` to `MetricPoint` with `f64` value
- domain/src/training_discipline_statistics.rs — Updated to use simplified types
- domain/src/profile.rs — Updated API signatures, replaced `typed_std_dev()` with `population_std_dev().map(Cents::new)`
- domain/src/lib.rs — Removed `WelfordMeasurement` from public exports
- domain/src/strategy.rs — Updated test MetricPoint construction
- domain/src/session/pitch_discrimination_session.rs — Updated test MetricPoint construction
- domain/src/session/pitch_matching_session.rs — Updated test MetricPoint construction
- domain/tests/profile_hydration.rs — Removed `Cents::new()` wrappers in MetricPoint construction
- domain/tests/strategy_convergence.rs — Removed `Cents::new()` wrapper in MetricPoint construction
- web/src/bridge.rs — Removed `domain::Cents::new()` wrappers in MetricPoint construction
- web/src/app.rs — Simplified `Vec<MetricPoint<domain::Cents>>` to `Vec<MetricPoint>`, removed `Cents::new()` wrappers

## Change Log

- 2026-03-24: Generalized statistics engine from Cents to f64. Removed WelfordMeasurement trait, simplified WelfordAccumulator and MetricPoint to concrete f64-based types. All 356 tests pass.
