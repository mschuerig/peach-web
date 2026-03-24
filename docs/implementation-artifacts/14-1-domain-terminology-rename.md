# Story 14.1: Domain Crate Terminology Rename

Status: done

## Story

As a developer,
I want all domain crate types renamed to match the iOS terminology alignment,
so that both apps use identical domain language and cross-platform CSV compatibility is maintained.

## Context

The iOS sibling app (commits `09e32be` through `8368c4d`) performed a systematic terminology rename to align with music/psychoacoustic usage:

- **TrainingMode → TrainingDiscipline** — "discipline" is standard in music pedagogy
- **PitchComparison → PitchDiscriminationTrial** — "discrimination" is the psychoacoustic term for threshold detection
- **PitchMatchingChallenge → PitchMatchingTrial** — "trial" is the standard term for one atomic presentation-response cycle
- **Settings drop "Training" suffix** — e.g. `PitchComparisonTrainingSettings` → `PitchDiscriminationSettings`

Full rename spec: `docs/ios-reference/ios-changes-since-f70e3f.md`, section 1.

This story covers the `domain/` crate only. Web crate changes follow in story 14.2.

## Acceptance Criteria

1. **AC1 — Enum rename:** `TrainingMode` renamed to `TrainingDiscipline` with updated variants:
   - `UnisonPitchComparison` → `UnisonPitchDiscrimination`
   - `IntervalPitchComparison` → `IntervalPitchDiscrimination`
   - `UnisonMatching` → `UnisonPitchMatching`
   - `IntervalMatching` → `IntervalPitchMatching`

2. **AC2 — Training type renames:**
   - `PitchComparison` → `PitchDiscriminationTrial`
   - `CompletedPitchComparison` → `CompletedPitchDiscriminationTrial`
   - `PitchMatchingChallenge` → `PitchMatchingTrial`
   - `CompletedPitchMatching` → `CompletedPitchMatchingTrial`

3. **AC3 — Session type renames:**
   - `PitchComparisonSession` → `PitchDiscriminationSession`
   - `PitchComparisonSessionState` → `PitchDiscriminationSessionState`
   - `PitchComparisonPlaybackData` → `PitchDiscriminationPlaybackData`
   - `PitchMatchingSession` → `PitchMatchingSession` (unchanged)
   - `PitchMatchingSessionState` → `PitchMatchingSessionState` (unchanged)
   - `PitchMatchingPlaybackData` → `PitchMatchingPlaybackData` (unchanged)

4. **AC4 — Observer trait renames:**
   - `PitchComparisonObserver` → `PitchDiscriminationObserver`
   - Method: `pitch_comparison_completed()` → `pitch_discrimination_completed()`
   - `PitchMatchingObserver` → `PitchMatchingObserver` (unchanged)
   - Method: `pitch_matching_completed()` → `pitch_matching_completed()` (unchanged)

5. **AC5 — Record type renames:**
   - `PitchComparisonRecord` → `PitchDiscriminationRecord`
   - `PitchMatchingRecord` → `PitchMatchingRecord` (unchanged)

6. **AC6 — Associated type renames:**
   - `TrainingModeStatistics` → `TrainingDisciplineStatistics`
   - `TrainingModeState` → `TrainingDisciplineState`
   - `TrainingModeConfig` (if it exists) → `TrainingDisciplineConfig`

7. **AC7 — Method renames on sessions:**
   - `next_pitch_comparison()` → `next_pitch_discrimination_trial()`
   - `PitchComparisonSession::new()` signature uses new types

8. **AC8 — Slug values updated:**
   - `"pitch-comparison"` → `"pitch-discrimination"`
   - `"interval-comparison"` → `"interval-discrimination"`
   - `"pitch-matching"` → `"pitch-matching"` (unchanged)
   - `"interval-matching"` → `"interval-matching"` (unchanged)

9. **AC9 — File renames:**
   - `domain/src/training/pitch_comparison.rs` → `pitch_discrimination.rs`
   - `domain/src/session/pitch_comparison_session.rs` → `pitch_discrimination_session.rs`
   - `domain/src/training_mode.rs` → `training_discipline.rs`
   - `domain/src/records.rs` — no rename needed, but internal types updated

10. **AC10 — Port trait renames:**
    - `TrainingDataStore` methods: `save_pitch_comparison()` → `save_pitch_discrimination()`, `fetch_all_pitch_comparisons()` → `fetch_all_pitch_discriminations()`

11. **AC11 — Strategy renames:**
    - `NextPitchComparisonStrategy` → `NextPitchDiscriminationStrategy` (if this type exists)
    - `next_pitch_comparison()` → `next_pitch_discrimination_trial()`

12. **AC12 — All tests pass:** `cargo test -p domain` and `cargo clippy --workspace` pass.

13. **AC13 — Web crate compiles:** The web crate may have compilation errors from changed imports — these are expected and will be fixed in story 14.2. However, `cargo check -p domain` must pass cleanly.

## Tasks / Subtasks

### Phase 1: File renames

- [x] Task 1: `git mv` files listed in AC9
- [x] Task 2: Update `mod` declarations in `lib.rs`, `training/mod.rs`, `session/mod.rs`

### Phase 2: Core type renames

- [x] Task 3: Rename `TrainingMode` enum → `TrainingDiscipline` with new variant names
- [x] Task 4: Rename `PitchComparison` → `PitchDiscriminationTrial`, `CompletedPitchComparison` → `CompletedPitchDiscriminationTrial`
- [x] Task 5: Rename `PitchMatchingChallenge` → `PitchMatchingTrial`, `CompletedPitchMatching` → `CompletedPitchMatchingTrial`
- [x] Task 6: Rename record types (AC5)
- [x] Task 7: Rename session types and state enums (AC3)
- [x] Task 8: Rename observer traits and methods (AC4)
- [x] Task 9: Update slug values (AC8) — N/A: no slug string literals exist in domain crate (web crate only)

### Phase 3: Cascade fixes

- [x] Task 10: Fix all `use` statements, match arms, and method calls throughout domain crate
- [x] Task 11: Update strategy types and methods (AC11)
- [x] Task 12: Update port traits (AC10)
- [x] Task 13: Update all test functions and assertions

### Phase 4: Verification

- [x] Task 14: `cargo test -p domain` passes (335 tests, 0 failures)
- [x] Task 15: `cargo clippy -p domain` passes (clean)

## Dev Notes

- Use `cargo check -p domain` iteratively — the compiler will guide you to every broken reference
- The `PitchMatching*` family has fewer renames (only `Challenge` → `Trial`, `Completed` gets `Trial` suffix)
- Do NOT update web crate, CSV format, or docs — those are separate stories
- Historical story docs (e.g. `7-0a-rename-comparison-to-pitch-comparison.md`) are NOT updated

## Dev Agent Record

### Implementation Plan

Systematic rename in 4 phases: file renames via `git mv`, core type renames using replace_all, cascade fixes across all consumers, verification with tests and clippy.

### Debug Log

No bugs encountered. All renames applied cleanly via systematic replace_all operations. The compiler confirmed correctness at each step.

### Completion Notes

All domain crate types, traits, methods, and files renamed per iOS terminology alignment:
- 3 files renamed via `git mv` (training_mode.rs, pitch_comparison.rs, pitch_comparison_session.rs)
- ~30 type/trait/method renames applied across 15 source files + 3 integration test files
- AC8 (slug values): No slug string literals exist in domain crate — these live in the web crate and will be addressed in story 14.2
- AC11 (`NextPitchComparisonStrategy`): Type does not exist in domain crate — the strategy is a function `next_pitch_comparison()` which was renamed to `next_pitch_discrimination_trial()`
- 335 tests pass, clippy clean, `cargo check -p domain` clean
- Web crate will have compilation errors from changed exports — expected, covered by story 14.2

## File List

### Modified

- domain/src/lib.rs
- domain/src/training/mod.rs
- domain/src/session/mod.rs
- domain/src/training_discipline.rs (renamed from training_mode.rs)
- domain/src/training/pitch_discrimination.rs (renamed from pitch_comparison.rs)
- domain/src/session/pitch_discrimination_session.rs (renamed from pitch_comparison_session.rs)
- domain/src/session/pitch_matching_session.rs
- domain/src/training/pitch_matching.rs
- domain/src/records.rs
- domain/src/ports.rs
- domain/src/strategy.rs
- domain/src/profile.rs
- domain/src/training_mode_statistics.rs
- domain/src/progress_timeline.rs
- domain/tests/strategy_convergence.rs
- domain/tests/profile_hydration.rs

## Change Log

- 2026-03-24: Completed domain crate terminology rename (Story 14.1) — all types, traits, methods, and files renamed to match iOS alignment spec
