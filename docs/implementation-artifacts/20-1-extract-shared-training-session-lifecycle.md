# Story 20.1: Extract Shared Training Session Lifecycle

Status: pending

## Story

As a developer,
I want shared training lifecycle infrastructure extracted from the four training views,
so that cross-cutting behavior (audio interruption, visibility changes, error handling, cleanup) is defined once and reused.

## Context

Adversarial review (2025-03-25) identified ARV-001 and ARV-002 as the highest-impact findings: the four training views (`pitch_matching_view.rs`, `pitch_discrimination_view.rs`, `rhythm_offset_detection_view.rs`, `continuous_rhythm_matching_view.rs`) are 700-854 line monoliths that duplicate ~600 lines of shared infrastructure.

### iOS Reference

The iOS sibling app (`peach-ios`) solves this with:
- **`SessionLifecycle`** (`Peach/Core/Training/SessionLifecycle.swift`) — shared task cancellation, audio interruption handling, background/foreground hooks
- **Observer + Adapter pattern** — sessions notify observers; adapters bridge to persistence/profile
- **Composition root** (`PeachApp.swift`) — all wiring in one place; views are thin consumers

This story focuses on extracting the web equivalent. It does NOT restructure how sessions are created or how observers work (those may follow in later stories).

### Duplicated Code Inventory

The following patterns are copy-pasted across all four training views:

1. **AudioContext state-change handler** (~50 lines x 4 = 200 lines)
   - Listens for `onstatechange` on AudioContext
   - Pauses training on `suspended`/`closed`, resumes on `running`
   - Sets `AudioNeedsGesture` signal when gesture required

2. **Visibility-change handler** (~20 lines x 4 = 80 lines)
   - Listens for `visibilitychange` on document
   - Pauses training when tab hidden, resumes when visible

3. **Error auto-dismiss Effects** (~15 lines x 2 signals x 4 = 120 lines)
   - `audio_error` signal: set to Some, auto-clear after 5s
   - `storage_error` signal: set to Some, auto-clear after 5s

4. **Help-modal pause coordination** (~15 lines x 4 = 60 lines)
   - `help_paused` signal toggles `cancelled` flag
   - Training loop checks `cancelled` before advancing

5. **Cancellation/termination flag setup** (~10 lines x 4 = 40 lines)
   - `cancelled: Rc<Cell<bool>>` for pause
   - `terminated: Rc<Cell<bool>>` for navigation exit

6. **Navigation cleanup** (~15 lines x 4 = 60 lines)
   - `on_cleanup` closure: set terminated, stop session, sync state

**Total duplicated:** ~560 lines

## Acceptance Criteria

1. **AC1 — `training_common` module:** A new `web/src/components/training_common.rs` module exists, exported from `components/mod.rs`.

2. **AC2 — Cancellation/termination flags:** A struct (e.g., `TrainingFlags`) encapsulates `cancelled: Rc<Cell<bool>>` and `terminated: Rc<Cell<bool>>` with methods `pause()`, `resume()`, `terminate()`, `is_cancelled()`, `is_terminated()`.

3. **AC3 — AudioContext state handler:** A function (e.g., `setup_audio_state_handler`) accepts the training flags and `AudioNeedsGesture` signal, installs the `onstatechange` listener, and returns a cleanup handle. Identical behavior to the current per-view implementation.

4. **AC4 — Visibility-change handler:** A function (e.g., `setup_visibility_handler`) accepts the training flags, installs the `visibilitychange` listener, and returns a cleanup handle.

5. **AC5 — Error auto-dismiss:** A function (e.g., `setup_error_autodismiss`) accepts an `RwSignal<Option<String>>` and installs an Effect that clears it after 5 seconds using `spawn_local_scoped_with_cancellation`.

6. **AC6 — Help-modal coordination:** A function (e.g., `setup_help_pause`) accepts `help_paused: RwSignal<bool>` and the training flags, installs an Effect that toggles `cancelled` on help open/close.

7. **AC7 — All four training views refactored:** Each training view calls the shared functions instead of inlining the logic. Net line reduction per view: ~140 lines minimum.

8. **AC8 — No behavior change:** All training interactions work identically before and after. No regressions in audio state handling, visibility pausing, error display, help modal, or navigation cleanup.

9. **AC9 — Cleanup correctness:** All event listeners installed by shared functions are properly removed in `on_cleanup`, matching current behavior.

## Tasks / Subtasks

- [ ] Task 1: Create `training_common.rs` with `TrainingFlags` struct (AC: 1, 2)
  - [ ] 1.1 Define `TrainingFlags` with `cancelled` and `terminated` fields
  - [ ] 1.2 Add `new()`, `pause()`, `resume()`, `terminate()`, `is_cancelled()`, `is_terminated()` methods
  - [ ] 1.3 Export from `components/mod.rs`

- [ ] Task 2: Extract AudioContext state handler (AC: 3, 9)
  - [ ] 2.1 Create `setup_audio_state_handler()` function
  - [ ] 2.2 Return cleanup handle (stored in `StoredValue::new_local()`)
  - [ ] 2.3 Verify identical behavior to existing inline code

- [ ] Task 3: Extract visibility-change handler (AC: 4, 9)
  - [ ] 3.1 Create `setup_visibility_handler()` function
  - [ ] 3.2 Return cleanup handle

- [ ] Task 4: Extract error auto-dismiss Effect (AC: 5)
  - [ ] 4.1 Create `setup_error_autodismiss()` function
  - [ ] 4.2 Uses `spawn_local_scoped_with_cancellation` + `TimeoutFuture`

- [ ] Task 5: Extract help-modal pause coordination (AC: 6)
  - [ ] 5.1 Create `setup_help_pause()` function

- [ ] Task 6: Refactor `pitch_discrimination_view.rs` (AC: 7, 8)
  - [ ] 6.1 Replace inline code with shared function calls
  - [ ] 6.2 Verify no behavior change

- [ ] Task 7: Refactor `pitch_matching_view.rs` (AC: 7, 8)
  - [ ] 7.1 Replace inline code with shared function calls
  - [ ] 7.2 Verify no behavior change

- [ ] Task 8: Refactor `rhythm_offset_detection_view.rs` (AC: 7, 8)
  - [ ] 8.1 Replace inline code with shared function calls
  - [ ] 8.2 Verify no behavior change

- [ ] Task 9: Refactor `continuous_rhythm_matching_view.rs` (AC: 7, 8)
  - [ ] 9.1 Replace inline code with shared function calls
  - [ ] 9.2 Verify no behavior change

- [ ] Task 10: Final validation (AC: 8)
  - [ ] 10.1 `cargo clippy --workspace` clean
  - [ ] 10.2 `cargo test -p domain` passes
  - [ ] 10.3 Manual smoke test: each discipline starts, plays, pauses on help, resumes, navigates back cleanly

## Dev Notes

### Approach

Extract helpers bottom-up: start with the simplest shared code (`TrainingFlags`, error auto-dismiss), then the more complex handlers (AudioContext, visibility). Refactor one view at a time, verifying behavior after each.

Do NOT attempt to create a generic `<TrainingView<S>>` wrapper component in this story. That would require restructuring how sessions and signals flow, which is a larger change. This story focuses purely on extracting duplicated utility functions.

### Key Patterns to Preserve

- `StoredValue::new_local()` for event listener cleanup handles (prevents premature GC)
- `spawn_local_scoped_with_cancellation` for timer Effects (NOT `spawn_local`)
- `terminated` flag checked before any signal writes in cleanup paths
- `SendWrapper` for closures passed to Leptos context

### iOS Architecture Reference

- `peach-ios/Peach/Core/Training/SessionLifecycle.swift` — Task management and interruption handling
- `peach-ios/docs/arc42.md` sections 5-8 — Architecture decisions and patterns

### Related Findings

- ARV-001: Massive code duplication across training views
- ARV-002: No shared abstraction for training session lifecycle
- ARV-005: No centralized error/notification system (partially addressed by AC5)
- ARV-008: RefCell borrow overlap risk (NOT addressed in this story)

### References

- [Adversarial review: docs/pre-existing-findings.adversarial-review-2025-03-25.md]
- [iOS architecture: ../peach-ios/docs/arc42.md]
- [iOS SessionLifecycle: ../peach-ios/Peach/Core/Training/SessionLifecycle.swift]
