# Story 1.2: Domain Value Types & Tuning System

Status: done

## Story

As a developer,
I want all domain value types and the tuning system implemented with unit tests,
so that the foundational types are correct, precise, and ready for use by sessions and profiles.

## Acceptance Criteria

1. **AC1 — MIDINote construction and naming:** `MIDINote::new(60).name()` returns `"C4"`, `MIDINote::new(69).name()` returns `"A4"`
2. **AC2 — MIDINote out-of-range invariant:** `MIDINote::new(128)` panics (programming error invariant)
3. **AC3 — MIDINote random generation:** `MIDINote::random(36..=84)` returns a value within that range
4. **AC4 — MIDINote transposition:** `MIDINote::new(60).transposed(DirectedInterval::new(Interval::PerfectFifth, Direction::Up))` returns raw_value 67
5. **AC5 — Value type constraints:** Cents (unrestricted f64), Frequency panics on <= 0, NoteDuration clamps to 0.3-3.0, UnitInterval clamps to 0.0-1.0, MIDIVelocity panics outside 1-127, AmplitudeDB clamps to -90.0..12.0, SoundSourceID defaults to `"sf2:8:80"` if empty
6. **AC6 — Interval::between:** `Interval::between(MIDINote::new(60), MIDINote::new(67))` returns `Interval::PerfectFifth`; distance > 12 returns `Err`
7. **AC7 — DirectedInterval::between:** `DirectedInterval::between(MIDINote::new(60), MIDINote::new(67))` returns `(PerfectFifth, Up)`; reversed args return `(PerfectFifth, Down)`
8. **AC8 — Equal temperament frequency:** `TuningSystem::EqualTemperament.frequency(DetunedMIDINote::from(MIDINote::new(69)), Frequency::CONCERT_440)` returns exactly 440.0 Hz
9. **AC9 — Precision (NFR2):** All frequency conversions accurate to within 0.1 cent of mathematically correct values
10. **AC10 — Just intonation offsets:** `TuningSystem::JustIntonation.cent_offset(Interval::PerfectFifth)` returns 701.955
11. **AC11 — Full test suite:** `cargo test -p domain` passes with zero browser dependencies

## Tasks / Subtasks

- [x] Task 1: Create module structure (AC: all)
  - [x] Create `domain/src/types/mod.rs` with re-exports
  - [x] Create individual type files under `domain/src/types/`
  - [x] Create `domain/src/tuning.rs`
  - [x] Update `domain/src/lib.rs` to declare and re-export all modules
- [x] Task 2: Implement MIDINote (AC: 1,2,3,4)
  - [x] `domain/src/types/midi.rs` — MIDINote struct with new(), name(), random(), transposed()
  - [x] MIDIVelocity struct in same file
  - [x] Inline unit tests for all MIDINote operations
- [x] Task 3: Implement simple value types (AC: 5)
  - [x] `domain/src/types/cents.rs` — Cents with magnitude()
  - [x] `domain/src/types/frequency.rs` — Frequency with CONCERT_440 constant
  - [x] `domain/src/types/duration.rs` — NoteDuration with clamping
  - [x] `domain/src/types/amplitude.rs` — AmplitudeDB (f32!) with clamping, UnitInterval with clamping
  - [x] `domain/src/types/sound_source.rs` — SoundSourceID with empty-default
  - [x] Inline unit tests for each type
- [x] Task 4: Implement Interval, Direction, DirectedInterval (AC: 6,7)
  - [x] `domain/src/types/interval.rs` — Interval enum (13 variants), Direction enum, DirectedInterval struct
  - [x] Interval::between() returning Result
  - [x] DirectedInterval::between() with prime-always-up rule
  - [x] Inline unit tests
- [x] Task 5: Implement DetunedMIDINote (AC: 8)
  - [x] `domain/src/types/detuned.rs` — DetunedMIDINote with From<MIDINote> (offset=0.0)
  - [x] Inline unit tests
- [x] Task 6: Implement TuningSystem (AC: 8,9,10)
  - [x] `domain/src/tuning.rs` — TuningSystem enum with frequency() and cent_offset()
  - [x] Equal temperament formula: `ref_pitch * 2^((midi - 69 + cents/100) / 12)`
  - [x] Just intonation cent offset lookup table
  - [x] Inline unit tests + integration test `domain/tests/tuning_accuracy.rs`
- [x] Task 7: Verify full suite (AC: 11)
  - [x] Run `cargo test -p domain` — all pass
  - [x] Run `cargo clippy -p domain` — no warnings

## Dev Notes

### Architecture Compliance

- **Crate boundary:** ALL code in `domain/` crate only. Zero `web-sys`, `wasm-bindgen`, or `leptos` imports. The compiler enforces this — if you add browser deps, `cargo test -p domain` will fail on native.
- **Type name fidelity:** Use EXACT names from domain blueprint. `MIDINote` not `MidiNote`. `DetunedMIDINote` not `PitchOffset`. `CompletedComparison` not `ComparisonResult`. This is the shared language across all agents and documentation.
- **Field naming:** Blueprint uses `rawValue` — in Rust use `raw_value` (snake_case per clippy). Document the mapping explicitly.

### Numeric Precision Rules

- All cent values, frequency values: **`f64`** — no `f32` anywhere in domain calculations
- MIDI note values: **`u8`** (0-127)
- **Exception:** `AmplitudeDB` uses **`f32`** per blueprint Section 2.10
- Sample counts: **`u32`**

### Error Handling Pattern

- `Result<T, E>` for all fallible operations (e.g., `Interval::between` when distance > 12)
- Custom error enums via `thiserror` — create `domain/src/error.rs` or per-type errors
- `unwrap()`/`expect()` ONLY for invariant violations (programming errors): MIDINote outside 0-127, Frequency <= 0, MIDIVelocity outside 1-127
- Clamping types (NoteDuration, UnitInterval, AmplitudeDB) silently clamp — no errors
- `let _ = fallible_call()` is **forbidden** — every Result must be handled

### Serialization

- `#[derive(Serialize, Deserialize)]` on all types
- Struct fields serialize as snake_case (serde default)
- Enum variants serialize as camelCase: `#[serde(rename_all = "camelCase")]` — matches blueprint storage identifiers (`"equalTemperament"`, `"justIntonation"`, `"perfectFifth"`, etc.)

### Testing Standards

- **Inline unit tests:** `#[cfg(test)] mod tests { ... }` at the bottom of each source file
- **Integration test:** `domain/tests/tuning_accuracy.rs` for frequency conversion precision
- **Test naming:** `test_` prefix + descriptive snake_case (e.g., `test_midi_note_c4_name`, `test_frequency_panics_on_zero`)
- **Run natively:** `cargo test -p domain` — fast, no browser, no WASM

### Crate Dependencies (already in Cargo.toml)

```toml
[dependencies]
serde = { workspace = true }   # version "1" with "derive" feature
thiserror = "2"
rand = "0.9"
```

No additional dependencies needed. Do NOT add any.

### Project Structure Notes

Target file layout:

```
domain/src/
├── lib.rs                  # mod declarations + pub use re-exports
├── types/
│   ├── mod.rs              # Re-exports all types
│   ├── midi.rs             # MIDINote, MIDIVelocity
│   ├── cents.rs            # Cents
│   ├── frequency.rs        # Frequency
│   ├── interval.rs         # Interval, Direction, DirectedInterval
│   ├── detuned.rs          # DetunedMIDINote
│   ├── duration.rs         # NoteDuration
│   ├── amplitude.rs        # AmplitudeDB, UnitInterval
│   └── sound_source.rs     # SoundSourceID
├── tuning.rs               # TuningSystem enum, frequency conversion, cent offsets
domain/tests/
└── tuning_accuracy.rs      # Integration test: frequency precision within 0.1 cent
```

Existing `domain/src/lib.rs` contains only an empty `#[cfg(test)] mod tests {}` — replace entirely with module declarations and re-exports.

### Type Implementation Reference

#### MIDINote (domain/src/types/midi.rs)
- `raw_value: u8` — panic if outside 0-127
- `name() -> String`: `NOTE_NAMES[raw_value % 12]` + `(raw_value / 12 - 1)` where `NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]`
- `random(range: RangeInclusive<u8>) -> MIDINote` — uniform random
- `transposed(by: DirectedInterval) -> MIDINote` — add/subtract semitones, panic if result outside 0-127
- Derive: `Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize`

#### Cents (domain/src/types/cents.rs)
- `raw_value: f64` — unrestricted
- `magnitude() -> f64`: `raw_value.abs()`
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`

#### DetunedMIDINote (domain/src/types/detuned.rs)
- `note: MIDINote`, `offset: Cents`
- `impl From<MIDINote>` — offset defaults to `Cents(0.0)`
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`

#### Frequency (domain/src/types/frequency.rs)
- `raw_value: f64` — panic if <= 0.0
- `const CONCERT_440: Frequency = Frequency { raw_value: 440.0 }` (or associated const)
- Derive: `Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize`

#### Interval (domain/src/types/interval.rs)
- Enum with 13 variants: `Prime = 0, MinorSecond = 1, ... Octave = 12`
- `semitones(&self) -> u8` — returns the discriminant value
- `between(reference: MIDINote, target: MIDINote) -> Result<Interval, DomainError>` — `abs(diff)`, error if > 12
- `#[serde(rename_all = "camelCase")]`
- Derive: `Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize`

#### Direction (domain/src/types/interval.rs)
- Enum: `Up = 0, Down = 1`
- `#[serde(rename_all = "camelCase")]`
- Derive: `Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize`

#### DirectedInterval (domain/src/types/interval.rs)
- `interval: Interval`, `direction: Direction`
- `between(ref: MIDINote, target: MIDINote) -> Result<Self, DomainError>` — uses `Interval::between()`, direction is Up if target >= ref, Down otherwise. Prime is always Up.
- Derive: `Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`

#### NoteDuration (domain/src/types/duration.rs)
- `raw_value: f64` — clamped to 0.3..=3.0 on construction
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`

#### MIDIVelocity (domain/src/types/midi.rs)
- `raw_value: u8` — panic outside 1-127
- Derive: `Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize`

#### AmplitudeDB (domain/src/types/amplitude.rs)
- `raw_value: f32` — clamped to -90.0..=12.0 (**f32, not f64!**)
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`

#### UnitInterval (domain/src/types/amplitude.rs)
- `raw_value: f64` — clamped to 0.0..=1.0
- Derive: `Clone, Copy, Debug, PartialEq, Serialize, Deserialize`

#### SoundSourceID (domain/src/types/sound_source.rs)
- `raw_value: String` — defaults to `"sf2:8:80"` if empty
- Derive: `Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`

#### TuningSystem (domain/src/tuning.rs)
- Enum: `EqualTemperament, JustIntonation`
- `cent_offset(&self, interval: Interval) -> f64` — lookup table
- `frequency(&self, note: DetunedMIDINote, reference_pitch: Frequency) -> Frequency`
  - Formula: `ref_pitch * 2.0_f64.powf((midi - 69.0 + cents / 100.0) / 12.0)`
  - Convenience: `frequency_for_note(&self, note: MIDINote, ref_pitch: Frequency) -> Frequency` treats as DetunedMIDINote with 0 offset
- `#[serde(rename_all = "camelCase")]`
- Derive: `Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`

### Just Intonation Cent Offset Table

| Interval | Equal Temperament | Just Intonation |
|---|---|---|
| Prime | 0.0 | 0.0 |
| MinorSecond | 100.0 | 111.731 |
| MajorSecond | 200.0 | 203.910 |
| MinorThird | 300.0 | 315.641 |
| MajorThird | 400.0 | 386.314 |
| PerfectFourth | 500.0 | 498.045 |
| Tritone | 600.0 | 590.224 |
| PerfectFifth | 700.0 | 701.955 |
| MinorSixth | 800.0 | 813.686 |
| MajorSixth | 900.0 | 884.359 |
| MinorSeventh | 1000.0 | 1017.596 |
| MajorSeventh | 1100.0 | 1088.269 |
| Octave | 1200.0 | 1200.0 |

### Previous Story Intelligence (1.1 Project Scaffold)

**Key learnings from Story 1.1:**
- Rust edition 2024 for both crates
- `.cargo/config.toml` has `--cfg getrandom_backend="wasm_js"` rustflags — this applies to ALL crates in workspace. The `rand` crate in domain uses `getrandom` internally, and this config ensures WASM compatibility when domain is compiled as part of the web crate, while still allowing native `cargo test -p domain`
- Domain crate currently has empty `lib.rs` with just `#[cfg(test)] mod tests {}` — replace entirely
- Code review feedback from 1.1: use conditional log levels, always add explicit build configs, remove placeholder tests

**Patterns established:**
- Workspace with `resolver = "2"`
- Shared deps via `[workspace.dependencies]`
- Release profile: `opt-level = 'z'`, `lto = true`, `codegen-units = 1`
- Leptos 0.8 CSR in web crate (not relevant to this story but context)

### Anti-Patterns (DO NOT)

- Do NOT add browser dependencies to domain crate
- Do NOT rename domain types from the blueprint
- Do NOT use `f32` for cents, frequency, or statistical values (except AmplitudeDB)
- Do NOT use `anyhow` in domain crate — use `thiserror` only
- Do NOT swallow errors with `let _ =`
- Do NOT implement types from later stories (Comparison, CompletedComparison, PerceptualProfile, etc. — those are Stories 1.3+)
- Do NOT add gamification, scores, streaks, or session summaries

### References

- [Source: docs/ios-reference/domain-blueprint.md#2-domain-value-types] — Authoritative type definitions
- [Source: docs/ios-reference/domain-blueprint.md#3-tuning-system-the-bridge] — TuningSystem, frequency formula, cent offset table
- [Source: docs/planning-artifacts/architecture.md] — Two-crate structure, naming conventions, file layout
- [Source: docs/project-context.md] — Coding rules, error handling, serialization, testing standards
- [Source: docs/planning-artifacts/ux-design-specification.md] — Settings constraints (note duration, reference pitch, vary loudness)
- [Source: docs/implementation-artifacts/1-1-project-scaffold.md] — Previous story learnings and patterns

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Clippy caught `if_same_then_else` in `DirectedInterval::between()` where prime-always-up and target>=reference cases both returned `Direction::Up`. Simplified to single condition since equal notes (prime) satisfy `target >= reference`.

### Completion Notes List

- All 11 domain value types implemented with exact blueprint names: MIDINote, MIDIVelocity, Cents, Frequency, NoteDuration, AmplitudeDB, UnitInterval, SoundSourceID, Interval, Direction, DirectedInterval, DetunedMIDINote, TuningSystem
- Error handling via `DomainError` enum with `thiserror` for `Interval::between` out-of-range
- Panic invariants for programming errors: MIDINote >127, Frequency <=0, MIDIVelocity outside 1-127
- Clamping types silently clamp: NoteDuration (0.3-3.0), AmplitudeDB (-90.0..12.0 f32), UnitInterval (0.0-1.0)
- SoundSourceID defaults to "sf2:8:80" on empty string
- All serde: struct fields snake_case, enum variants camelCase
- 79 inline unit tests + 4 integration tests = 83 total, all pass
- Zero clippy warnings
- Zero browser dependencies — pure native Rust
- Added `serde_json` as dev-dependency for serialization round-trip tests
- Equal temperament precision verified to within 0.1 cent for all 128 MIDI notes
- Just intonation cent offset lookup table with all 13 intervals

### File List

- domain/src/lib.rs (modified — replaced empty test stub with module declarations and re-exports)
- domain/src/error.rs (new — DomainError enum)
- domain/src/types/mod.rs (new — module re-exports)
- domain/src/types/midi.rs (new — MIDINote, MIDIVelocity)
- domain/src/types/cents.rs (new — Cents)
- domain/src/types/frequency.rs (new — Frequency)
- domain/src/types/duration.rs (new — NoteDuration)
- domain/src/types/amplitude.rs (new — AmplitudeDB, UnitInterval)
- domain/src/types/sound_source.rs (new — SoundSourceID)
- domain/src/types/interval.rs (new — Interval, Direction, DirectedInterval)
- domain/src/types/detuned.rs (new — DetunedMIDINote)
- domain/src/tuning.rs (new — TuningSystem with frequency() and cent_offset())
- domain/tests/tuning_accuracy.rs (new — integration test for frequency precision)
- domain/Cargo.toml (modified — added serde_json dev-dependency)
- Cargo.lock (modified — auto-generated dependency lockfile)

## Senior Developer Review (AI)

**Reviewer:** Code Review Agent (Claude Opus 4.6) | **Date:** 2026-03-03

**Findings (6 fixed, 1 deferred):**

- **[FIXED][HIGH] H1 — Public fields bypassed constructor invariants:** `raw_value` was `pub` on MIDINote, MIDIVelocity, Frequency, NoteDuration, AmplitudeDB, UnitInterval, SoundSourceID. Direct struct construction bypassed panic/clamping invariants. Fixed: fields made private, `pub fn raw_value()` getters added. All cross-module consumers updated.
- **[FIXED][MEDIUM] M1 — `TuningSystem::frequency()` ignored `self`:** Both variants produced identical 12-TET results. Fixed: decompose MIDI distance into octaves + remainder interval, look up tuning-system-specific cent offset via `cent_offset()`, convert to Hz via `ref * 2^(cents/1200)`. JI now produces correct pure-ratio frequencies (e.g., P5 = 3/2 = 660 Hz, M3 = 5/4 = 550 Hz). 7 new JI tests added.
- **[FIXED][MEDIUM] M2 — NaN passed through clamping constructors:** `f64::NAN.clamp()` returns NaN, bypassing clamping on NoteDuration, AmplitudeDB, UnitInterval. Fixed: `assert!(!raw_value.is_nan())` added before clamping.
- **[FIXED][MEDIUM] M3 — Missing serde round-trip tests:** Only enum types had serde tests. Fixed: added round-trip tests for all 10 struct types (MIDINote, MIDIVelocity, Cents, Frequency, NoteDuration, AmplitudeDB, UnitInterval, SoundSourceID, DetunedMIDINote, DirectedInterval).
- **[FIXED][LOW] L1 — Cargo.lock not in File List:** Added to File List.
- **[FIXED][LOW] L2 — Missing MIDINote::transposed Prime test:** Added `test_midi_note_transposed_by_prime`.
- **[FIXED][LOW] L3 — Missing MIDINote::random edge cases:** Added `test_midi_note_random_single_element_range` and `test_midi_note_random_full_range`.

**Test count:** 60 → 83 (23 new tests: 10 serde round-trip, 3 NaN panic, 3 MIDINote edge cases, 7 JI frequency)

**Outcome:** All issues resolved.

## Change Log

- 2026-03-03: Implemented all domain value types and tuning system (Story 1.2) — 13 new files, 60 tests
- 2026-03-03: Code review fixes — private fields with getters (H1), NaN guards (M2), serde round-trip tests (M3), edge case tests (L2, L3), TuningSystem JI frequency fix (M1) — 83 tests total
