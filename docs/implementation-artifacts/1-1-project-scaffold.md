# Story 1.1: Project Scaffold

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want a working Cargo workspace with Leptos CSR, Trunk, and Tailwind CSS configured,
so that I have a solid foundation to build the application on.

## Acceptance Criteria

1. **Given** a fresh checkout of the repository, **When** I run `trunk serve`, **Then** the application compiles and serves a page at localhost, **And** the page displays "Peach" as a heading.

2. **Given** the workspace is set up, **When** I inspect the project structure, **Then** there is a `domain` crate with `src/lib.rs` and zero browser dependencies in Cargo.toml, **And** there is a `web` crate with `src/main.rs` that depends on the domain crate, **And** there is a workspace-level `Cargo.toml`.

3. **Given** Tailwind CSS is configured, **When** the app is built with `trunk build`, **Then** Tailwind utility classes are processed and included in the output CSS.

4. **Given** the project builds, **When** I run `cargo test -p domain`, **Then** the domain crate compiles and tests pass (even if empty), **And** the domain crate has no `web-sys`, `wasm-bindgen`, or `leptos` dependencies.

5. **Given** the `index.html` entry point, **When** I inspect it, **Then** it includes `<meta name="viewport" content="width=device-width, initial-scale=1">`.

## Tasks / Subtasks

- [x] Task 1: Install toolchain prerequisites (AC: #1, #4)
  - [x] 1.1 Install Trunk: `cargo install trunk`
  - [x] 1.2 Add WASM target: `rustup target add wasm32-unknown-unknown`
- [x] Task 2: Create Cargo workspace with two crates (AC: #2)
  - [x] 2.1 Create root `Cargo.toml` with workspace members `["domain", "web"]` and `resolver = "2"`
  - [x] 2.2 Create `domain/Cargo.toml` — pure Rust crate, dependencies: `serde` (with derive feature), `thiserror`, `rand`
  - [x] 2.3 Create `domain/src/lib.rs` — empty module with a placeholder test
  - [x] 2.4 Create `web/Cargo.toml` — depends on `leptos = { version = "0.8", features = ["csr"] }`, `leptos_router = "0.8"`, `console_log`, `console_error_panic_hook`, and `domain = { path = "../domain" }`
  - [x] 2.5 Create `web/src/main.rs` — Leptos mount point that renders a "Peach" heading
- [x] Task 3: Configure Trunk build pipeline (AC: #1)
  - [x] 3.1 Create `Trunk.toml` at project root with `[tools] tailwindcss = "4"` and build target pointing to `index.html`
  - [x] 3.2 Create `index.html` at project root with `data-trunk` attributes, viewport meta tag, and `<link data-trunk rel="tailwind-css" href="input.css" />`
- [x] Task 4: Configure Tailwind CSS (AC: #3)
  - [x] 4.1 Create `input.css` with Tailwind v4 directives: `@import 'tailwindcss';` and `@source` pointing to Rust source files
  - [x] 4.2 Apply a Tailwind utility class to the "Peach" heading to verify CSS processing works
- [x] Task 5: Add project scaffolding files (AC: #1)
  - [x] 5.1 Create `.gitignore` with `dist/`, `target/`, and OS-specific entries
  - [x] 5.2 Create `rust-toolchain.toml` specifying stable toolchain
- [x] Task 6: Verify all acceptance criteria (AC: #1-#5)
  - [x] 6.1 Run `trunk serve` — confirm page loads with "Peach" heading
  - [x] 6.2 Run `trunk build` — confirm Tailwind classes are in output CSS
  - [x] 6.3 Run `cargo test -p domain` — confirm domain crate compiles and tests pass natively
  - [x] 6.4 Inspect `domain/Cargo.toml` — confirm zero browser dependencies
  - [x] 6.5 Inspect `index.html` — confirm viewport meta tag present

## Dev Notes

### Architecture Compliance

**Crate Separation (compiler-enforced, non-negotiable):**
- `domain` crate: Pure Rust. Zero browser dependencies. No `web-sys`, `wasm-bindgen`, or `leptos` in Cargo.toml. Must compile and pass tests with native `cargo test -p domain`.
- `web` crate: Depends on `domain` via `path = "../domain"`. All browser-specific code lives here.
- The compiler enforces this boundary — if someone adds `web-sys` to the domain crate, it will fail native `cargo test`.

**Project Structure (from architecture document):**
```
peach-web/
├── Cargo.toml              # Workspace definition
├── index.html              # Trunk entry point
├── Trunk.toml              # Build pipeline config
├── input.css               # Tailwind directives
├── .gitignore
├── rust-toolchain.toml
├── domain/
│   ├── Cargo.toml          # Pure Rust — no WASM deps
│   └── src/
│       └── lib.rs
├── web/
│   ├── Cargo.toml          # Leptos + web-sys + adapters
│   └── src/
│       └── main.rs         # Leptos mount, composition root
└── dist/                   # Build output (git-ignored)
```
[Source: docs/planning-artifacts/architecture.md — Project Structure section]

### Technology Versions

| Technology | Version | Notes |
|---|---|---|
| Rust | stable (latest) | Via `rust-toolchain.toml` |
| Leptos | 0.8 | CSR mode (`features = ["csr"]`). Latest patch: 0.8.16 |
| leptos_router | 0.8 | Must match Leptos version exactly |
| Trunk | latest (0.21+) | WASM bundler with built-in Tailwind support |
| Tailwind CSS | 4.x | Via Trunk's `[tools]` section — no npm needed |
| wasm-bindgen | 0.2 | Dev dependency only at this stage |
| Compile target | wasm32-unknown-unknown | Added via `rustup target add` |

[Source: docs/project-context.md — Technology Stack & Versions]

### Tailwind CSS v4 Setup (No npm Required)

Trunk has built-in Tailwind CSS support via the `tailwind-css` asset type. Configuration:

**`index.html`** — use `data-trunk` attributes:
```html
<link data-trunk rel="tailwind-css" href="input.css" />
```

**`input.css`** — Tailwind v4 directives (NOT v3 `@tailwind` directives):
```css
@import 'tailwindcss';

@source "./web/src/**/*.rs";
@source "./domain/src/**/*.rs";
```

**`Trunk.toml`** — pin Tailwind version:
```toml
[tools]
tailwindcss = "4"
```

Trunk auto-downloads the Tailwind standalone CLI binary. No Node.js, no npm, no `tailwind.config.js` needed with v4.

**Important:** The architecture doc mentions `tailwind.config.js` — this is a v3 pattern. With Tailwind v4, configuration moves into the CSS file itself. If the team decides to use Tailwind v3 instead, use `@tailwind base; @tailwind components; @tailwind utilities;` in `input.css` and add a `tailwind.config.js` with `content: ["./web/src/**/*.rs"]`.

### Cargo Workspace Configuration

**Root `Cargo.toml`:**
```toml
[workspace]
members = ["domain", "web"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }

[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
```

**`domain/Cargo.toml`:**
```toml
[package]
name = "domain"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
thiserror = "2"
rand = "0.9"
```

**`web/Cargo.toml`:**
```toml
[package]
name = "web"
version = "0.1.0"
edition = "2024"

[dependencies]
domain = { path = "../domain" }
leptos = { version = "0.8", features = ["csr"] }
leptos_router = "0.8"
console_log = "1"
log = "0.4"
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-test = "0.3"
```

**Do NOT add** `web-sys`, `wasm-bindgen`, `wasm-bindgen-futures`, `gloo-timers`, `oxisynth`, or `serde_json` to the web crate yet — those are needed by later stories. Only add what this story requires.

[Source: docs/planning-artifacts/architecture.md — Project Structure; web research on Leptos 0.8 starter template]

### Leptos CSR Minimal Setup

**`web/src/main.rs`:**
```rust
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <main class="flex min-h-screen items-center justify-center">
            <h1 class="text-4xl font-bold">"Peach"</h1>
        </main>
    }
}
```

This satisfies AC #1 (displays "Peach" as a heading) and AC #3 (uses Tailwind utility classes to verify CSS processing). Routing setup is deferred to Story 1.4 (App Shell & Routing).

[Source: docs/planning-artifacts/epics.md — Story 1.1 acceptance criteria]

### `index.html` Template

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Peach</title>
    <link data-trunk rel="tailwind-css" href="input.css" />
    <link data-trunk rel="rust" data-wasm-opt="z" />
  </head>
  <body></body>
</html>
```

The `<link data-trunk rel="rust" ...>` tells Trunk to compile the Rust crate to WASM. The `data-wasm-opt="z"` enables size optimization in release builds.

[Source: Leptos start-trunk template; docs/planning-artifacts/architecture.md]

### Anti-Patterns — Do NOT

- Do NOT use `cargo generate` from the Leptos starter template — we are manually creating the workspace structure because the template generates a single-crate project, not a workspace. Extract patterns from the template but build the workspace by hand.
- Do NOT add dependencies for later stories (web-sys features, IndexedDB, audio APIs, etc.) — only add what is needed to satisfy this story's acceptance criteria.
- Do NOT add routing, multiple views, or the UIObserver bridge — that is Story 1.4.
- Do NOT add any domain types or logic — that is Story 1.2.
- Do NOT install Node.js or npm for Tailwind — Trunk handles it.

### Build & Verification Commands

```bash
# Development
trunk serve --open          # Dev server with auto-rebuild, opens browser

# Testing
cargo test -p domain        # Domain tests run natively (fast)

# Production build
trunk build --release        # Outputs to dist/

# Lint check (domain crate only — web crate is linted during trunk build)
cargo clippy -p domain       # Run before committing
```

[Source: docs/project-context.md — Development Workflow Rules]

### Project Structure Notes

- This story establishes the foundational directory structure that all subsequent stories build on
- The `domain/` and `web/` crates at the root level (not inside a `crates/` or `src/` subdirectory) matches the architecture document
- `dist/` is the build output directory and must be git-ignored
- `target/` is the Cargo build directory and must be git-ignored

### References

- [Source: docs/planning-artifacts/architecture.md — Starter Template Evaluation, Project Structure, Frontend Architecture, Development Workflow]
- [Source: docs/planning-artifacts/epics.md — Epic 1: Core Comparison Training, Story 1.1: Project Scaffold]
- [Source: docs/planning-artifacts/prd.md — Phase 1 Foundation, Technical Constraints]
- [Source: docs/planning-artifacts/ux-design-specification.md — Responsive Design, Accessibility, Visual Design Foundation]
- [Source: docs/project-context.md — Technology Stack, Development Workflow Rules, Critical Implementation Rules]
- [Source: docs/ios-reference/domain-blueprint.md — Two-World Model, Ports and Adapters, Composition Root]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Trunk 0.20.2 did not support workspace builds; upgraded to 0.21.14
- Trunk `<link data-trunk rel="rust">` requires `href="web"` to locate the web crate in a workspace (not `data-cargo-package`)
- `getrandom` 0.3.x requires explicit WASM backend config: `.cargo/config.toml` with `--cfg getrandom_backend="wasm_js"` and `getrandom = { version = "0.3", features = ["wasm_js"] }` in web crate
- Trunk.toml `tailwindcss = "4"` caused 404; must specify exact version (e.g., `"4.2.1"`)

### Completion Notes List

- Created Cargo workspace with `domain` (pure Rust) and `web` (Leptos CSR) crates
- Domain crate has zero browser dependencies; compiles and passes tests natively with `cargo test -p domain`
- Web crate renders "Peach" heading with Tailwind CSS utility classes
- Trunk 0.21.14 configured with Tailwind CSS v4.2.1 standalone CLI (no npm)
- All 5 acceptance criteria verified and passing
- Added `.cargo/config.toml` for WASM getrandom backend (required by `rand` 0.9 in domain crate when compiled as WASM dependency)

### Change Log

- 2026-03-03: Initial implementation — project scaffold with Cargo workspace, Leptos CSR, Trunk, and Tailwind CSS v4

### File List

- `Cargo.toml` (new) — workspace definition
- `Cargo.lock` (new) — dependency lock file
- `Trunk.toml` (new) — Trunk build pipeline config
- `index.html` (new) — Trunk entry point with viewport meta tag
- `input.css` (new) — Tailwind v4 directives
- `.gitignore` (new) — dist/, target/, OS files
- `rust-toolchain.toml` (new) — stable Rust toolchain
- `.cargo/config.toml` (new) — WASM getrandom backend config
- `domain/Cargo.toml` (new) — pure Rust domain crate
- `domain/src/lib.rs` (new) — empty module with placeholder test
- `web/Cargo.toml` (new) — Leptos CSR web crate
- `web/src/main.rs` (new) — Leptos mount point with "Peach" heading
- `docs/implementation-artifacts/1-1-project-scaffold.md` (modified) — story status updated
