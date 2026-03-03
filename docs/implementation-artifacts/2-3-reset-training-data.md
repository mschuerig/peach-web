# Story 2.3: Reset Training Data

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to reset all my training data with a confirmation step,
so that I can start fresh if I choose to, without accidentally losing my data.

## Acceptance Criteria

1. **AC1 — Destructive button styling (FR29):** When the Settings view loads, the "Reset all training data" button is visually distinct as a destructive action (red/danger styling, separated from other settings).

2. **AC2 — Confirmation dialog:** When I click the reset button, a confirmation dialog appears that clearly states all training data will be permanently deleted and requires explicit confirmation before proceeding.

3. **AC3 — Cancel preserves data:** When I cancel the confirmation dialog, it closes and no data is deleted. The Settings view remains unchanged.

4. **AC4 — Full data reset:** When I confirm the reset, all comparison records are deleted from IndexedDB, the PerceptualProfile is reset (all 128 notes to defaults, matching accumulators zeroed), the TrendAnalyzer is reset, and the ThresholdTimeline is reset.

5. **AC5 — Active session stopped:** If a training session is active when reset executes, it is stopped first before data is cleared.

6. **AC6 — Cold-start after reset:** After a reset, when I start training, the adaptive algorithm begins at cold-start difficulty (100 cents) and the profile preview shows the empty state.

## Tasks / Subtasks

- [ ] Task 1: Add reset button to SettingsView (AC: 1)
  - [ ] 1.1 Add a "Reset all training data" button at the bottom of the settings form, visually separated from other controls
  - [ ] 1.2 Style as destructive action: red/danger color scheme with Tailwind (`bg-red-600 hover:bg-red-700 text-white dark:bg-red-700 dark:hover:bg-red-800`), 44px touch target
  - [ ] 1.3 Add a warning description text below or near the button explaining the action is permanent

- [ ] Task 2: Add confirmation dialog (AC: 2,3)
  - [ ] 2.1 Use native HTML `<dialog>` element for the confirmation — semantic, accessible, supports `Escape` to cancel
  - [ ] 2.2 Dialog text: "This will permanently delete all training data, including your perceptual profile and comparison history. This cannot be undone."
  - [ ] 2.3 Two buttons: "Delete All Data" (destructive, red) and "Cancel" (neutral)
  - [ ] 2.4 Cancel closes dialog, no side effects
  - [ ] 2.5 Dialog must be keyboard-accessible: focus trapped inside when open, Escape closes

- [ ] Task 3: Implement reset handler (AC: 4,5,6)
  - [ ] 3.1 Retrieve contexts via `use_context()`: `db_store` signal, `profile`, `trend_analyzer`, `timeline`
  - [ ] 3.2 On confirm: `spawn_local` an async block that calls `IndexedDbStore::delete_all().await`
  - [ ] 3.3 On success: call `profile.borrow_mut().reset()`, `profile.borrow_mut().reset_matching()`, `trend_analyzer.borrow_mut().reset()`, `timeline.borrow_mut().reset()`
  - [ ] 3.4 On IndexedDB error: log via `web_sys::console::warn_1()` and show user-facing error (NFR8 — no silent data loss)
  - [ ] 3.5 Close dialog after reset completes
  - [ ] 3.6 Show brief success feedback (e.g., button text changes to "Data Reset" momentarily, or a subtle confirmation)

- [ ] Task 4: Verify and validate (AC: all)
  - [ ] 4.1 `cargo clippy -p domain` — zero warnings
  - [ ] 4.2 `cargo clippy -p web` — zero warnings
  - [ ] 4.3 `cargo test -p domain` — all tests pass
  - [ ] 4.4 `trunk build` — successful WASM compilation
  - [ ] 4.5 Manual browser smoke test: open Settings, verify reset button visible, click reset, verify dialog appears, cancel → no change, confirm → data cleared, start training → cold-start difficulty

## Dev Notes

### Core Approach: Add Reset UI to SettingsView + Wire Existing Domain Reset Methods

This story adds a destructive "Reset all training data" button with a confirmation dialog to the existing `SettingsView`, then wires it to the already-implemented domain reset infrastructure. **The domain layer is fully prepared** — all reset methods exist and are tested. The work is entirely in the web crate's `SettingsView` component.

**Settings are NOT reset.** FR29 says "reset all training data" — this means IndexedDB comparison records and in-memory profile/trend/timeline state. User settings (note range, duration, pitch, intervals, tuning system, etc.) in localStorage are preserved.

### Existing Infrastructure (DO NOT recreate)

The following already exists and must be reused — do NOT reimplement any of this:

- **`Resettable` trait** — `domain/src/ports.rs` (line 36): `fn reset(&mut self)`. Already exported via `domain::Resettable`.
- **`PerceptualProfile::reset(&mut self)`** — `domain/src/profile.rs` (line 221): clears all 128 `PerceptualNote` entries back to `Default`. Has tests at line 442.
- **`PerceptualProfile::reset_matching(&mut self)`** — `domain/src/profile.rs` (line 226): zeroes pitch matching accumulators. Has tests.
- **`TrendAnalyzer::reset(&mut self)`** — `domain/src/trend.rs` (line 70): clears `abs_offsets` vec. Has test at line 161.
- **`ThresholdTimeline::reset(&mut self)`** — `domain/src/timeline.rs` (line 124): clears `data_points` vec. Has test at line 202.
- **`TrainingDataStore::delete_all`** — `domain/src/ports.rs` (line 73): trait method for deleting all stored records.
- **`IndexedDbStore::delete_all(&self)`** — `web/src/adapters/indexeddb_store.rs` (lines 129-148): fully implemented, calls `store.clear()` on the IndexedDB object store, async, returns `Result<(), StorageError>`. Currently has `#[allow(dead_code)]` — remove this attribute.
- **`StorageError::DeleteFailed`** — `domain/src/ports.rs` (line 57): error variant for delete operations.

### Context Wiring Pattern (from `app.rs`)

All required state is already provided via `provide_context` in `App` (lines 32-37 of `web/src/app.rs`):

| Context Type | How to Access | What to Call |
|---|---|---|
| `SendWrapper<Rc<RefCell<PerceptualProfile>>>` | `use_context::<SendWrapper<Rc<RefCell<PerceptualProfile>>>>()` | `.borrow_mut().reset()` then `.borrow_mut().reset_matching()` |
| `SendWrapper<Rc<RefCell<TrendAnalyzer>>>` | `use_context::<SendWrapper<Rc<RefCell<TrendAnalyzer>>>>()` | `.borrow_mut().reset()` |
| `SendWrapper<Rc<RefCell<ThresholdTimeline>>>` | `use_context::<SendWrapper<Rc<RefCell<ThresholdTimeline>>>>()` | `.borrow_mut().reset()` |
| `RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage>` | `use_context::<RwSignal<Option<Rc<IndexedDbStore>>>>()` | `.get_untracked()` then `.delete_all().await` |

**Note:** `SendWrapper` dereferences to the inner type via `Deref`, so `Rc::clone(&*context_value)` gets the inner `Rc`. Follow the same pattern as `comparison_view.rs` (lines 42-51) for context retrieval.

### Reset Flow Implementation

```rust
// In SettingsView, after user confirms in dialog:
let db_store = use_context::<RwSignal<Option<Rc<IndexedDbStore>>>>()
    .expect("db_store context");
let profile = use_context::<SendWrapper<Rc<RefCell<PerceptualProfile>>>>()
    .expect("profile context");
let trend = use_context::<SendWrapper<Rc<RefCell<TrendAnalyzer>>>>()
    .expect("trend context");
let timeline = use_context::<SendWrapper<Rc<RefCell<ThresholdTimeline>>>>()
    .expect("timeline context");

spawn_local(async move {
    if let Some(store) = db_store.get_untracked() {
        match store.delete_all().await {
            Ok(()) => {
                profile.borrow_mut().reset();
                profile.borrow_mut().reset_matching();
                trend.borrow_mut().reset();
                timeline.borrow_mut().reset();
                // Show success feedback, close dialog
            }
            Err(e) => {
                web_sys::console::warn_1(
                    &format!("Failed to delete training data: {e:?}").into()
                );
                // Show error to user (NFR8)
            }
        }
    }
});
```

### Confirmation Dialog Pattern

Use native HTML `<dialog>` element — it's semantic, accessible, and supports `Escape` to close:

```rust
// Create a NodeRef to the dialog element
let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
let show_dialog = RwSignal::new(false);

// Open dialog
let open_dialog = move |_| {
    if let Some(dialog) = dialog_ref.get() {
        let _ = dialog.show_modal(); // Centers + adds backdrop
    }
};

// In view!:
view! {
    <dialog node_ref=dialog_ref class="...">
        <p>"This will permanently delete all training data..."</p>
        <button on:click=handle_confirm>"Delete All Data"</button>
        <button on:click=handle_cancel>"Cancel"</button>
    </dialog>
    <button on:click=open_dialog class="destructive styling...">"Reset all training data"</button>
}
```

**`<dialog>` benefits:**
- `show_modal()` traps focus and adds backdrop
- Native `Escape` key closes the dialog (fires `close` event)
- Screen reader support built in
- No custom modal implementation needed

### Tailwind Styling for Destructive Action

Follow the existing form control patterns from stories 2.1 and 2.2, with destructive action styling:

```
// Reset button (at bottom of settings, visually separated)
class="mt-8 w-full min-h-[44px] rounded-lg bg-red-600 px-4 py-3 font-semibold text-white
       hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2
       dark:bg-red-700 dark:hover:bg-red-800 dark:ring-offset-gray-900"

// Dialog backdrop
class="backdrop:bg-black/50 rounded-lg p-6 max-w-md mx-auto
       bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"

// Confirm (destructive) button in dialog
class="min-h-[44px] rounded-lg bg-red-600 px-4 py-2 font-semibold text-white
       hover:bg-red-700 dark:bg-red-700 dark:hover:bg-red-800"

// Cancel button in dialog
class="min-h-[44px] rounded-lg bg-gray-200 px-4 py-2 font-semibold text-gray-700
       hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
```

### What NOT to Implement (Separate Stories)

- **Profile visualization** — Story 3.2. The empty state after reset is just the current absence of data.
- **Profile preview on start page** — Story 3.3. After reset, the existing start page simply won't show profile data.
- **Settings reset** — FR29 is about "training data" only. Do NOT clear localStorage settings.
- **Undo/restore** — Not in scope. The confirmation dialog is the safety mechanism.

### UX Requirements (from UX Spec)

- **Destructive action confirmation:** "Reset all training data" is the ONLY destructive action in the app and requires a confirmation dialog before executing (UX spec: Settings View → Form Behavior).
- **No save/cancel pattern:** The reset button is a standalone action, not part of the auto-save settings pattern.
- **Dark mode:** All Tailwind color utilities must use `dark:` variants.
- **Touch targets:** Minimum 44x44px on all interactive elements.
- **Keyboard accessible:** Button reachable via Tab, dialog keyboard-navigable, Escape closes dialog.
- **No gamification/session framing:** No "are you sure you want to lose your progress?" guilt language. Factual description only: "all training data will be permanently deleted."

### Project Structure Notes

- Modified: `web/src/components/settings_view.rs` — Add reset button, confirmation dialog, reset handler with context wiring
- Modified: `web/src/adapters/indexeddb_store.rs` — Remove `#[allow(dead_code)]` from `delete_all()`
- No new files needed
- No domain crate changes needed — all reset methods already exist and are tested
- New imports in `settings_view.rs`: `SendWrapper`, `Rc`, `RefCell`, `spawn_local`, `IndexedDbStore`, `PerceptualProfile`, `TrendAnalyzer`, `ThresholdTimeline`, `web_sys`

### References

- [Source: docs/planning-artifacts/epics.md#Story 2.3] — Full acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/architecture.md#Storage Boundaries] — IndexedDB `peach` database with `comparison_records` object store
- [Source: docs/planning-artifacts/architecture.md#Error Handling] — Storage write/delete failures must inform user (NFR8)
- [Source: docs/planning-artifacts/ux-design-specification.md#Settings View] — "Reset all training data" button with confirmation dialog
- [Source: docs/planning-artifacts/ux-design-specification.md#Form Behavior] — Destructive action requires explicit confirmation
- [Source: docs/project-context.md#Error Handling] — `let _ = fallible_call()` is forbidden; every Result must be handled
- [Source: domain/src/ports.rs#Resettable] — `Resettable` trait (line 36), `TrainingDataStore::delete_all` (line 73), `StorageError::DeleteFailed` (line 57)
- [Source: domain/src/profile.rs#reset] — `PerceptualProfile::reset()` (line 221), `reset_matching()` (line 226)
- [Source: domain/src/trend.rs#reset] — `TrendAnalyzer::reset()` (line 70)
- [Source: domain/src/timeline.rs#reset] — `ThresholdTimeline::reset()` (line 124)
- [Source: web/src/adapters/indexeddb_store.rs#delete_all] — `IndexedDbStore::delete_all()` (line 129), fully implemented with `#[allow(dead_code)]`
- [Source: web/src/app.rs#provide_context] — All reset targets provided as contexts (lines 25-37)
- [Source: web/src/components/settings_view.rs] — Existing settings form to extend with reset section
- [Source: web/src/components/comparison_view.rs#context_retrieval] — Pattern for `use_context` with `SendWrapper` types

### Previous Story Intelligence (from Story 2.2)

**Patterns established:**
- `RwSignal` for UI state, initialized from localStorage on mount — same for reset button state (normal/resetting/success)
- `on:click` handler for actions — same pattern for reset button
- `LocalStorageSettings` methods are public — but NOT needed here (settings are preserved)
- All Tailwind classes follow pattern: dark mode variants, 44px touch targets, focus rings, consistent spacing
- `<fieldset>` and `<legend>` for grouped form sections (added in code review for 2.2) — use same pattern for reset section

**Code review learnings from stories 2.1 and 2.2:**
- Accent colors on form controls: `accent-indigo-600 dark:accent-indigo-400`
- Dark mode ring offsets: `dark:ring-offset-gray-900`
- Use `<fieldset>`/`<legend>` for accessible grouped controls
- Disabled state for interactive elements when constraints apply
- Error logging via `web_sys::console::warn_1()`

### Git Intelligence

Recent commit pattern: "Implement story X" → "Apply code review fixes for story X and mark as done". Follow same pattern.

Files most relevant to this story:
- `web/src/components/settings_view.rs` — Add reset button + confirmation dialog + handler
- `web/src/adapters/indexeddb_store.rs` — Remove `#[allow(dead_code)]` from `delete_all()`

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
