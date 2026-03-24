# Pre-Existing Findings Catalog

Single source of truth for all known pre-existing issues. Every finding has a unique ID and a disposition.

**Dispositions:** OPEN — needs fixing or a story. CLOSED — fixed, with commit/story reference. WONT-FIX — accepted risk with rationale.

## Findings

### PEF-001: NaN/infinity propagation through WelfordAccumulator

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/welford.rs` — `WelfordAccumulator::update()`
- **Description:** If a NaN or infinity `f64` is passed to `update()`, it permanently poisons `mean`, `m2`, and all downstream statistics (EWMA, trend, std dev). Callers currently pass values derived from `.abs()` on known-finite fields, but there is no guard in the accumulator itself.
- **Recommendation:** Add `debug_assert!(value.is_finite())` in `update()`, or return early / return `Result`.

### PEF-002: NaN timestamp from parse_iso8601_to_epoch breaks session bucketing

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/training_discipline_statistics.rs:82`, `web/src/app.rs`
- **Description:** If `parse_iso8601_to_epoch` returns NaN (e.g., malformed timestamp from IndexedDB), the session gap comparison `point.timestamp - prev.timestamp >= session_gap` evaluates to `false`, silently merging bad points into sessions. The `partial_cmp` sort also swallows NaN via `unwrap_or(Equal)`.
- **Recommendation:** Validate or filter out non-finite timestamps at the hydration boundary in `app.rs`.

### PEF-003: No non-negative validation on MetricPoint::value

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/metric_point.rs`
- **Description:** The statistics engine assumes values represent absolute errors (non-negative). Callers do call `.abs()` before constructing points, but `MetricPoint::new` accepts any `f64` including negative values. No compile-time or runtime signal that negative inputs are a logic error.
- **Recommendation:** Add `debug_assert!(value >= 0.0)` in `MetricPoint::new`, or document the contract.

### PEF-004: WelfordAccumulator::mean() returns 0.0 on empty accumulator

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/welford.rs` — `mean()`
- **Description:** `mean()` returns `0.0` when `count == 0` rather than `Option<f64>`. All current callers guard with `count > 0` checks, but the public API is a footgun for future callers.
- **Recommendation:** Change return type to `Option<f64>`, or add a doc comment making the 0.0 contract explicit.
