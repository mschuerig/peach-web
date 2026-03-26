---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-03-02'
inputDocuments:
  - docs/planning-artifacts/prd.md
  - docs/ios-reference/domain-blueprint.md
  - docs/ios-reference/peach-web-prd.md
  - docs/ios-reference/peach-web-ux-spec.md
workflowType: 'architecture'
project_name: 'peach-web'
user_name: 'Michael'
date: '2026-03-02'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**

49 FRs across 8 categories. The domain blueprint specifies the complete behavior for each — the PRD adds web-specific considerations (keyboard shortcuts, AudioContext activation, browser storage). Architecturally, the FRs divide into two layers:

- **Domain layer (Rust):** Training state machines, adaptive algorithm, perceptual profile, trend analysis, value types, tuning system — all specified in the domain blueprint with exact algorithms and formulas
- **Adapter layer (Rust/WASM + browser APIs):** Audio playback (Web Audio API), data persistence (IndexedDB/localStorage), settings storage, UI rendering and reactivity

**Non-Functional Requirements:**

NFRs that will drive architectural decisions:

| NFR | Architectural Impact |
|---|---|
| Audio onset <50ms (NFR1) | Audio adapter design, Web Audio node graph, WASM-to-JS call overhead |
| Frequency accuracy <0.1 cent (NFR2) | Mathematical precision in Rust (f64), Web Audio API parameter resolution |
| State transitions <16ms (NFR3) | Efficient observer pattern, minimal allocation in hot paths |
| Profile hydration <1s for 10k records (NFR4) | Rust performance for sequential replay, storage read efficiency |
| Real-time pitch adjustment (NFR5) | Continuous WASM-to-Web Audio communication during slider drag |
| Data survives crashes (NFR6-8) | Storage write strategy, atomicity guarantees |
| Full offline after load (NFR10-11) | Service Worker, asset caching strategy |
| Cross-browser (NFR13-14) | Web Audio API compatibility, AudioContext policy handling |

**Scale & Complexity:**

- Primary domain: Web SPA (Rust/WebAssembly)
- Complexity level: Medium — well-specified domain, non-trivial platform integration
- Estimated architectural components: ~12-15 (domain types, 2 session managers, profile, strategy, trend analyzer, timeline, audio adapter, storage adapter, settings adapter, UI views, composition root, service worker)

### Technical Constraints & Dependencies

| Constraint | Source | Impact |
|---|---|---|
| Rust compiled to WebAssembly | PRD hard constraint | All domain logic in Rust; UI framework must be Rust/WASM-native or interop cleanly |
| No backend logic | PRD hard constraint | All state management, audio, and persistence are browser-local |
| Independent from iOS codebase | PRD hard constraint | Reimplementation from blueprint, no shared code |
| Web Audio API for audio | PRD technology stack | Browser API with autoplay restrictions, tab suspension, varying latency |
| UI framework undecided | PRD open decision D1 | Leptos vs Dioxus — affects reactivity model, component patterns, ecosystem |
| Storage strategy undecided | PRD open decision D3 | IndexedDB vs localStorage — affects async model, data volume limits |
| Audio approach: oscillators first | PRD recommendation D2 | Simplifies Phase 1; SoundFont deferred |
| Browser compatibility (Chrome, Firefox, Safari, Edge) | PRD NFR13 | Must handle AudioContext policy differences |
| Learning Rust/WASM is primary goal | PRD G1 | Favor idiomatic Rust patterns over convenience shortcuts |

### Cross-Cutting Concerns Identified

1. **Audio context lifecycle** — AudioContext creation requires user gesture; suspension on tab hide; resumption on return. Affects both session state machines and the audio adapter. Must be handled consistently across comparison and pitch matching modes.

2. **Async model across WASM boundary** — Rust async (via `wasm-bindgen-futures`) coordinating with Web Audio API scheduling, browser timers, and UI events. The session state machines are inherently async (note playback, delays, user input waits).

3. **Reactivity and state observation** — The domain uses an observer pattern (fire-and-forget). The UI framework will use signals or similar reactive primitives. The bridge between domain observers and UI reactivity needs a clean pattern.

4. **WASM-JS interop overhead** — Every Web Audio API call crosses the WASM boundary. During pitch matching, `adjustFrequency` is called continuously. The interop cost must be negligible.

5. **Storage consistency** — Training records must persist atomically. Profile hydration replays all records on startup. IndexedDB is async; localStorage is sync but limited. The choice affects the entire persistence adapter.

6. **Accessibility (WCAG 2.1 AA)** — Keyboard navigation, screen reader announcements, focus management, contrast ratios, `prefers-reduced-motion`. Affects every UI component.

7. **Offline capability** — Service Worker registration and caching strategy. Must cache WASM binary + all static assets. Deferred to Phase 4 but influences deployment architecture from the start.

## Starter Template Evaluation

### Primary Technology Domain

Rust/WebAssembly client-side SPA — static deployment, no backend, all logic in browser.

### Starter Options Considered

**Option A: Leptos 0.8 CSR + Trunk** — Fine-grained reactive signals, no virtual DOM, web-focused, official CSR starter template via `cargo generate`. Standard Rust/WASM toolchain (Trunk bundler, wasm32-unknown-unknown target).

**Option B: Dioxus 0.7 Web + dx CLI** — Virtual DOM with signals, cross-platform framework used in web-only mode, integrated dx CLI with hot-patching. RSX syntax (JSX-like).

### Selected Starter: Leptos CSR + Trunk

**Rationale:**
- Fine-grained reactivity maps directly to the domain blueprint's observer pattern — signal changes update individual DOM nodes without virtual DOM diffing
- Web-only focus means no abstraction layer between Rust code and browser APIs (Web Audio, IndexedDB)
- Trunk is the standard Rust/WASM bundler — transferable knowledge beyond Leptos
- Larger web-focused Rust community with more examples and crate compatibility
- Closer to 1.0 stability (0.8 described as "final form")

**Initialization Command:**

```bash
cargo install trunk
cargo install cargo-generate
cargo generate --git https://github.com/leptos-rs/start-trunk
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:** Rust compiled to wasm32-unknown-unknown, running in browser

**Reactivity:** Leptos signal-based fine-grained reactivity (no virtual DOM)

**Build Tooling:** Trunk — compiles Rust to WASM, bundles assets, serves with live-reload

**Styling:** None pre-configured — CSS approach to be decided (minimal custom CSS per UX spec)

**Testing:** Standard Rust testing (`cargo test`) for domain logic; browser testing TBD

**Code Organization:** Leptos component-based structure with `view!` macros; domain logic in separate Rust modules

**Development Workflow:** `trunk serve --open` for dev server with auto-rebuild and browser reload

**Note:** Project initialization using this command should be the first implementation story.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Storage strategy (split: IndexedDB + localStorage)
- Async model (`wasm-bindgen-futures` + `gloo-timers`)
- Domain-UI bridge (UIObserver pattern)
- Project structure (two-crate workspace)
- Audio architecture (hybrid: oscillator fallback + OxiSynth SoundFont)

**Important Decisions (Shape Architecture):**
- Client-side routing (`leptos_router`)
- Styling (Tailwind CSS)
- Hosting (static files on Apache)
- Error handling (idiomatic Rust)

**Deferred Decisions:**
- CI/CD pipeline — future learning opportunity; automated local testing for now
- Data export/import format — Phase 5 (PRD D5)
- Service Worker caching strategy — Phase 4 (PRD D6)
- SoundFont file delivery — Phase 2 implementation detail

### Data Architecture

**Decision:** Split storage — IndexedDB for training records, localStorage for settings.

- **IndexedDB** implements `TrainingDataStore` port. Stores `PitchDiscriminationRecord` and `PitchMatchingRecord` in object stores with timestamp indexes. Async access via `web-sys` IndexedDB bindings. Handles 10k+ records with capacity to spare.
- **localStorage** implements `UserSettings` port. ~10 key-value pairs, JSON-serialized. Synchronous reads allow settings to be read live on every trial without async overhead, exactly as the domain blueprint specifies.
- **Rationale:** Two distinct access patterns, two distinct port interfaces, two adapters. IndexedDB's capacity handles growing record sets; localStorage's sync access matches the settings read pattern.

### Authentication & Security

**Not applicable.** No backend, no user accounts, no network communication after initial load. All data is browser-local.

### API & Communication Patterns

**Not applicable.** No server-side API. The only "communication" is WASM-JS interop within the browser, covered by the async model and audio architecture decisions.

### Frontend Architecture

**State Management:**
- Domain state lives in domain types (sessions, profile, strategy, trend analyzer) — owned by the domain crate
- A `UIObserver` implements the domain's observer protocols and bridges state changes to Leptos `RwSignal`s
- UI components read signals only — never mutate domain state directly
- User actions (answer, start/stop, settings change) call domain methods, which trigger observer notifications, which update signals
- **Rationale:** Preserves domain blueprint's ports-and-adapters architecture. Domain logic is testable with `cargo test` (no browser, no framework).

**Routing:**
- `leptos_router` with URL-based routes (`/`, `/training/pitch-discrimination`, `/training/pitch-matching`, `/profile`, `/settings`, `/info`)
- Hub-and-spoke: start page is root, all views one level deep
- **Rationale:** Enables bookmarkable training links. Future option to accept configuration via query parameters (e.g. `?tuning=just`).

**View Entry Paths (all must be handled):**

Any story touching navigation, audio init, or loading states must consider ALL entry paths:

| View | Entry Paths |
|---|---|
| Start Page (`/`) | App launch, back button from any view, direct URL |
| Pitch Discrimination (`/training/pitch-discrimination`) | Start Page card click, direct URL/bookmark, page refresh |
| Pitch Matching (`/training/pitch-matching`) | Start Page card click, direct URL/bookmark, page refresh |
| Profile (`/profile`) | Start Page icon, direct URL/bookmark, page refresh |
| Settings (`/settings`) | Start Page icon, direct URL/bookmark, page refresh |
| Info (`/info`) | Start Page icon, direct URL/bookmark, page refresh |

For training views, direct URL entry means no prior user gesture has occurred — AudioContext will be in `suspended` state. The `AudioGateOverlay` pattern handles this. For settings, the worklet bridge may be `None` (cold-start) if no training view has been visited yet.

**Styling:**
- Tailwind CSS integrated with Trunk build
- System font stack, `prefers-color-scheme` dark mode support via Tailwind's dark mode utilities
- Minimal custom CSS — Tailwind utilities cover the UX spec's spacing scale and color system

**Component Architecture:**
- Leptos `view!` macro components, one per view (StartPage, PitchDiscriminationView, PitchMatchingView, ProfileView, SettingsView, InfoView)
- Custom components: ProfileVisualization (Canvas/SVG), ProfilePreview, FeedbackIndicator, VerticalPitchSlider
- Components receive signals as props — no internal state management beyond local UI concerns

### Audio Architecture

**Decision:** Hybrid approach — oscillator fallback + OxiSynth SoundFont.

**Architecture:**
- `NotePlayer` trait in domain crate defines the port interface (play, stop, adjust frequency)
- `OscillatorNotePlayer` — simple Web Audio `OscillatorNode` + `GainNode` implementation. Fast to build, zero external dependencies. Serves as fallback while SoundFont loads and as resilient default if SoundFont loading fails.
- `SoundFontNotePlayer` — OxiSynth crate renders SoundFont presets into `AudioBuffer`s. Playback via `AudioBufferSourceNode`. Real-time pitch adjustment for pitch matching via `.detune` (±20 cents range, ~1.2% playback speed change — imperceptible).
- **SoundFont loading:** 31 MB file loaded async via `fetch()`, cached by browser. On first visit, oscillator fallback plays while SoundFont downloads. On subsequent visits, SoundFont loads from cache instantly.
- **Upgrade path:** If pre-rendered AudioBuffers prove limiting, upgrade to AudioWorklet streaming synthesis without changing the domain-side interface.

**Web Audio API access:** `web-sys` bindings directly from Rust. No JS glue code. All audio adapter code in the `web` crate.

### Web MIDI API (Progressive Enhancement)

**Decision:** Web MIDI API for hardware controller input — note-on events as rhythm tap input, pitch bend as pitch matching slider control. Progressive enhancement: MIDI is never required, and the app functions identically when MIDI is unavailable.

- **Feature detection:** `is_midi_available()` checks `Navigator.request_midi_access` existence before attempting access
- **Adapter:** `web/src/adapters/midi_input.rs` encapsulates all MIDI parsing and listener management
- **Event flow:** `midimessage` event → adapter parsing (`is_note_on`, `is_pitch_bend`, `parse_pitch_bend`) → existing tap/slider pipeline
- **Cleanup:** `MidiCleanupHandle` removes all `midimessage` listeners on drop/cleanup, stored in `StoredValue::new_local(SendWrapper::new(handle))`
- **Pitch bend commit:** return-to-center detection uses ±3.125% dead-zone (±1/32 of full range) to trigger answer commit
- **Failure handling:** MIDI setup failure → `log::warn!`, training continues with pointer/keyboard only. No error UI.
- **`web-sys` MIDI feature flags:** `MidiAccess`, `MidiInput`, `MidiInputMap`, `MidiMessageEvent`, `MidiOptions`, `MidiPort`, `MidiConnectionEvent`

### Async Model

**Decision:** `wasm-bindgen-futures` for spawning Rust futures on the browser microtask queue. `gloo-timers` for delays (note durations, feedback display).

- Session training loops expressed as async functions: `play_note().await`, `sleep(duration).await`, `wait_for_answer().await`
- Session cancellation via `Rc<Cell<bool>>` flag checked between await points, matching the domain blueprint's "IF cancelled or stopped: RETURN" pattern
- **Rationale:** Lightweight, standard Rust/WASM async. No heavy runtime (no tokio). Training loop code reads naturally as sequential logic.

### Project Structure

**Decision:** Cargo workspace with two crates.

```
peach-web/
├── Cargo.toml              (workspace)
├── domain/
│   ├── Cargo.toml          (pure Rust — no wasm, no leptos)
│   └── src/
│       ├── lib.rs
│       ├── types/           (MIDINote, Cents, Frequency, etc.)
│       ├── tuning.rs        (TuningSystem — the bridge)
│       ├── profile.rs       (PerceptualProfile, Welford's)
│       ├── strategy.rs      (KazezNoteStrategy)
│       ├── session/         (PitchDiscriminationSession, PitchMatchingSession)
│       ├── trend.rs         (TrendAnalyzer, ThresholdTimeline)
│       └── ports.rs         (NotePlayer, TrainingDataStore, etc.)
├── web/
│   ├── Cargo.toml          (leptos, web-sys, wasm-bindgen, gloo)
│   └── src/
│       ├── main.rs          (mount, composition root)
│       ├── adapters/        (AudioAdapter, IndexedDbStore, LocalStorageSettings)
│       ├── bridge.rs        (UIObserver — domain observers → Leptos signals)
│       ├── components/      (Leptos view components)
│       └── app.rs           (routing, top-level app component)
├── index.html              (Trunk entry point)
└── Trunk.toml
```

- **domain crate:** Pure Rust, no WASM or browser dependencies. Compiler enforces separation. Testable with native `cargo test`.
- **web crate:** Depends on domain crate. Contains all browser-specific code: Leptos UI, adapters, composition root.
- **Rationale:** The domain blueprint's two-world model (logical/physical) maps naturally to two crates. Compiler-enforced boundary prevents accidental coupling.

### Infrastructure & Deployment

**Hosting:** Static files served from Apache on personal website. `trunk build --release` produces a `dist/` folder; contents uploaded to web server.

**CI/CD:** Deferred. Automated local testing (`cargo test` for domain, manual browser testing for web) is sufficient initially. CI/CD to be added later as a learning opportunity.

**Offline support:** Service Worker caching deferred to Phase 4. Architecture does not preclude it — `trunk build` output is static and cacheable.

### Error Handling

**Decision:** Idiomatic Rust throughout. The domain blueprint's error semantics are respected, but expressed with Rust idioms rather than literal translation of Swift patterns.

- `Result<T, E>` for all fallible operations
- Custom error enums per domain area (e.g. `AudioError`, `StorageError`)
- `thiserror` or similar for ergonomic error type definitions
- `?` operator for error propagation
- Observers must not panic — errors caught internally and logged
- Storage write failures surface to the user (no silent data loss, per NFR8)

### Decision Impact Analysis

**Implementation Sequence:**
1. Workspace setup + Leptos CSR + Trunk + Tailwind (project scaffold)
2. Domain crate: value types, tuning system (pure Rust, testable immediately)
3. Domain crate: perceptual profile, Kazez strategy (pure Rust, testable)
4. Web crate: OscillatorNotePlayer adapter (first sound in browser)
5. Domain crate: PitchDiscriminationSession state machine
6. Web crate: UIObserver bridge, pitch discrimination training UI
7. Web crate: IndexedDB + localStorage adapters (persistence)
8. Web crate: SoundFontNotePlayer via OxiSynth (audio quality upgrade)
9. Domain crate: PitchMatchingSession, remaining features

**Cross-Component Dependencies:**
- UIObserver bridge depends on both domain observer traits and Leptos signals — central integration point
- Audio adapters depend on domain `NotePlayer` trait but not on UI
- Storage adapters depend on domain `TrainingDataStore` trait but not on UI
- Composition root in `main.rs` wires everything together per domain blueprint Section 11

## Implementation Patterns & Consistency Rules

### Critical Conflict Points Identified

7 areas where AI agents could make inconsistent choices if not specified.

### Naming Patterns

**Rust Code Naming (enforced by `clippy`):**

| Element | Convention | Example |
|---|---|---|
| Types, structs, enums | `PascalCase` | `MIDINote`, `PitchDiscriminationSession`, `TuningSystem` |
| Functions, methods | `snake_case` | `next_pitch_discrimination_trial()`, `handle_answer()` |
| Variables, fields | `snake_case` | `reference_note`, `sample_count` |
| Constants | `SCREAMING_SNAKE_CASE` | `FEEDBACK_DURATION`, `REFERENCE_MIDI_NOTE` |
| Modules, files | `snake_case` | `tuning.rs`, `pitch_matching.rs` |
| Trait names | `PascalCase` | `NotePlayer`, `PitchDiscriminationObserver` |
| Enum variants | `PascalCase` | `PlayingReferenceNote`, `AwaitingAnswer` |
| Type parameters | Single uppercase or `PascalCase` | `T`, `E` |

**Leptos Component Functions:**
- `PascalCase` — they're Rust functions but used as component tags in `view!` macros
- Example: `fn StartPage()`, `fn PitchDiscriminationView()`, `fn FeedbackIndicator()`

**Domain Type Names:**
- Use the exact names from the domain blueprint. No renaming, no abbreviation.
- `MIDINote`, not `MidiNote` or `Note`. `DetunedMIDINote`, not `PitchOffset`. `CompletedPitchDiscriminationTrial`, not `TrialResult`.
- **Rationale:** The blueprint is the shared language. If an agent reads "CompletedPitchDiscriminationTrial" in the blueprint, it should find `CompletedPitchDiscriminationTrial` in the code.

**Storage Keys:**

| Store | Key Convention | Example |
|---|---|---|
| localStorage | `peach.` prefix + `snake_case` | `peach.note_range_min`, `peach.tuning_system` |
| IndexedDB | Database name: `peach` | Object stores: `comparison_records` (legacy name), `pitch_matching_records` |

**Route Paths:**
- Lowercase, hyphenated: `/training/pitch-discrimination`, `/training/pitch-matching`, `/profile`, `/settings`, `/info`

### Structure Patterns

**Test Location:**
- Domain crate: `#[cfg(test)] mod tests` inline within each source file for unit tests. `domain/tests/` directory for integration tests that span multiple modules.
- Web crate: `#[cfg(test)] mod tests` inline where feasible. Browser-specific tests deferred until testing infrastructure is established.
- Test function naming: `test_` prefix + descriptive name. Example: `test_kazez_narrow_reduces_difficulty()`, `test_welford_mean_with_single_sample()`

**Module Organization:**
- One public type per file when the type is substantial (e.g. `profile.rs` for `PerceptualProfile`)
- Related small types can share a file (e.g. `types/mod.rs` re-exports from `types/midi.rs`, `types/cents.rs`, etc.)
- `mod.rs` used only for module directories, not as a dumping ground

**Adapter Organization:**
- One file per adapter: `adapters/audio_oscillator.rs`, `adapters/audio_soundfont.rs`, `adapters/indexeddb_store.rs`, `adapters/localstorage_settings.rs`
- Adapters implement domain port traits — the trait import is always from the `domain` crate

### Format Patterns

**Serialization:**
- `serde` + `serde_json` for all serialization (IndexedDB records, localStorage values, future export/import)
- Struct field names serialize as `snake_case` (serde default) — matches Rust field names
- Enum variants serialize as `camelCase` strings (serde `rename_all = "camelCase"`) — matches the domain blueprint's storage identifiers (e.g. `"equalTemperament"`)
- Timestamps: ISO 8601 strings (`"2026-03-02T14:30:00Z"`) — human-readable, sortable, compatible with future iOS interchange

**Numeric Precision:**
- All cent values, frequency values, and statistical accumulators use `f64` — no `f32` anywhere in domain calculations
- MIDI note values use `u8` (0-127 fits naturally)
- Sample counts use `u32` (sufficient for 10k+ records)

### Communication Patterns

**Observer Method Signatures:**
- Follow the domain blueprint exactly: `fn pitch_discrimination_completed(&mut self, completed: &CompletedPitchDiscriminationTrial)`
- Observers take `&CompletedPitchDiscriminationTrial` by reference (not owned) — the session owns the data, observers read it
- Observers must not return errors — any internal errors are logged via `web_sys::console::warn_1()`

**Leptos Signal Naming:**
- Signals that mirror domain state: `session_state`, `show_feedback`, `is_last_correct`
- Signals use `RwSignal` when both read and write are needed, `ReadSignal`/`WriteSignal` when access should be restricted
- Signal setter calls happen only inside the `UIObserver` bridge — components never call `.set()` on domain-related signals

**Component Props:**
- Components receive signals as props via `#[component]` function parameters
- No prop drilling beyond one level — if a deeply nested component needs data, it reads from a context provider

### Process Patterns

**Error Handling:**
- Domain crate: custom error enums with `thiserror`. Example: `AudioError`, `StorageError`
- Functions that can fail return `Result<T, E>` — never panic on recoverable errors
- `unwrap()` / `expect()` permitted only for invariants that are genuine programming errors (e.g. `MIDINote` construction with value > 127)
- Web crate: adapter errors converted to user-visible messages at the composition root / UI boundary level
- `anyhow` permitted in the web crate for ad-hoc error aggregation; not in the domain crate

**Loading & Initialization:**
- App startup sequence: mount Leptos → show loading state → hydrate profile from IndexedDB → begin SoundFont download → show start page
- SoundFont loading: non-blocking. Start page is interactive with oscillator fallback immediately. SoundFont swap happens silently when download completes.
- Loading states are Leptos signals: `is_profile_loaded: RwSignal<bool>`, `soundfont_status: RwSignal<SoundFontStatus>` (where `SoundFontStatus` is `Loading`, `Ready`, `Failed`)

**AudioContext Lifecycle:**
- Created on first training button click (user gesture requirement)
- Stored in a shared reference (`Rc<RefCell<AudioContext>>` or similar)
- Tab visibility change → stop active session, return to start page (per UX spec)
- AudioContext suspension detected via state change event → same behavior as tab hide

### Enforcement Guidelines

**All AI Agents MUST:**
- Use domain type names exactly as specified in the domain blueprint — no renaming
- Place domain logic in the `domain` crate with zero browser dependencies
- Implement port interfaces as Rust traits in the domain crate, with adapters in the web crate
- Use `Result<T, E>` for fallible operations — no silent failures, no unwrap on recoverable errors
- Write inline `#[cfg(test)]` unit tests for all domain logic
- Follow the observer pattern as specified — observers are injected, fire-and-forget, must not panic

**Pattern Enforcement:**
- `cargo clippy` catches naming violations (non-idiomatic Rust)
- Crate boundary enforces domain/web separation at compile time
- Code review (manual or via `/bmad-bmm-code-review`) verifies blueprint adherence

### Pattern Examples

**Good:**
```rust
// Domain crate — pure Rust, no browser deps
pub struct MIDINote { raw_value: u8 }  // Name matches blueprint
pub fn kazez_narrow(p: f64) -> f64 {   // Name matches blueprint
    p * (1.0 - 0.05 * p.sqrt())        // Formula matches blueprint
}
```

**Anti-Pattern:**
```rust
// DON'T: Rename domain types
pub struct Note { value: i32 }         // Wrong name, wrong type
// DON'T: Put browser code in domain crate
use web_sys::AudioContext;             // Compiler error — web-sys not in domain deps
// DON'T: Swallow errors silently
let _ = store.save(record);           // Silent failure violates NFR8
```

## Project Structure & Boundaries

### Complete Project Directory Structure

```
peach-web/
├── Cargo.toml                          # Workspace definition
├── index.html                          # Trunk entry point (loads WASM)
├── Trunk.toml                          # Trunk build configuration
├── tailwind.config.js                  # Tailwind CSS configuration
├── input.css                           # Tailwind directives (@tailwind base, etc.)
├── .gitignore
├── docs/                               # Project documentation (existing)
│   ├── planning-artifacts/
│   │   ├── prd.md
│   │   └── architecture.md
│   └── ios-reference/
│       ├── domain-blueprint.md
│       ├── peach-web-prd.md
│       └── peach-web-ux-spec.md
│
├── domain/                             # Pure Rust domain crate
│   ├── Cargo.toml                      # Dependencies: serde, thiserror, rand
│   ├── src/
│   │   ├── lib.rs                      # Public API: re-exports all domain types
│   │   │
│   │   ├── types/                      # Domain value types (Blueprint §2)
│   │   │   ├── mod.rs                  # Re-exports all types
│   │   │   ├── midi.rs                 # MIDINote, MIDIVelocity
│   │   │   ├── cents.rs               # Cents
│   │   │   ├── frequency.rs           # Frequency
│   │   │   ├── interval.rs            # Interval, Direction, DirectedInterval
│   │   │   ├── detuned.rs             # DetunedMIDINote
│   │   │   ├── duration.rs            # NoteDuration
│   │   │   ├── amplitude.rs           # AmplitudeDB, UnitInterval
│   │   │   └── sound_source.rs        # SoundSourceID
│   │   │
│   │   ├── tuning.rs                  # TuningSystem, frequency conversion (Blueprint §3)
│   │   │
│   │   ├── training/                  # Training domain entities (Blueprint §4)
│   │   │   ├── mod.rs
│   │   │   ├── pitch_discrimination.rs # PitchDiscriminationTrial, CompletedPitchDiscriminationTrial
│   │   │   └── pitch_matching.rs      # PitchMatchingTrial, CompletedPitchMatchingTrial
│   │   │
│   │   ├── profile.rs                 # PerceptualProfile, PerceptualNote, Welford's (Blueprint §5)
│   │   │
│   │   ├── strategy.rs               # KazezNoteStrategy, kazez_narrow/widen (Blueprint §6)
│   │   │
│   │   ├── session/                   # Session state machines (Blueprint §7)
│   │   │   ├── mod.rs
│   │   │   ├── pitch_discrimination_session.rs  # PitchDiscriminationSession + PitchDiscriminationSessionState
│   │   │   └── pitch_matching_session.rs  # PitchMatchingSession + states
│   │   │
│   │   ├── trend.rs                   # TrendAnalyzer (Blueprint §8.1)
│   │   │
│   │   ├── timeline.rs               # ThresholdTimeline (Blueprint §8.2)
│   │   │
│   │   ├── ports.rs                   # All port trait definitions (Blueprint §9)
│   │   │                              # NotePlayer, PlaybackHandle, TrainingDataStore,
│   │   │                              # UserSettings, SoundSourceProvider,
│   │   │                              # PitchDiscriminationObserver, PitchMatchingObserver, Resettable
│   │   │
│   │   ├── records.rs                 # PitchDiscriminationRecord, PitchMatchingRecord (Blueprint §10)
│   │   │
│   │   └── error.rs                   # Domain error types (AudioError, etc.)
│   │
│   └── tests/                         # Integration tests
│       ├── profile_hydration.rs       # Replay records → verify profile state
│       ├── strategy_convergence.rs    # Kazez algorithm convergence behavior
│       └── tuning_accuracy.rs         # Frequency conversion precision tests
│
├── web/                                # Browser-specific crate
│   ├── Cargo.toml                      # Dependencies: leptos, leptos_router, web-sys,
│   │                                   # wasm-bindgen, wasm-bindgen-futures, gloo-timers,
│   │                                   # serde, serde_json, oxisynth, domain (path dep)
│   ├── src/
│   │   ├── main.rs                    # Leptos mount, composition root (Blueprint §11)
│   │   │
│   │   ├── app.rs                     # Top-level App component, router setup
│   │   │
│   │   ├── adapters/                  # Port implementations
│   │   │   ├── mod.rs
│   │   │   ├── audio_oscillator.rs    # OscillatorNotePlayer (Web Audio OscillatorNode)
│   │   │   ├── audio_soundfont.rs     # SoundFontNotePlayer (OxiSynth + AudioBuffer)
│   │   │   ├── audio_context.rs       # AudioContext lifecycle management
│   │   │   ├── indexeddb_store.rs     # IndexedDB TrainingDataStore implementation
│   │   │   ├── localstorage_settings.rs  # localStorage UserSettings implementation
│   │   │   ├── default_settings.rs   # Default UserSettings values
│   │   │   ├── note_player.rs        # Unified NotePlayer facade (oscillator + SoundFont)
│   │   │   ├── sound_preview.rs      # Sound source preview playback
│   │   │   ├── audio_latency.rs      # AudioContext output latency helpers
│   │   │   ├── rhythm_scheduler.rs   # Click track scheduling for rhythm training
│   │   │   └── midi_input.rs         # Web MIDI API adapter (note-on, pitch bend, feature detection)
│   │   │
│   │   ├── bridge.rs                  # UIObserver: domain observers → Leptos signals
│   │   │
│   │   ├── components/                # Leptos UI components
│   │   │   ├── mod.rs
│   │   │   ├── start_page.rs          # StartPage: training buttons, profile preview
│   │   │   ├── pitch_discrimination_view.rs  # PitchDiscriminationView: higher/lower, feedback
│   │   │   ├── pitch_matching_view.rs # PitchMatchingView: slider, feedback
│   │   │   ├── profile_view.rs        # ProfileView: full visualization, stats
│   │   │   ├── settings_view.rs       # SettingsView: all training settings
│   │   │   ├── info_view.rs           # InfoView: app info
│   │   │   ├── profile_visualization.rs  # Canvas/SVG piano keyboard + confidence band
│   │   │   ├── profile_preview.rs     # Compact clickable profile miniature
│   │   │   ├── feedback_indicator.rs  # Thumbs up/down (comparison) + arrow/cents (matching)
│   │   │   ├── pitch_slider.rs        # Vertical pitch adjustment slider
│   │   │   ├── continuous_rhythm_matching_view.rs  # ContinuousRhythmMatchingView: rhythm tap training
│   │   │   ├── rhythm_offset_detection_view.rs  # RhythmOffsetDetectionView: offset measurement
│   │   │   ├── progress_sparkline.rs  # Compact inline sparkline charts
│   │   │   ├── training_stats.rs      # Training statistics display
│   │   │   └── help_content.rs        # HelpContent: context-aware help overlay
│   │   │
│   │   └── signals.rs                 # Shared signal type definitions and context providers
│   │
│   └── assets/                        # Static assets
│       └── soundfont/                 # SoundFont .sf2 file(s)
│
└── dist/                               # Build output (git-ignored)
```

### Architectural Boundaries

**Crate Boundary (compiler-enforced):**

```
┌─────────────────────────────────┐
│         domain crate            │
│  Pure Rust. No browser deps.    │
│  Defines: types, traits, logic  │
│  Tests: cargo test (native)     │
├─────────────────────────────────┤
│         Port Traits             │
│  NotePlayer, TrainingDataStore, │
│  UserSettings, Observers        │
└──────────────┬──────────────────┘
               │ implements
┌──────────────▼──────────────────┐
│          web crate              │
│  Leptos + web-sys + adapters    │
│  Implements port traits         │
│  Owns: UI, audio, storage      │
│  Tests: wasm-pack test          │
└─────────────────────────────────┘
```

**Data Flow:**

```
User Input (click/key)
    → Leptos event handler
    → domain session method (handle_answer, adjust_pitch)
    → session broadcasts to observers
    → UIObserver writes Leptos signals
    → Leptos re-renders affected DOM nodes

    → TrainingDataStore observer persists record to IndexedDB
    → PerceptualProfile observer updates statistics
    → TrendAnalyzer observer updates trend
```

**Audio Flow:**

```
Session calls NotePlayer.play(frequency, duration, velocity, amplitude)
    → OscillatorNotePlayer: creates OscillatorNode + GainNode → destination
    → SoundFontNotePlayer: OxiSynth renders AudioBuffer → AudioBufferSourceNode → destination
    ← returns PlaybackHandle (stop, adjust_frequency)
```

**Storage Boundaries:**

```
IndexedDB "peach"
├── comparison_records    (object store, legacy name, timestamp index)
└── pitch_matching_records (object store, timestamp index)

localStorage
├── peach.note_range_min
├── peach.note_range_max
├── peach.note_duration
├── peach.reference_pitch
├── peach.sound_source
├── peach.vary_loudness
├── peach.intervals        (JSON array)
├── peach.tuning_system
└── peach.version          (for future migration)
```

### Requirements to Structure Mapping

**FR → File Mapping:**

| FR Category | Domain Crate | Web Crate |
|---|---|---|
| Pitch Discrimination Training (FR1-8) | `session/pitch_discrimination_session.rs`, `strategy.rs`, `training/pitch_discrimination.rs` | `components/pitch_discrimination_view.rs`, `bridge.rs` |
| Pitch Matching (FR9-15) | `session/pitch_matching_session.rs`, `training/pitch_matching.rs` | `components/pitch_matching_view.rs`, `components/pitch_slider.rs` |
| Perceptual Profile (FR16-21) | `profile.rs` | `components/profile_view.rs`, `components/profile_visualization.rs`, `components/profile_preview.rs` |
| Settings (FR22-30) | `ports.rs` (UserSettings trait) | `components/settings_view.rs`, `adapters/localstorage_settings.rs` |
| Audio Engine (FR31-35) | `ports.rs` (NotePlayer, PlaybackHandle traits) | `adapters/audio_oscillator.rs`, `adapters/audio_soundfont.rs`, `adapters/audio_context.rs` |
| Data Persistence (FR36-40) | `ports.rs` (TrainingDataStore trait), `records.rs` | `adapters/indexeddb_store.rs` |
| Input & Accessibility (FR41-46) | — | `components/pitch_discrimination_view.rs`, `components/pitch_matching_view.rs` (keyboard handlers, ARIA) |
| MIDI Input (FR50-52) | — | `adapters/midi_input.rs`, `components/continuous_rhythm_matching_view.rs`, `components/pitch_matching_view.rs` |
| Navigation (FR47-49) | — | `app.rs` (router), all component files |

**Domain Blueprint § → File Mapping:**

| Blueprint Section | File |
|---|---|
| §2 Domain Value Types | `types/*.rs` |
| §3 Tuning System | `tuning.rs` |
| §4 Training Entities | `training/*.rs` |
| §5 Perceptual Profile | `profile.rs` |
| §6 Adaptive Algorithm | `strategy.rs` |
| §7 Session State Machines | `session/*.rs` |
| §8 Trend Analysis | `trend.rs`, `timeline.rs` |
| §9 Port Interfaces | `ports.rs` |
| §10 Persistence Schemas | `records.rs` |
| §11 Composition Rules | `web/src/main.rs` |

### Development Workflow

**Development:**
```bash
cd peach-web
trunk serve --open          # Starts dev server, opens browser, watches for changes
cargo test -p domain        # Run domain tests (native, fast)
```

**Production Build:**
```bash
trunk build --release       # Outputs to dist/
# Upload dist/ contents to Apache
```

**Build Pipeline:**
```
Trunk reads index.html
  → Compiles web crate to WASM (which depends on domain crate)
  → Processes Tailwind CSS (input.css → output)
  → Copies static assets
  → Outputs: dist/index.html, dist/*.wasm, dist/*.js, dist/*.css
```

## Architecture Validation Results

### Coherence Validation

**Decision Compatibility:** All technology choices are mutually compatible. Leptos 0.8 + Trunk + Tailwind + `wasm-bindgen-futures` + `gloo-timers` + `web-sys` + OxiSynth form a coherent, standard Rust/WASM stack. No version conflicts, no contradictory patterns.

**Pattern Consistency:** All patterns align with Rust idioms (compiler-enforced via `clippy`). Domain type names match the blueprint exactly. Serialization conventions are uniform. Signal naming follows a consistent scheme.

**Structure Alignment:** Two-crate workspace directly reflects the blueprint's ports-and-adapters architecture. Every blueprint section maps to a specific domain crate file. Every FR maps to specific files in both crates.

### Requirements Coverage Validation

**Functional Requirements:** All 49 FRs have architectural support. FR36-40 (export/import) partially deferred to Phase 5 per PRD.

**Non-Functional Requirements:** All 15 NFRs addressed. Offline capability (NFR10-12) deferred to Phase 4 per PRD; architecture does not preclude it.

### Implementation Readiness Validation

**Decision Completeness:** All critical decisions documented with versions and rationale. Deferred decisions listed with explicit timeline.

**Structure Completeness:** Complete directory tree with every file specified. Blueprint-to-file and FR-to-file mappings provided.

**Pattern Completeness:** Naming, structure, format, communication, and process patterns defined with examples and anti-patterns.

### Gap Analysis Results

**Critical gaps:** None.

**Implementation notes for agents:**
1. Sessions require `Rc<RefCell<...>>` for shared ownership between async training loop and UI event handlers — standard Rust/WASM interior mutability pattern
2. Observer injection can use trait objects (`Box<dyn PitchDiscriminationObserver>`) or enum dispatch — either is architecturally consistent; decide during implementation
3. Trunk + Tailwind requires a build hook in `Trunk.toml` — detail for the project scaffold story

### Architecture Completeness Checklist

**Requirements Analysis**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed (medium)
- [x] Technical constraints identified (Rust/WASM, no backend, browser-local)
- [x] Cross-cutting concerns mapped (7 concerns)

**Architectural Decisions**
- [x] Critical decisions documented with versions (8 decisions)
- [x] Technology stack fully specified (Leptos 0.8 + Trunk + Tailwind + web-sys + OxiSynth)
- [x] Integration patterns defined (UIObserver bridge, adapter pattern)
- [x] Performance considerations addressed (all 5 performance NFRs)

**Implementation Patterns**
- [x] Naming conventions established (Rust idioms + blueprint fidelity)
- [x] Structure patterns defined (test location, module organization, adapter organization)
- [x] Communication patterns specified (observer signatures, signal naming, component props)
- [x] Process patterns documented (error handling, loading states, AudioContext lifecycle)

**Project Structure**
- [x] Complete directory structure defined
- [x] Component boundaries established (domain crate / web crate, compiler-enforced)
- [x] Integration points mapped (data flow, audio flow, storage boundaries)
- [x] Requirements to structure mapping complete (49 FRs + 12 blueprint sections)

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High — the domain blueprint eliminates the usual ambiguity in what to build. Architectural decisions focus on how to build it on the web platform, and all choices are standard, well-documented Rust/WASM patterns.

**Key Strengths:**
- Domain blueprint provides exceptional specification clarity — no ambiguity in domain logic
- Compiler-enforced crate boundary prevents architectural erosion
- Fine-grained Leptos signals + observer bridge give optimal reactivity without coupling
- Hybrid audio approach (oscillator fallback + SoundFont) provides resilience and quality
- Split storage matches the two distinct data access patterns perfectly

**Areas for Future Enhancement:**
- CI/CD pipeline (deferred — learning opportunity)
- Service Worker for offline support (Phase 4)
- SoundFont upgrade to AudioWorklet streaming if pre-rendered buffers prove limiting
- Data export/import format definition (Phase 5)

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all components
- Respect the domain/web crate boundary — domain crate must have zero browser dependencies
- Use domain type names from the blueprint verbatim — no renaming
- Refer to this document for all architectural questions

**First Implementation Priority:**
Project scaffold — Cargo workspace, Leptos CSR + Trunk + Tailwind, domain crate with `lib.rs`, web crate with `main.rs`, verify `trunk serve` runs successfully.
