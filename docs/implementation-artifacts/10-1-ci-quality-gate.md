# Story 10.1: CI Quality Gate

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want automated quality checks to run on every push to `main`,
so that broken code is never deployed and I get fast feedback on regressions.

## Acceptance Criteria

1. A GitHub Actions workflow file exists at `.github/workflows/ci.yml` and triggers on push to `main`
2. `cargo fmt --check` runs and fails the pipeline if formatting is wrong
3. `cargo clippy --workspace` runs and fails the pipeline if there are warnings
4. `cargo test -p domain` runs and fails the pipeline if any test fails
5. The workflow uses caching for the Cargo registry and build artifacts to speed up subsequent runs
6. If any check fails, the pipeline stops — no build or deploy step runs

## Tasks / Subtasks

- [ ] Task 1: Create GitHub Actions workflow file (AC: 1)
  - [ ] 1.1 Create `.github/workflows/ci.yml` with trigger on `push` to `main`
  - [ ] 1.2 Set up the job to run on `ubuntu-latest`
  - [ ] 1.3 Install Rust stable toolchain with `wasm32-unknown-unknown` target (needed for `cargo clippy --workspace` which includes the `web` and `synth-worklet` crates)
  - [ ] 1.4 Add `rustfmt` and `clippy` components to the toolchain installation
- [ ] Task 2: Add Cargo caching (AC: 5)
  - [ ] 2.1 Use `Swatinem/rust-cache@v2` to cache `~/.cargo/registry`, `~/.cargo/git`, and `target/`
  - [ ] 2.2 Use workspace `Cargo.lock` as cache key input
- [ ] Task 3: Add quality check steps in order (AC: 2, 3, 4, 6)
  - [ ] 3.1 Step: `cargo fmt --check` — fails pipeline on formatting violations
  - [ ] 3.2 Step: `cargo clippy --workspace -- -D warnings` — fails pipeline on any warning
  - [ ] 3.3 Step: `cargo test -p domain` — fails pipeline on any test failure
- [ ] Task 4: Push workflow and verify pipeline catches issues (AC: 1, 6)
  - [ ] 4.1 Commit and push the `.github/workflows/ci.yml` file
  - [ ] 4.2 Confirm the pipeline triggers and `cargo fmt --check` fails (existing code has formatting issues)
  - [ ] 4.3 Confirm the pipeline stops — clippy and test steps do not run after fmt failure
- [ ] Task 5: Fix formatting and verify green pipeline (AC: 2, 3, 4, 5) — separate commit
  - [ ] 5.1 Run `cargo fmt` locally to fix all formatting issues
  - [ ] 5.2 Run `cargo clippy --workspace -- -D warnings` locally to confirm clean
  - [ ] 5.3 Run `cargo test -p domain` locally to confirm all tests pass
  - [ ] 5.4 Commit formatting fixes separately and push
  - [ ] 5.5 Confirm the pipeline passes all three checks

## Dev Notes

### Technical Requirements

- **GitHub Actions only** — no other CI provider. Workflow file goes in `.github/workflows/ci.yml`
- **Rust toolchain**: stable channel, with components `rustfmt` and `clippy`, and target `wasm32-unknown-unknown`
- The `wasm32-unknown-unknown` target is required because `cargo clippy --workspace` compiles all three workspace members (`domain`, `web`, `synth-worklet`), and both `web` and `synth-worklet` target WASM
- `cargo clippy` must use `-- -D warnings` to promote warnings to errors so the pipeline fails on any warning
- No SoundFont file or Trunk needed — this story is compilation and linting only

### Architecture Compliance

- This story creates infrastructure only — no Rust code changes, no domain or web crate modifications
- The workflow must respect the workspace structure: `domain/`, `web/`, `synth-worklet/` as defined in root `Cargo.toml`
- `cargo test -p domain` runs native tests only (no WASM, no browser) — this is by design per architecture.md
- Story 10.2 will add a second job (build + deploy) that depends on this quality gate job passing — design the workflow file so a second job can be appended later with a `needs: quality-gate` dependency

### Library & Framework Requirements

- **`actions/checkout@v4`** — check out the repository
- **`dtolnay/rust-toolchain@stable`** — install Rust stable with components and targets. Preferred over `actions-rs/toolchain` (deprecated)
- **`Swatinem/rust-cache@v2`** — caches `~/.cargo/registry`, `~/.cargo/git`, and `target/` keyed on `Cargo.lock`. Simpler and more effective than manual `actions/cache` for Rust projects
- No other actions or dependencies required for the quality gate

### File Structure Requirements

- **Create:** `.github/workflows/ci.yml` — the only new file in this story
- **No existing files modified** — this is a pure infrastructure addition
- The `.github/` directory does not exist yet — it must be created along with the `workflows/` subdirectory

### Testing Requirements

- No automated tests to write — this story creates CI infrastructure, not application code
- **Verification approach:** run all three checks locally before committing to confirm the pipeline will pass on first push:
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test -p domain`
- After pushing, verify the GitHub Actions run completes successfully via the repository's Actions tab

### Project Structure Notes

- Workspace has three crates: `domain` (pure Rust), `web` (Leptos CSR + browser APIs), `synth-worklet` (WASM AudioWorklet module)
- `Cargo.lock` exists at workspace root — use as cache key
- No existing CI/CD configuration — greenfield setup
- `bin/download-sf2.sh` exists for SoundFont download but is not needed for this story (deferred to 10.2)

### References

- [Source: docs/planning-artifacts/epics.md#Epic 10] Story 10.1 acceptance criteria and 10.2 dependency description
- [Source: docs/planning-artifacts/architecture.md#Infrastructure & Deployment] CI/CD deferred decision, now being implemented
- [Source: docs/project-context.md#Code Quality & Style Rules] `cargo clippy` and `cargo test -p domain` as standard quality checks
- [Source: Cargo.toml] Workspace members: `domain`, `web`, `synth-worklet`
- [Source: Trunk.toml] Pre-build hook for synth-worklet WASM — not relevant for quality gate (Trunk not invoked)

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
