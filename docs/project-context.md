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
- Leptos 0.8 requires `Send + Sync` for `provide_context`, `on_cleanup`, and all closures captured by reactive primitives (signals, effects, view macros). Wrap `Rc<RefCell<T>>` in `SendWrapper` at the definition site — this is safe because WASM is single-threaded. Example: `let player = SendWrapper::new(Rc::new(RefCell::new(None::<Player>)));`

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
- Context shadowing: `use_context` is type-based. Multiple `RwSignal<bool>` contexts shadow each other — only the last `provide_context` call wins. Always wrap in newtypes (e.g. `struct IsProfileLoaded(pub RwSignal<bool>)`)
- Navigation: always use `leptos_router::components::A` for internal links, never raw `<a>`. Raw `<a>` breaks client-side routing and user-gesture propagation for Web Audio

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
- `cargo fmt` — format code before every commit (CI enforces `cargo fmt --check`)
- `cargo clippy` — lint check before committing

**Project Structure:**

- Cargo workspace with three crates: `domain/` (pure Rust), `web/` (Leptos + browser), and `synth-worklet/` (WASM AudioWorklet)
- `index.html` at project root — Trunk entry point
- `Trunk.toml` — build pipeline configuration
- `input.css` — Tailwind directives, processed via Trunk build hook
- `dist/` — build output, git-ignored

**Deployment:**

- GitHub Pages at `https://mschuerig.github.io/peach-web/` — deployed automatically via CI on push to `main`
- CI pipeline (`.github/workflows/ci.yml`): `quality-gate` job (fmt, clippy, tests) → `build-deploy` job (Trunk build + deploy to Pages)
- `trunk build --release --public-url /peach-web/` — the `--public-url` flag prefixes asset URLs but does NOT auto-insert a `<base>` tag (see Subpath Deployment below)
- Manual browser testing for web crate behavior

**Subpath Deployment (GitHub Pages):**

- The app is served from `/peach-web/`, not root `/`. This requires three things to work:
  1. `<base data-trunk-public-url/>` in `index.html` — Trunk replaces this with `<base href="/peach-web/">` at build time. Without it, the `<base>` tag is missing entirely. During local dev (`trunk serve`), it becomes `<base href="/">`.
  2. `<Router base=base_path()>` in `app.rs` — reads the `<base href>` from the DOM and passes it to the Leptos Router so it strips the `/peach-web` prefix before matching routes. Without this, the router sees `/peach-web/` and falls through to the fallback.
  3. `cp dist/index.html dist/404.html` after build — GitHub Pages serves `404.html` for unknown paths, enabling SPA deep-link routing (e.g. `/peach-web/profile`).
- All runtime asset fetches in Rust code must use relative paths (`./soundfont/...`, `./GeneralUser-GS.sf2`), never root-absolute (`/soundfont/...`). Trunk only rewrites paths in HTML, not in compiled WASM. Relative paths resolve against the `<base href>` and work correctly under any base path.
- `AudioWorklet.addModule()` may not respect `<base href>` — resolve to absolute URL via `document.base_uri()` before calling it.
- Leptos Router 0.8 `<A href="/path">` does NOT prepend the base for `/`-prefixed hrefs. All `<A>` hrefs, `NavBar back_href`, `NavIconButton href`, and raw `<a href>` must use `base_href("/path")` (from `app.rs`) which reads the `BasePath` context.
- **Exception:** `navigate()` (from `use_navigate()`) DOES resolve against the base internally (via `resolve_path`). Do NOT use `base_href()` with `navigate()` — it would double-prefix. Use plain paths: `navigate("/", Default::default())`.

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
- SoundFont load failure → fall back to oscillators with brief user notification
- Two-phase worklet init: Phase 1 (fetch assets at app mount, no AudioContext) and Phase 2 (connect worklet at training start, after `ensure_running()`). The worklet bridge is `None` until Phase 2 completes — any code using SF2 playback must handle the cold-start case
- User-gesture chain: switching from `<button onclick>` to `<A href>` breaks gesture propagation. Any navigation that leads to AudioContext creation must preserve the gesture chain
- Entry paths: users can reach training views via Start Page click, direct URL, bookmark, or page refresh. All paths must handle AudioContext gesture requirements (the `AudioGateOverlay` pattern)

**Disabled Interactive Elements:**

- Disabled `<a>` elements: remove the `href` attribute entirely and set `tabindex="-1"`. CSS-only disabling or `prevent_default` on click is insufficient — middle-click and right-click bypass JS handlers
- Disabled buttons: use the native `disabled` attribute

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

## Common Pitfalls (Epic 8 Retrospective)

These patterns caused repeated failures. Check this section before implementing any story.

| Pitfall | What Goes Wrong | Fix |
|---|---|---|
| Raw `Rc<RefCell<T>>` in Leptos closures | Compilation fails with `Send + Sync` error | Wrap in `SendWrapper` at definition site, not at clone site |
| Multiple `RwSignal<bool>` in context | `use_context` returns the wrong signal (type-based lookup) | Use newtypes: `struct IsProfileLoaded(pub RwSignal<bool>)` |
| Raw `<a>` for internal navigation | Breaks client-side routing and user-gesture chain for Web Audio | Use `leptos_router::components::A` |
| Disabled `<a>` with `href` still set | Users can navigate via middle-click, right-click, or keyboard | Remove `href` and set `tabindex="-1"` when disabled |
| `<button>` to `<A href>` migration | Breaks user-gesture propagation for AudioContext | Audit all paths that lead to AudioContext creation |
| Worklet bridge assumed available | Bridge is `None` until first training view connects it | Check for `None` and connect on-demand (cold-start pattern) |
| Guard logic created but not wired | Enum variant exists but fetch/action runs unconditionally | After creating a guard enum, grep for every call site and add the guard check |
| Story tasks taken literally | Dev Notes say "no changes needed" but edge cases exist | Always ask: what are ALL entry paths? What state can signals be in? |
| Root-absolute asset paths in Rust | Fetch calls like `/soundfont/foo.wasm` 404 on subpath deployments | Use relative paths (`./soundfont/foo.wasm`) — they resolve against `<base href>` |
| Missing `<base data-trunk-public-url/>` | Trunk `--public-url` rewrites HTML asset URLs but does NOT insert a `<base>` tag | Add the tag to `index.html` — without it, router base and relative fetches break |
| No `404.html` for GitHub Pages SPA | Direct navigation to deep routes (e.g. `/peach-web/profile`) returns GitHub's 404 | Copy `index.html` to `404.html` in build output |
| `<A href="/path">` ignores router base | Leptos Router 0.8 returns `/`-prefixed paths as-is without prepending base | Use `base_href("/path")` for `<A>` hrefs, but NOT for `navigate()` which resolves internally |
| `addModule()` ignores `<base href>` | AudioWorklet module URL resolves against document URL, not `<base>` | Resolve to absolute URL via `document.base_uri()` before calling `addModule()` |

## Implementation Edge-Case Checklist

Before writing ANY code for a story task, answer these questions:

1. What are ALL entry paths to this code? (start page click, direct URL, bookmark, back button, page refresh)
2. What state can each signal/resource be in when this code runs? (None, Loading, Ready, Failed, stale)
3. What happens if an async operation is still in progress when this code executes?
4. What happens if the user navigates away mid-operation?
5. Does this change break any user-gesture chains required by Web Audio or other browser APIs?
6. If a guard enum/status is created, are ALL call sites checked? Grep every usage.

Challenge story Dev Notes: if they say "no changes needed" for a module, verify this is actually true for all entry paths. Check MEMORY.md and the Common Pitfalls table above for known patterns that apply.

**Verification honesty:** NEVER mark a manual testing task as complete unless you actually performed the test. If you cannot run the browser, mark the task as UNCHECKED and note "deferred to user — agent cannot verify in browser."

## Code Review: Feed Back Patterns

After completing a code review, check whether any HIGH or MEDIUM finding reflects a repeatable pattern (not a one-off typo). Ask: "Would a different agent making a different change hit this same problem?" If yes, and the pattern is not already documented, add it to the Common Pitfalls table above.

## Debugging Protocol

When a runtime bug appears during implementation, follow this protocol in order:

1. **Check MEMORY.md first.** The bug pattern may already be documented.
2. **Isolate your changes.** `git stash` and test the original code. If the bug exists without your changes, it is pre-existing — document it and move on. If it disappears, your changes caused it.
3. **Form a hypothesis tree.** List all possible causes, then eliminate them systematically. Do not guess sequentially.
4. **Binary search.** If you changed multiple files, revert half and test. Narrow down to the specific change that introduced the bug.
5. **3-attempt limit.** If you cannot resolve the bug within 3 focused attempts, stop and ask the user for diagnostic help. Do not flail for 20+ rounds.

Never blame caching, console filters, or pre-existing conditions without evidence. Never mark a hypothesis as "CONFIRMED" from code reading alone — reproduce and observe.

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
