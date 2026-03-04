# Story 4.2: Vertical Pitch Slider

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want a vertical slider that I can drag to tune a note by ear,
so that I can practice pitch matching using only my hearing as a guide.

## Acceptance Criteria

1. **AC1 — Vertical layout:** Given the VerticalPitchSlider component, when rendered, then it is vertically oriented, occupying most of the training view height, has a large thumb/handle for comfortable dragging, and has no markings, tick marks, or center indicators.

2. **AC2 — Mouse drag:** Given the slider is in active state, when I drag the thumb with a mouse, then the slider value changes continuously as I drag, with up = sharper (positive) and down = flatter (negative).

3. **AC3 — Touch drag:** Given the slider is in active state, when I drag with touch, then the slider responds to touch drag identically to mouse drag.

4. **AC4 — Keyboard fine adjust:** Given the slider is in active state, when I press Arrow Up, then the pitch adjusts by a fine increment (upward/sharper), and when I press Arrow Down, then the pitch adjusts by a fine increment (downward/flatter).

5. **AC5 — Release commits:** Given the slider is being dragged, when I release (mouse up / touch end), then the commit event fires with the current slider value.

6. **AC6 — Keyboard commit:** Given the slider has focus and a value has been adjusted via keyboard, when I press Enter or Space, then the commit event fires with the current slider value.

7. **AC7 — Center reset:** Given the slider, when it starts a new challenge, then it always begins at the same physical center position regardless of the pitch offset.

8. **AC8 — Disabled state:** Given the slider in inactive state (during reference playback), when displayed, then it appears dimmed/disabled and does not respond to any input.

9. **AC9 — ARIA accessibility:** Given the slider, when I inspect the HTML, then it has ARIA role "slider", aria-label "Pitch adjustment", aria-orientation "vertical", aria-valuemin/aria-valuemax/aria-valuenow reflecting the current state, and it is keyboard-operable.

10. **AC10 — Touch targets:** Given the slider thumb/handle, when measured, then it meets the minimum 44x44px touch target requirement.

## Tasks / Subtasks

- [ ] Task 1: Create `web/src/components/pitch_slider.rs` with component skeleton (AC: 1, 9)
  - [ ] 1.1 Define `VerticalPitchSlider` component function with props: `enabled: Signal<bool>`, `on_change: impl Fn(f64) + 'static`, `on_commit: impl Fn(f64) + 'static`, `reset_trigger: Signal<u32>`
  - [ ] 1.2 Render track container `<div>` with vertical layout (tall, narrow) and thumb `<div>` positioned by internal value signal
  - [ ] 1.3 Apply ARIA attributes: `role="slider"`, `aria-label="Pitch adjustment"`, `aria-orientation="vertical"`, `aria-valuemin="-1"`, `aria-valuemax="1"`, `aria-valuenow` bound to current value
  - [ ] 1.4 Set `tabindex="0"` for keyboard focusability

- [ ] Task 2: Implement pointer-based drag interaction (AC: 2, 3, 5, 10)
  - [ ] 2.1 On `pointerdown`: if enabled, compute value from Y position, set dragging flag, call `setPointerCapture()` on the track element, fire `on_change(value)`
  - [ ] 2.2 On `pointermove`: if dragging, compute value from Y position, update thumb position, fire `on_change(value)`
  - [ ] 2.3 On `pointerup`: if dragging, clear dragging flag, call `releasePointerCapture()`, fire `on_commit(value)`
  - [ ] 2.4 Value computation: map pointer Y relative to track bounds to [-1.0, +1.0] where top = +1.0 (sharper) and bottom = -1.0 (flatter), clamped to range

- [ ] Task 3: Implement keyboard interaction (AC: 4, 6)
  - [ ] 3.1 On `keydown` ArrowUp: increment value by fine step (0.05), clamp to [-1.0, +1.0], fire `on_change(value)`
  - [ ] 3.2 On `keydown` ArrowDown: decrement value by fine step (0.05), clamp to [-1.0, +1.0], fire `on_change(value)`
  - [ ] 3.3 On `keydown` Enter or Space: fire `on_commit(value)` with current value
  - [ ] 3.4 Prevent default on all handled keys to avoid page scrolling

- [ ] Task 4: Implement center reset and disabled state (AC: 7, 8)
  - [ ] 4.1 Watch `reset_trigger` signal — when it changes, reset internal value to 0.0 and update thumb to center position
  - [ ] 4.2 When `enabled` is false: apply dimmed/disabled styling (opacity, pointer-events none), ignore all pointer and keyboard events
  - [ ] 4.3 When `enabled` transitions from false to true: ensure value is at center (0.0)

- [ ] Task 5: Apply Tailwind styling (AC: 1, 8, 10)
  - [ ] 5.1 Track: tall vertical container (`h-[60vh]` or similar), narrow width, rounded background, centered in parent
  - [ ] 5.2 Thumb: large circular handle (min 48x48px), positioned absolutely within track, `touch-action: none` to prevent browser scroll interference
  - [ ] 5.3 Disabled state: reduced opacity (`opacity-40`), `cursor-not-allowed`
  - [ ] 5.4 Active state: full opacity, `cursor-grab` (normal) / `cursor-grabbing` (dragging)
  - [ ] 5.5 Focus ring on track element for keyboard focus visibility
  - [ ] 5.6 Dark mode: appropriate color variants

- [ ] Task 6: Register component in mod.rs (AC: all)
  - [ ] 6.1 Add `mod pitch_slider;` to `web/src/components/mod.rs`
  - [ ] 6.2 Add `pub use pitch_slider::VerticalPitchSlider;` to exports

- [ ] Task 7: Verify and validate (AC: all)
  - [ ] 7.1 `cargo clippy -p web` — zero warnings
  - [ ] 7.2 `trunk build` — successful WASM compilation
  - [ ] 7.3 Verify component renders with a temporary test mount (can add to PitchMatchingView stub temporarily for visual testing, then revert)

## Dev Notes

### Core Approach: Custom Pointer-Events-Based Vertical Slider Component

This story creates a standalone `VerticalPitchSlider` Leptos component. It is a pure UI component that emits normalized values — it does NOT connect to the `PitchMatchingSession` domain state machine (that's story 4.3).

**Why custom instead of `<input type="range">`:** The requirements demand no markings, no tick marks, a blank instrument appearance, and consistent cross-browser behavior. Native range inputs have notoriously inconsistent styling across browsers, especially when vertical. A custom component using the Pointer Events API provides full control and unified mouse/touch handling.

### Component Props Design

```rust
#[component]
pub fn VerticalPitchSlider(
    /// Whether the slider accepts input. When false, appears dimmed.
    enabled: Signal<bool>,
    /// Called continuously during drag with normalized value [-1.0, +1.0].
    /// Up = positive (sharper), down = negative (flatter).
    on_change: Box<dyn Fn(f64)>,
    /// Called when the user releases the slider (commit). Also fired on Enter/Space.
    on_commit: Box<dyn Fn(f64)>,
    /// Increment this signal to reset the slider to center (0.0).
    reset_trigger: Signal<u32>,
) -> impl IntoView
```

**Note on callback types:** Leptos components cannot use generic `impl Fn()` in props directly because component props must be `'static`. Use `Box<dyn Fn(f64)>` or `Callback<f64>` (Leptos provides `Callback<T>` for component callback props). Check which pattern the existing codebase uses — ComparisonView uses closures and `Rc`, but those are internal to the component. For component props, Leptos `Callback` is the idiomatic choice if available in 0.8.x.

**Alternative prop approach:** If `Callback` proves problematic, use `WriteSignal<Option<f64>>` for `on_change` and `on_commit` — the parent writes `None` initially, the slider writes `Some(value)`. The parent watches these signals with `Effect`. This avoids callback boxing entirely.

### Pointer Events Strategy

Use the Pointer Events API (`PointerEvent`) for unified mouse + touch handling. This is a modern, well-supported API that eliminates the need for separate mouse/touch event handlers.

**Pointer capture** (`element.setPointerCapture(pointerId)`) is critical: it ensures `pointermove` and `pointerup` events continue firing on the slider element even when the pointer moves outside its bounds. Without capture, dragging outside the slider would lose tracking.

**`touch-action: none`** CSS on the slider container prevents the browser from intercepting touch events for scrolling/zooming. This is essential for touch drag to work.

### Value Computation

```
Pointer Y position → normalized value [-1.0, +1.0]

Given:
  - track_top: top edge Y of the track container (getBoundingClientRect().top)
  - track_height: height of the track container
  - pointer_y: current pointer Y (event.clientY)

Relative position (0.0 = top, 1.0 = bottom):
  relative = clamp((pointer_y - track_top) / track_height, 0.0, 1.0)

Value (top = +1.0 sharper, bottom = -1.0 flatter):
  value = 1.0 - 2.0 * relative

Thumb position (CSS top percentage):
  thumb_top_pct = relative * 100.0
```

**Important:** The slider maps top-to-bottom as +1.0 to -1.0 (up = sharper, down = flatter). This is the musically intuitive direction.

### Keyboard Fine Step Size

Arrow Up/Down adjusts by **0.05** per keypress. Since the full range is [-1.0, +1.0] mapping to ±20 cents, each keypress = 1 cent. This provides fine control for keyboard-only users.

### web-sys Features Required

The following `web-sys` features are needed (add to `web/Cargo.toml` if not already present):

- `PointerEvent` — for `pointer_id()`, `client_y()` methods
- `Element` — for `set_pointer_capture()`, `release_pointer_capture()`, `get_bounding_client_rect()`
- `DomRect` — for bounding rect `top()`, `height()` methods
- `HtmlElement` — for focus management

Check existing `web/Cargo.toml` web-sys features — `PointerEvent` may not be enabled yet since ComparisonView uses only click and keyboard events.

### Existing Pattern: How ComparisonView Handles Events

ComparisonView uses:
- `on:click` Leptos event attributes for button clicks
- `Closure<dyn Fn(KeyboardEvent)>` + `document.add_event_listener` for global keyboard
- `Closure<dyn FnMut(web_sys::Event)>` for visibility change
- `StoredValue::new_local()` to keep closures alive

For the slider, pointer events should be handled directly on the element using Leptos `on:pointerdown`, `on:pointermove`, `on:pointerup` attributes rather than document-level listeners, because pointer capture keeps events on the element.

Keyboard events for the slider should use `on:keydown` on the slider element itself (not document-level), since the slider will have focus when the user interacts with it via keyboard.

### Styling Approach

The slider is intentionally minimal — "a blank instrument" per the UX spec. Tailwind classes with dark mode variants:

- **Track:** A subtle vertical bar (e.g., `w-2 rounded-full bg-gray-200 dark:bg-gray-700`) centered in a wider touch area
- **Thumb:** A larger circle (e.g., `w-12 h-12 rounded-full bg-indigo-600 dark:bg-indigo-400 shadow-md`) positioned absolutely
- **Disabled:** `opacity-40` on the entire component, `pointer-events-none`
- **Active:** `opacity-100`, thumb shows `cursor-grab` / `cursor-grabbing`
- **Container:** `h-[60vh]` or `flex-1` to fill available height, `relative` for absolute thumb positioning, `touch-action-none` for touch support

### What NOT to Implement

- **No domain session integration** — the slider only emits values via callbacks. Story 4.3 wires it to `PitchMatchingSession.adjust_pitch()`.
- **No audio playback** — the slider does not play or adjust any audio. That's story 4.3.
- **No feedback display** — the pitch matching feedback indicator is part of story 4.3.
- **No training loop** — the async loop orchestrating reference → slider → commit → feedback is story 4.3.
- **No persistence** — saving pitch matching records to IndexedDB is story 4.4.
- **No visual proximity feedback during tuning** — per UX spec, the ear is the only guide. No color changes, no markers, no "getting warmer" indicators.

### Project Structure Notes

**New files:**
- `web/src/components/pitch_slider.rs` — VerticalPitchSlider component

**Modified files:**
- `web/src/components/mod.rs` — Add module declaration and re-export
- `web/Cargo.toml` — May need additional `web-sys` features (`PointerEvent`, `DomRect`)

**No changes to:**
- Domain crate (no domain logic in this story)
- `web/src/components/pitch_matching_view.rs` (remains a stub until story 4.3)
- `web/src/bridge.rs` (no observer changes)
- `web/src/adapters/` (no adapter changes)

### References

- [Source: docs/planning-artifacts/epics.md#Story 4.2] — Acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/ux-design-specification.md#Vertical Pitch Slider] — Component specification, states, accessibility
- [Source: docs/planning-artifacts/ux-design-specification.md#Pitch Matching Loop Mechanics] — Interaction flow and timing
- [Source: docs/planning-artifacts/architecture.md#Component Architecture] — VerticalPitchSlider listed as custom component in `pitch_slider.rs`
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture] — Signal-based reactivity, component props pattern
- [Source: docs/ios-reference/domain-blueprint.md#§7.2] — adjustPitch(value) takes [-1.0, +1.0], ±20 cents mapping
- [Source: docs/project-context.md] — Leptos component rules, naming conventions, accessibility requirements

### Previous Story Intelligence (from Story 4.1)

**Patterns to follow:**
- Component registration in `mod.rs`: `mod pitch_slider;` + `pub use pitch_slider::VerticalPitchSlider;`
- Tailwind dark mode variants: `dark:bg-*`, `dark:text-*`
- ARIA attributes directly in `view!` macro: `aria-label`, `role`, etc.
- Touch target minimum 44x44px (existing buttons use `min-h-11 min-w-11` which is 44x44)

**Code review learnings from previous stories:**
- Collapse `else { if }` blocks per clippy `collapsible_else_if` lint
- String ownership: clone before use if needed in multiple places
- Leptos event attributes use `on:eventname` syntax directly in view macro
- `StoredValue::new_local()` to keep JS closures alive for component lifetime

**Domain API for story 4.3 integration context (NOT for this story):**
- `PitchMatchingSession::adjust_pitch(value: f64) -> Option<Frequency>` — value in [-1.0, +1.0]
- `PitchMatchingSession::commit_pitch(value: f64, timestamp: String)` — called on slider release
- Slider `on_change` → `adjust_pitch`, slider `on_commit` → `commit_pitch`

### Git Intelligence

Recent commits (last 5):
- `c0f028d` Apply code review fixes for story 4.1 and mark as done
- `a040bf5` Implement story 4.1 Pitch Matching Session State Machine
- `5cab0a8` Rename note1/note2 to referenceNote/targetNote across codebase
- `4f1597e` Add quick tech spec to rename note1/note2 terminology
- `cf70b5a` Fix is_target_higher incorrectly ignoring interval for non-unison comparisons

**Commit pattern:** "Add story X.Y ..." for story creation, "Implement story X.Y ..." for implementation.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
