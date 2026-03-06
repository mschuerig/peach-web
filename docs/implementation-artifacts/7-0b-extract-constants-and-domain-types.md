# Story 7.0b: Extract Constants and Thread Domain Types

Status: review

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

- [x] Task 1: Add Cents associated constants (AC: 1)
  - [x] In `domain/src/types/cents.rs`, add:
    ```rust
    impl Cents {
        pub const PER_OCTAVE: f64 = 1200.0;
        pub const PER_SEMITONE_ET: f64 = 100.0;
    }
    ```
  - [x] Replace `1200.0` in `domain/src/tuning.rs` (lines ~43, ~67) with `Cents::PER_OCTAVE`
  - [x] Replace `100.0` in `domain/src/tuning.rs` (line ~55) with `Cents::PER_SEMITONE_ET`
  - [x] Replace `1200.0` in `domain/src/session/pitch_matching_session.rs` (lines ~268, ~328) with `Cents::PER_OCTAVE`
  - [x] Replace `100.0` in `domain/src/training/comparison.rs` (line ~38, in `is_target_higher`) with `Cents::PER_SEMITONE_ET`

- [x] Task 2: Create training constants (AC: 2, 3)
  - [x] Add constants to appropriate locations. Options:
    - Add to existing types (e.g. `PerceptualNote::COLD_START_DIFFICULTY`)
    - Create `domain/src/training/constants.rs` for algorithm-specific constants
  - [x] Replace in `domain/src/profile.rs`: `100.0` default difficulty → `PerceptualNote::COLD_START_DIFFICULTY` (or `TrainingConstants::COLD_START_DIFFICULTY`)
  - [x] Replace in `domain/src/strategy.rs`:
    - `0.05` → `KAZEZ_NARROW_FACTOR`
    - `0.09` → `KAZEZ_WIDEN_FACTOR`
    - `0.1` min_cent_difference default → `MIN_CENT_DIFFERENCE`
    - `100.0` max_cent_difference default → `COLD_START_DIFFICULTY` (same value)
  - [x] Replace in `domain/src/session/pitch_matching_session.rs`:
    - `40.0` offset range → `INITIAL_OFFSET_RANGE`
    - `20.0` slider scaling → `PITCH_SLIDER_CENTS_RANGE`
  - [x] Replace in `domain/src/session/comparison_session.rs`:
    - `5.0` amplitude scaling → `AMPLITUDE_VARY_SCALING`
    - `0.1` and `100.0` duplicated defaults → use constants from strategy/profile

- [x] Task 3: Thread Cents through profile API (AC: 4, 6)
  - [x] Change `PerceptualProfile::update(&mut self, note: MIDINote, cent_offset: f64, is_correct: bool)` to `cent_offset: Cents`
  - [x] Internally use `cent_offset.raw_value` for Welford arithmetic (AC7)
  - [x] Change `update_matching(&mut self, _note: MIDINote, cent_error: f64)` to `cent_error: Cents`
  - [x] Internally use `cent_error.magnitude()` for arithmetic
  - [x] Change `average_threshold()` return from `Option<f64>` to `Option<Cents>`
  - [x] Update callers in bridge.rs (`ProfileObserver::comparison_completed`)
  - [x] Update callers in hydration code
  - [x] Update all tests

- [x] Task 4: Thread Cents through strategy API (AC: 5, 6)
  - [x] Change `kazez_narrow(p: f64) -> f64` to `kazez_narrow(p: Cents) -> Cents`
  - [x] Internally unwrap to `f64` for math, rewrap result
  - [x] Change `kazez_widen(p: f64) -> f64` to `kazez_widen(p: Cents) -> Cents`
  - [x] Update callers in session code
  - [x] Update all tests

- [x] Task 5: Update module system if needed
  - [x] If `training/constants.rs` created, add `mod constants` to `training/mod.rs`
  - [x] Add re-exports to `lib.rs` if constants are public

- [x] Task 6: Verify (AC: 8)
  - [x] `cargo test -p domain` — all pass
  - [x] `cargo clippy -p domain` — no warnings
  - [x] `cargo clippy -p web` — no warnings
  - [x] `trunk build` — succeeds

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

## Dev Agent Record

### Implementation Notes

**Constants placement strategy:** Instead of creating a separate `training/constants.rs` module, constants were placed as associated constants on their owning types following the Dev Notes guidance:

- `Cents::PER_OCTAVE`, `Cents::PER_SEMITONE_ET` — on the `Cents` type in `types/cents.rs`
- `PerceptualNote::COLD_START_DIFFICULTY` — on the `PerceptualNote` type in `profile.rs`
- `KAZEZ_NARROW_FACTOR`, `KAZEZ_WIDEN_FACTOR`, `MIN_CENT_DIFFERENCE` — module-level in `strategy.rs` (algorithm parameters without a single owning type)
- `PITCH_SLIDER_CENTS_RANGE`, `INITIAL_OFFSET_RANGE` — module-level in `pitch_matching_session.rs`
- `AMPLITUDE_VARY_SCALING` — module-level in `pitch_comparison_session.rs`

**Type threading approach:** `Cents` was threaded through public APIs while keeping raw `f64` for internal Welford arithmetic (AC7). At persistence boundaries (bridge observers, hydration code), `Cents::new()` wrapping is applied at the call site. The `kazez_narrow`/`kazez_widen` functions unwrap to `f64` internally for math, rewrap result.

**No separate constants module needed:** All constants found natural homes on existing types or in their consuming modules. No `training/constants.rs` was created. Re-exports in `session/mod.rs` were updated for the new session constants.

### Completion Notes

All 6 tasks completed. 306 domain tests pass (291 unit + 15 integration). Zero clippy warnings on both domain and web crates. Trunk build succeeds. All 8 acceptance criteria satisfied.

## File List

- `domain/src/types/cents.rs` — Added `PER_OCTAVE` and `PER_SEMITONE_ET` associated constants
- `domain/src/tuning.rs` — Replaced `1200.0` and `100.0` magic numbers with `Cents::PER_OCTAVE` and `Cents::PER_SEMITONE_ET`
- `domain/src/profile.rs` — Added `PerceptualNote::COLD_START_DIFFICULTY`, changed `update()` to take `Cents`, `update_matching()` to take `Cents`, `average_threshold()` to return `Option<Cents>`, updated all tests
- `domain/src/strategy.rs` — Added `KAZEZ_NARROW_FACTOR`, `KAZEZ_WIDEN_FACTOR`, `MIN_CENT_DIFFERENCE` constants; changed `kazez_narrow`/`kazez_widen` to `Cents -> Cents`; replaced defaults; updated all tests
- `domain/src/session/pitch_matching_session.rs` — Added `PITCH_SLIDER_CENTS_RANGE`, `INITIAL_OFFSET_RANGE` constants; replaced `1200.0`, `20.0`, `40.0` magic numbers; updated `update_matching` call
- `domain/src/session/pitch_comparison_session.rs` — Added `AMPLITUDE_VARY_SCALING` constant; replaced `5.0`, `0.1`, `100.0` magic numbers
- `domain/src/session/mod.rs` — Updated re-exports for new session constants
- `domain/src/training/pitch_comparison.rs` — Replaced `100.0` in `is_target_higher` with `Cents::PER_SEMITONE_ET`
- `domain/tests/profile_hydration.rs` — Updated all `update`/`update_matching` calls to use `Cents`
- `domain/tests/strategy_convergence.rs` — Updated `profile.update` call to use `Cents`
- `web/src/bridge.rs` — Updated `ProfileObserver` to wrap cent offset in `Cents::new()`
- `web/src/app.rs` — Updated hydration code to wrap values in `Cents::new()`
- `docs/implementation-artifacts/sprint-status.yaml` — Updated story status
- `docs/implementation-artifacts/7-0b-extract-constants-and-domain-types.md` — Updated story file

## Change Log

- 2026-03-06: Implemented all tasks — extracted magic numbers into named constants, threaded `Cents` type through profile and strategy APIs, updated all callers and tests
