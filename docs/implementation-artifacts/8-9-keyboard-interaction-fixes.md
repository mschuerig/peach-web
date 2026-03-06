# Story 8.9: Keyboard Interaction Fixes

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want keyboard shortcuts on the pitch comparison screen to only trigger without browser-modifier keys, and all help/info screens to close when Escape is pressed,
so that I can use standard browser shortcuts like Cmd-L / Ctrl-L without interference, and dismiss overlays with Escape consistently.

## Acceptance Criteria

1. On the pitch comparison screen, pressing H/h or L/l answers "higher"/"lower" ONLY when no modifier keys are held (no Ctrl, no Cmd/Meta, no Alt) — Shift alone is allowed
2. On the pitch comparison screen, pressing ArrowUp/ArrowDown answers ONLY when no modifier keys are held — Shift alone is allowed
3. Cmd-L (macOS) and Ctrl-L (Windows/Linux) no longer trigger the "lower" answer and correctly focus the browser address bar
4. The InfoView (`/info` route) closes (navigates back to start page) when Escape is pressed
5. All help modals already close on Escape (native `<dialog>` behavior) — verify this still works
6. On the pitch matching screen, Escape handling remains unchanged (already interrupts training correctly)
7. No behavioral changes to any other keyboard interactions

## Tasks / Subtasks

- [ ] Task 1: Add modifier-key guard to pitch comparison keydown handler (AC: #1, #2, #3)
  - [ ] 1.1 In `web/src/components/pitch_comparison_view.rs`, modify the keydown closure (~line 325-345) to check `ev.ctrl_key()`, `ev.meta_key()`, and `ev.alt_key()` before matching H/L/ArrowUp/ArrowDown
  - [ ] 1.2 If any of Ctrl, Meta, or Alt is pressed, return early without calling `ev.prevent_default()` or `on_answer()` — let the browser handle the event
  - [ ] 1.3 Shift is explicitly allowed (users may have caps lock or press Shift+H/Shift+L)
  - [ ] 1.4 The Escape key handler in the same closure does NOT need the modifier guard (Escape with modifiers is not a standard browser shortcut)
- [ ] Task 2: Add Escape key handler to InfoView (AC: #4)
  - [ ] 2.1 In `web/src/components/info_view.rs`, add a document-level keydown event listener that navigates to `/` when Escape is pressed
  - [ ] 2.2 Follow the exact same pattern used in `pitch_comparison_view.rs` and `pitch_matching_view.rs`: `Closure::<dyn Fn(KeyboardEvent)>`, `document.add_event_listener_with_callback("keydown", ...)`, `StoredValue::new_local()` to keep closure alive
  - [ ] 2.3 Clean up the listener in `on_cleanup()` using the same pattern as the training views
  - [ ] 2.4 Use `leptos_router::hooks::use_navigate()` to navigate to `/` on Escape
- [ ] Task 3: Verify existing Escape behavior (AC: #5, #6, #7)
  - [ ] 3.1 Manual test: open help modal on pitch comparison screen, press Escape — modal closes
  - [ ] 3.2 Manual test: open help modal on pitch matching screen, press Escape — modal closes
  - [ ] 3.3 Manual test: open help modal on settings screen, press Escape — modal closes
  - [ ] 3.4 Manual test: Escape on pitch comparison training screen still interrupts training
  - [ ] 3.5 Manual test: Escape on pitch matching training screen still interrupts training
  - [ ] 3.6 Manual test: Cmd-L / Ctrl-L on pitch comparison screen focuses address bar (not "lower" answer)
  - [ ] 3.7 `cargo clippy --workspace` clean

## Dev Notes

### Key Implementation Detail: Modifier Guard Pattern

The modifier guard should be applied BEFORE the match statement in the pitch comparison keydown handler. The pattern is:

```rust
// Inside the keydown closure:
let has_modifier = ev.ctrl_key() || ev.meta_key() || ev.alt_key();
// Note: ev.shift_key() is intentionally NOT checked — Shift is allowed

match ev.key().as_str() {
    "Escape" => {
        // Escape does NOT check modifiers
        ev.prevent_default();
        (*interrupt)();
    }
    _ if has_modifier => {
        // Any other key with a modifier: let the browser handle it
        return;
    }
    "ArrowUp" | "h" | "H" => {
        ev.prevent_default();
        on_answer(true);
    }
    "ArrowDown" | "l" | "L" => {
        ev.prevent_default();
        on_answer(false);
    }
    _ => {}
}
```

This approach ensures Escape always works for training interruption while preventing H/L/Arrow from interfering with browser shortcuts.

### InfoView Escape Handler Pattern

The InfoView is a full-page route (`/info`), not a modal. To "close" it on Escape, add a document-level keydown listener that navigates back to `/`. Follow the same lifecycle pattern used in training views:

```rust
// In InfoView component body:
let navigate = use_navigate();
let keydown_handler = Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
    if ev.key() == "Escape" {
        ev.prevent_default();
        navigate("/", Default::default());
    }
});
// ... add_event_listener, StoredValue, on_cleanup pattern
```

### Files to Modify

| File | Change |
|---|---|
| `web/src/components/pitch_comparison_view.rs` | Add modifier guard to keydown handler (~line 325-345) |
| `web/src/components/info_view.rs` | Add Escape keydown listener with navigate-back |

### What NOT to Change

- `pitch_matching_view.rs` — Escape handler is fine as-is (no H/L keys used)
- `pitch_slider.rs` — Arrow keys bound to slider element via Leptos `on:keydown`, not document; modifier guard not needed for focused element interaction
- `help_content.rs` — Native `<dialog>` already handles Escape
- `settings_view.rs` — Dialogs already handle Escape natively

### web-sys Features Required

The `KeyboardEvent` methods `ctrl_key()`, `meta_key()`, `alt_key()` are available via `web_sys::KeyboardEvent` which is already imported in `pitch_comparison_view.rs`. No new web-sys features needed.

### Previous Story Intelligence

Story 8.8 (most recent) cleaned up export/import architecture. Story 8.7 extracted business logic from settings view. Neither directly impacts this story, but they confirm the pattern of keeping views thin and focused on presentation.

### Git Intelligence

Recent commits follow the pattern: "Implement story X.Y: short description". Code changes are clean, well-structured, and pass clippy.

### Project Structure Notes

- Alignment with project structure: keyboard handling lives in view components (appropriate — it's UI interaction code)
- The modifier guard is a view-level concern, not domain logic
- InfoView Escape handler follows established patterns from training views

### References

- [Source: web/src/components/pitch_comparison_view.rs:325-345] Keydown handler to add modifier guard
- [Source: web/src/components/pitch_comparison_view.rs:351-352] Event listener registration pattern
- [Source: web/src/components/pitch_comparison_view.rs:445-448] Cleanup pattern
- [Source: web/src/components/pitch_matching_view.rs:427-435] Reference pattern for Escape-only handler
- [Source: web/src/components/info_view.rs] InfoView — needs Escape handler added
- [Source: web/src/components/help_content.rs:103-109] Native dialog close handler — already works
- [Source: web/src/help_sections.rs:33] Help text documents H/L keys
- [Source: docs/project-context.md] Project coding conventions

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
