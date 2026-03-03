# Story 1.3: Perceptual Profile & Adaptive Algorithm

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the perceptual profile, adaptive algorithm, trend analyzer, and threshold timeline implemented with unit tests,
so that the core intelligence of the app is correct and ready for integration.

## Acceptance Criteria

1. **AC1 — PerceptualProfile single update:** `profile.update(MIDINote::new(60), 50.0, true)` results in note 60's mean = 50.0, sample_count = 1, std_dev = 0.0
2. **AC2 — Welford's correctness:** Multiple updates for the same note produce mean and std_dev matching manual computation of Welford's online algorithm (sample standard deviation, dividing M2 by n-1)
3. **AC3 — Weak spots ranking:** `weak_spots(10)` returns untrained notes first (infinite score), then trained notes sorted by highest mean (higher threshold = weaker)
4. **AC4 — Summary statistics:** `overall_mean` is the average of per-note means across trained notes; `overall_std_dev` is sample std dev of per-note means; both return `None` when no notes are trained
5. **AC5 — Pitch matching accumulators:** `update_matching` tracks running mean of absolute errors via Welford's; `matching_std_dev` correct for 2+ samples; returns `None` when count is 0
6. **AC6 — Profile reset:** `profile.reset()` returns all 128 notes to defaults; `reset_matching()` zeroes matching accumulators
7. **AC7 — Kazez narrow (correct answer):** At 50 cents, `kazez_narrow(50.0)` = 50 * (1.0 - 0.05 * sqrt(50)) ≈ 32.3 cents
8. **AC8 — Kazez widen (incorrect answer):** At 50 cents, `kazez_widen(50.0)` = 50 * (1.0 + 0.09 * sqrt(50)) ≈ 81.8 cents
9. **AC9 — Cold start strategy:** No previous comparison and no profile data → magnitude defaults to `max_cent_difference` (100 cents)
10. **AC10 — Warm start strategy:** No previous comparison but profile has `overall_mean` → magnitude starts at `overall_mean`, clamped to difficulty range
11. **AC11 — Reference note in range:** `next_comparison` selects reference note within configured note range, and target note stays within MIDI 0-127 after interval transposition
12. **AC12 — TrendAnalyzer insufficient data:** Fewer than 20 data points → trend returns `None`
13. **AC13 — TrendAnalyzer improving:** 20+ points where later half mean is >5% lower than earlier half → returns `Improving`
14. **AC14 — ThresholdTimeline data recording:** Data points recorded with timestamp, cent_offset, is_correct, reference_note; daily aggregation produces correct mean_threshold and counts
15. **AC15 — Training entity types:** `Comparison`, `CompletedComparison`, `PitchMatchingChallenge`, `CompletedPitchMatching` implemented per blueprint §4
16. **AC16 — Full test suite:** `cargo test -p domain` passes with zero browser dependencies

## Tasks / Subtasks

- [x] Task 1: Implement training domain entities — Blueprint §4 (AC: 15)
  - [x] Create `domain/src/training/mod.rs` with re-exports
  - [x] Create `domain/src/training/comparison.rs` — `Comparison`, `CompletedComparison`
  - [x] Create `domain/src/training/pitch_matching.rs` — `PitchMatchingChallenge`, `CompletedPitchMatching`
  - [x] Update `domain/src/lib.rs` to declare `training` module and re-export types
  - [x] Inline unit tests for all training entity operations
- [x] Task 2: Implement PerceptualProfile — Blueprint §5 (AC: 1,2,3,4,5,6)
  - [x] Create `domain/src/profile.rs` — `PerceptualNote`, `PerceptualProfile`
  - [x] Implement Welford's online algorithm for `update()`
  - [x] Implement `weak_spots(count)` with untrained-first ranking
  - [x] Implement `overall_mean`, `overall_std_dev`, `average_threshold(range)`
  - [x] Implement pitch matching accumulators: `update_matching`, `matching_mean`, `matching_std_dev`
  - [x] Implement `reset()` and `reset_matching()`
  - [x] Update `domain/src/lib.rs` to declare and re-export
  - [x] Comprehensive inline unit tests (Welford's correctness, weak spots, summary stats, reset, matching)
- [x] Task 3: Implement KazezNoteStrategy — Blueprint §6 (AC: 7,8,9,10,11)
  - [x] Create `domain/src/strategy.rs` — `kazez_narrow`, `kazez_widen`, `TrainingSettings`, `next_comparison`
  - [x] Implement cold start, warm start, and adaptive difficulty logic
  - [x] Implement note range clamping and interval-aware note selection
  - [x] Update `domain/src/lib.rs` to declare and re-export
  - [x] Inline unit tests (formulas, cold start, warm start, note range, interval modes)
- [x] Task 4: Implement TrendAnalyzer — Blueprint §8.1 (AC: 12,13)
  - [x] Create `domain/src/trend.rs` — `TrendAnalyzer`, `Trend` enum
  - [x] Implement half-split comparison algorithm with 5% threshold
  - [x] Update `domain/src/lib.rs` to declare and re-export
  - [x] Inline unit tests (insufficient data, improving, stable, declining)
- [x] Task 5: Implement ThresholdTimeline — Blueprint §8.2 (AC: 14)
  - [x] Create `domain/src/timeline.rs` — `ThresholdTimeline`, `TimelineDataPoint`, `PeriodAggregate`
  - [x] Implement data point recording and daily aggregation
  - [x] Update `domain/src/lib.rs` to declare and re-export
  - [x] Inline unit tests (data recording, aggregation, rolling statistics)
- [x] Task 6: Add integration tests (AC: 16)
  - [x] Create `domain/tests/strategy_convergence.rs` — Kazez convergence behavior over sequences
  - [x] Create `domain/tests/profile_hydration.rs` — Replay records → verify profile state
- [x] Task 7: Verify full suite (AC: 16)
  - [x] Run `cargo test -p domain` — all pass (170 tests)
  - [x] Run `cargo clippy -p domain` — no warnings

## Dev Notes

### Architecture Compliance

- **Crate boundary:** ALL code in `domain/` crate only. Zero `web-sys`, `wasm-bindgen`, or `leptos` imports. `cargo test -p domain` runs natively.
- **Type name fidelity:** Use EXACT names from domain blueprint §4-8. `PerceptualProfile` not `Profile`. `CompletedComparison` not `ComparisonResult`. `KazezNoteStrategy` not `AdaptiveStrategy`.
- **Field naming:** Blueprint uses `camelCase` fields — in Rust use `snake_case` (e.g. `rawValue` → `raw_value`, `sampleCount` → `sample_count`, `centOffset` → `cent_offset`).

### Existing Code Patterns (from Story 1.2 — follow exactly)

- **Private fields with public getters:** All struct fields are private. Add `pub fn field_name(&self) -> Type` accessor methods. Never expose raw fields publicly.
- **Constructor pattern:** `pub fn new(...)` with `assert!()` for programming-error invariants, `.clamp()` for bounded types.
- **NaN guard:** Floating-point clamping types MUST `assert!(!value.is_nan())` before clamping (learned from 1.2 code review fix M2).
- **Derives:** Use `Clone, Copy, Debug, PartialEq, Serialize, Deserialize` for float-containing types (no `Eq`/`Ord`). Add `Eq, Hash, PartialOrd, Ord` for integer/enum types.
- **Serde:** `#[serde(rename_all = "camelCase")]` on enums. Struct fields serialize as `snake_case` (serde default).
- **Error handling:** `Result<T, DomainError>` for fallible operations. Add new variants to existing `DomainError` enum in `domain/src/error.rs`.

### Numeric Precision Rules

- All cent values, frequency values, means, standard deviations, M2 accumulators: **`f64`**
- MIDI note values: **`u8`** (via `MIDINote`)
- Sample counts: **`u32`** (sufficient for 10k+ records)
- No `f32` anywhere in this story

### Type Implementation Reference

#### Comparison (domain/src/training/comparison.rs) — Blueprint §4.1

```rust
pub struct Comparison {
    reference_note: MIDINote,
    target_note: DetunedMIDINote,
}
```
- `is_target_higher() -> bool`: `target_note.offset.raw_value() > 0.0` (NOTE: Cents uses `raw_value` pub field, not getter — check actual impl)
- `is_correct(user_answered_higher: bool) -> bool`: `user_answered_higher == self.is_target_higher()`
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`
- **Private fields, public getters** for `reference_note()` and `target_note()`

#### CompletedComparison (domain/src/training/comparison.rs) — Blueprint §4.2

```rust
pub struct CompletedComparison {
    comparison: Comparison,
    user_answered_higher: bool,
    tuning_system: TuningSystem,
    timestamp: String,  // ISO 8601
}
```
- `is_correct() -> bool`: delegates to `comparison.is_correct(user_answered_higher)`
- Derive: `Clone, Debug, PartialEq, Serialize, Deserialize` (no Copy — contains String timestamp)
- Private fields, public getters

#### PitchMatchingChallenge (domain/src/training/pitch_matching.rs) — Blueprint §4.3

```rust
pub struct PitchMatchingChallenge {
    reference_note: MIDINote,
    target_note: MIDINote,
    initial_cent_offset: f64,  // random in -20.0..=+20.0
}
```
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`
- Private fields, public getters

#### CompletedPitchMatching (domain/src/training/pitch_matching.rs) — Blueprint §4.4

```rust
pub struct CompletedPitchMatching {
    reference_note: MIDINote,
    target_note: MIDINote,
    initial_cent_offset: f64,
    user_cent_error: f64,  // positive = sharp, negative = flat
    tuning_system: TuningSystem,
    timestamp: String,  // ISO 8601
}
```
- Derive: `Clone, Debug, PartialEq, Serialize, Deserialize`
- Private fields, public getters

#### PerceptualNote (domain/src/profile.rs) — Blueprint §5.1

```rust
pub struct PerceptualNote {
    mean: f64,              // Welford's running mean, default 0.0
    std_dev: f64,           // Standard deviation, default 0.0
    m2: f64,                // Welford's M2 accumulator, default 0.0
    sample_count: u32,      // Number of comparisons, default 0
    current_difficulty: f64, // Active difficulty (cents), default 100.0
}
```
- `is_trained() -> bool`: `sample_count > 0`
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`
- All fields private with public getters

#### PerceptualProfile (domain/src/profile.rs) — Blueprint §5.2-5.4

```rust
pub struct PerceptualProfile {
    notes: [PerceptualNote; 128],
    matching_count: u32,
    matching_mean_abs: f64,
    matching_m2: f64,
}
```

**Critical methods:**
- `update(&mut self, note: MIDINote, cent_offset: f64, is_correct: bool)` — Welford's online algorithm. `cent_offset` is always the **absolute** magnitude of the cent difference presented (NOT signed).
- `weak_spots(&self, count: usize) -> Vec<MIDINote>` — untrained = infinity score, trained = mean score, sort descending, return top `count`
- `overall_mean(&self) -> Option<f64>` — average of per-note means across trained notes, `None` if no trained notes
- `overall_std_dev(&self) -> Option<f64>` — sample std dev of per-note means, `None` if fewer than 2 trained notes
- `average_threshold(&self, range: RangeInclusive<u8>) -> Option<f64>` — average mean across trained notes in MIDI range
- `update_matching(&mut self, note: MIDINote, cent_error: f64)` — Welford's on `abs(cent_error)` for aggregate pitch matching stats
- `matching_mean(&self) -> Option<f64>` — `matching_mean_abs` if count > 0, else `None`
- `matching_std_dev(&self) -> Option<f64>` — `sqrt(matching_m2 / (matching_count - 1))` if count >= 2, else `None`
- `reset(&mut self)` — all 128 notes to defaults
- `reset_matching(&mut self)` — zero matching accumulators
- `note_stats(&self, note: MIDINote) -> &PerceptualNote` — read-only access to per-note stats

**Welford's Algorithm (exact implementation):**
```
update(note, cent_offset, is_correct):
    stats = &mut self.notes[note.raw_value() as usize]
    stats.sample_count += 1
    delta = cent_offset - stats.mean
    stats.mean += delta / stats.sample_count as f64
    delta2 = cent_offset - stats.mean
    stats.m2 += delta * delta2
    variance = if sample_count < 2 { 0.0 } else { stats.m2 / (stats.sample_count - 1) as f64 }
    stats.std_dev = variance.sqrt()
```

**NOTE:** The `is_correct` parameter is currently unused by the profile update but is part of the interface for future use. Accept it but don't use it in the computation — the profile only tracks thresholds, not correctness.

#### KazezNoteStrategy (domain/src/strategy.rs) — Blueprint §6

**Public functions (stateless):**
```rust
pub fn kazez_narrow(p: f64) -> f64 {
    p * (1.0 - 0.05 * p.sqrt())
}

pub fn kazez_widen(p: f64) -> f64 {
    p * (1.0 + 0.09 * p.sqrt())
}
```

**TrainingSettings struct:**
```rust
pub struct TrainingSettings {
    note_range_min: MIDINote,      // default: 36 (C2)
    note_range_max: MIDINote,      // default: 84 (C6)
    reference_pitch: Frequency,     // default: 440.0
    min_cent_difference: Cents,     // default: 0.1
    max_cent_difference: Cents,     // default: 100.0
}
```
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`
- Private fields, public getters, `Default` impl with the defaults above
- Constructor: `pub fn new(...)` — validate `note_range_min <= note_range_max`

**next_comparison function:**
```rust
pub fn next_comparison(
    profile: &PerceptualProfile,
    settings: &TrainingSettings,
    last_comparison: Option<&CompletedComparison>,
    interval: DirectedInterval,
) -> Comparison
```

Algorithm (from Blueprint §6.3):
1. Determine magnitude from `last_comparison` (kazez_narrow/widen), or `profile.overall_mean()`, or cold-start (100 cents)
2. Clamp to `[min_cent_difference, max_cent_difference]`
3. Random sign (equally likely positive/negative)
4. Select reference note within range, accounting for interval transposition
5. Return `Comparison { reference_note, target_note: DetunedMIDINote(transposed, Cents(signed)) }`

#### TrendAnalyzer (domain/src/trend.rs) — Blueprint §8.1

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Trend {
    Improving,
    Stable,
    Declining,
}

pub struct TrendAnalyzer {
    abs_offsets: Vec<f64>,
}
```

- `MINIMUM_RECORD_COUNT = 20`
- `CHANGE_THRESHOLD = 0.05`
- `push(&mut self, abs_offset: f64)` — append to `abs_offsets`
- `trend(&self) -> Option<Trend>` — half-split mean comparison
- `reset(&mut self)` — clear all data
- Derive on struct: `Clone, Debug, PartialEq, Serialize, Deserialize`

#### ThresholdTimeline (domain/src/timeline.rs) — Blueprint §8.2

```rust
pub struct TimelineDataPoint {
    timestamp: String,       // ISO 8601
    cent_offset: f64,        // absolute
    is_correct: bool,
    reference_note: u8,      // raw MIDI value
}

pub struct PeriodAggregate {
    period_start: String,    // ISO 8601 date
    mean_threshold: f64,
    comparison_count: u32,
    correct_count: u32,
}

pub struct ThresholdTimeline {
    data_points: Vec<TimelineDataPoint>,
}
```

- `push(&mut self, timestamp: &str, cent_offset: f64, is_correct: bool, reference_note: u8)` — record data point
- `aggregate_daily(&self) -> Vec<PeriodAggregate>` — group by date, compute means and counts
- `reset(&mut self)` — clear all data
- Derive on structs: `Clone, Debug, PartialEq, Serialize, Deserialize`

### Dependency Notes

**No new crate dependencies needed.** Story 1.2 already added everything required:
- `serde` (with derive) — serialization
- `thiserror` — error types
- `rand` — random note selection, random sign

**Expand `DomainError` enum** in `domain/src/error.rs` if needed (e.g., `InvalidNoteRange` for strategy settings validation).

### Project Structure Notes

New files to create:
```
domain/src/
├── training/
│   ├── mod.rs                  # Re-exports Comparison, CompletedComparison, etc.
│   ├── comparison.rs           # Comparison, CompletedComparison
│   └── pitch_matching.rs       # PitchMatchingChallenge, CompletedPitchMatching
├── profile.rs                  # PerceptualNote, PerceptualProfile
├── strategy.rs                 # kazez_narrow, kazez_widen, TrainingSettings, next_comparison
├── trend.rs                    # TrendAnalyzer, Trend
├── timeline.rs                 # ThresholdTimeline, TimelineDataPoint, PeriodAggregate
domain/tests/
├── strategy_convergence.rs     # Integration: Kazez convergence over sequences
├── profile_hydration.rs        # Integration: Replay records → verify profile state
```

Files to modify:
- `domain/src/lib.rs` — add `pub mod training;`, `pub mod profile;`, `pub mod strategy;`, `pub mod trend;`, `pub mod timeline;` and re-exports
- `domain/src/error.rs` — add variants if needed

### Testing Standards

- **Inline unit tests:** `#[cfg(test)] mod tests { ... }` at the bottom of each new source file
- **Integration tests:** `domain/tests/strategy_convergence.rs` and `domain/tests/profile_hydration.rs`
- **Test naming:** `test_` prefix + descriptive snake_case
- **Run natively:** `cargo test -p domain` — fast, no browser, no WASM
- **Welford's verification:** Manually compute expected values for 3-5 data points and assert equality within f64 epsilon
- **Strategy tests:** Use fixed seeds or deterministic inputs (e.g., mock profile with known `overall_mean`)

### Anti-Patterns (DO NOT)

- Do NOT add browser dependencies to domain crate
- Do NOT rename domain types from the blueprint (`PerceptualProfile`, NOT `Profile` or `UserProfile`)
- Do NOT use `f32` for any computation in this story
- Do NOT implement observer traits here — observers are Story 1.6+ (session integration). This story implements the pure domain logic only.
- Do NOT implement session state machines — those are Story 1.6
- Do NOT implement port traits (NotePlayer, TrainingDataStore, etc.) — those are later stories
- Do NOT add `ComparisonRecord` or `PitchMatchingRecord` persistence schemas — those are Story 1.8
- Do NOT add gamification, scores, streaks, or session summaries
- Do NOT use `anyhow` in domain crate — use `thiserror` only
- Do NOT swallow errors with `let _ =`
- Do NOT make `PerceptualNote.m2` or internal Welford state public — expose only `mean`, `std_dev`, `sample_count`, `current_difficulty`, and `is_trained`

### Timestamp Handling

- Use `String` for timestamps (ISO 8601 format: `"2026-03-03T14:30:00Z"`)
- Domain crate does NOT import chrono or any time library — timestamps are opaque strings created by callers
- `ThresholdTimeline::aggregate_daily` groups by the date portion of the ISO 8601 string (first 10 characters: `"2026-03-03"`)

### Previous Story Intelligence (1.2 Domain Value Types)

**Key learnings from Story 1.2:**
- Private fields + public getters pattern was enforced after code review (H1 fix). Follow this pattern from the start.
- NaN guards on clamping constructors were added after code review (M2 fix). Any bounded float inputs should check for NaN.
- Serde round-trip tests were missing initially (M3 fix). Include serde tests from the start for all serializable types.
- `TuningSystem::frequency()` initially ignored `self` and produced identical 12-TET results for both variants (M1 fix). Double-check that all enum dispatch actually uses the variant.
- Clippy caught `if_same_then_else` in `DirectedInterval::between()`. Run clippy early and often.

**Patterns established (reuse exactly):**
- Module structure: `types/mod.rs` re-exports, `lib.rs` declares modules with `pub mod` and re-exports with `pub use`
- Constructor: `Self { field }` with validation
- Error pattern: `DomainError` enum with `#[error("...")]` messages
- Test pattern: inline `#[cfg(test)] mod tests { use super::*; ... }`
- Serde dev-dependency: `serde_json = "1"` already in domain/Cargo.toml

**Existing type APIs the new code will use:**
- `MIDINote::new(u8)`, `MIDINote::raw_value()`, `MIDINote::random(RangeInclusive<u8>)`, `MIDINote::transposed(DirectedInterval)`
- `Cents::new(f64)`, `Cents.raw_value` (pub field on Cents), `Cents::magnitude()`
- `DetunedMIDINote { note, offset }` (pub fields), `From<MIDINote> for DetunedMIDINote`
- `DirectedInterval::new(Interval, Direction)`, `DirectedInterval.interval`, `DirectedInterval.direction`
- `Interval::semitones()`, `Interval::between(MIDINote, MIDINote) -> Result<Interval, DomainError>`
- `Frequency::new(f64)`, `Frequency::CONCERT_440`
- `TuningSystem::frequency(DetunedMIDINote, Frequency) -> Frequency`

**Field access patterns (verified from 1.2 code):**
- `Cents`: `pub raw_value: f64` — public field, access as `cents.raw_value`
- `DetunedMIDINote`: `pub note: MIDINote`, `pub offset: Cents` — public fields
- `DirectedInterval`: `pub interval: Interval`, `pub direction: Direction` — public fields
- All other types: private fields with `pub fn raw_value()` getters (MIDINote, Frequency, etc.)

### Git Intelligence

Recent commits show:
- `c5fa82a` — Update docs/claude-audit
- `6246c33` — Apply code review fixes for story 1.2 (private fields, NaN guards, serde tests, JI frequency fix)
- `12894a9` — Implement story 1.2 domain value types and tuning system
- All on `main` branch, linear history

### References

- [Source: docs/ios-reference/domain-blueprint.md#4-training-domain-entities] — Comparison, CompletedComparison, PitchMatchingChallenge, CompletedPitchMatching
- [Source: docs/ios-reference/domain-blueprint.md#5-perceptual-profile-user-model] — PerceptualNote, Welford's algorithm, weak spots, summary stats, pitch matching accumulators, reset
- [Source: docs/ios-reference/domain-blueprint.md#6-adaptive-algorithm-kazez-note-strategy] — nextComparison, kazez_narrow/widen, TrainingSettings
- [Source: docs/ios-reference/domain-blueprint.md#8-trend-analysis] — TrendAnalyzer, ThresholdTimeline
- [Source: docs/planning-artifacts/architecture.md] — Two-crate structure, file locations (profile.rs, strategy.rs, trend.rs, timeline.rs, training/)
- [Source: docs/project-context.md] — Coding rules, error handling, serialization, testing standards
- [Source: docs/planning-artifacts/epics.md#story-1-3] — BDD acceptance criteria
- [Source: docs/implementation-artifacts/1-2-domain-value-types-and-tuning-system.md] — Previous story learnings, patterns, code review fixes

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Serde does not natively support `[T; 128]` arrays — resolved with custom `note_array_serde` module that serializes/deserializes as `Vec<PerceptualNote>` without adding external dependencies.
- Clippy `collapsible_if` warning in `timeline.rs` — fixed by merging nested `if let` + `if` into single `if let && condition` expression.

### Completion Notes List

- **Task 1:** Implemented `Comparison`, `CompletedComparison`, `PitchMatchingChallenge`, `CompletedPitchMatching` in `training/` module. All types use private fields with public getters, correct derives, serde roundtrip tests. 17 unit tests.
- **Task 2:** Implemented `PerceptualNote` and `PerceptualProfile` with Welford's online algorithm for both comparison updates and pitch matching accumulators. Includes `weak_spots`, `overall_mean`, `overall_std_dev`, `average_threshold`, `reset`, `reset_matching`, `note_stats`. 22 unit tests.
- **Task 3:** Implemented `kazez_narrow`, `kazez_widen`, `TrainingSettings`, and `next_comparison` with cold start, warm start, adaptive difficulty, note range clamping, and interval-aware note selection. 18 unit tests.
- **Task 4:** Implemented `TrendAnalyzer` with half-split comparison algorithm (20-point minimum, 5% threshold) and `Trend` enum. 10 unit tests.
- **Task 5:** Implemented `ThresholdTimeline` with `TimelineDataPoint`, `PeriodAggregate`, daily aggregation by ISO 8601 date prefix. 9 unit tests.
- **Task 6:** Created `strategy_convergence.rs` (4 integration tests: convergence, divergence, oscillation, interval transposition) and `profile_hydration.rs` (7 integration tests: replay, order independence, multi-note, weak spots, matching, reset, large dataset).
- **Task 7:** Full suite passes: 170 tests (155 unit + 15 integration), 0 clippy warnings.

### File List

**New files:**
- domain/src/training/mod.rs
- domain/src/training/comparison.rs
- domain/src/training/pitch_matching.rs
- domain/src/profile.rs
- domain/src/strategy.rs
- domain/src/trend.rs
- domain/src/timeline.rs
- domain/tests/strategy_convergence.rs
- domain/tests/profile_hydration.rs

**Modified files:**
- domain/src/lib.rs
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/1-3-perceptual-profile-and-adaptive-algorithm.md

## Senior Developer Review (AI)

**Reviewer:** Code Review Workflow | **Date:** 2026-03-03 | **Outcome:** Approved (after fixes)

**Git vs Story Discrepancies:** 0

**Issues Found:** 3 High, 3 Medium, 2 Low — 6 fixed automatically, 2 Low deferred.

### Fixes Applied

- **H1** `PerceptualProfile::update()` and `update_matching()` — Added NaN guards (`assert!(!cent_offset.is_nan())`) per Story 1.2 established pattern. Added 2 panic tests.
- **H2** `ThresholdTimeline::push()` — Added assertion that timestamp is at least 10 characters (ISO 8601). Added panic test.
- **H3** `kazez_narrow` and `kazez_widen` — Added `assert!(p >= 0.0)` guards (covers NaN since NaN >= 0.0 is false). Added 4 panic tests.
- **M1** `ThresholdTimeline::aggregate_daily()` — Replaced consecutive-group Vec with BTreeMap for correct handling of non-chronological data. Added non-chronological grouping test.
- **M2** `PerceptualNote::default()` — Converted from inherent method to `impl Default for PerceptualNote` for consistency with codebase pattern.
- **M3** `TrendAnalyzer::push()` — Added `assert!(abs_offset >= 0.0)` guard. Added 2 panic tests.

### Low Issues (Deferred)

- **L1** No public constructor for `TimelineDataPoint` and `PeriodAggregate` — acceptable since they are produced internally; can be added when integration code needs them.
- **L2** Test `test_perceptual_note_serde_roundtrip` uses unreachable internal state — cosmetic, does not affect correctness.

### Post-Fix Test Results

- **180 tests passing** (165 unit + 15 integration), 0 clippy warnings
- All 16 ACs verified as implemented

## Change Log

- 2026-03-03: Implemented story 1.3 — Perceptual Profile & Adaptive Algorithm. Added training entities (Comparison, CompletedComparison, PitchMatchingChallenge, CompletedPitchMatching), PerceptualProfile with Welford's online algorithm, KazezNoteStrategy with adaptive difficulty, TrendAnalyzer, ThresholdTimeline. 170 tests passing, 0 clippy warnings.
- 2026-03-03: Code review fixes — Added NaN guards (profile, strategy, trend), timestamp validation (timeline), BTreeMap grouping (timeline), Default trait for PerceptualNote. 10 new tests added. 180 tests passing, 0 clippy warnings.
