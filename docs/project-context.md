---
project_name: 'peach-web'
user_name: 'Michael'
date: '2026-03-03'
status: 'complete'
sections_completed: ['technology_stack', 'language_rules', 'framework_rules', 'testing_rules', 'code_quality', 'workflow_rules', 'critical_rules']
rule_count: 42
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

| Technology | Version | Role |
|---|---|---|
| Rust | stable (latest) | Language — domain + web crates |
| Leptos | 0.8.x | UI framework (CSR mode, fine-grained signals) |
| Trunk | latest | WASM bundler, dev server, asset pipeline |
| Tailwind CSS | 3.x or 4.x | Utility-first styling |
| `wasm-bindgen` | latest | Rust-WASM-JS interop |
| `wasm-bindgen-futures` | latest | Async futures on browser microtask queue |
| `web-sys` | latest | Direct Web API bindings (Web Audio, IndexedDB) |
| `gloo-timers` | latest | Browser timer abstractions |
| `serde` + `serde_json` | latest | Serialization (storage, future export) |
| `thiserror` | latest | Error types (domain crate only) |
| `anyhow` | latest | Error aggregation (web crate only) |
| OxiSynth | latest | SoundFont synthesis (Phase 2+) |
| `leptos_router` | 0.8.x | Client-side routing |

**Version Constraints:**

- Leptos and `leptos_router` versions must match exactly
- `web-sys` features are opt-in — enable only the specific Web APIs needed (e.g. `AudioContext`, `OscillatorNode`, `GainNode`, `IdbDatabase`)
- No `tokio` — the only async runtime is `wasm-bindgen-futures`
- Compile target: `wasm32-unknown-unknown`
- `thiserror` in domain crate only; `anyhow` in web crate only — never mix

## Critical Implementation Rules

### Rust Language Rules

**Crate Separation (compiler-enforced):**

- `domain` crate: pure Rust — zero browser dependencies. No `web-sys`, no `wasm-bindgen`, no `leptos`. Must compile and pass tests with native `cargo test`.
- `web` crate: depends on `domain` via path dependency. All browser-specific code lives here.
- This boundary is non-negotiable — the compiler enforces it through `Cargo.toml` dependencies.

**Numeric Precision:**

- All cent, frequency, and statistical accumulator values: `f64` — never `f32`
- MIDI note values: `u8` (0-127)
- Sample counts: `u32`

**Error Handling:**

- `Result<T, E>` for all fallible operations — never silent failures
- `unwrap()` / `expect()` only for genuine programming errors (invariants that cannot happen at runtime)
- `let _ = fallible_call()` is forbidden — every `Result` must be handled or propagated with `?`
- Custom error enums per domain area via `thiserror`

**Async Model:**

- `wasm-bindgen-futures::spawn_local()` to spawn async tasks
- Session cancellation via `Rc<Cell<bool>>` flag checked between await points
- No `tokio`, no heavy async runtime — WASM is single-threaded

**Interior Mutability:**

- `Rc<RefCell<...>>` for shared ownership between async training loop and UI event handlers
- No `Arc`/`Mutex` — single-threaded WASM environment, these are unnecessary overhead

**Serialization:**

- Struct fields serialize as `snake_case` (serde default)
- Enum variants serialize as `camelCase` (`#[serde(rename_all = "camelCase")]`)
- Timestamps: ISO 8601 strings (`"2026-03-02T14:30:00Z"`)

### Leptos Framework Rules

**Reactivity & Signals:**

- `RwSignal` when both read and write needed; `ReadSignal`/`WriteSignal` to restrict access
- Signal setters (`.set()`) for domain-related signals happen ONLY inside the `UIObserver` bridge — components never call `.set()` on domain signals
- Signal naming mirrors domain state: `session_state`, `show_feedback`, `is_last_correct`
- Loading state signals: `is_profile_loaded: RwSignal<bool>`, `soundfont_status: RwSignal<SoundFontStatus>` (enum: `Loading`, `Ready`, `Failed`)

**Component Architecture:**

- Component functions use `PascalCase` (e.g. `fn StartPage()`, `fn PitchComparisonView()`)
- Components receive signals as props via `#[component]` function parameters
- No prop drilling beyond one level — use Leptos context providers for deeply nested data
- One view component per route, composing smaller custom components

**Domain-UI Bridge (UIObserver):**

- `UIObserver` implements domain observer traits, translates domain events into Leptos signal updates
- This is the ONLY bridge between domain state and UI reactivity
- Data flow: Leptos event handler → domain method → observer notification → UIObserver writes signal → Leptos re-renders
- Components are read-only consumers of signals — they trigger domain methods but never mutate domain state

**Routing:**

- `leptos_router` with URL-based routes, hub-and-spoke model
- Start page is root `/`, all views one level deep
- Routes: `/`, `/training/comparison`, `/training/pitch-matching`, `/profile`, `/settings`, `/info`
- Interval mode via query parameter: `?intervals=<codes>` (e.g., `?intervals=M3u,M3d,m6u,M6d` where each code encodes interval quality, size, and direction u/d)

**AudioContext Lifecycle:**

- Created on first training button click (user gesture requirement)
- Stored in `Rc<RefCell<AudioContext>>` or similar shared reference
- Tab visibility change → stop active session, return to start page
- AudioContext suspension detected via state change event → same behavior as tab hide

### Testing Rules

**Test Location:**

- Domain crate unit tests: `#[cfg(test)] mod tests` inline within each source file
- Domain crate integration tests: `domain/tests/` directory for cross-module tests (e.g. `profile_hydration.rs`, `strategy_convergence.rs`, `tuning_accuracy.rs`)
- Web crate: inline `#[cfg(test)]` where feasible; browser-specific tests deferred

**Test Naming:**

- `test_` prefix + descriptive `snake_case` name
- Examples: `test_kazez_narrow_reduces_difficulty()`, `test_welford_mean_with_single_sample()`

**Test Commands:**

- `cargo test -p domain` — domain tests run natively (fast, no browser)
- `wasm-pack test` — web crate browser tests (when established)

**Test Requirements:**

- All domain algorithms must produce identical results to the iOS implementation given the same inputs
- Every domain type with non-trivial logic gets inline unit tests
- Integration tests verify cross-module behavior (profile hydration, strategy convergence)
- Web crate: manual browser testing is acceptable initially

### Code Quality & Style Rules

**Naming Conventions:**

| Element | Convention | Example |
|---|---|---|
| Types, structs, enums | `PascalCase` | `MIDINote`, `PitchComparisonSession` |
| Functions, methods | `snake_case` | `next_pitch_comparison()`, `handle_answer()` |
| Variables, fields | `snake_case` | `reference_note`, `sample_count` |
| Constants | `SCREAMING_SNAKE_CASE` | `FEEDBACK_DURATION`, `REFERENCE_MIDI_NOTE` |
| Modules, files | `snake_case` | `tuning.rs`, `pitch_matching.rs` |
| Enum variants | `PascalCase` | `PlayingReferenceNote`, `AwaitingAnswer` |
| Leptos components | `PascalCase` functions | `fn StartPage()`, `fn PitchComparisonView()` |

**Domain Blueprint Fidelity (critical):**

- Use EXACT type names from the domain blueprint — no renaming, no abbreviation
- `MIDINote`, not `MidiNote` or `Note`. `DetunedMIDINote`, not `PitchOffset`. `CompletedPitchComparison`, not `ComparisonResult`.
- The blueprint is the shared language between agents and documentation

**Module Organization:**

- One public type per file when substantial (e.g. `profile.rs` for `PerceptualProfile`)
- Related small types can share a file (e.g. `types/midi.rs`, `types/cents.rs`)
- `mod.rs` only for module directories, not as a dumping ground
- One file per adapter: `audio_oscillator.rs`, `indexeddb_store.rs`, etc.

**Storage Key Conventions:**

- localStorage: `peach.` prefix + `snake_case` (e.g. `peach.note_range_min`, `peach.tuning_system`)
- IndexedDB: database `peach`, object stores `comparison_records` and `pitch_matching_records`

**Linting:**

- `cargo clippy` enforces idiomatic Rust — run before every commit
- Code review via `/bmad-bmm-code-review` verifies blueprint adherence

### Development Workflow Rules

**Build & Dev Commands:**

- `trunk serve --open` — dev server with auto-rebuild and browser reload
- `trunk build --release` — production build, outputs to `dist/`
- `cargo test -p domain` — run domain tests natively (fast feedback loop)
- `cargo clippy` — lint check before committing

**Project Structure:**

- Cargo workspace with two crates: `domain/` (pure Rust) and `web/` (Leptos + browser)
- `index.html` at project root — Trunk entry point
- `Trunk.toml` — build pipeline configuration
- `input.css` — Tailwind directives, processed via Trunk build hook
- `dist/` — build output, git-ignored

**Deployment:**

- Static files from `dist/` uploaded to Apache on personal website
- No CI/CD pipeline (deferred)
- Manual browser testing for web crate behavior

### Critical Don't-Miss Rules

**Anti-Patterns — NEVER Do These:**

- DO NOT rename domain types — `MIDINote` is `MIDINote`, never `MidiNote`, `Note`, or `Pitch`
- DO NOT put browser dependencies in the `domain` crate — no `web-sys`, `wasm-bindgen`, or `leptos`
- DO NOT swallow errors — `let _ = store.save(record);` violates NFR8 (no silent data loss)
- DO NOT use `f32` for cent, frequency, or statistical values — `f64` everywhere
- DO NOT call `.set()` on domain signals from components — only `UIObserver` writes domain signals
- DO NOT add gamification, scores, streaks, or session summaries — "training, not testing" philosophy
- DO NOT add onboarding, tutorials, or "welcome back" messages — the UX is intentionally sparse
- DO NOT put business logic in view components — views are pure presentation. Constants, data transformations, persistence calls, encoding/decoding, and orchestration belong in domain types or adapter modules. Views only declare signals, wire event handlers, and render DOM.

**Observer Contract:**

- Signatures follow the blueprint exactly: `fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison)`
- Observers take data by reference (`&`), not owned — the session owns the data
- Observers must never panic — internal errors logged via `web_sys::console::warn_1()`
- Observers must never return errors — fire-and-forget pattern

**Web Audio Edge Cases:**

- AudioContext requires user gesture to create — training start button is the gesture
- Tab visibility change → stop active session, return to start page
- SoundFont loading is non-blocking — oscillator fallback always available
- SoundFont load failure → fall back silently to oscillators (no error shown)

**Storage Edge Cases:**

- Storage write failures must inform the user (NFR8) — non-blocking, training continues
- Profile hydration replays ALL stored records on every launch — order matters, results must be deterministic
- localStorage is sync; IndexedDB is async — adapters hide this behind port traits

**Performance Constraints:**

- Audio onset < 50ms from trigger
- State transitions, observer notifications, profile updates < 16ms (single frame)
- Profile hydration < 1s for 10,000 records
- Real-time pitch adjustment: no perceptible lag between slider and frequency change

---

## Usage Guidelines

**For AI Agents:**

- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Refer to `docs/planning-artifacts/architecture.md` for full architectural details
- Refer to `docs/ios-reference/domain-blueprint.md` for exact domain type definitions and algorithms

**For Humans:**

- Keep this file lean and focused on agent needs
- Update when technology stack or patterns change
- Remove rules that become obvious over time

Last Updated: 2026-03-03
