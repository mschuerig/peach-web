# Story 7.1: TrainingMode Enum and TrainingModeConfig

Status: ready-for-dev

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

- [ ] Task 1: Create `domain/src/training_mode.rs` (AC: 1, 2, 3, 4, 8, 9)
  - [ ] Define `TrainingMode` enum with four variants, derive `Clone, Copy, Debug, PartialEq, Eq, Hash`
  - [ ] Define `TrainingModeConfig` struct with display_name, unit_label, optimal_baseline, ewma_halflife_secs, session_gap_secs
  - [ ] Implement `TrainingMode::config(&self) -> &'static TrainingModeConfig` returning static configs
  - [ ] Implement `TrainingMode::ALL: [TrainingMode; 4]` constant
  - [ ] Define `TrainingModeState` enum with `NoData` and `Active` variants (AC: 7)
- [ ] Task 2: Implement metric extraction methods (AC: 5, 6)
  - [ ] `TrainingMode::extract_comparison_metric(&self, record: &ComparisonRecord) -> Option<f64>` — unison variants match interval==0, interval variants match interval!=0, returns abs(cent_offset)
  - [ ] `TrainingMode::extract_matching_metric(&self, record: &PitchMatchingRecord) -> Option<f64>` — same interval logic, returns abs(user_cent_error)
- [ ] Task 3: Wire into module system (AC: 10)
  - [ ] Add `pub mod training_mode;` to `domain/src/lib.rs`
  - [ ] Add re-exports: `TrainingMode`, `TrainingModeConfig`, `TrainingModeState`
- [ ] Task 4: Write tests (AC: all)
  - [ ] Test each variant's config returns correct values
  - [ ] Test `TrainingMode::ALL` contains all four variants
  - [ ] Test `extract_comparison_metric` returns Some for matching records, None for non-matching
  - [ ] Test `extract_matching_metric` returns Some for matching records, None for non-matching
  - [ ] Test boundary: interval==0 is unison, interval==1 is interval
  - [ ] Test negative cent_offset returns positive magnitude
  - [ ] Run `cargo test -p domain` and `cargo clippy -p domain`

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
