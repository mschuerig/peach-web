# Story 1.3: Perceptual Profile & Adaptive Algorithm

Status: ready-for-dev

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

- [ ] Task 1: Implement training domain entities — Blueprint §4 (AC: 15)
  - [ ] Create `domain/src/training/mod.rs` with re-exports
  - [ ] Create `domain/src/training/comparison.rs` — `Comparison`, `CompletedComparison`
  - [ ] Create `domain/src/training/pitch_matching.rs` — `PitchMatchingChallenge`, `CompletedPitchMatching`
  - [ ] Update `domain/src/lib.rs` to declare `training` module and re-export types
  - [ ] Inline unit tests for all training entity operations
- [ ] Task 2: Implement PerceptualProfile — Blueprint §5 (AC: 1,2,3,4,5,6)
  - [ ] Create `domain/src/profile.rs` — `PerceptualNote`, `PerceptualProfile`
  - [ ] Implement Welford's online algorithm for `update()`
  - [ ] Implement `weak_spots(count)` with untrained-first ranking
  - [ ] Implement `overall_mean`, `overall_std_dev`, `average_threshold(range)`
  - [ ] Implement pitch matching accumulators: `update_matching`, `matching_mean`, `matching_std_dev`
  - [ ] Implement `reset()` and `reset_matching()`
  - [ ] Update `domain/src/lib.rs` to declare and re-export
  - [ ] Comprehensive inline unit tests (Welford's correctness, weak spots, summary stats, reset, matching)
- [ ] Task 3: Implement KazezNoteStrategy — Blueprint §6 (AC: 7,8,9,10,11)
  - [ ] Create `domain/src/strategy.rs` — `kazez_narrow`, `kazez_widen`, `TrainingSettings`, `next_comparison`
  - [ ] Implement cold start, warm start, and adaptive difficulty logic
  - [ ] Implement note range clamping and interval-aware note selection
  - [ ] Update `domain/src/lib.rs` to declare and re-export
  - [ ] Inline unit tests (formulas, cold start, warm start, note range, interval modes)
- [ ] Task 4: Implement TrendAnalyzer — Blueprint §8.1 (AC: 12,13)
  - [ ] Create `domain/src/trend.rs` — `TrendAnalyzer`, `Trend` enum
  - [ ] Implement half-split comparison algorithm with 5% threshold
  - [ ] Update `domain/src/lib.rs` to declare and re-export
  - [ ] Inline unit tests (insufficient data, improving, stable, declining)
- [ ] Task 5: Implement ThresholdTimeline — Blueprint §8.2 (AC: 14)
  - [ ] Create `domain/src/timeline.rs` — `ThresholdTimeline`, `TimelineDataPoint`, `PeriodAggregate`
  - [ ] Implement data point recording and daily aggregation
  - [ ] Update `domain/src/lib.rs` to declare and re-export
  - [ ] Inline unit tests (data recording, aggregation, rolling statistics)
- [ ] Task 6: Add integration tests (AC: 16)
  - [ ] Create `domain/tests/strategy_convergence.rs` — Kazez convergence behavior over sequences
  - [ ] Create `domain/tests/profile_hydration.rs` — Replay records → verify profile state
- [ ] Task 7: Verify full suite (AC: 16)
  - [ ] Run `cargo test -p domain` — all pass
  - [ ] Run `cargo clippy -p domain` — no warnings

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

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
