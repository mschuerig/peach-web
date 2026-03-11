---
title: 'Fix i18n reactive tracking warnings on settings page'
slug: 'fix-settings-i18n-reactive-tracking'
created: '2026-03-11'
status: 'dev-complete'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['leptos-fluent 0.3.1', 'leptos 0.8.17', 'reactive_graph 0.2.13']
files_to_modify: ['web/src/components/settings_view.rs', 'docs/project-context.md']
code_patterns: ['move_tr!() for reactive i18n', 'Signal::derive(move || tr!()) for parameterized i18n', 'untrack(|| tr!()) for intentionally non-reactive captures']
test_patterns: ['manual browser verification — zero console warnings on settings page']
---

# Tech-Spec: Fix i18n reactive tracking warnings on settings page

**Created:** 2026-03-11

## Overview

### Problem Statement

Navigating to the settings page floods the browser console with warnings:

```
you access a reactive_graph::signal::rw::RwSignal<&leptos_fluent::Language>
(defined at web/src/app.rs:300:5) outside a reactive tracking context
```

**Root cause:** The `tr!()` macro expands to `expect_context::<I18n>().tr(text_id)`, and `I18n::tr()` calls `self.language.get()` (at `leptos-fluent-0.3.1/src/lib.rs:581`). When `tr!()` is used bare in view construction — not inside a `move || {}` closure or `Signal::derive` — the `.get()` executes outside a reactive tracking context.

The settings page has ~20 bare `tr!()` calls in view construction (option labels, dialog text, aria-labels, pre-captured strings). Other pages use `move_tr!()` (which wraps in `Signal::derive`) and are unaffected.

**Secondary issue:** No project rule prevents this pattern from recurring.

### Solution

1. Replace bare `tr!()` calls in view construction with `move_tr!()` or `move || tr!(...)` so the signal access happens inside a reactive tracking context.
2. For pre-captured i18n strings (used before `spawn_local`), wrap in `untrack(|| tr!(...))` to suppress tracking without changing behavior.
3. Add a prevention rule to `project-context.md` Common Pitfalls table and Anti-Patterns section.

### Scope

**In Scope:**

- Fix all bare `tr!()` calls in `settings_view.rs` that execute outside reactive contexts
- Add prevention rule to project-context.md
- Verify zero console warnings on settings page after fix

**Out of Scope:**

- Changing leptos-fluent internals
- Other components (verified safe — their `tr!()` calls are all inside `move || {}` closures)

## Context for Development

### Codebase Patterns

**Safe patterns (signal access inside reactive context):**

```rust
// move_tr! wraps in Signal::derive — SAFE
<SettingsSection title=move_tr!("language-label")>

// Explicit Signal::derive — SAFE
let min_label = Signal::derive(move || tr!("lowest-note", {"note" => min_note_name.get()}));

// Inside move || closure — SAFE
{move || match ie_status.get() {
    ImportExportStatus::Exporting => tr!("exporting"),
    _ => tr!("export-training-data"),
}}

// untrack suppresses tracking warning — SAFE for intentional non-reactive reads
let msg = untrack(|| tr!("some-key"));
```

**Broken pattern (signal access outside reactive context):**

```rust
// Bare tr!() in view construction — BROKEN, fires warning
<option value="equalTemperament">{tr!("equal-temperament")}</option>

// Bare tr!() in format! during view construction — BROKEN
aria-label=format!("{} {}", interval.short_label(), tr!("ascending"))

// Bare tr!() in attribute — BROKEN
aria-label=tr!("loudness-variation-aria")

// Bare tr!() in component body block — BROKEN
let msg_exported = tr!("data-exported");
```

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `web/src/components/settings_view.rs` | All problematic bare `tr!()` calls (lines 408, 448, 584, 585, 601, 616, 618, 639-644, 698, 881, 908, 932, 938, 944, 958, 960, 968, 975) |
| `docs/project-context.md` | Anti-Patterns (line ~223) and Common Pitfalls (line ~273) sections to update |
| `~/.cargo/registry/.../leptos-fluent-0.3.1/src/lib.rs:580-581` | `I18n::tr()` calls `self.language.get()` — root cause of the warning |

### Technical Decisions

1. **`move_tr!()` over `move || tr!()`:** Prefer `move_tr!()` for simple key-only translations in view text content. It is more concise and idiomatic (`Signal::derive(move || tr!("key"))`). Use `move || tr!(...)` for HTML attributes that need a closure, or `Signal::derive(move || tr!(..., {args}))` for parameterized translations.

2. **`untrack(|| tr!(...))` for pre-captured strings:** The `untrack` function from `reactive_graph::graph` (re-exported via `leptos::prelude::*`) suppresses reactive tracking for signal accesses inside its closure. `tr!()` still reads the language signal and returns the correct translation, but no subscription is created and no warning fires. This is the idiomatic Leptos solution for intentionally non-reactive reads.

3. **Reactive aria-labels with format strings:** Convert `aria-label=format!("{} {}", x, tr!("key"))` to `aria-label=move || format!("{} {}", x, tr!("key"))`. This both fixes the warning and makes the aria-label update correctly on language change.

4. **Line 493 (`tr!("sine-oscillator")`) is safe:** It is inside the `move || { ... }` closure at line 491, which IS a reactive context. No change needed.

## Implementation Plan

### Tasks

- [x] **Task 1:** Replace bare `tr!()` with `move_tr!()` for view text content
  - File: `web/src/components/settings_view.rs`
  - Action: For each line below, replace `tr!(` with `move_tr!(`:
    - Line 584: `{tr!("equal-temperament")}` → `{move_tr!("equal-temperament")}`
    - Line 585: `{tr!("just-intonation")}` → `{move_tr!("just-intonation")}`
    - Line 601: `{tr!("off")}` → `{move_tr!("off")}`
    - Line 618: `{tr!("max")}` → `{move_tr!("max")}`
    - Line 908: `{tr!("import-dialog-title")}` → `{move_tr!("import-dialog-title")}`
    - Line 932: `{tr!("replace-all-data")}` → `{move_tr!("replace-all-data")}`
    - Line 938: `{tr!("merge-with-existing")}` → `{move_tr!("merge-with-existing")}`
    - Line 944: `{tr!("cancel")}` → `{move_tr!("cancel")}`
    - Line 958: `{tr!("reset-dialog-title")}` → `{move_tr!("reset-dialog-title")}`
    - Line 960: `{tr!("reset-dialog-message")}` → `{move_tr!("reset-dialog-message")}`
    - Line 968: `{tr!("cancel")}` → `{move_tr!("cancel")}`
    - Line 975: `{tr!("delete-all-data")}` → `{move_tr!("delete-all-data")}`
  - Notes: `move_tr!("key")` expands to `Signal::derive(move || tr!("key"))`, providing both reactive tracking and language-switch reactivity.

- [x] **Task 2:** Wrap bare `tr!()` in attributes with `move ||` closures
  - File: `web/src/components/settings_view.rs`
  - Action: For each line below, wrap the attribute value in `move ||`:
    - Line 408: `aria-label=format!("{} {}", interval.short_label(), tr!("ascending"))` → `aria-label=move || format!("{} {}", interval.short_label(), tr!("ascending"))`
    - Line 448: `aria-label=format!("{} {}", interval.short_label(), tr!("descending"))` → `aria-label=move || format!("{} {}", interval.short_label(), tr!("descending"))`
    - Line 616: `aria-label=tr!("loudness-variation-aria")` → `aria-label=move || tr!("loudness-variation-aria")`
    - Line 881: `aria-label=tr!("select-csv")` → `aria-label=move || tr!("select-csv")`
  - Notes: `move ||` creates a reactive closure. Leptos evaluates it in a tracking context, fixing the warning and making the attribute reactive to language changes.

- [x] **Task 3:** Move i18n strings from pre-capture into event handlers
  - File: `web/src/components/settings_view.rs`
  - Action: Removed 6 pre-captured `untrack(|| tr!(...))` variables from component body. Moved `tr!()` calls into each event handler body (before `spawn_local`), where they run in a valid context and read the current language. Eliminated clone chains.
  - Notes: Event handlers run synchronously with a valid reactive owner. `tr!()` is safe there and reads the current language at invocation time. Only `spawn_local` crosses the async boundary where `tr!()` would panic.

- [x] **Task 4:** Verify compilation and linting
  - Action: Run `cargo clippy --workspace` and `cargo fmt --check`
  - Notes: No new warnings expected. `untrack` and `move_tr!` are standard Leptos APIs.

- [x] **Task 5:** Add prevention rules to project-context.md
  - File: `docs/project-context.md`
  - Action 1: Add to the **Anti-Patterns** section (after the existing `DO NOT` list items around line 233):
    ```
    - DO NOT use bare `tr!()` in view construction or component body code outside a `move || {}` closure — `tr!()` calls `i18n.language.get()` which fires a reactive tracking warning and the text won't update on language switch. Use `move_tr!()` for text content, `move || tr!(...)` for attributes, or `Signal::derive(move || tr!(...))` for parameterized translations. For intentional one-time captures (e.g. before `spawn_local`), use `untrack(|| tr!(...))`.
    ```
  - Action 2: Add to the **Common Pitfalls** table (after the existing rows around line 291):
    ```
    | Bare `tr!()` outside reactive context | Console floods with "outside reactive tracking context" warnings; text won't update on language switch | Use `move_tr!()` for text, `move \|\| tr!(...)` for attributes. For one-time captures use `untrack(\|\| tr!(...))`. Bare `tr!()` is only safe inside `move \|\|` closures or `Signal::derive`. |
    ```

- [ ] **Task 6:** Manual browser verification
  - Action: Open settings page, check browser console for zero reactive tracking warnings. Switch language, verify all text updates. Test export/import/reset flows.
  - Notes: Deferred to user — agent cannot verify in browser.

### Acceptance Criteria

- [ ] **AC1:** Given the settings page is loaded, when the browser console is observed, then zero "outside a reactive tracking context" warnings related to `RwSignal<&leptos_fluent::Language>` appear.
- [ ] **AC2:** Given the language is changed via the settings picker, when viewing any translated text on the settings page (option labels, dialog buttons, aria-labels, slider labels), then all text updates to the new language without page reload.
- [ ] **AC3:** Given the export/import/reset buttons are clicked, when async operations complete, then the pre-captured i18n strings display correctly (no panics, correct translations in the active language).
- [ ] **AC4:** Given `project-context.md` is read by an AI agent, when the agent writes new i18n code, then the anti-pattern rule and common pitfall entry prevent bare `tr!()` outside reactive contexts.
- [ ] **AC5:** Given `cargo clippy --workspace` is run, when the build completes, then no new warnings are introduced.

## Additional Context

### Dependencies

- No new crate dependencies. `untrack` is already available via `leptos::prelude::*`.

### Testing Strategy

- **Compilation:** `cargo clippy --workspace` — no new warnings
- **Formatting:** `cargo fmt --check` — passes
- **Manual browser test:** Open settings page, verify zero reactive tracking warnings in console
- **Manual browser test:** Switch language on settings page, verify all text updates (option labels, dialog buttons, aria-labels, slider labels)
- **Manual browser test:** Test export, import, and reset flows to verify pre-captured strings work correctly

### Notes

- The `tr!()` macro cannot be modified — it's from leptos-fluent 0.3.1. The fix is always at the call site.
- `move_tr!("key")` expands to `Signal::derive(move || tr!("key"))` — it's a one-token change for simple translations.
- `untrack(|| tr!("key"))` is the idiomatic Leptos way to suppress tracking for intentionally non-reactive reads.
- Other components (start_page, profile_view, info_view, pitch_comparison_view, training_stats, progress_card, progress_sparkline) were audited — all their `tr!()` calls are inside `move || {}` closures or `Signal::derive`.
- Line 493 (`tr!("sine-oscillator")`) is inside the `move || { ... }` closure at line 491 — already safe, no change needed.
