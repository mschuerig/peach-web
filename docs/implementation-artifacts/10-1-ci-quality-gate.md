# Story 10.1: CI Quality Gate

Status: done

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

- [x] Task 1: Create GitHub Actions workflow file (AC: 1)
  - [x] 1.1 Create `.github/workflows/ci.yml` with trigger on `push` to `main`
  - [x] 1.2 Set up the job to run on `ubuntu-latest`
  - [x] 1.3 Install Rust stable toolchain with `wasm32-unknown-unknown` target (needed for `cargo clippy --workspace` which includes the `web` and `synth-worklet` crates)
  - [x] 1.4 Add `rustfmt` and `clippy` components to the toolchain installation
- [x] Task 2: Add Cargo caching (AC: 5)
  - [x] 2.1 Use `Swatinem/rust-cache@v2` to cache `~/.cargo/registry`, `~/.cargo/git`, and `target/`
  - [x] 2.2 Use workspace `Cargo.lock` as cache key input
- [x] Task 3: Add quality check steps in order (AC: 2, 3, 4, 6)
  - [x] 3.1 Step: `cargo fmt --check` — fails pipeline on formatting violations
  - [x] 3.2 Step: `cargo clippy --workspace -- -D warnings` — fails pipeline on any warning
  - [x] 3.3 Step: `cargo test -p domain` — fails pipeline on any test failure
- [x] Task 4: Push workflow and verify pipeline catches issues (AC: 1, 6)
  - [x] 4.1 Commit and push the `.github/workflows/ci.yml` file
  - [ ] 4.2 Confirm the pipeline triggers and `cargo fmt --check` fails (existing code has formatting issues) — **DEVIATION: combined with Task 5 into single commit per user decision; verified locally instead**
  - [ ] 4.3 Confirm the pipeline stops — clippy and test steps do not run after fmt failure — **DEVIATION: see 4.2**
- [x] Task 5: Fix formatting and verify green pipeline (AC: 2, 3, 4, 5) — separate commit
  - [x] 5.1 Run `cargo fmt` locally to fix all formatting issues
  - [x] 5.2 Run `cargo clippy --workspace -- -D warnings` locally to confirm clean
  - [x] 5.3 Run `cargo test -p domain` locally to confirm all tests pass
  - [x] 5.4 Commit formatting fixes separately and push — **DEVIATION: combined with CI file into single commit**
  - [ ] 5.5 Confirm the pipeline passes all three checks — **deferred to user: verify on GitHub Actions tab after push**

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

Claude Opus 4.6

### Debug Log References

(none)

### Completion Notes List

- Created `.github/workflows/ci.yml` with quality gate job: fmt check, clippy, domain tests
- Uses `dtolnay/rust-toolchain@stable` with `wasm32-unknown-unknown` target, `rustfmt` and `clippy` components
- Uses `Swatinem/rust-cache@v2` for Cargo caching
- Ran `cargo fmt` to fix existing formatting issues across the workspace
- All three checks verified locally: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p domain`
- **Deviation from story plan:** Tasks 4 & 5 were combined into a single commit per user decision instead of the two-commit approach (CI file first → verify red pipeline → fix formatting → verify green pipeline). Local verification was used instead.

### Code Review Notes (AI)

Reviewed by Claude Opus 4.6 on 2026-03-07. Findings and fixes:

- **[M1] Fixed:** Added `pull_request` trigger to CI workflow so PRs get checked before merge
- **[M2] Fixed:** Updated `docs/project-context.md` deployment section — removed stale "No CI/CD pipeline (deferred)"
- **[M3] Fixed:** Updated `docs/project-context.md` workspace description — added `synth-worklet/` (was missing)
- **[M4] Fixed:** Added `--all-targets` to clippy step — caught a `needless_range_loop` warning in test code (`domain/src/progress_timeline.rs:949`)
- **[L1] Acknowledged:** Tasks 4.2, 4.3, 5.5 remain deferred to user (GitHub Actions verification)
- **[L2] Fixed:** Split combined commit into separate formatting and CI commits for cleaner git history

### Change Log

- 2026-03-07: Created CI quality gate workflow and fixed formatting (combined commit)
- 2026-03-07: Code review fixes — PR trigger, --all-targets clippy, project-context.md updates, clippy fix in test code, split commits

### File List

- `.github/workflows/ci.yml` (new) — CI quality gate workflow
- `docs/project-context.md` (modified) — updated deployment and workspace descriptions
- `docs/implementation-artifacts/sprint-status.yaml` (modified) — status update
- `docs/implementation-artifacts/10-1-ci-quality-gate.md` (modified) — story tracking
- `domain/src/progress_timeline.rs` (modified) — formatting + clippy fix in test helper
- `domain/src/lib.rs` (modified) — formatting
- `domain/src/ports.rs` (modified) — formatting
- `domain/src/profile.rs` (modified) — formatting
- `domain/src/progress_timeline.rs` (modified) — formatting
- `domain/src/records.rs` (modified) — formatting
- `domain/src/session/mod.rs` (modified) — formatting
- `domain/src/session/pitch_comparison_session.rs` (modified) — formatting
- `domain/src/session/pitch_matching_session.rs` (modified) — formatting
- `domain/src/strategy.rs` (modified) — formatting
- `domain/src/training/pitch_comparison.rs` (modified) — formatting
- `domain/src/training/pitch_matching.rs` (modified) — formatting
- `domain/src/training_mode.rs` (modified) — formatting
- `domain/src/tuning.rs` (modified) — formatting
- `domain/src/types/interval.rs` (modified) — formatting
- `domain/src/types/midi.rs` (modified) — formatting
- `domain/src/types/note_range.rs` (modified) — formatting
- `domain/tests/strategy_convergence.rs` (modified) — formatting
- `domain/tests/tuning_accuracy.rs` (modified) — formatting
- `web/src/adapters/audio_context.rs` (modified) — formatting
- `web/src/adapters/audio_soundfont.rs` (modified) — formatting
- `web/src/adapters/csv_export_import.rs` (modified) — formatting
- `web/src/adapters/default_settings.rs` (modified) — formatting
- `web/src/adapters/indexeddb_store.rs` (modified) — formatting
- `web/src/adapters/localstorage_settings.rs` (modified) — formatting
- `web/src/adapters/note_player.rs` (modified) — formatting
- `web/src/adapters/sound_preview.rs` (modified) — formatting
- `web/src/app.rs` (modified) — formatting
- `web/src/bridge.rs` (modified) — formatting
- `web/src/components/help_content.rs` (modified) — formatting
- `web/src/components/info_view.rs` (modified) — formatting
- `web/src/components/mod.rs` (modified) — formatting
- `web/src/components/nav_bar.rs` (modified) — formatting
- `web/src/components/pitch_comparison_view.rs` (modified) — formatting
- `web/src/components/pitch_matching_view.rs` (modified) — formatting
- `web/src/components/pitch_slider.rs` (modified) — formatting
- `web/src/components/progress_chart.rs` (modified) — formatting
- `web/src/components/progress_sparkline.rs` (modified) — formatting
- `web/src/components/settings_view.rs` (modified) — formatting
- `web/src/components/start_page.rs` (modified) — formatting
- `web/src/components/training_stats.rs` (modified) — formatting
- `web/src/help_sections.rs` (modified) — formatting
- `web/src/interval_codes.rs` (modified) — formatting
