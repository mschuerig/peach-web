# Story 7.0b: Extract Constants and Thread Domain Types

Status: ready-for-dev

## Story

As a developer,
I want magic numbers replaced with named constants and raw `f64` parameters replaced with domain types (`Cents`, `Frequency`) where appropriate,
so that the code is self-documenting and type-safe.

## Context

The iOS sibling app (commit `11772e7e10b2`) performed a code quality pass based on a whole-project review. The two main changes were:
1. Extract magic number literals into named constants
2. Replace raw `Double`/`TimeInterval` with domain types (`Cents`, `Frequency`) in function signatures

This story ports the equivalent improvements to peach-web.

Depends on: Story 7.0a (rename must complete first so we're working with the final type names).

## Acceptance Criteria

1. **AC1 — Cents constants:** `Cents::PER_OCTAVE` (1200.0) and `Cents::PER_SEMITONE_ET` (100.0) exist as associated constants on the `Cents` type
2. **AC2 — Training constants module:** A `TrainingConstants` struct or module in the domain crate provides named constants for algorithm parameters:
   - `COLD_START_DIFFICULTY: f64 = 100.0`
   - `KAZEZ_NARROW_FACTOR: f64 = 0.05`
   - `KAZEZ_WIDEN_FACTOR: f64 = 0.09`
   - `MIN_CENT_DIFFERENCE: f64 = 0.1`
   - `PITCH_SLIDER_CENTS_RANGE: f64 = 20.0`
   - `INITIAL_OFFSET_RANGE: f64 = 40.0`
   - `AMPLITUDE_VARY_SCALING: f64 = 5.0`
3. **AC3 — Magic numbers replaced:** All occurrences of `1200.0`, `100.0` (in cents contexts), `0.05`, `0.09`, `20.0` (slider), `40.0` (offset range), `5.0` (amplitude scaling) in domain source files are replaced with the named constants
4. **AC4 — Profile methods use Cents:** `PerceptualProfile::update()` takes `cent_offset: Cents` instead of `f64`. `update_matching()` takes `cent_error: Cents`. `average_threshold()` returns `Option<Cents>`.
5. **AC5 — Strategy functions use Cents:** `kazez_narrow()` and `kazez_widen()` take and return `Cents` instead of `f64`
6. **AC6 — Callers updated:** All call sites in session code, bridge observers, and hydration code updated to pass/receive domain types
7. **AC7 — Raw types at boundaries:** Raw `f64` is kept at persistence boundaries (record fields, serde) and inside Welford's algorithm arithmetic (where unwrapping/rewrapping adds noise)
8. **AC8 — Tests pass:** `cargo test -p domain` passes, `cargo clippy` has no warnings, `trunk build` succeeds

## Tasks / Subtasks

- [ ] Task 1: Add Cents associated constants (AC: 1)
  - [ ] In `domain/src/types/cents.rs`, add:
    ```rust
    impl Cents {
        pub const PER_OCTAVE: f64 = 1200.0;
        pub const PER_SEMITONE_ET: f64 = 100.0;
    }
    ```
  - [ ] Replace `1200.0` in `domain/src/tuning.rs` (lines ~43, ~67) with `Cents::PER_OCTAVE`
  - [ ] Replace `100.0` in `domain/src/tuning.rs` (line ~55) with `Cents::PER_SEMITONE_ET`
  - [ ] Replace `1200.0` in `domain/src/session/pitch_matching_session.rs` (lines ~268, ~328) with `Cents::PER_OCTAVE`
  - [ ] Replace `100.0` in `domain/src/training/comparison.rs` (line ~38, in `is_target_higher`) with `Cents::PER_SEMITONE_ET`

- [ ] Task 2: Create training constants (AC: 2, 3)
  - [ ] Add constants to appropriate locations. Options:
    - Add to existing types (e.g. `PerceptualNote::COLD_START_DIFFICULTY`)
    - Create `domain/src/training/constants.rs` for algorithm-specific constants
  - [ ] Replace in `domain/src/profile.rs`: `100.0` default difficulty → `PerceptualNote::COLD_START_DIFFICULTY` (or `TrainingConstants::COLD_START_DIFFICULTY`)
  - [ ] Replace in `domain/src/strategy.rs`:
    - `0.05` → `KAZEZ_NARROW_FACTOR`
    - `0.09` → `KAZEZ_WIDEN_FACTOR`
    - `0.1` min_cent_difference default → `MIN_CENT_DIFFERENCE`
    - `100.0` max_cent_difference default → `COLD_START_DIFFICULTY` (same value)
  - [ ] Replace in `domain/src/session/pitch_matching_session.rs`:
    - `40.0` offset range → `INITIAL_OFFSET_RANGE`
    - `20.0` slider scaling → `PITCH_SLIDER_CENTS_RANGE`
  - [ ] Replace in `domain/src/session/comparison_session.rs`:
    - `5.0` amplitude scaling → `AMPLITUDE_VARY_SCALING`
    - `0.1` and `100.0` duplicated defaults → use constants from strategy/profile

- [ ] Task 3: Thread Cents through profile API (AC: 4, 6)
  - [ ] Change `PerceptualProfile::update(&mut self, note: MIDINote, cent_offset: f64, is_correct: bool)` to `cent_offset: Cents`
  - [ ] Internally use `cent_offset.raw_value` for Welford arithmetic (AC7)
  - [ ] Change `update_matching(&mut self, _note: MIDINote, cent_error: f64)` to `cent_error: Cents`
  - [ ] Internally use `cent_error.magnitude()` for arithmetic
  - [ ] Change `average_threshold()` return from `Option<f64>` to `Option<Cents>`
  - [ ] Update callers in bridge.rs (`ProfileObserver::comparison_completed`)
  - [ ] Update callers in hydration code
  - [ ] Update all tests

- [ ] Task 4: Thread Cents through strategy API (AC: 5, 6)
  - [ ] Change `kazez_narrow(p: f64) -> f64` to `kazez_narrow(p: Cents) -> Cents`
  - [ ] Internally unwrap to `f64` for math, rewrap result
  - [ ] Change `kazez_widen(p: f64) -> f64` to `kazez_widen(p: Cents) -> Cents`
  - [ ] Update callers in session code
  - [ ] Update all tests

- [ ] Task 5: Update module system if needed
  - [ ] If `training/constants.rs` created, add `mod constants` to `training/mod.rs`
  - [ ] Add re-exports to `lib.rs` if constants are public

- [ ] Task 6: Verify (AC: 8)
  - [ ] `cargo test -p domain` — all pass
  - [ ] `cargo clippy -p domain` — no warnings
  - [ ] `cargo clippy -p web` — no warnings
  - [ ] `trunk build` — succeeds

## Dev Notes

### iOS Mapping

| iOS Constant | peach-web Equivalent |
|---|---|
| `Cents.perOctave` (static) | `Cents::PER_OCTAVE` (associated const) |
| `TrainingConstants.coldStartDifficulty` | `PerceptualNote::COLD_START_DIFFICULTY` or separate constant |
| `TrainingConstants.pitchBendRangeSemitones` | Not applicable (web uses different audio model) |
| `KazezNoteStrategy` constants | `KAZEZ_NARROW_FACTOR`, `KAZEZ_WIDEN_FACTOR` |

### Design Decisions

- **Constants as associated consts vs module:** Prefer associated constants on the relevant type when there's a clear owner (e.g. `Cents::PER_OCTAVE`, `PerceptualNote::COLD_START_DIFFICULTY`). Use a separate constants module for algorithm parameters that don't belong to a single type.
- **Cents wrapping in strategy:** `kazez_narrow`/`kazez_widen` take and return `Cents` to enforce type safety at the API boundary, but internally unwrap for the arithmetic. This matches the iOS approach.
- **Raw f64 preserved in Welford's algorithm:** The running mean, M2, and variance computations in `PerceptualProfile` keep raw `f64` internally. Wrapping every intermediate value in `Cents` would add noise without safety benefit.
- **No changes to record types:** `PitchComparisonRecord.cent_offset` and `PitchMatchingRecord.user_cent_error` stay as `f64` because they're serde-serialized storage fields at the persistence boundary.

### Scope Boundaries

- **In scope:** Domain crate constants, profile and strategy type signatures, callers in bridge and sessions
- **Out of scope:** Web crate UI code (no magic numbers there), CSS values, test data generation
- **Logging:** The iOS app added lifecycle logging to PitchMatchingSession. If desired, add `web_sys::console::log_1()` calls to session start/stop/challenge in `pitch_matching_view.rs` — but this is optional and low priority.
