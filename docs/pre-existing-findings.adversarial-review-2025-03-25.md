# Adversarial Codebase Review — 2025-03-25

Full codebase review covering architecture, coding style, dependencies, and duplications.

## Findings

### ARV-001: Massive code duplication across training views (~600+ lines)

**Severity:** High
**Location:** `web/src/components/pitch_matching_view.rs`, `pitch_discrimination_view.rs`, `rhythm_offset_detection_view.rs`, `continuous_rhythm_matching_view.rs`

The four training views (700-854 lines each) copy-paste the same patterns: AudioContext state-change handlers (~50 lines × 4 = 200), error auto-dismiss Effects, help-modal pause/resume coordination, visibility-change handlers, cancellation/termination flag logic, and NavBar callback setup. A bug fix in one view must be manually replicated in three others.

---

### ARV-002: No shared abstraction for training session lifecycle

**Severity:** High
**Location:** `web/src/components/` (all 4 training views)

Each training view independently implements the same lifecycle: create signals → setup audio → run training loop → sync state → handle navigation cleanup. No `TrainingViewBase`, no shared hook, no extracted utility. The "sync session to signals" function is redefined in each view with slight variations.

---

### ARV-003: `debug_assert!` used for numerical safety in production paths

**Severity:** Medium
**Location:** `domain/src/welford.rs`, `domain/src/metric_point.rs`

`WelfordAccumulator::update()` and `MetricPoint::new()` use `debug_assert!` to check for NaN/infinity. These are stripped in release builds, meaning production WASM will silently accept poisoned float values that corrupt running statistics. Should be runtime `assert!` or return `Result`.

---

### ~~ARV-004: Lossy bucket reconstruction in ProgressTimeline~~ WONT-FIX

**Severity:** Medium
**Location:** `domain/src/progress_timeline.rs`

`TimeBucket` stores only `mean` and `stddev`, then reconstructs `m2` from `stddev * stddev * count` during incremental updates. This loses numerical precision over many updates — the very problem Welford's algorithm exists to solve. The domain crate has a proper `WelfordAccumulator` that stores `m2` directly, but `TimeBucket` doesn't use it.

**WONT-FIX rationale:** The `sqrt -> square` round-trip error is ~1 ULP per update (~10^-16 relative). Even over thousands of updates the accumulated drift is negligible for statistics displayed to 1 decimal place. Technically correct finding but practically a non-issue.

---

### ~~ARV-005: No centralized error/notification system~~ Covered by Story 20.1

**Severity:** Medium
**Location:** `web/src/components/` (all 4 training views)

Audio errors and storage errors each use manually duplicated signal + auto-dismiss Effects in every view. No `<ErrorNotification>` component, no toast system, no error boundary. A single extracted component would eliminate ~80 lines of duplicated Effects.

**Covered by Story 20.1 AC5:** shared `setup_error_autodismiss()` function eliminates the duplication. A full toast component would be an enhancement beyond what the finding demands.

---

### ~~ARV-006: Inconsistent button enable/disable patterns~~ WONT-FIX

**Severity:** Low
**Location:** `web/src/components/pitch_matching_view.rs`, `pitch_discrimination_view.rs`, `rhythm_offset_detection_view.rs`

`pitch_matching_view` controls interactivity via CSS `pointer-events`. Other views use the HTML `disabled` attribute. No consistent approach.

**WONT-FIX rationale:** The views use fundamentally different controls (continuous slider vs. discrete buttons). The different disable mechanisms are appropriate for each case.

---

### ~~ARV-007: Dead code — `SchedulerMode::Loop` variant~~ FIXED

**Severity:** Low
**Location:** `web/src/adapters/rhythm_scheduler.rs`

`SchedulerMode::Loop` is marked `#[allow(dead_code)]` — only `SinglePass` is ever used. Speculative code for a nonexistent feature.

**FIXED:** Removed `SchedulerMode` enum entirely. Scheduler now always does single-pass (the only mode ever used). Removed `mode` field from `SchedulerConfig` and `SchedulerState`, simplified `schedule_ahead`.

---

### ~~ARV-008: `RefCell` borrow overlap risk in event handlers~~ WONT-FIX

**Severity:** Medium
**Location:** `web/src/components/` (training views), `web/src/adapters/audio_context.rs`

Audio context manager, sessions, and note players held in `Rc<RefCell<>>`. If a browser event handler fires during an existing `.borrow_mut()` scope (e.g., `onstatechange` during audio setup), it will panic at runtime. No `try_borrow_mut()` guards anywhere.

**WONT-FIX rationale:** In single-threaded WASM, browser callbacks (`onstatechange`, `visibilitychange`) are event-loop dispatched, not fired synchronously during Rust execution. All 73 `borrow_mut()` call sites are short-lived method calls that drop immediately — none are held across `.await` points. The overlap scenario described cannot occur in this runtime model.

---

### ~~ARV-009: IndexedDB writes are fire-and-forget with no retry~~ WONT-FIX

**Severity:** Medium
**Location:** `web/src/adapters/indexeddb_store.rs`

Failed writes log an error and show a vague message. No retry logic, no queue, no offline buffer. Training data is silently lost on transient failures.

**WONT-FIX rationale:** IndexedDB failure modes (quota exceeded, database closed, upgrade needed) are persistent, not transient — a retry wouldn't help. Current behavior (warn user, continue training) is reasonable. A retry queue adds complexity for a rare scenario with no practical benefit.

---

### ~~ARV-010: LocalStorage settings have no schema versioning~~ WONT-FIX

**Severity:** Low
**Location:** `web/src/adapters/localstorage_settings.rs`

Settings persisted as JSON with no version field. Schema changes will cause deserialization failures or silent defaults for existing users. No migration path.

**WONT-FIX rationale:** Serde's `#[serde(default)]` handles additive schema changes gracefully. Breaking changes (renames, type changes) haven't occurred. Add versioning when a breaking change is actually needed, not preemptively.

---

### ~~ARV-011: Service worker cache version injection is fragile~~ FIXED

**Severity:** Low
**Location:** `sw.js`, `.github/workflows/ci.yml`

CI uses `sed` to replace a literal string in `sw.js`. If anyone reformats the file, the replacement silently fails and ships a stale cache key. No CI step verifies the replacement succeeded.

**FIXED:** Added `grep` verification after `sed` — build fails with a clear error if the replacement didn't take.

---

### ~~ARV-012: Test helper duplication across integration tests~~ FALSE POSITIVE

**Severity:** Low
**Location:** `domain/tests/profile_hydration.rs`, `domain/tests/strategy_convergence.rs`

Both files define their own `discrimination_record()` and `matching_record()` helpers. Should live in a shared `tests/common/` module.

**FALSE POSITIVE:** Neither file defines these helpers. Both construct test data inline using domain constructors directly. No duplication exists.

---

### ~~ARV-013: `RhythmOffset` zero-check relies on exact float equality~~ WONT-FIX

**Severity:** Low
**Location:** `domain/src/types/rhythm_offset.rs`

`RhythmOffset::new(0.0).direction()` returns `OnBeat` only for exactly `0.0`. Near-zero results from float arithmetic classify as `Early` or `Late`. No tolerance/epsilon.

**WONT-FIX rationale:** By design. `OnBeat` is a degenerate statistics bucket — `difficulty_pct(OnBeat)` returns `0.0` and `update(OnBeat, _)` is a no-op. Real taps never land at exactly `0.0ms`. The `Early`/`Late` classification at sub-millisecond offsets is correct and harmless.

---

### ~~ARV-014: synth-worklet FFI has no null-pointer checks~~ FALSE POSITIVE

**Severity:** Low
**Location:** `synth-worklet/src/lib.rs`

All 9 exported C functions dereference raw `*mut Synth` pointers without null checks. A JS caller passing `0` or a freed pointer causes undefined behavior.

**FALSE POSITIVE:** All functions taking `*mut Synth` already use `synth.as_mut()` which returns `None` for null pointers. Functions silently no-op on null. Guards are already in place.

---

### ~~ARV-015: Trunk pre-build hook relies on implicit `cmp` binary~~ WONT-FIX

**Severity:** Low
**Location:** `Trunk.toml`

The pre-build hook uses `cmp -s` which exists on macOS and most Linux distros but is not guaranteed on minimal CI images. Works today on `ubuntu-latest` but is an implicit dependency.

**WONT-FIX rationale:** `cmp` is part of GNU diffutils, installed on every `ubuntu-latest` runner and all standard macOS/Linux environments.

---

### ~~ARV-016: No web crate tests~~ WONT-FIX

**Severity:** High
**Location:** `web/`

`wasm-bindgen-test` is a dev dependency but zero test files exist. All 3,200+ lines of adapter code and component logic are completely untested. Only `domain/` has tests.

**WONT-FIX rationale:** Web crate is thin glue between domain logic and browser APIs (Web Audio, IndexedDB, AudioWorklet). All testable business logic lives in the domain crate (341+ automated tests). Web crate behavior is covered by manual acceptance testing per story. Browser API integration is inherently hard to unit test without a full browser harness.

---

### ~~ARV-017: CSS custom styles bypass Tailwind's design system~~ WONT-FIX

**Severity:** Low
**Location:** `input.css`

60+ lines of hand-written CSS use hardcoded colors (`rgba(120,120,120,0.4)`, `#e5e7eb`) instead of Tailwind theme tokens. Won't respond to theme changes and create a parallel styling system.

**WONT-FIX rationale:** The custom CSS fills legitimate gaps: `prefers-contrast: more` accessibility overrides, SVG `fill` on programmatically generated chart elements, scrollbar pseudo-element styling, and high-contrast media queries. These are contexts where Tailwind utility classes can't reach. The hardcoded colors are gray-scale values matching Tailwind's gray palette.
