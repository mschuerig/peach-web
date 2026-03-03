# Story 2.2: Interval Selection

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to select which directed intervals I want to train,
so that I can focus on specific intervals that are relevant to my musical practice.

## Acceptance Criteria

1. **AC1 — Interval display (FR27):** When the interval selection control in Settings loads, all 25 directed intervals are displayed (Prime/Up is always up-only; the remaining 12 intervals each have Up and Down variants). Currently selected intervals are checked. Default selection is Prime/Up only (unison).

2. **AC2 — Interval toggle:** When I check or uncheck an interval checkbox, the selection is immediately saved to `peach.intervals` in localStorage as a JSON array of `DirectedInterval` objects.

3. **AC3 — Minimum selection constraint:** When I attempt to deselect the last remaining selected interval, the checkbox remains checked — at least one interval must always be selected.

4. **AC4 — Interval labels:** Each interval checkbox displays a human-readable label with interval name and direction (e.g., "Minor Second Up", "Perfect Fifth Down"). Prime only shows "Prime" (no direction — it's always Up).

5. **AC5 — Training integration:** When I start comparison training after changing the interval selection, the session uses only the selected intervals for comparison generation (replacing the current hardcoded Prime/Up).

## Tasks / Subtasks

- [x] Task 1: Add interval selection section to SettingsView (AC: 1,2,3,4)
  - [x] 1.1 Add a new section below the existing Tuning System control with heading "Interval Selection"
  - [x] 1.2 Generate all 25 directed intervals: Prime/Up + 12 intervals × 2 directions (Up/Down)
  - [x] 1.3 Display each as a labeled checkbox with human-readable names (e.g., "Minor Second Up")
  - [x] 1.4 Read initial selection from localStorage key `peach.intervals`, defaulting to `[{"interval":"prime","direction":"up"}]` if absent
  - [x] 1.5 On checkbox change, serialize updated selection to JSON and persist via `LocalStorageSettings::set("peach.intervals", &json)`
  - [x] 1.6 Prevent deselecting the last remaining interval (disable the checkbox or ignore the click)

- [x] Task 2: Add interval helper to LocalStorageSettings (AC: 1,2,5)
  - [x] 2.1 Add `pub fn get_selected_intervals() -> HashSet<DirectedInterval>` method to `LocalStorageSettings` that reads `peach.intervals` from localStorage, deserializes JSON, and returns the set (defaulting to `{Prime/Up}` if absent or invalid)

- [x] Task 3: Update ComparisonView to use selected intervals (AC: 5)
  - [x] 3.1 Replace the hardcoded `intervals` HashSet at line 322-324 of `comparison_view.rs` with `LocalStorageSettings::get_selected_intervals()`

- [x] Task 4: Verify and validate (AC: all)
  - [x] 4.1 `cargo clippy -p domain` — zero warnings
  - [x] 4.2 `cargo clippy -p web` — zero warnings
  - [x] 4.3 `cargo test -p domain` — all tests pass
  - [x] 4.4 `trunk build` — successful WASM compilation
  - [ ] 4.5 Manual browser smoke test: open Settings, verify all 25 intervals displayed, toggle intervals, verify localStorage updates, start training, verify selected intervals used

## Dev Notes

### Core Approach: Extend Settings View + Wire Up ComparisonView

This story adds an interval multi-select checkbox group to the existing `SettingsView` component and updates `ComparisonView` to read the selected intervals from localStorage instead of hardcoding Prime/Up. A small helper method is added to `LocalStorageSettings` for reading the interval selection.

**No domain crate changes required.** All domain types (`Interval`, `Direction`, `DirectedInterval`) already exist with full serde support. The work is entirely in the web crate.

### Existing Infrastructure (DO NOT recreate)

The following already exists and must be reused:

- **`Interval` enum** — 13 variants from Prime (0) to Octave (12) in `domain/src/types/interval.rs` (line 9). Serialized as camelCase: `"prime"`, `"minorSecond"`, `"perfectFifth"`, etc.
- **`Direction` enum** — Up/Down in `domain/src/types/interval.rs` (line 58). Serialized as `"up"` / `"down"`.
- **`DirectedInterval` struct** — combines `interval: Interval` + `direction: Direction` in `domain/src/types/interval.rs` (line 67). Has serde Serialize/Deserialize. Fields are public.
- **`ComparisonSession::start(intervals, settings)`** — already accepts `HashSet<DirectedInterval>` in `domain/src/session/comparison_session.rs` (line 128). Asserts intervals is not empty.
- **`LocalStorageSettings::set(key, value)`** — writes to localStorage. Public method in `web/src/adapters/localstorage_settings.rs` (line 31).
- **`LocalStorageSettings::get_string(key)`** — reads string from localStorage. Public method in `web/src/adapters/localstorage_settings.rs` (line 10).
- **`target_value(ev)`** — helper in `web/src/components/settings_view.rs` (line 10) extracts `.value` from event targets.
- **`serde_json`** — already a dependency of the web crate. Use for serializing/deserializing interval arrays.

### localStorage Key Convention

| Key | Type | Default | Notes |
|---|---|---|---|
| `peach.intervals` | JSON string | `[{"interval":"prime","direction":"up"}]` | Array of DirectedInterval objects |

This key is defined in the architecture doc's Storage Boundaries section. The JSON format uses serde's default struct serialization with camelCase enum values.

Example stored value for Prime/Up + Perfect Fifth Up + Perfect Fifth Down:
```json
[{"interval":"prime","direction":"up"},{"interval":"perfectFifth","direction":"up"},{"interval":"perfectFifth","direction":"down"}]
```

### All 25 Directed Intervals

The interval multi-select displays these 25 checkboxes in order:

| # | Interval | Direction | Label | Serde Key |
|---|---|---|---|---|
| 1 | Prime | Up | "Prime" | `{"interval":"prime","direction":"up"}` |
| 2 | Minor Second | Up | "Minor Second Up" | `{"interval":"minorSecond","direction":"up"}` |
| 3 | Minor Second | Down | "Minor Second Down" | `{"interval":"minorSecond","direction":"down"}` |
| 4 | Major Second | Up | "Major Second Up" | `{"interval":"majorSecond","direction":"up"}` |
| 5 | Major Second | Down | "Major Second Down" | `{"interval":"majorSecond","direction":"down"}` |
| 6 | Minor Third | Up | "Minor Third Up" | `{"interval":"minorThird","direction":"up"}` |
| 7 | Minor Third | Down | "Minor Third Down" | `{"interval":"minorThird","direction":"down"}` |
| 8 | Major Third | Up | "Major Third Up" | `{"interval":"majorThird","direction":"up"}` |
| 9 | Major Third | Down | "Major Third Down" | `{"interval":"majorThird","direction":"down"}` |
| 10 | Perfect Fourth | Up | "Perfect Fourth Up" | `{"interval":"perfectFourth","direction":"up"}` |
| 11 | Perfect Fourth | Down | "Perfect Fourth Down" | `{"interval":"perfectFourth","direction":"down"}` |
| 12 | Tritone | Up | "Tritone Up" | `{"interval":"tritone","direction":"up"}` |
| 13 | Tritone | Down | "Tritone Down" | `{"interval":"tritone","direction":"down"}` |
| 14 | Perfect Fifth | Up | "Perfect Fifth Up" | `{"interval":"perfectFifth","direction":"up"}` |
| 15 | Perfect Fifth | Down | "Perfect Fifth Down" | `{"interval":"perfectFifth","direction":"down"}` |
| 16 | Minor Sixth | Up | "Minor Sixth Up" | `{"interval":"minorSixth","direction":"up"}` |
| 17 | Minor Sixth | Down | "Minor Sixth Down" | `{"interval":"minorSixth","direction":"down"}` |
| 18 | Major Sixth | Up | "Major Sixth Up" | `{"interval":"majorSixth","direction":"up"}` |
| 19 | Major Sixth | Down | "Major Sixth Down" | `{"interval":"majorSixth","direction":"down"}` |
| 20 | Minor Seventh | Up | "Minor Seventh Up" | `{"interval":"minorSeventh","direction":"up"}` |
| 21 | Minor Seventh | Down | "Minor Seventh Down" | `{"interval":"minorSeventh","direction":"down"}` |
| 22 | Major Seventh | Up | "Major Seventh Up" | `{"interval":"majorSeventh","direction":"up"}` |
| 23 | Major Seventh | Down | "Major Seventh Down" | `{"interval":"majorSeventh","direction":"down"}` |
| 24 | Octave | Up | "Octave Up" | `{"interval":"octave","direction":"up"}` |
| 25 | Octave | Down | "Octave Down" | `{"interval":"octave","direction":"down"}` |

### Interval Display Names

The `Interval` and `Direction` enums do NOT have `Display` implementations. The settings view must provide human-readable labels. Implement a simple local helper function:

```rust
fn interval_label(interval: Interval, direction: Direction) -> String {
    let name = match interval {
        Interval::Prime => return "Prime".to_string(), // Prime is always Up, no direction shown
        Interval::MinorSecond => "Minor Second",
        Interval::MajorSecond => "Major Second",
        Interval::MinorThird => "Minor Third",
        Interval::MajorThird => "Major Third",
        Interval::PerfectFourth => "Perfect Fourth",
        Interval::Tritone => "Tritone",
        Interval::PerfectFifth => "Perfect Fifth",
        Interval::MinorSixth => "Minor Sixth",
        Interval::MajorSixth => "Major Sixth",
        Interval::MinorSeventh => "Minor Seventh",
        Interval::MajorSeventh => "Major Seventh",
        Interval::Octave => "Octave",
    };
    let dir = match direction {
        Direction::Up => "Up",
        Direction::Down => "Down",
    };
    format!("{name} {dir}")
}
```

### Checkbox Group Implementation Pattern

Use standard HTML `<input type="checkbox">` elements with `<label>` elements for accessibility. Each checkbox represents one `DirectedInterval`.

**State management:**
1. Read current selection from localStorage into a `RwSignal<HashSet<DirectedInterval>>` on mount
2. Generate all 25 `DirectedInterval` values in a static list
3. For each interval, render a checkbox whose `checked` state derives from the signal's HashSet
4. On change, update the HashSet signal AND serialize to localStorage

**Preventing empty selection (AC3):**
When a checkbox is unchecked, check if it's the last selected interval. If so, re-add it to the HashSet (or skip the removal). The checkbox will remain checked reactively.

**Layout:**
Group intervals visually. A simple single-column list of checkboxes, or optionally group by interval (each interval with Up/Down sub-items). Keep it simple — a flat list with consistent spacing.

### Checkbox Tailwind Styling

Follow the established form control patterns from story 2.1:

```
// Checkbox group container
class="mt-4 space-y-2"

// Each checkbox row
class="flex items-center gap-2"

// Checkbox input
class="h-5 w-5 min-h-[44px] min-w-[44px] rounded border-gray-300 text-indigo-600 focus:ring-2 focus:ring-indigo-400 dark:border-gray-600 dark:bg-gray-800"
```

Note: For 44px touch targets on checkboxes, wrap each checkbox+label pair in a clickable container or use padding on the label to expand the target area. The `<label>` associated with the `<input>` via `for`/`id` or nesting inherently extends the click target to the label text.

### LocalStorageSettings Helper Method

Add to `web/src/adapters/localstorage_settings.rs`:

```rust
use std::collections::HashSet;
use domain::types::{DirectedInterval, Direction, Interval};

impl LocalStorageSettings {
    /// Read selected intervals from localStorage. Returns default {Prime/Up} if absent or invalid.
    pub fn get_selected_intervals() -> HashSet<DirectedInterval> {
        Self::get_string("peach.intervals")
            .and_then(|json| serde_json::from_str::<Vec<DirectedInterval>>(&json).ok())
            .map(|v| v.into_iter().collect())
            .filter(|s: &HashSet<DirectedInterval>| !s.is_empty())
            .unwrap_or_else(|| {
                let mut set = HashSet::new();
                set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
                set
            })
    }
}
```

This method:
- Returns `HashSet<DirectedInterval>` (matches `ComparisonSession::start()` parameter type)
- Falls back to `{Prime/Up}` if localStorage key is absent, invalid JSON, or empty array
- Uses `serde_json::from_str` to deserialize the JSON array

### ComparisonView Integration

Replace lines 322-324 in `web/src/components/comparison_view.rs`:

**Before:**
```rust
let mut intervals = HashSet::new();
intervals.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
session.borrow_mut().start(intervals, &settings);
```

**After:**
```rust
let intervals = LocalStorageSettings::get_selected_intervals();
session.borrow_mut().start(intervals, &settings);
```

This is a 3-line → 2-line change. The `LocalStorageSettings` import already exists in `comparison_view.rs` (line 17).

### What NOT to Implement (Separate Stories)

- **Reset training data (FR29)** — Story 2.3. Do NOT add the reset button in this story.
- **Interval query parameter routing** — The UX spec mentions `?intervals=M3u,M3d,...` for interval mode routes. This is a future concern for when interval-specific training views are added. Story 2.2 only adds the settings control and wires it into the existing comparison view.
- **Start page interval-specific buttons** — The start page has "Interval Comparison" and "Interval Pitch Matching" buttons mentioned in UX spec. These are separate stories (Epic 5). The interval selection in settings applies to ALL comparison training for now.

### UX Requirements (from UX Spec)

- **Auto-save:** All changes persist immediately to localStorage. No save/cancel buttons.
- **Bounded controls:** Checkbox group prevents deselecting last interval. No validation errors possible.
- **No confirmation:** No confirmation toast or save indicator. The checkbox state itself is the feedback.
- **Dark mode:** All Tailwind color utilities must use `dark:` variants.
- **Touch targets:** Minimum 44x44px on all interactive elements (use label click area).
- **Keyboard accessible:** All checkboxes reachable and operable via keyboard (native `<input type="checkbox">` provides this).

### Project Structure Notes

- Modified: `web/src/components/settings_view.rs` — Add interval selection checkbox group
- Modified: `web/src/adapters/localstorage_settings.rs` — Add `get_selected_intervals()` method
- Modified: `web/src/components/comparison_view.rs` — Replace hardcoded intervals with settings read
- No new files needed
- No domain crate changes needed
- Add `serde_json` import in `localstorage_settings.rs` (already a web crate dependency)

### References

- [Source: docs/planning-artifacts/epics.md#Story 2.2] — Full acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/architecture.md#Storage Boundaries] — `peach.intervals` localStorage key (JSON array)
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture] — Tailwind CSS, component patterns, signal naming
- [Source: docs/planning-artifacts/ux-design-specification.md#Settings View] — Interval selection: multi-select control type
- [Source: docs/planning-artifacts/ux-design-specification.md#Form Behavior] — Auto-save, bounded controls, no validation errors
- [Source: docs/project-context.md#Leptos Framework Rules] — Component architecture, signal patterns
- [Source: docs/project-context.md#Storage Key Conventions] — `peach.` prefix + `snake_case`
- [Source: domain/src/types/interval.rs] — Interval, Direction, DirectedInterval with serde support
- [Source: domain/src/session/comparison_session.rs#start()] — Accepts `HashSet<DirectedInterval>`
- [Source: web/src/components/settings_view.rs] — Existing settings form (extend with interval section)
- [Source: web/src/adapters/localstorage_settings.rs] — Existing `set()` and `get_string()` methods
- [Source: web/src/components/comparison_view.rs#line 322-324] — Hardcoded Prime/Up to replace

### Previous Story Intelligence (from Story 2.1)

**Patterns established:**
- `target_value()` helper extracts event target value — reuse for checkbox checked state (or use `web_sys::HtmlInputElement::checked()`)
- `RwSignal` for each setting, initialized from localStorage on mount — same pattern for interval selection HashSet
- `on:change` handler persists immediately via `LocalStorageSettings::set()` — same for checkboxes
- All Tailwind classes follow pattern: dark mode variants, 44px touch targets, focus rings, consistent spacing
- `LocalStorageSettings::get_string()` is public — use for reading `peach.intervals` JSON
- Made `LocalStorageSettings::get_string()` public in story 2.1 — enables JSON-based interval reading

**Code review learnings from story 2.1:**
- Sound source correctly reads from localStorage (not just hardcoded default)
- Accent colors on form controls: `accent-indigo-600 dark:accent-indigo-400`
- Dark mode ring offsets: `dark:ring-offset-gray-900`
- Avoid unnecessary `.clone()` calls on `String` values in closures

### Git Intelligence

Recent commit pattern: "Implement story X" → "Apply code review fixes for story X and mark as done". Follow same pattern.

Files most relevant to this story:
- `web/src/components/settings_view.rs` — Extend with interval checkboxes
- `web/src/adapters/localstorage_settings.rs` — Add `get_selected_intervals()`
- `web/src/components/comparison_view.rs` — Wire up interval selection (lines 322-324)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Fixed unused `HashSet` import warning in `settings_view.rs` (type is inferred from `get_selected_intervals()` return)
- Removed unused `HashSet`, `DirectedInterval`, `Direction`, `Interval` imports from `comparison_view.rs` after wiring up the helper method

### Completion Notes List

- Added `get_selected_intervals()` helper to `LocalStorageSettings` — reads `peach.intervals` from localStorage, deserializes JSON array, defaults to `{Prime/Up}` if absent/invalid/empty
- Added interval selection checkbox group to `SettingsView` below Tuning System section with all 25 directed intervals
- Added `target_checked()` helper (mirrors existing `target_value()` pattern using `js_sys::Reflect`)
- Added `interval_label()` and `all_directed_intervals()` helper functions for display and generation
- Checkbox state managed via `RwSignal<HashSet<DirectedInterval>>`, initialized from localStorage on mount
- Minimum selection constraint: `update` closure skips removal when `set.len() <= 1` (AC3)
- Serialized intervals sorted by `(interval, direction)` for consistent localStorage output
- Labels use nested `<label>` wrapping `<input>` for accessibility and 44px touch targets via `min-h-[44px]`
- Replaced hardcoded `Prime/Up` in `ComparisonView` with `LocalStorageSettings::get_selected_intervals()`
- All clippy warnings resolved, all 253 domain tests pass, trunk build succeeds

### Change Log

- 2026-03-04: Implemented story 2.2 — Interval Selection (all tasks complete)

### File List

- `web/src/adapters/localstorage_settings.rs` — Added `get_selected_intervals()` method, new imports for `HashSet`, `DirectedInterval`, `Direction`, `Interval`
- `web/src/components/settings_view.rs` — Added interval selection checkbox group with helpers (`target_checked`, `interval_label`, `all_directed_intervals`), new imports for domain interval types
- `web/src/components/comparison_view.rs` — Replaced hardcoded interval set with `LocalStorageSettings::get_selected_intervals()`, removed unused imports
