# Story 10.2: Build & Deploy to GitHub Pages

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want the app to be automatically built and deployed to GitHub Pages after all checks pass,
so that the latest version of the app is always live without manual intervention.

## Acceptance Criteria

1. A `build-deploy` job in `.github/workflows/ci.yml` runs only after the `quality-gate` job passes (`needs: quality-gate`)
2. Rust toolchain with `wasm32-unknown-unknown` target and Trunk are installed in the build job
3. The SoundFont file is downloaded via `bin/download-sf2.sh` with GitHub Actions caching so it is fetched only once (or when the cache expires)
4. `trunk build --release --public-url /peach-web/` produces the deployable WASM artifacts in `dist/`
5. The build output is deployed to GitHub Pages using `actions/deploy-pages`
6. The GitHub repository is configured with GitHub Pages source set to "GitHub Actions"
7. The app is accessible and functional at `https://<username>.github.io/peach-web/`
8. Cargo registry, build artifacts, and SoundFont caches are shared between the quality gate and build jobs where possible

## Tasks / Subtasks

- [ ] Task 1: Add `build-deploy` job to CI workflow (AC: 1, 2, 3, 4, 8)
  - [ ] 1.1 Add `build-deploy` job with `needs: quality-gate` dependency in `.github/workflows/ci.yml`
  - [ ] 1.2 Add `permissions: pages: write, id-token: write, contents: read` at workflow level (required by `actions/deploy-pages`)
  - [ ] 1.3 Add `concurrency` group `pages` with `cancel-in-progress: false` to prevent overlapping deployments
  - [ ] 1.4 Install Rust stable toolchain with `wasm32-unknown-unknown` target (reuse same action as quality-gate)
  - [ ] 1.5 Add Cargo caching via `Swatinem/rust-cache@v2` (each job needs its own cache step; caches are shared by key)
  - [ ] 1.6 Install Trunk via download of pre-built binary from GitHub releases (faster than `cargo install trunk`)
- [ ] Task 2: Download and cache SoundFont file (AC: 3, 8)
  - [ ] 2.1 Cache `.cache/` directory using `actions/cache@v4` with key based on `hashFiles('bin/sf2-sources.conf')` hash
  - [ ] 2.2 Run `bin/download-sf2.sh` — script skips download if cached file exists with correct checksum
  - [ ] 2.3 Verify the SF2 file exists at `.cache/GeneralUser-GS.sf2` after the step
- [ ] Task 3: Build WASM app with Trunk (AC: 4)
  - [ ] 3.1 Run `trunk build --release --public-url /peach-web/` to produce `dist/` output
  - [ ] 3.2 Verify build includes: index.html, WASM binaries, CSS, synth_worklet.wasm, GeneralUser-GS.sf2, sw.js
- [ ] Task 4: Deploy to GitHub Pages (AC: 5, 6, 7)
  - [ ] 4.1 Upload `dist/` as artifact using `actions/upload-pages-artifact@v3` with `path: dist`
  - [ ] 4.2 Deploy using `actions/deploy-pages@v4`
  - [ ] 4.3 Add `environment: github-pages` with the deployment URL to the job
- [ ] Task 5: Verify deployment (AC: 7)
  - [ ] 5.1 Push to `main` and confirm GitHub Actions pipeline runs both jobs — deferred to user: verify on GitHub Actions tab
  - [ ] 5.2 Confirm app loads at `https://<username>.github.io/peach-web/` — deferred to user: manual browser verification
  - [ ] 5.3 Confirm SoundFont loads and audio playback works — deferred to user: manual browser verification

## Dev Notes

### Technical Requirements

- **Extend existing workflow** — do NOT create a separate workflow file. Add the `build-deploy` job to `.github/workflows/ci.yml`
- **Job dependency:** `needs: quality-gate` ensures build never runs if quality checks fail (Story 10.1 designed for this)
- **GitHub Pages permissions:** The workflow needs `pages: write` and `id-token: write` permissions at the top level for OIDC token-based deployment
- **Trunk `--public-url`:** Critical for GitHub Pages subpath deployment. Without this flag, all asset paths would be root-relative (`/index.js`) instead of subpath-relative (`/peach-web/index.js`), causing 404s
- **Service Worker path:** The `sw.js` registered in `index.html` uses `'./sw.js'` (relative), which should work correctly with `--public-url`
- **SoundFont file is ~31 MB** — caching is essential to avoid re-downloading on every CI run. Cache by `bin/sf2-sources.conf` content hash so the cache invalidates only when the SF2 source changes

### Architecture Compliance

- This story modifies infrastructure only — no Rust code changes, no domain or web crate modifications
- The Trunk pre-build hook in `Trunk.toml` compiles `synth-worklet` to WASM automatically — no extra CI step needed
- `index.html` references `.cache/GeneralUser-GS.sf2` via `<link data-trunk rel="copy-file">` — Trunk copies this into `dist/` during build
- Build output (`dist/`) is git-ignored — deployment is from CI artifact, not committed files
- The `--release` flag activates workspace profile: `opt-level = 'z'`, `lto = true`, `codegen-units = 1` (size-optimized WASM)

### Library & Framework Requirements

| Action/Tool | Version | Purpose |
|---|---|---|
| `actions/checkout@v4` | v4 | Check out repository |
| `dtolnay/rust-toolchain@stable` | stable | Install Rust with `wasm32-unknown-unknown` target |
| `Swatinem/rust-cache@v2` | v2 | Cargo caching (shared cache key with quality-gate job) |
| `actions/cache@v4` | v4 | Cache `.cache/` directory for SoundFont (key: `hashFiles('bin/sf2-sources.conf')`) |
| `actions/upload-pages-artifact@v3` | v3 | Upload `dist/` as deployable artifact |
| `actions/deploy-pages@v4` | v4 | Deploy uploaded artifact to GitHub Pages |
| Trunk | latest | WASM bundler — install pre-built binary from GitHub releases for speed |

### File Structure Requirements

- **Modify:** `.github/workflows/ci.yml` — add `build-deploy` job, permissions, and concurrency
- **No new files created**
- **No existing Rust code modified**

### Testing Requirements

- No automated tests to write — this is CI/CD infrastructure only
- Verification: review YAML before push, confirm Actions runs both jobs, confirm deployment URL serves the app

### Project Structure Notes

- Workspace: `domain/`, `web/`, `synth-worklet/` — all three compile during `trunk build`
- Trunk entry: `index.html` at project root
- SoundFont path: `.cache/GeneralUser-GS.sf2` — must exist before `trunk build`
- Synth worklet: compiled by Trunk pre-build hook → `web/assets/soundfont/synth_worklet.wasm`
- Tailwind CSS: version 4.2.1 pinned in `Trunk.toml` — Trunk downloads automatically

### Previous Story Intelligence (10.1)

- Story 10.1 created `.github/workflows/ci.yml` with a `quality-gate` job — explicitly designed for 10.2 to append a second job with `needs: quality-gate`
- Code review added `pull_request` trigger and `--all-targets` to clippy
- `Swatinem/rust-cache@v2` already configured — the build job benefits from shared cache keys
- All three checks pass locally and in CI

### GitHub Pages Manual Setup Required

The repository must have GitHub Pages enabled with source set to **"GitHub Actions"** (not "Deploy from branch"). This is a one-time manual setting: **Settings > Pages > Source > GitHub Actions**. The dev agent cannot configure this — the user must do it in the GitHub UI before the first deployment will work.

### Existing CI Workflow (current state)

```yaml
name: CI Quality Gate

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  quality-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --check
      - name: Clippy lints
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Run domain tests
        run: cargo test -p domain
```

The `build-deploy` job must be added below the `quality-gate` job in this file.

### References

- [Source: docs/planning-artifacts/epics.md#Epic 10] Story 10.2 acceptance criteria
- [Source: docs/implementation-artifacts/10-1-ci-quality-gate.md] Previous story learnings, CI workflow design
- [Source: docs/planning-artifacts/architecture.md#Infrastructure & Deployment] Trunk build process, static deployment
- [Source: docs/project-context.md#Development Workflow Rules] Build commands, project structure
- [Source: .github/workflows/ci.yml] Existing quality-gate job to extend
- [Source: Trunk.toml] Build config with pre-build hook for synth-worklet
- [Source: index.html] Asset references: SF2, synth_worklet, sw.js, Tailwind CSS
- [Source: bin/sf2-sources.conf] SoundFont URL and SHA256 for cache key
- [Source: Cargo.toml] Workspace members, release profile optimization

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
