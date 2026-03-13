# Story 12.1: Progress Data Pipeline

Status: review

## Story

As a developer,
I want the domain crate to extract metric points from training records, bucket them into multi-granularity display zones, compute EWMA over session-level buckets, and determine trend direction,
So that the chart rendering layer receives fully computed, ready-to-render data structures.

## Acceptance Criteria

1. **Metric extraction** — Given PitchComparisonRecord entries with interval == 0, metric extraction for UnisonPitchComparison produces (timestamp, abs(centOffset)) pairs; records with interval != 0 are excluded. Same pattern for IntervalPitchComparison (interval != 0), UnisonMatching (interval == 0, abs(userCentError)), IntervalMatching (interval != 0, abs(userCentError)).

2. **Three-zone display bucketing** — Given metric points spanning months, recent days, and today, with current time T and local-midnight epoch `start_of_today`:
   - Session zone: timestamps >= `start_of_today`, grouped by 30-minute session gap rule
   - Day zone: timestamps >= `start_of_today - 7*86400` AND < `start_of_today`, grouped by calendar day (floor to day boundary)
   - Month zone: timestamps < `start_of_today - 7*86400`, grouped by calendar month
   - Last monthly bucket's end date truncated to the day zone boundary
   - All buckets concatenated chronologically: months, then days, then sessions

3. **Per-bucket aggregation** — Each bucket computes mean = sum(values)/count and population stddev = sqrt(sum((v-mean)^2)/count). Single-record buckets have stddev = 0.

4. **Separate EWMA pipeline** — Session-level buckets (30-min gap rule across ALL records, independent of display zones) used for EWMA: ewma[0] = bucket[0].mean; for i>0: alpha = 1.0 - exp(-ln(2) * dt / 604800), ewma[i] = alpha * bucket[i].mean + (1-alpha) * ewma[i-1]. Current EWMA = ewma[last].

5. **Trend direction** — Requires >= 2 individual records. Running mean and population stddev computed across all individual metric values (Welford's). If latestBucketMean > runningMean + runningStddev: Declining. Else if latestBucketMean >= currentEWMA: Stable. Else: Improving. With < 2 records: trend = None.

6. **Per-mode chart parameters** — TrainingMode.config() returns: UnisonComparison (baseline=8, halfLife=604800, gap=1800), IntervalComparison (baseline=12, halfLife=604800, gap=1800), UnisonMatching (baseline=5, halfLife=604800, gap=1800), IntervalMatching (baseline=8, halfLife=604800, gap=1800).

7. **Pure domain** — All pipeline code runs with `cargo test -p domain`, zero browser dependencies.

## Tasks / Subtasks

- [x] Task 1: Refactor display bucketing to three-zone calendar-day-snapped (AC: 2, 3)
  - [x] 1.1 Add `start_of_today: f64` parameter to `rebuild()` and `build_display_buckets()` (local midnight epoch, passed from web layer via `js_sys::Date`)
  - [x] 1.2 Implement `build_display_buckets(points, start_of_today, session_gap)` with three zones: Month (< start_of_today - 7d), Day (>= start_of_today - 7d AND < start_of_today), Session (>= start_of_today)
  - [x] 1.3 Calendar-day grouping for Day zone: floor timestamp to day boundary relative to start_of_today
  - [x] 1.4 Calendar-month grouping for Month zone: group by (year, month) derived from epoch
  - [x] 1.5 Truncate last monthly bucket's end to the day zone boundary
  - [x] 1.6 Session grouping within Session zone using 30-min gap rule
  - [x] 1.7 Change stddev to population formula: `sqrt(m2 / n)` instead of `sqrt(m2 / (n-1))`
  - [x] 1.8 Remove `BucketSize::Week` variant (no longer used in display pipeline)
- [x] Task 2: Separate EWMA session pipeline (AC: 4)
  - [x] 2.1 Add `ewma_buckets: Vec<TimeBucket>` field to ModeState (session-level only, independent of display)
  - [x] 2.2 Implement `build_ewma_session_buckets(points, session_gap)` — pure 30-min gap grouping with NO zone logic
  - [x] 2.3 EWMA computation reads from `ewma_buckets`, not `display_buckets`
  - [x] 2.4 Both pipelines rebuilt on `rebuild()` and updated on `add_comparison()`/`add_matching()`
- [x] Task 3: Fix trend stddev to population formula (AC: 5)
  - [x] 3.1 Change `running_stddev` from `sqrt(m2 / (n-1))` to `sqrt(m2 / n)` in `recompute_trend()`
- [x] Task 4: Add public API for chart rendering (AC: 1, 2, 4, 5, 6)
  - [x] 4.1 `display_buckets(mode) -> Vec<TimeBucket>` — returns three-zone display buckets
  - [x] 4.2 `latest_bucket_stddev(mode) -> Option<f64>` — stddev of last display bucket (for headline ±)
  - [x] 4.3 Ensure existing `current_ewma(mode)`, `trend(mode)`, `state(mode)` still work
  - [x] 4.4 Rename old `buckets()` to `display_buckets()` or deprecate
- [x] Task 5: Update and add tests (AC: 1-7)
  - [x] 5.1 Update existing tests for population stddev (divisor n, not n-1)
  - [x] 5.2 Test three-zone bucketing: records spanning months/days/today correctly assigned
  - [x] 5.3 Test calendar-day boundary snapping (not age-based)
  - [x] 5.4 Test monthly bucket end truncation at day zone boundary
  - [x] 5.5 Test EWMA uses session pipeline, not display pipeline
  - [x] 5.6 Test trend with population running stddev
  - [x] 5.7 Test empty zones (e.g., no session data today, only historical)
  - [x] 5.8 Test incremental add_comparison/add_matching produce same results as rebuild
  - [x] 5.9 Verify `cargo test -p domain` passes, `cargo clippy --workspace` zero warnings

## Dev Notes

### What Already Exists (DO NOT REINVENT)

The following are **already implemented and working** — extend, don't recreate:

- **`domain/src/progress_timeline.rs`** — `ProgressTimeline`, `TimeBucket`, `BucketSize`, `ModeState` with EWMA, Welford's running stats, trend computation, incremental updates, ISO 8601 parser (`parse_iso8601_to_epoch`), epoch-to-ISO helper (test only)
- **`domain/src/training_mode.rs`** — `TrainingMode` enum with `ALL` array, `TrainingModeConfig` with per-mode constants, `extract_comparison_metric()`, `extract_matching_metric()`, `TrainingModeState`
- **`domain/src/trend.rs`** — `Trend` enum (Improving, Stable, Declining) with serde camelCase
- **`domain/src/records.rs`** — `PitchComparisonRecord` (cent_offset, interval, timestamp), `PitchMatchingRecord` (user_cent_error, interval, timestamp)

### What Must Change

**`progress_timeline.rs` refactoring (primary file):**

1. **Remove `BucketSize::Week`** — The iOS v3 spec uses three zones only (Session, Day, Month). Check for any downstream usage of `Week` in `web/src/components/progress_chart.rs` and update.

2. **Replace `build_adaptive_buckets()` with `build_display_buckets()`** — New function signature:
   ```rust
   fn build_display_buckets(
       points: &[(f64, f64)],    // sorted (timestamp, metric)
       start_of_today: f64,      // local midnight epoch (NEW parameter)
       session_gap_secs: f64,
   ) -> Vec<TimeBucket>
   ```
   Zone boundaries:
   - `day_zone_start = start_of_today - 7 * 86400`
   - Session zone: `ts >= start_of_today`
   - Day zone: `ts >= day_zone_start && ts < start_of_today`
   - Month zone: `ts < day_zone_start`

3. **Add `build_ewma_session_buckets()`** — Separate function for EWMA pipeline, pure session-gap grouping with no zone logic. Returns session-level buckets for EWMA computation only.

4. **Change all stddev to population formula** — Three locations:
   - `build_display_buckets()` aggregation: `sqrt(m2 / n)` not `sqrt(m2 / (n-1))`
   - `ModeState::add_point()` incremental session update: `sqrt(m2 / n)` not `sqrt(m2 / (n-1))`
   - `ModeState::recompute_trend()` running stddev: `sqrt(m2 / n)` not `sqrt(m2 / (n-1))`

5. **Add `start_of_today` parameter** to `rebuild()` and `add_point()` — The web layer computes this from `js_sys::Date` as local midnight epoch. For `add_comparison()`/`add_matching()`, the caller already passes `now: f64`; derive `start_of_today` from it or pass separately.

6. **Calendar month grouping** — For Month zone, group by (year, month). Use `parse_year_month_from_epoch()` helper (derive year/month from epoch seconds, similar to existing `parse_iso8601_to_epoch` logic). The period_start for a month bucket = epoch of first day of that month.

### Critical Algorithm Details

**Population stddev (NOT sample stddev):**
```rust
// CORRECT (population, divide by n):
let stddev = if n > 0 { (m2 / n as f64).sqrt() } else { 0.0 };

// WRONG (sample, divide by n-1) — this is what the current code does:
let stddev = if n > 1 { (m2 / (n - 1) as f64).sqrt() } else { 0.0 };
```

**EWMA formula (unchanged, but reads from session pipeline):**
```rust
ewma[0] = session_buckets[0].mean;
for i in 1..session_buckets.len() {
    let dt = session_buckets[i].period_start - session_buckets[i-1].period_start;
    let alpha = 1.0 - (-f64::ln(2.0) * dt / halflife).exp();
    ewma[i] = alpha * session_buckets[i].mean + (1.0 - alpha) * ewma[i-1];
}
```

**Trend direction (order matters):**
```rust
if latest > running_mean + running_pop_stddev { Declining }
else if latest >= ewma { Stable }
else { Improving }
```

### start_of_today — Bridging Domain and Web

The domain crate must remain pure Rust (no `js_sys`). The web layer provides `start_of_today` as a parameter:

```rust
// Web layer (app.rs or bridge.rs):
let now_ms = js_sys::Date::now(); // milliseconds
let date = js_sys::Date::new_0();
date.set_hours(0); date.set_minutes(0); date.set_seconds(0); date.set_milliseconds(0);
let start_of_today = date.get_time() / 1000.0; // epoch seconds, local midnight
```

This preserves the domain crate's testability — tests can pass any `start_of_today` value.

### Downstream Impact

- **`web/src/components/progress_chart.rs`** — Currently uses `BucketSize::Week` for axis label formatting. Must update to handle only Session/Day/Month. The chart component calls `timeline.buckets(mode)` — rename to `display_buckets(mode)`.
- **`web/src/app.rs`** — `rebuild()` call needs the new `start_of_today` parameter.
- **`web/src/bridge.rs`** — `add_comparison()`/`add_matching()` observers need to pass `start_of_today` or derive from `now`.

These downstream changes are **out of scope** for this story (they belong to story 12.2), but the API design must accommodate them. Ensure backward compatibility or document the breaking change clearly.

### Project Structure Notes

- All new code in `domain/src/progress_timeline.rs` (extend existing module)
- No new files needed — this is a refactoring of the existing data pipeline
- Tests inline in `#[cfg(test)] mod tests` within `progress_timeline.rs`
- No new dependencies needed in `domain/Cargo.toml`

### References

- [Source: docs/planning-artifacts/epics.md#Story 12.1: Progress Data Pipeline] — Full BDD acceptance criteria
- [Source: docs/ios-reference/profile-screen-specification.md] — Authoritative reference for bucketing, EWMA, trend algorithms
- [Source: docs/planning-artifacts/epics.md#Epic 12: Profile Progress Charts] — PFR1-PFR16, PNFR1-PNFR5, additional requirements
- [Source: domain/src/progress_timeline.rs] — Existing implementation (story 7.2)
- [Source: domain/src/training_mode.rs] — TrainingMode enum and config
- [Source: domain/src/trend.rs] — Trend enum
- [Source: docs/project-context.md] — Coding conventions, numeric precision (f64), crate separation rules

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

No debug issues encountered.

### Completion Notes List

- Replaced `build_adaptive_buckets()` with `build_display_buckets()` implementing three-zone bucketing (Month/Day/Session) based on `start_of_today` parameter
- Added `build_ewma_session_buckets()` for separate EWMA pipeline using pure 30-min gap grouping independent of display zones
- Changed all stddev computations from sample (n-1 divisor) to population (n divisor) in three locations: `aggregate_bucket()`, `welford_update_bucket()`, and `recompute_trend()`
- Removed `BucketSize::Week` variant; updated `progress_chart.rs` match arm
- Added `start_of_today: f64` parameter to `rebuild()`, `add_comparison()`, `add_matching()`
- Renamed `buckets()` to `display_buckets()` on `ProgressTimeline`
- Added `latest_bucket_stddev(mode)` public API method
- Added `ewma_buckets` field to `ModeState` for separate EWMA pipeline
- Added helper functions: `floor_to_day()`, `epoch_to_month_start()`, `epoch_to_year_month()`, `year_month_to_epoch()`, `aggregate_bucket()`, `welford_update_bucket()`, `update_session_bucket()`, `update_or_create_bucket()`
- Updated web crate callers: `app.rs` (rebuild), `bridge.rs` (add_comparison/add_matching), `progress_card.rs` (display_buckets), `progress_sparkline.rs` (display_buckets)
- Added `compute_start_of_today()` helper in `bridge.rs` for local midnight epoch
- 27 tests in progress_timeline module covering all acceptance criteria
- All 343 domain tests pass, zero clippy warnings across workspace

### File List

- domain/src/progress_timeline.rs (modified)
- web/src/app.rs (modified)
- web/src/bridge.rs (modified)
- web/src/components/progress_chart.rs (modified)
- web/src/components/progress_card.rs (modified)
- web/src/components/progress_sparkline.rs (modified)
- docs/implementation-artifacts/sprint-status.yaml (modified)
- docs/implementation-artifacts/12-1-progress-data-pipeline.md (modified)

## Change Log

- 2026-03-13: Story 12.1 implemented — three-zone display bucketing, separate EWMA pipeline, population stddev, BucketSize::Week removed, public API added
