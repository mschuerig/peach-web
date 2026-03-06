# Story 7.2: ProgressTimeline with EWMA and Adaptive Bucketing

Status: done

## Story

As a musician,
I want my training progress tracked with exponentially weighted moving averages and adaptive time bucketing,
so that recent training sessions have more weight and the timeline adapts its granularity to data density.

## Context

This story replaces the existing `ThresholdTimeline` (simple daily aggregation) and the standalone `TrendAnalyzer` (half-split comparison) with a unified `ProgressTimeline` that tracks each `TrainingMode` independently using EWMA smoothing and adaptive time bucketing.

The iOS implementation lives in `Peach/Core/Profile/ProgressTimeline.swift`. The key algorithms are:
- **EWMA:** Time-weighted exponential moving average with configurable half-life (7 days). The decay factor alpha = 1 - exp(-ln(2) * dt / halflife) where dt is the time between consecutive buckets.
- **Adaptive bucketing:** Recent records (< 24h) grouped by session (30-min gap), 1-7 days by day, 1-4 weeks by week, older by month.
- **Trend:** Stddev-based — if latest value > running_mean + running_stddev then declining, if latest value < ewma then improving, else stable. Requires >= 2 records.
- **Incremental updates:** New records append to the correct mode and recompute EWMA/trend without full rebuild.

Depends on: Story 7.1 (TrainingMode, TrainingModeConfig).

## Acceptance Criteria

1. **AC1 — ProgressTimeline struct:** `ProgressTimeline` exists in the domain crate with a `new()` constructor
2. **AC2 — Rebuild from records:** `rebuild(comparison_records: &[ComparisonRecord], matching_records: &[PitchMatchingRecord], now: f64)` populates per-mode data from stored records. The `now` parameter is seconds since epoch (passed from the web layer via `Date.now()`).
3. **AC3 — Per-mode state:** `state(mode: TrainingMode) -> TrainingModeState` returns `NoData` when no records exist for that mode, `Active` otherwise
4. **AC4 — Adaptive bucketing:** `buckets(mode: TrainingMode) -> Vec<TimeBucket>` returns time-grouped buckets. Records < 24h old are session-bucketed (30-min gap), < 7 days by day, < 30 days by week, older by month.
5. **AC5 — EWMA computation:** `current_ewma(mode: TrainingMode) -> Option<f64>` returns the exponentially weighted moving average across buckets with 7-day half-life
6. **AC6 — Trend computation:** `trend(mode: TrainingMode) -> Option<Trend>` returns `None` if < 2 records; otherwise `Improving` if latest < ewma, `Declining` if latest > running_mean + running_stddev, `Stable` otherwise
7. **AC7 — TimeBucket struct:** `TimeBucket` has fields: `period_start: f64` (epoch secs), `period_end: f64`, `bucket_size: BucketSize`, `mean: f64`, `stddev: f64`, `record_count: usize`
8. **AC8 — BucketSize enum:** `BucketSize` enum with `Session`, `Day`, `Week`, `Month` variants
9. **AC9 — Incremental comparison update:** `add_comparison(&mut self, record: &ComparisonRecord, now: f64)` updates the relevant mode(s) without full rebuild
10. **AC10 — Incremental matching update:** `add_matching(&mut self, record: &PitchMatchingRecord, now: f64)` updates the relevant mode(s) without full rebuild
11. **AC11 — Reset:** `reset(&mut self)` clears all per-mode data
12. **AC12 — Bridge integration:** `bridge.rs` updated — replace `TrendObserver` and `TimelineObserver` with a `ProgressTimelineObserver` that calls `add_comparison`/`add_matching`
13. **AC13 — Hydration integration:** During app startup, `ProgressTimeline::rebuild()` is called with all stored records (same as `PerceptualProfile` hydration)
14. **AC14 — Old types retained:** `ThresholdTimeline` and `TrendAnalyzer` remain in the codebase (they may still be used by profile_view until story 7.4 replaces it). If no code references them after bridge changes, they can be removed.
15. **AC15 — Tests pass:** `cargo test -p domain` passes, `cargo clippy -p domain` has no warnings

## Tasks / Subtasks

- [x] Task 1: Create `domain/src/progress_timeline.rs` (AC: 1, 7, 8)
  - [x] Define `BucketSize` enum with `Session`, `Day`, `Week`, `Month`
  - [x] Define `TimeBucket` struct with period_start, period_end, bucket_size, mean, stddev, record_count
  - [x] Define internal `ModeState` struct: buckets, ewma, record_count, computed_trend, all_metrics, running_mean, running_m2
  - [x] Define `ProgressTimeline` struct with `HashMap<TrainingMode, ModeState>` (or array-indexed by mode)
  - [x] Implement `new()` initializing empty state for all four modes

- [x] Task 2: Implement rebuild (AC: 2, 3, 4, 5, 6)
  - [x] `rebuild()` — for each TrainingMode, extract metrics from records using `extract_comparison_metric`/`extract_matching_metric`, sort by timestamp, assign to adaptive buckets, compute EWMA and trend
  - [x] Bucket assignment: age = now - timestamp. < 86400 → Session (group by 1800s gap), < 604800 → Day, < 2592000 → Week, else → Month
  - [x] Day bucketing: floor timestamp to start of day (epoch secs / 86400 * 86400)
  - [x] Week bucketing: floor to start of week (Monday-based or simplified 7-day blocks)
  - [x] Month bucketing: floor to start of month (simplified 30-day blocks, exact calendar not needed in WASM)
  - [x] Per-bucket mean and stddev via Welford's online algorithm
  - [x] EWMA: iterate buckets chronologically, alpha = 1 - exp(-ln(2) * dt / halflife_secs)
  - [x] Trend: use running mean/stddev (Welford across ALL metrics, not per-bucket) + ewma

- [x] Task 3: Implement per-mode accessors (AC: 3, 4, 5, 6)
  - [x] `state()` — `NoData` if record_count == 0, else `Active`
  - [x] `buckets()` — return clone of mode's bucket vec
  - [x] `current_ewma()` — return mode's ewma Option
  - [x] `trend()` — return mode's computed_trend Option (None if < 2 records)

- [x] Task 4: Implement incremental updates (AC: 9, 10)
  - [x] `add_comparison()` — for each mode, try extract_comparison_metric; if Some, append to mode state
  - [x] `add_matching()` — same for matching records
  - [x] ModeState::add_point(): update running mean/m2, append to or extend last session bucket (if within gap), recompute EWMA and trend

- [x] Task 5: Implement reset (AC: 11)
  - [x] `reset()` — reinitialize all mode states to empty

- [x] Task 6: Wire into module system
  - [x] Add `pub mod progress_timeline;` to `domain/src/lib.rs`
  - [x] Add re-exports: `ProgressTimeline`, `TimeBucket`, `BucketSize`

- [x] Task 7: Update bridge.rs (AC: 12)
  - [x] Create `ProgressTimelineObserver` wrapping `Rc<RefCell<ProgressTimeline>>`
  - [x] Implement `ComparisonObserver` — calls `add_comparison()` with current timestamp
  - [x] Implement `PitchMatchingObserver` — calls `add_matching()` with current timestamp
  - [x] Add `ProgressTimelineObserver` to comparison_view.rs observers (kept TrendObserver + TimelineObserver — still referenced)
  - [x] Add `ProgressTimelineObserver` to pitch_matching_view.rs observers
  - [x] Get current time via `js_sys::Date::now() / 1000.0` (epoch seconds)

- [x] Task 8: Update hydration (AC: 13)
  - [x] In app startup (where PerceptualProfile hydration happens), also call `ProgressTimeline::rebuild()` with all fetched records
  - [x] Store ProgressTimeline in Leptos context (same pattern as PerceptualProfile)

- [x] Task 9: Clean up old types (AC: 14)
  - [x] Check if `ThresholdTimeline` and `TrendAnalyzer` are still referenced after bridge changes
  - [x] Still referenced by profile_view.rs, settings_view.rs, pitch_comparison_view.rs — left in place
  - [x] If still referenced by profile_view.rs (until story 7.4), leave them

- [x] Task 10: Write tests (AC: all)
  - [x] Test rebuild with empty records → all modes NoData
  - [x] Test rebuild with comparison records (interval=0) → unison comparison is Active, others NoData
  - [x] Test rebuild with matching records (interval!=0) → interval matching is Active
  - [x] Test bucket assignment: session-gap grouping (< 1800s gap → same bucket)
  - [x] Test bucket assignment: day grouping for records 2-6 days old
  - [x] Test EWMA: two buckets with known dt → verify alpha and weighted result
  - [x] Test trend: latest < ewma → Improving; latest > mean + stddev → Declining; else Stable
  - [x] Test incremental add_comparison updates the correct mode
  - [x] Test reset clears all data
  - [x] Run `cargo test -p domain` and `cargo clippy -p domain`

## Dev Notes

### iOS Mapping

| iOS (Swift) | peach-web (Rust) |
|---|---|
| `ProgressTimeline` (@Observable class) | `ProgressTimeline` (struct, no reactivity — signal updates happen in bridge) |
| `Date` timestamps | `f64` epoch seconds (from `js_sys::Date::now() / 1000.0`) |
| `Calendar`-based week/month intervals | Simplified arithmetic (86400s/day, 604800s/week, 2592000s/month) |
| `PitchComparisonObserver` protocol | `ComparisonObserver` trait |
| `PitchMatchingObserver` protocol | `PitchMatchingObserver` trait |

### Design Decisions

- **f64 epoch seconds for timestamps:** The existing `ComparisonRecord` and `PitchMatchingRecord` store timestamps as ISO 8601 strings. For ProgressTimeline, we parse these to f64 epoch seconds during rebuild. For incremental updates, the bridge passes `js_sys::Date::now() / 1000.0` directly.
- **Simplified calendar math:** The iOS app uses `Calendar.current` for exact week/month boundaries. In WASM, we use simpler arithmetic (7-day blocks, 30-day blocks) since exact calendar alignment is not critical for visualization.
- **No Observable/Signal in domain:** ProgressTimeline is a plain struct. The web layer wraps it in `Rc<RefCell<>>` and updates Leptos signals through the bridge observer, same pattern as PerceptualProfile.
- **Timestamp parsing:** Add a helper to parse ISO 8601 strings to epoch seconds. This can use a simple parser since our timestamps are always in the format "YYYY-MM-DDTHH:MM:SSZ".

### Architecture Compliance

- **Domain crate:** ProgressTimeline, TimeBucket, BucketSize are pure Rust with no browser dependencies. All time is passed as f64 epoch seconds.
- **Web crate:** Bridge observers and hydration wiring use `js_sys::Date::now()` and Leptos context.
- **Record types unchanged:** Uses existing `ComparisonRecord` and `PitchMatchingRecord` fields.

## Dev Agent Record

### Implementation Plan

- Created `ProgressTimeline` as a pure-Rust domain struct using `HashMap<TrainingMode, ModeState>` for per-mode tracking
- Implemented adaptive bucketing via `build_adaptive_buckets()` — groups sorted (timestamp, metric) pairs by age-based bucket size with session-gap detection
- EWMA computed chronologically across buckets with alpha = 1 - exp(-ln(2) * dt / halflife)
- Trend uses Welford running mean/stddev across all metrics + EWMA comparison
- ISO 8601 parser added for rebuild path (simple parser for "YYYY-MM-DDTHH:MM:SSZ" format)
- Bridge: `ProgressTimelineObserver` implements both `PitchComparisonObserver` and `PitchMatchingObserver` traits
- Hydration: refactored app.rs to store fetched records and reuse them for ProgressTimeline rebuild (avoids double IndexedDB fetch)
- Old types (ThresholdTimeline, TrendAnalyzer) retained — still used by profile_view, settings_view

### Completion Notes

All 10 tasks completed. 20 new unit tests in `progress_timeline.rs`. Full test suite passes (339 tests). Zero clippy warnings on both domain and web crates. Web crate compiles cleanly with WASM target.

## File List

- domain/src/progress_timeline.rs (new) — ProgressTimeline, TimeBucket, BucketSize, ModeState, adaptive bucketing, EWMA, trend, ISO 8601 parser, 20 unit tests
- domain/src/lib.rs (modified) — added `pub mod progress_timeline` and re-exports
- web/src/bridge.rs (modified) — added ProgressTimelineObserver (implements both observer traits)
- web/src/app.rs (modified) — added ProgressTimeline context, hydration via rebuild()
- web/src/components/pitch_comparison_view.rs (modified) — added ProgressTimelineObserver to observers
- web/src/components/pitch_matching_view.rs (modified) — added ProgressTimelineObserver to observers
- docs/implementation-artifacts/sprint-status.yaml (modified) — status updated
- docs/implementation-artifacts/7-2-progress-timeline-ewma-adaptive-bucketing.md (modified) — story file updated

## Change Log

- 2026-03-06: Implemented ProgressTimeline with EWMA smoothing and adaptive time bucketing (story 7.2). Added ProgressTimelineObserver bridge for both comparison and pitch matching modes. Integrated hydration at app startup. Old ThresholdTimeline/TrendAnalyzer retained for existing views.
- 2026-03-06: Code review fixes: (H1) fixed off-by-one in incremental session bucket stddev m2 recovery, (M1) wired per-mode TrainingModeConfig for halflife/session_gap instead of hardcoded constants, (M1/L2) added TrainingMode to ModeState, (L1) removed unused all_metrics Vec, (M3) added week and month bucket tests, added incremental stddev consistency test. Mark done.
