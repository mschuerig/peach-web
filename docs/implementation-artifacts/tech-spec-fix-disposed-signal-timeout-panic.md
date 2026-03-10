---
title: 'Fix disposed reactive value panic on navigation'
slug: 'fix-disposed-signal-timeout-panic'
created: '2026-03-10'
status: 'completed'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['leptos 0.8', 'gloo-timers', 'wasm-bindgen-futures']
files_to_modify:
  - 'web/src/components/pitch_matching_view.rs'
  - 'web/src/components/pitch_comparison_view.rs'
  - 'web/src/components/start_page.rs'
code_patterns: ['spawn_local + TimeoutFuture']
test_patterns: ['manual navigation test']
---

# Tech-Spec: Fix disposed reactive value panic on navigation

**Created:** 2026-03-10

## Overview

### Problem Statement

Navigating from a training page (or start page) back to the start page triggers repeated "Tried to access a reactive value that has already been disposed" panics in the console. The error cascades via `setTimeout` callbacks until it eventually hits an `unreachable` WASM trap. The root cause is `gloo_timers::callback::Timeout::new(...).forget()` inside `Effect::new` — the `.forget()` detaches the timer from component lifecycle, so the callback fires after Leptos has disposed the component's signals.

### Solution

Replace all `Timeout::new(...).forget()` auto-dismiss patterns with `spawn_local` + `TimeoutFuture::new().await`. Leptos-spawned tasks are tied to the reactive owner and are cancelled at `.await` points when the owner is disposed. This pattern is already used throughout the training loop code in the same files.

### Scope

**In Scope:**
- Replace 5 instances of the buggy `Effect::new` + `Timeout.forget()` pattern across 3 files
- Add missing imports to `start_page.rs`

**Out of Scope:**
- Other uses of `gloo_timers` (e.g., `audio_soundfont.rs` `play_for_duration` — safe because it doesn't access signals)
- Refactoring the training loop async code
- Adding new error handling or dismissal UX

## Context for Development

### Codebase Patterns

- Training views already use `spawn_local` + `TimeoutFuture::new(ms).await` for polling and feedback delays
- Both training views already import `gloo_timers::future::TimeoutFuture` and `wasm_bindgen_futures::spawn_local`
- `start_page.rs` does NOT currently import these — needs `use gloo_timers::future::TimeoutFuture;` and `use wasm_bindgen_futures::spawn_local;`
- The `Effect::new` is still needed to reactively trigger on signal changes — the `spawn_local` goes inside the effect

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `web/src/components/pitch_matching_view.rs` | Lines 107-126: two buggy auto-dismiss effects |
| `web/src/components/pitch_comparison_view.rs` | Lines 118-137: two buggy auto-dismiss effects |
| `web/src/components/start_page.rs` | Lines 89-98: one buggy auto-dismiss effect |

### Technical Decisions

- **Why `spawn_local` + `TimeoutFuture` over stored handles + `on_cleanup`:** Less boilerplate, already proven in the codebase, and Leptos handles cancellation automatically at `.await` suspension points when the owner disposes.

## Implementation Plan

### Tasks

- [x] **Task 1:** Fix `pitch_matching_view.rs` auto-dismiss effects
  - File: `web/src/components/pitch_matching_view.rs`
  - Action: Replace `Timeout::new(5000, ...).forget()` with `spawn_local(async move { TimeoutFuture::new(5000).await; signal.set(None); })` in both the `storage_error` effect (line 107) and `audio_error` effect (line 118)
  - Before (each effect):
    ```rust
    Effect::new(move || {
        if storage_error.get().is_some() {
            let signal = storage_error;
            gloo_timers::callback::Timeout::new(5000, move || {
                signal.set(None);
            })
            .forget();
        }
    });
    ```
  - After (each effect):
    ```rust
    Effect::new(move || {
        if storage_error.get().is_some() {
            spawn_local(async move {
                TimeoutFuture::new(5000).await;
                storage_error.set(None);
            });
        }
    });
    ```

- [x] **Task 2:** Fix `pitch_comparison_view.rs` auto-dismiss effects
  - File: `web/src/components/pitch_comparison_view.rs`
  - Action: Identical transformation as Task 1 for `storage_error` (line 118) and `audio_error` (line 129) effects

- [x] **Task 3:** Fix `start_page.rs` auto-dismiss effect
  - File: `web/src/components/start_page.rs`
  - Action: Add imports `use gloo_timers::future::TimeoutFuture;` and `use wasm_bindgen_futures::spawn_local;`, then replace the `sf2_error_dismissed` effect (line 89)
  - Before:
    ```rust
    Effect::new(move || {
        if matches!(sf2_status.get(), SoundFontLoadStatus::Failed(_)) && !sf2_error_dismissed.get()
        {
            let signal = sf2_error_dismissed;
            gloo_timers::callback::Timeout::new(5000, move || {
                signal.set(true);
            })
            .forget();
        }
    });
    ```
  - After:
    ```rust
    Effect::new(move || {
        if matches!(sf2_status.get(), SoundFontLoadStatus::Failed(_)) && !sf2_error_dismissed.get()
        {
            spawn_local(async move {
                TimeoutFuture::new(5000).await;
                sf2_error_dismissed.set(true);
            });
        }
    });
    ```

- [x] **Task 4:** Remove unused `gloo_timers::callback::Timeout` imports
  - Files: All 3 files above
  - Action: Check each file for remaining uses of `gloo_timers::callback::Timeout`. If none remain, remove the import.

- [x] **Task 5:** Add convention rule to `project-context.md`
  - File: `docs/project-context.md`
  - Action: Add rule: "Never use `gloo_timers::callback::Timeout` with `.forget()` inside Leptos components — the timer outlives the component and causes disposed-signal panics. Use `spawn_local` + `TimeoutFuture::new(ms).await` instead, which is automatically cancelled when the reactive owner disposes."

### Acceptance Criteria

- [x] **AC1:** Given a user is on a training page, when they navigate back to the start page before the 5-second auto-dismiss fires, then no "reactive value already disposed" panic appears in the console.
- [x] **AC2:** Given a user is on the start page with a soundfont load failure, when they navigate away before the 5-second dismiss fires, then no panic appears.
- [x] **AC3:** Given an error is displayed and the user stays on the page, when 5 seconds elapse, then the error is still auto-dismissed as before.
- [x] **AC4:** Given the codebase compiles, when `cargo clippy --workspace` runs, then no new warnings are introduced.

## Additional Context

### Dependencies

No new dependencies. `gloo_timers::future::TimeoutFuture` and `wasm_bindgen_futures::spawn_local` are already in the dependency tree.

### Testing Strategy

- Manual: Navigate away from training mid-session, verify console is clean
- Manual: Trigger a storage/audio error, stay on page, verify auto-dismiss still works
- `cargo clippy --workspace` passes without new warnings

### Notes

The `gloo_timers::callback::Timeout` in `audio_soundfont.rs` `play_for_duration` is safe — it owns the `SoundFontPlaybackHandle` and never accesses reactive signals.

## Review Notes

- Adversarial review completed
- Findings: 6 total, 3 fixed (F-01 critical, F-02 high, F-03 high), 3 skipped (pre-existing/benign)
- Resolution approach: auto-fix
- Key correction: tech spec originally specified `spawn_local` but adversarial review identified it lacks owner-disposal cancellation. Changed to `leptos::task::spawn_local_scoped_with_cancellation` which uses `AbortHandle` + `on_cleanup`.
- Additional fix (beyond original scope): training loop's `spawn_local` task accessed `help_paused` signal after navigation disposal. Added `terminated.get()` guard before `help_paused.get_untracked()` and wrapped final cleanup in `!terminated.get()` check in both training views.
