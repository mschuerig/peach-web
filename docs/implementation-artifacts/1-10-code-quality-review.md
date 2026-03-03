# Story 1.10: Code Quality Review

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the Epic 1 codebase reviewed for idiomatic Rust, best practices, fallible constructors, concurrency safety, and resource management,
so that the foundation is solid before building Epic 2 features on top of it.

## Acceptance Criteria

1. **AC1 — Idiomatic Rust:** All code follows standard Rust idioms: proper use of iterators, pattern matching, Option/Result, traits, and ownership patterns. No anti-patterns such as non-idiomatic Clone implementations or unnecessary type wrapping.

2. **AC2 — Best practices:** Module visibility is correctly scoped (no unnecessarily public fields), documentation exists for public APIs, naming conventions match Rust and project standards, and clippy passes with zero warnings including pre-existing ones.

3. **AC3 — Obvious improvements:** Code duplication is eliminated where a helper method or shared abstraction would reduce maintenance burden without over-engineering. Unnecessary complexity is simplified.

4. **AC4 — Fallible constructors (try_new):** Domain value types that currently panic on invalid input via assert!/unwrap in new() provide a fallible try_new() or TryFrom alternative returning Result. Callers in both crates migrate to fallible constructors where input is not statically guaranteed valid.

5. **AC5 — Concurrency and interior mutability:** All Rc<RefCell<>> usage is verified safe from overlapping borrow panics. RefCell borrow scopes are minimized. No unnecessary SendWrapper wrapping. Cell vs RefCell usage is correct throughout.

6. **AC6 — Resource management:** All event listeners are properly cleaned up. Audio resources are stopped/disconnected on Drop or cleanup. No closure leaks. IndexedDB callback chains handle all error paths.

7. **AC7 — Professional design review:** An experienced Rust developer would look at this codebase and consider it well-designed. Evaluate overall architecture, abstraction choices, error modeling, type design, module boundaries, and API ergonomics. Flag anything that would make a seasoned Rustacean raise an eyebrow — even if it technically works.

## Tasks / Subtasks

- [ ] Task 1: Fallible constructors — domain value types (AC: 4,7)
  - [ ] 1.1 Expand `DomainError` in `domain/src/error.rs` with variants: `InvalidFrequency(f64)`, `InvalidMIDINote(u8)`, `InvalidMIDIVelocity(u8)`, `InvalidSettings(String)`, `TranspositionOutOfRange { note: u8, semitones: i16 }`
  - [ ] 1.2 Add `Frequency::try_new(f64) -> Result<Self, DomainError>` in `domain/src/types/frequency.rs` (lines 18-21 currently panic on `<= 0.0`)
  - [ ] 1.3 Add `MIDINote::try_new(u8) -> Result<Self, DomainError>` in `domain/src/types/midi.rs` (lines 26-29 currently panic on `> 127`)
  - [ ] 1.4 Add `MIDIVelocity::try_new(u8) -> Result<Self, DomainError>` in `domain/src/types/midi.rs` (lines 72-78 currently panic on `0` or `> 127`)
  - [ ] 1.5 Add `TrainingSettings::try_new(...) -> Result<Self, DomainError>` in `domain/src/strategy.rs` (lines 34-54 currently panic on `min > max`)
  - [ ] 1.6 Change `MIDINote::transposed(semitones) -> MIDINote` to return `Result<MIDINote, DomainError>` in `domain/src/types/midi.rs` (lines 47-54 currently panic on out-of-range) — update all callers
  - [ ] 1.7 Refactor `new()` to delegate to `try_new().expect(...)` — validation logic lives in one place
  - [ ] 1.8 Audit ALL callers of `new()` across both crates. Migrate to `try_new()` + `?` propagation where input comes from: user settings, browser APIs, deserialization, computed values (e.g. transposition, frequency calculation). Keep `new()` only where input is a hardcoded literal or already validated upstream (e.g. test fixtures like `MIDINote::new(60)`)
  - [ ] 1.9 Add unit tests for all new `try_new()` methods: valid input, boundary values, and error cases

- [ ] Task 2: Idiomatic Rust fixes (AC: 1,7)
  - [ ] 2.1 Replace `clone_shared()` method in `web/src/adapters/audio_oscillator.rs` (lines 33-37) with `#[derive(Clone)]` on `OscillatorPlaybackHandle` — standard Clone trait handles `Rc::clone` automatically
  - [ ] 2.2 Replace `eprintln!` in `domain/src/session/comparison_session.rs` (lines 118-123) with proper error handling — change `current_interval(&self)` to return `Option<DirectedInterval>` using `.ok()` instead of printing to stderr
  - [ ] 2.3 Review `Interval::between(...).map(|i| i.semitones()).unwrap_or(0)` in `domain/src/records.rs` (line 29-30) — document why 0 is safe fallback, or propagate the error if it could mask bugs

- [ ] Task 3: Best practices and visibility (AC: 2,7)
  - [ ] 3.1 Make tuple struct fields private in `web/src/bridge.rs`: `ProfileObserver(pub ...)` (line 14), `TrendObserver(pub ...)` (line 68), `TimelineObserver(pub ...)` (line 77) — add `pub fn new(...)` constructors and accessor methods instead. `DataStoreObserver` (line 28) is already correct.
  - [ ] 3.2 Resolve the 3 pre-existing `dead_code` clippy warnings in the web crate (mentioned in story 1.9 task 4.2) — either use the items, add `#[allow(dead_code)]` with a comment explaining why, or remove them
  - [ ] 3.3 Evaluate unused parameters `_is_correct` in `PerceptualProfile::update()` (profile.rs line 98) and `_note` in `update_matching()` (line 194) — remove if not needed for future stories, or add doc comment explaining planned usage

- [ ] Task 4: Code simplification (AC: 3)
  - [ ] 4.1 Extract `trained_means(&self) -> Vec<f64>` helper in `domain/src/profile.rs` to deduplicate the trained-note filtering in `overall_mean()` (lines 143-155) and `overall_std_dev()` (lines 159-174)
  - [ ] 4.2 In `web/src/components/comparison_view.rs`, extract the `sync_signals` closure (lines 87-106, called 12+ times) into a named helper function for readability
  - [ ] 4.3 Replace `window().unwrap().document().unwrap()` chains in `comparison_view.rs` (lines 218, 222, 231, 241) with a shared helper or single binding at the top of the setup block

- [ ] Task 5: Concurrency and interior mutability audit (AC: 5)
  - [ ] 5.1 Audit all `borrow()` / `borrow_mut()` call sites in `comparison_view.rs` (lines 296-417 async loop) — verify no overlapping borrows are possible. Add comments documenting safety invariant where borrow scopes are non-obvious.
  - [ ] 5.2 Evaluate `SendWrapper` usage in `app.rs` (lines 22-25) — determine if Leptos 0.8 signals make it redundant. Remove if unnecessary, document if required.
  - [ ] 5.3 Verify `Cell<bool>` vs `RefCell` usage is appropriate for all cancellation flags

- [ ] Task 6: Resource management and error handling (AC: 6)
  - [ ] 6.1 Review `unwrap()` calls in IndexedDB callback chains in `web/src/adapters/indexeddb_store.rs` (lines 34, 37, 45, 48, 149, 151, 155, 160) — replace with proper error handling where feasible within JS closure constraints
  - [ ] 6.2 Review `use_context().expect(...)` calls in `comparison_view.rs` (lines 32-40, 5 instances) and `start_page.rs` (line 8) — evaluate if Leptos `from_context` prop attribute is applicable or add more descriptive expect messages

- [ ] Task 7: Professional design review pass (AC: 7)
  - [ ] 7.1 Evaluate overall type design: are newtypes used consistently? Is the DomainError enum expressive enough after Task 1 expansion? Would a `Timestamp` newtype improve safety over bare `String`?
  - [ ] 7.2 Evaluate API ergonomics: are public interfaces intuitive? Would a builder pattern help anywhere? Are trait boundaries clean?
  - [ ] 7.3 Evaluate module boundaries: does the domain/web split feel natural? Any leaky abstractions?
  - [ ] 7.4 Document findings and recommendations in this story's Dev Agent Record — even items deferred for later epics

- [ ] Task 8: Verify and validate (AC: all)
  - [ ] 8.1 `cargo clippy -p domain` — zero warnings
  - [ ] 8.2 `cargo clippy -p web` — zero warnings (including resolved pre-existing ones)
  - [ ] 8.3 `cargo test -p domain` — all existing tests pass plus new try_new tests
  - [ ] 8.4 `trunk build` — successful WASM compilation
  - [ ] 8.5 `trunk serve` — manual browser smoke test: start training, complete a comparison, verify persistence, interrupt via tab switch, verify clean recovery

## Dev Notes

### Core Approach: Review-Driven Refactoring

This is NOT a feature story — it's a codebase quality pass across the entire Epic 1 implementation (stories 1.1-1.9). The dev agent acts as reviewer AND implementer. The approach is:

1. Read each file systematically
2. Apply fixes per the tasks above
3. Run tests after each logical group of changes
4. Document any findings deferred to later epics in the Dev Agent Record

**Key principle:** Improve what's there without changing behavior. All existing tests must continue to pass. No new features, no architecture changes, no UI changes.

### Fallible Constructor Design (Task 1)

The domain crate heavily uses panicking `new()` constructors for value types. These are acceptable for internal code where invariants are guaranteed by the caller, but the web crate passes user-derived or browser-derived data that could violate invariants. Adding `try_new()` provides a safe path for untrusted input.

**Pattern to follow:**

```rust
// In domain/src/types/frequency.rs:
impl Frequency {
    /// Creates a Frequency, panicking if value <= 0.0.
    /// Use try_new() when input is not guaranteed valid.
    pub fn new(raw_value: f64) -> Self {
        Self::try_new(raw_value).expect("Frequency must be positive")
    }

    pub fn try_new(raw_value: f64) -> Result<Self, DomainError> {
        if raw_value <= 0.0 || raw_value.is_nan() || raw_value.is_infinite() {
            Err(DomainError::InvalidFrequency(raw_value))
        } else {
            Ok(Self { raw_value })
        }
    }
}
```

**Refactor `new()` to delegate to `try_new()`** — validation logic lives in one place. Then audit every caller: if the input could be invalid at runtime (user settings, deserialization, computed values, browser API results), migrate to `try_new()` with proper error propagation. Only retain `new()` at call sites where the value is a hardcoded literal or already validated (e.g. `MIDINote::new(60)` in tests).

**`MIDINote::transposed()` is different** — this is the one signature change. It currently panics if the transposition goes out of 0-127 range. Since transposition inputs come from the adaptive algorithm and can legitimately produce out-of-range values, this MUST return `Result`. All callers must be updated to handle the error.

### DomainError Expansion

Current state (`domain/src/error.rs`): only `IntervalOutOfRange(u8)`.

**Target state:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("interval out of range: {0} semitones")]
    IntervalOutOfRange(u8),

    #[error("invalid frequency: {0} (must be positive, finite)")]
    InvalidFrequency(f64),

    #[error("invalid MIDI note: {0} (must be 0-127)")]
    InvalidMIDINote(u8),

    #[error("invalid MIDI velocity: {0} (must be 1-127)")]
    InvalidMIDIVelocity(u8),

    #[error("invalid training settings: {0}")]
    InvalidSettings(String),

    #[error("transposition out of MIDI range: note {note} + {semitones} semitones")]
    TranspositionOutOfRange { note: u8, semitones: i16 },
}
```

### Bridge Visibility Fix (Task 3.1)

Current bridge.rs exposes inner state via public tuple struct fields:

```rust
pub struct ProfileObserver(pub Rc<RefCell<PerceptualProfile>>);   // BAD: inner is public
pub struct TrendObserver(pub Rc<RefCell<TrendAnalyzer>>);         // BAD
pub struct TimelineObserver(pub Rc<RefCell<ThresholdTimeline>>);   // BAD
```

**Fix:** Make the inner field private and add constructor:

```rust
pub struct ProfileObserver(Rc<RefCell<PerceptualProfile>>);

impl ProfileObserver {
    pub fn new(profile: Rc<RefCell<PerceptualProfile>>) -> Self {
        Self(profile)
    }
}
```

Then update callers in `comparison_view.rs` that currently access `.0` directly to use `::new()`.

### SendWrapper Evaluation (Task 5.2)

`app.rs` wraps domain objects in `SendWrapper<Rc<RefCell<T>>>`. In Leptos 0.8, `provide_context` requires `Send + Sync` for non-local contexts. `SendWrapper` satisfies this for WASM (single-threaded). **This is likely still required** — but verify by checking whether `RwSignal::new_local()` or `provide_context` with `LocalResource` could replace the pattern. If `SendWrapper` is required, add a comment explaining why.

### What NOT to Change

- **No behavior changes** — all existing functionality must work identically
- **No architecture changes** — the domain/web split, observer pattern, and signal bridge remain as-is
- **No new dependencies** — improvements use only what's already in Cargo.toml
- **No UI changes** — this is purely internal code quality
- **Don't remove the panicking `new()` constructors** — they're convenient for test code and internal callers where invariants are guaranteed. `try_new()` is additive. But `new()` delegates to `try_new()` internally.
- **Don't refactor ComparisonView into sub-components** — that's a larger structural change better suited for a dedicated story. Task 4.2 only extracts a helper function within the same file.

### Project Structure Notes

- Primary changes in `domain/src/`: error.rs, types/frequency.rs, types/midi.rs, strategy.rs, profile.rs, session/comparison_session.rs, records.rs
- Primary changes in `web/src/`: bridge.rs, adapters/audio_oscillator.rs, components/comparison_view.rs, components/start_page.rs, adapters/indexeddb_store.rs, app.rs
- No new files created — all edits to existing files
- Alignment with project-context.md rule: "Result<T, E> for all fallible operations"

### Files to Modify

| File | Change |
|---|---|
| `domain/src/error.rs` | Expand DomainError with new variants |
| `domain/src/types/frequency.rs` | Add `try_new()`, refactor `new()` to delegate |
| `domain/src/types/midi.rs` | Add `MIDINote::try_new()`, `MIDIVelocity::try_new()`, change `transposed()` return type |
| `domain/src/strategy.rs` | Add `TrainingSettings::try_new()`, refactor `new()` to delegate |
| `domain/src/profile.rs` | Extract `trained_means()` helper, evaluate unused params |
| `domain/src/session/comparison_session.rs` | Replace `eprintln!` with proper error handling, update `transposed()` callers |
| `domain/src/records.rs` | Document or fix `unwrap_or(0)` fallback |
| `web/src/bridge.rs` | Make observer tuple struct fields private, add constructors |
| `web/src/adapters/audio_oscillator.rs` | Replace `clone_shared()` with `#[derive(Clone)]` |
| `web/src/adapters/indexeddb_store.rs` | Improve error handling in JS callback chains |
| `web/src/components/comparison_view.rs` | Extract sync_signals helper, consolidate document access, audit borrow scopes |
| `web/src/components/start_page.rs` | Improve context unwrap handling |
| `web/src/app.rs` | Evaluate/document SendWrapper usage |

### References

- [Source: docs/planning-artifacts/architecture.md#Error Handling] — "Result<T, E> for all fallible operations, custom error enums per domain area via thiserror"
- [Source: docs/planning-artifacts/architecture.md#Implementation Patterns] — "unwrap()/expect() only for invariants that are genuine programming errors"
- [Source: docs/project-context.md#Rust Language Rules] — "unwrap()/expect() only for genuine programming errors"
- [Source: docs/project-context.md#Code Quality & Style Rules] — clippy enforcement, naming conventions
- [Source: docs/planning-artifacts/architecture.md#Project Structure] — domain crate pure Rust, web crate browser-specific
- [Source: docs/implementation-artifacts/1-7-comparison-training-ui.md] — Established event listener cleanup patterns
- [Source: docs/implementation-artifacts/1-8-persistence-and-profile-hydration.md] — Observer injection patterns
- [Source: docs/implementation-artifacts/1-9-interruption-handling.md] — Shared closure pattern, StoredValue lifecycle

### Previous Story Intelligence (from Story 1.9)

**Patterns established:**
- `interrupt_and_navigate` shared closure with `Rc` wrapping — good pattern for shared logic
- `StoredValue::new_local()` for closure lifetime management
- `on_cleanup` for comprehensive event listener removal
- `Rc<Cell<bool>>` cancellation flag — correct use of `Cell` for `Copy` type

**Code review learnings from 1.8 and 1.9:**
- Keep web-sys feature lists explicit and minimal
- 3 pre-existing `dead_code` warnings accumulated — this story resolves them
- Pattern: implementation commit -> code review fixes -> story marked done

### Git Intelligence

Recent commit pattern (last 10 commits):
- Alternating: "Implement story X" -> "Apply code review fixes for story X and mark as done"
- Consistent single-story-per-branch approach
- Most recent: stories 1.7, 1.8, 1.9 each follow this pattern

Files most touched in Epic 1:
- `web/src/components/comparison_view.rs` — central to stories 1.6, 1.7, 1.8, 1.9
- `domain/src/session/comparison_session.rs` — core state machine
- `web/src/bridge.rs` — observer bridge, modified in 1.8
- `domain/src/types/*.rs` — established in 1.2, stable since

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
