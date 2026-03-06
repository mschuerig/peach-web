# Story 7.1: TrainingMode Enum and TrainingModeConfig

Status: done

## Story

As a developer,
I want a `TrainingMode` enum with four variants and per-mode configuration,
so that profile visualization, progress tracking, and UI can distinguish between the four training modes independently.

## Context

The iOS sibling app (repo `mschuerig/peach`) introduced `TrainingMode` and `TrainingModeConfig` to track each training mode's progress independently. Previously, all comparison training was lumped together; now unison and interval modes are separate for both comparison and matching.

This is a foundational domain change that stories 7.2-7.6 depend on.

**iOS reference files:**
- `Peach/Core/Profile/ProgressTimeline.swift` — defines `TrainingMode`, `TrainingModeState`, `BucketSize`, `TimeBucket`
- `Peach/Core/Profile/TrainingModeConfig.swift` — defines per-mode configuration

## Acceptance Criteria

1. **AC1 — TrainingMode enum:** `TrainingMode` enum exists in the domain crate with four variants: `UnisonPitchComparison`, `IntervalPitchComparison`, `UnisonMatching`, `IntervalMatching`
2. **AC2 — TrainingModeConfig struct:** `TrainingModeConfig` struct exists with fields: `display_name: &'static str`, `unit_label: &'static str`, `optimal_baseline: f64` (cents), `ewma_halflife_secs: f64`, `session_gap_secs: f64`
3. **AC3 — Config accessor:** `TrainingMode::config()` returns the correct `TrainingModeConfig` for each variant
4. **AC4 — Config values:** Unison comparison optimal baseline is 8.0 cents, interval comparison 12.0, unison matching 5.0, interval matching 8.0. EWMA halflife is 7 days (604800 secs) for all modes. Session gap is 30 minutes (1800 secs) for all modes.
5. **AC5 — Metric extraction from ComparisonRecord:** `TrainingMode::extract_comparison_metric(&ComparisonRecord) -> Option<f64>` returns `Some(abs(cent_offset))` when the record's interval matches the mode (interval==0 for unison, interval!=0 for interval), `None` otherwise
6. **AC6 — Metric extraction from PitchMatchingRecord:** `TrainingMode::extract_matching_metric(&PitchMatchingRecord) -> Option<f64>` returns `Some(abs(user_cent_error))` when the record's interval matches the mode, `None` otherwise
7. **AC7 — TrainingModeState enum:** `TrainingModeState` enum with `NoData` and `Active` variants
8. **AC8 — All cases iteration:** `TrainingMode::ALL` constant provides an array of all four variants
9. **AC9 — Display names:** Unison comparison: "Hear & Compare -- Single Notes", interval comparison: "Hear & Compare -- Intervals", unison matching: "Tune & Match -- Single Notes", interval matching: "Tune & Match -- Intervals"
10. **AC10 — Tests pass:** `cargo test -p domain` passes, `cargo clippy -p domain` has no warnings

## Tasks / Subtasks

- [x] Task 1: Create `domain/src/training_mode.rs` (AC: 1, 2, 3, 4, 8, 9)
  - [x] Define `TrainingMode` enum with four variants, derive `Clone, Copy, Debug, PartialEq, Eq, Hash`
  - [x] Define `TrainingModeConfig` struct with display_name, unit_label, optimal_baseline, ewma_halflife_secs, session_gap_secs
  - [x] Implement `TrainingMode::config(&self) -> &'static TrainingModeConfig` returning static configs
  - [x] Implement `TrainingMode::ALL: [TrainingMode; 4]` constant
  - [x] Define `TrainingModeState` enum with `NoData` and `Active` variants (AC: 7)
- [x] Task 2: Implement metric extraction methods (AC: 5, 6)
  - [x] `TrainingMode::extract_comparison_metric(&self, record: &ComparisonRecord) -> Option<f64>` — unison variants match interval==0, interval variants match interval!=0, returns abs(cent_offset)
  - [x] `TrainingMode::extract_matching_metric(&self, record: &PitchMatchingRecord) -> Option<f64>` — same interval logic, returns abs(user_cent_error)
- [x] Task 3: Wire into module system (AC: 10)
  - [x] Add `pub mod training_mode;` to `domain/src/lib.rs`
  - [x] Add re-exports: `TrainingMode`, `TrainingModeConfig`, `TrainingModeState`
- [x] Task 4: Write tests (AC: all)
  - [x] Test each variant's config returns correct values
  - [x] Test `TrainingMode::ALL` contains all four variants
  - [x] Test `extract_comparison_metric` returns Some for matching records, None for non-matching
  - [x] Test `extract_matching_metric` returns Some for matching records, None for non-matching
  - [x] Test boundary: interval==0 is unison, interval==1 is interval
  - [x] Test negative cent_offset returns positive magnitude
  - [x] Run `cargo test -p domain` and `cargo clippy -p domain`

## Dev Notes

### iOS Mapping

| iOS (Swift) | peach-web (Rust) |
|---|---|
| `TrainingMode` (enum, CaseIterable) | `TrainingMode` (enum + `ALL` const) |
| `TrainingModeConfig` (struct with Duration) | `TrainingModeConfig` (struct with f64 seconds) |
| `TrainingModeState` (enum) | `TrainingModeState` (enum) |
| `mode.config` (computed property) | `mode.config()` (method) |
| `mode.extractMetrics(...)` (returns array) | `mode.extract_comparison_metric(record)` / `mode.extract_matching_metric(record)` — per-record instead of batch |

### Design Decisions

- **Per-record extraction instead of batch:** The iOS app takes arrays and returns arrays. In Rust, we use per-record methods that return `Option<f64>`, which composes better with iterators (`records.iter().filter_map(|r| mode.extract_comparison_metric(r))`).
- **f64 seconds instead of Duration:** Rust's `std::time::Duration` doesn't implement the traits we need for const contexts, and we're in WASM where system time is browser-provided anyway. Plain f64 seconds is simpler.
- **Static config references:** Config values are compile-time constants, so `config()` returns `&'static TrainingModeConfig`.
- **No serde on TrainingMode:** This enum is not persisted — it's a runtime classification. Records already store the `interval` field, which is sufficient to classify.

### Architecture Compliance

- **Domain crate only:** No browser dependencies. This is pure Rust.
- **Record types unchanged:** `ComparisonRecord` and `PitchMatchingRecord` already have the `interval` field needed for mode classification. No changes to existing types.
- **Naming fidelity:** Display names match the iOS app's localized strings exactly (minus localization — peach-web is English-only).

## Dev Agent Record

### Implementation Plan

- Created `domain/src/training_mode.rs` with `TrainingMode` enum (4 variants), `TrainingModeConfig` struct, and `TrainingModeState` enum
- Static config constants with `&'static` references for zero-cost access
- Per-record metric extraction using `matches_interval()` helper for DRY interval classification
- 14 inline unit tests covering all ACs: config values, ALL constant, metric extraction, boundary conditions, negative-to-positive magnitude conversion

### Completion Notes

All 4 tasks completed in a single pass. 305 domain tests pass (14 new), zero clippy warnings. Implementation follows all architecture constraints: domain-crate-only, f64 for cent values, no serde on TrainingMode (runtime classification only).

## File List

- `domain/src/training_mode.rs` — NEW: TrainingMode enum, TrainingModeConfig, TrainingModeState, metric extraction methods, tests
- `domain/src/lib.rs` — MODIFIED: added `pub mod training_mode` and re-exports
- `docs/implementation-artifacts/sprint-status.yaml` — MODIFIED: story status updated
- `docs/implementation-artifacts/7-1-training-mode-enum-and-config.md` — MODIFIED: task checkboxes, status, dev agent record

## Change Log

- 2026-03-06: Implemented TrainingMode enum with four variants, per-mode config, metric extraction from comparison and matching records, and comprehensive test suite (14 tests). All ACs satisfied.
- 2026-03-06: Code review fixes: added category guards to extract_comparison_metric/extract_matching_metric (matching modes no longer process comparison records and vice versa), added Copy derive to TrainingModeConfig, replaced misleading test with 2 cross-mode guard tests (15 tests total). Marked done.
