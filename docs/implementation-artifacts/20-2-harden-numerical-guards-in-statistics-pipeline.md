# Story 20.2: Harden Numerical Guards in Statistics Pipeline

Status: pending

## Story

As a user,
I want the statistics engine to gracefully reject invalid float values at runtime,
so that a single poisoned measurement (NaN, infinity, negative) cannot silently corrupt my training profile.

## Context

Adversarial review finding ARV-003: `WelfordAccumulator::update()` and `MetricPoint::new()` use `debug_assert!` to guard against non-finite and negative values. These checks are stripped in release builds (`opt-level = 'z'`), meaning production WASM silently accepts poisoned values.

A NaN entering Welford's algorithm poisons `mean` and `m2` irreversibly — every subsequent update produces NaN. Since `WelfordAccumulator` feeds `TrainingDisciplineStatistics`, which feeds the profile and progress charts, one bad value corrupts the user's entire discipline history.

### Root Cause

The original `debug_assert!` was a reasonable choice during initial development, but the statistics pipeline is now a production data path receiving values derived from browser audio timing (`audioContext.currentTime`), pointer events, and float arithmetic — all of which can produce edge-case non-finite results.

## Acceptance Criteria

1. **AC1 — Welford rejects non-finite values:** `WelfordAccumulator::update()` silently skips (early return, no state change) if `value` is NaN or infinity. A `log::warn!` is emitted behind the `training-log` feature flag.

2. **AC2 — MetricPoint rejects invalid values:** `MetricPoint::new()` returns `Option<Self>`, returning `None` for negative, NaN, or infinite values. Callers updated accordingly.

3. **AC3 — Callers handle None:** All call sites of `MetricPoint::new()` use `if let Some(point)` or equivalent, skipping the metric point if invalid. No panics, no silent corruption.

4. **AC4 — Tests for rejection:** Unit tests confirm:
   - `WelfordAccumulator::update(f64::NAN)` leaves state unchanged
   - `WelfordAccumulator::update(f64::INFINITY)` leaves state unchanged
   - `WelfordAccumulator` state remains valid after reject (subsequent valid updates work)
   - `MetricPoint::new(ts, f64::NAN)` returns `None`
   - `MetricPoint::new(ts, -1.0)` returns `None`
   - `MetricPoint::new(ts, f64::INFINITY)` returns `None`

5. **AC5 — No behavior change for valid inputs:** All existing tests pass unchanged.

## Tasks / Subtasks

- [ ] Task 1: Harden `WelfordAccumulator::update()` (AC: 1, 4, 5)
  - [ ] 1.1 Replace `debug_assert!` with early-return guard on `!value.is_finite()`
  - [ ] 1.2 Add `log::warn!` behind `training-log` feature
  - [ ] 1.3 Update doc comment (remove "Panics" section, add "Skips" note)
  - [ ] 1.4 Add tests for NaN, infinity, and post-reject valid update

- [ ] Task 2: Change `MetricPoint::new()` to return `Option<Self>` (AC: 2, 4, 5)
  - [ ] 2.1 Change signature to `pub fn new(timestamp: f64, value: f64) -> Option<Self>`
  - [ ] 2.2 Return `None` for negative, NaN, or infinite values
  - [ ] 2.3 Update doc comment
  - [ ] 2.4 Add tests for all rejection cases

- [ ] Task 3: Update callers of `MetricPoint::new()` (AC: 3)
  - [ ] 3.1 Find all call sites (likely in `training_discipline.rs`, `progress_timeline.rs`, `profile.rs`)
  - [ ] 3.2 Replace direct construction with `if let Some(point) = MetricPoint::new(...)` or `.and_then()`
  - [ ] 3.3 Verify no silent data loss — log a warning if a point is dropped

- [ ] Task 4: Validation (AC: 5)
  - [ ] 4.1 `cargo test -p domain` passes
  - [ ] 4.2 `cargo clippy --workspace` clean

## Dev Notes

### Design Decision: Skip vs. Panic vs. Result

- **Panic** (`assert!`): Too harsh — a single browser timing glitch shouldn't crash the app.
- **Result/Option** on Welford: Overly infectious — every caller would need error handling for an accumulator update. Early-return-and-skip is the standard approach for streaming statistics (sensor data, metrics pipelines).
- **Option on MetricPoint**: Appropriate — construction is a natural validation boundary, and callers already handle optional data flows.

### Why Not Also Guard Profile Key Lookups?

The `debug_assert!` calls in `profile.rs` and `progress_timeline.rs` guard structural invariants (keys pre-populated at initialization). These are programmer bugs, not data bugs. `debug_assert!` is correct for those — they should be caught in development and can never be triggered by user data.

### References

- [ARV-003 in docs/pre-existing-findings.adversarial-review-2025-03-25.md]
- [domain/src/welford.rs — WelfordAccumulator::update]
- [domain/src/metric_point.rs — MetricPoint::new]
