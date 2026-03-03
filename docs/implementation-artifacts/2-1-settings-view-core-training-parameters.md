# Story 2.1: Settings View — Core Training Parameters

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to configure my training note range, note duration, reference pitch, sound source, loudness variation, and tuning system,
so that I can personalize my training experience to match my instrument and goals.

## Acceptance Criteria

1. **AC1 — Settings display current values:** When the Settings view loads, all controls display their current values from localStorage. If no stored values exist, defaults are shown (noteRangeMin=36/C2, noteRangeMax=84/C6, noteDuration=1.0s, referencePitch=440Hz, varyLoudness=0%, tuningSystem=Equal Temperament).

2. **AC2 — Note range lower bound (FR22):** The lower bound control is constrained to valid MIDI notes and cannot exceed the current upper bound. Changes are immediately saved to `peach.note_range_min` in localStorage.

3. **AC3 — Note range upper bound (FR22):** The upper bound control is constrained to valid MIDI notes and cannot go below the current lower bound. Changes are immediately saved to `peach.note_range_max` in localStorage.

4. **AC4 — Note duration (FR23):** The duration control is constrained to 0.3-3.0 seconds. Changes are immediately saved to `peach.note_duration` in localStorage.

5. **AC5 — Reference pitch (FR24):** The user can select a reference pitch from predefined options (440Hz, 442Hz, 432Hz, 415Hz). Changes are immediately saved to `peach.reference_pitch` in localStorage.

6. **AC6 — Sound source (FR25):** The sound source dropdown shows available sources. Currently only "Sine Oscillator" is available (SoundFont deferred to Epic 5). Changes are immediately saved to `peach.sound_source` in localStorage.

7. **AC7 — Loudness variation (FR26):** The slider is constrained to 0-100%. Changes are immediately saved to `peach.vary_loudness` in localStorage as a 0.0-1.0 float.

8. **AC8 — Tuning system (FR28):** The user can select Equal Temperament or Just Intonation. Changes are immediately saved to `peach.tuning_system` in localStorage.

9. **AC9 — Settings take effect:** After changing any setting and returning to training, the new value takes effect on the next comparison.

10. **AC10 — Navigation from training (FR49):** When accessing Settings from within a training view, training has already stopped (handled by Epic 1 navigation). The fully functional Settings view loads.

11. **AC11 — Back navigation:** Navigating back from Settings returns to the Start Page.

12. **AC12 — Accessibility:** All controls have minimum 44x44px touch targets, are keyboard-accessible, and use semantic HTML (`<select>`, `<input>`, `<label>`). Dark mode is fully supported.

## Tasks / Subtasks

- [x] Task 1: Replace SettingsView stub with full component (AC: 1,11,12)
  - [x] 1.1 Replace the stub in `web/src/components/settings_view.rs` with the complete settings form layout
  - [x] 1.2 Add section heading "Settings" and back navigation link to "/"
  - [x] 1.3 Use semantic HTML: `<label>` + `<select>`, `<label>` + `<input type="range">` — no `<div>` click handlers
  - [x] 1.4 Apply Tailwind classes for layout (single column, centered, consistent spacing), dark mode (`dark:` variants), and 44x44px minimum touch targets (`min-h-11 min-w-11`)
  - [x] 1.5 Add `aria-label` attributes where labels are not visually adjacent to controls

- [x] Task 2: Note range controls (AC: 2,3)
  - [x] 2.1 Add two `<select>` dropdowns for lower bound and upper bound
  - [x] 2.2 Populate options with MIDI notes using `MIDINote::name()` for labels (e.g., "C2 (36)", "C#2 (37)", ..., "G9 (127)"). Practical range: MIDI 21 (A0) to 108 (C8)
  - [x] 2.3 Read initial values from `LocalStorageSettings` on mount — lower defaults to 36, upper defaults to 84
  - [x] 2.4 Constrain lower bound: max selectable value is current upper bound value
  - [x] 2.5 Constrain upper bound: min selectable value is current lower bound value
  - [x] 2.6 On change, call `LocalStorageSettings::set()` to persist immediately
  - [x] 2.7 Use Leptos signals to keep both dropdowns reactive to each other's changes

- [x] Task 3: Note duration control (AC: 4)
  - [x] 3.1 Add `<input type="range">` slider with min=0.3, max=3.0, step=0.1
  - [x] 3.2 Display current value as text next to slider (e.g., "1.0s")
  - [x] 3.3 Read initial value from localStorage key `peach.note_duration`, default 1.0
  - [x] 3.4 On change, persist via `LocalStorageSettings::set("peach.note_duration", &value.to_string())`

- [x] Task 4: Reference pitch control (AC: 5)
  - [x] 4.1 Add `<select>` dropdown with options: "440 Hz (Concert)", "442 Hz", "432 Hz", "415 Hz (Baroque)"
  - [x] 4.2 Read initial value from `peach.reference_pitch`, default 440.0
  - [x] 4.3 On change, persist the numeric Hz value to localStorage

- [x] Task 5: Sound source control (AC: 6)
  - [x] 5.1 Add `<select>` dropdown with single option: "Sine Oscillator"
  - [x] 5.2 Read initial value from `peach.sound_source`, default "oscillator:sine"
  - [x] 5.3 On change, persist to localStorage (currently only one option — control is present for future SoundFont support in Epic 5)

- [x] Task 6: Loudness variation control (AC: 7)
  - [x] 6.1 Add `<input type="range">` slider with min=0, max=100, step=1
  - [x] 6.2 Display current value as text next to slider (e.g., "25%")
  - [x] 6.3 Read initial value from `peach.vary_loudness` (stored as 0.0-1.0 float), convert to 0-100 for display
  - [x] 6.4 On change, convert percentage back to 0.0-1.0 and persist via `LocalStorageSettings::set()`

- [x] Task 7: Tuning system control (AC: 8)
  - [x] 7.1 Add `<select>` dropdown with options: "Equal Temperament", "Just Intonation"
  - [x] 7.2 Read initial value from `peach.tuning_system`, default "equalTemperament"
  - [x] 7.3 On change, persist value — use `"equalTemperament"` or `"justIntonation"` as serialized values (matching the existing `serde(rename_all = "camelCase")` on `TuningSystem` enum)

- [x] Task 8: Verify and validate (AC: all)
  - [x] 8.1 `cargo clippy -p domain` — zero warnings
  - [x] 8.2 `cargo clippy -p web` — zero warnings
  - [x] 8.3 `cargo test -p domain` — all tests pass (253 tests)
  - [x] 8.4 `trunk build` — successful WASM compilation
  - [ ] 8.5 `trunk serve` — manual browser smoke test: open Settings, change each control, verify localStorage values update, return to training, verify settings take effect

## Dev Notes

### Core Approach: Replace Stub Component

This story replaces the existing `SettingsView` stub (`web/src/components/settings_view.rs`) with a fully functional settings form. No domain crate changes are required — all domain types and the `UserSettings` trait already exist. The work is entirely in the web crate's settings component.

### Existing Infrastructure (DO NOT recreate)

The following already exists and must be reused:

- **`LocalStorageSettings::set(key, value)`** — writes to localStorage. Already implemented in `web/src/adapters/localstorage_settings.rs` (line 29). Currently `#[allow(dead_code)]` — this story activates it.
- **`LocalStorageSettings` getters** — `get_string()`, `get_f64()`, `get_u8()` private helpers (lines 13-27). Use these patterns for reading current values.
- **`UserSettings` trait** — read-only trait in `domain/src/ports.rs` (line 41). Do NOT modify this trait. Settings are read by the comparison session via this trait; writing is a web-crate concern handled by `LocalStorageSettings::set()`.
- **`DefaultSettings`** — fallback defaults in `web/src/adapters/default_settings.rs`. Reference for default values.
- **`MIDINote::name()`** — returns human-readable note names like "C4", "A4" in `domain/src/types/midi.rs` (line 43). Use for dropdown labels.
- **`NoteRange::try_new()`** — validates min <= max in `domain/src/types/note_range.rs` (line 21). Use to validate range before persisting.
- **`NoteDuration`** — clamps to 0.3-3.0 in `domain/src/types/duration.rs`. Clamp input values to this range.
- **`TuningSystem`** — enum with `EqualTemperament` and `JustIntonation`, serialized as `"equalTemperament"` / `"justIntonation"` via `serde(rename_all = "camelCase")` in `domain/src/tuning.rs` (line 27).

### localStorage Key Conventions

All keys use `peach.` prefix + `snake_case`. Existing keys (from architecture doc and `LocalStorageSettings` implementation):

| Key | Type | Default | Notes |
|---|---|---|---|
| `peach.note_range_min` | u8 | 36 | MIDI note number |
| `peach.note_range_max` | u8 | 84 | MIDI note number |
| `peach.note_duration` | f64 | 1.0 | Seconds (0.3-3.0) |
| `peach.reference_pitch` | f64 | 440.0 | Hz |
| `peach.vary_loudness` | f64 | 0.0 | 0.0-1.0 (displayed as 0-100%) |
| `peach.tuning_system` | String | (absent = ET) | `"justIntonation"` or absent (defaults to ET) |
| `peach.sound_source` | String | (absent = default) | Currently unused; store for future Epic 5 |

### Component Architecture Pattern

Follow the existing component patterns established in Epic 1:

```rust
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn SettingsView() -> impl IntoView {
    // 1. Read current values from localStorage into signals
    // 2. Create on_change handlers that persist via LocalStorageSettings::set()
    // 3. Return view! macro with semantic HTML form controls
}
```

**Signal pattern for auto-save:** Each setting gets an `RwSignal` initialized from localStorage. On change events, update both the signal (for reactivity) and localStorage (for persistence). No save button needed.

**Do NOT use `provide_context` or `use_context` for settings** — unlike the comparison session state, settings are read fresh from localStorage each time training starts. The `SettingsView` component reads/writes localStorage directly.

### Note Range Dropdown Implementation

The note range dropdowns need special handling to prevent invalid ranges:

1. Read current min/max from localStorage (defaults: 36, 84)
2. Store both as `RwSignal<u8>`
3. Lower bound dropdown: options from MIDI 21 (A0) to current upper bound value
4. Upper bound dropdown: options from current lower bound value to MIDI 108 (C8)
5. When lower bound changes → re-filter upper bound options
6. When upper bound changes → re-filter lower bound options
7. Use `MIDINote::try_new(value)` for validation, `MIDINote::name()` for option labels
8. Practical range 21-108 covers the full piano keyboard (A0 to C8) — sufficient for all instruments

### Reference Pitch Options

Predefined options only (no custom input for simplicity):

| Label | Value (Hz) | Context |
|---|---|---|
| "440 Hz (Concert)" | 440.0 | Modern standard |
| "442 Hz" | 442.0 | Common orchestral tuning |
| "432 Hz" | 432.0 | Alternative tuning |
| "415 Hz (Baroque)" | 415.0 | Historical pitch |

### Sound Source — Current Limitations

Only the oscillator audio adapter is implemented. SoundFont support (`SoundFontNotePlayer` via OxiSynth) is deferred to Epic 5. The dropdown shows a single option "Sine Oscillator" with the value `"oscillator:sine"`. This control exists to prepare the UI for future sound source selection without requiring rework.

### UX Requirements (from UX Spec)

- **Auto-save:** All changes persist immediately to localStorage. No save/cancel buttons.
- **Bounded controls:** All inputs are constrained to valid ranges. No validation errors possible.
- **No confirmation:** No confirmation toast or save indicator. The control state itself is the feedback.
- **Default values:** All settings have sensible defaults. First-time users never need to visit settings.
- **Dark mode:** All Tailwind color utilities must use `dark:` variants.
- **Touch targets:** Minimum 44x44px on all interactive elements.
- **Keyboard accessible:** All controls reachable and operable via keyboard.

### What NOT to Implement (Separate Stories)

- **Interval selection (FR27)** — Story 2.2. Do NOT add interval multi-select to this component.
- **Reset training data (FR29)** — Story 2.3. Do NOT add the reset button to this component.
- **Custom reference pitch** — Keep it simple with predefined dropdown options only.

### Tailwind Styling Patterns (from existing components)

Follow the established styling patterns from `start_page.rs` and `comparison_view.rs`:

- Container: `class="py-12"` (padding)
- Headings: `class="text-2xl font-bold dark:text-white"`
- Links: `class="min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"`
- Form controls: apply `min-h-11` for touch targets, `rounded`, `border`, `bg-white dark:bg-gray-800`, `dark:text-white`
- Labels: `class="text-sm font-medium text-gray-700 dark:text-gray-300"`
- Consistent spacing between form groups: `class="space-y-6"` or `class="flex flex-col gap-6"`

### Project Structure Notes

- Only file modified: `web/src/components/settings_view.rs` (replace stub with full implementation)
- No new files needed
- No domain crate changes needed
- Remove `#[allow(dead_code)]` from `LocalStorageSettings::set()` in `web/src/adapters/localstorage_settings.rs` since it will now be used
- No changes to routing (`app.rs` already has `/settings` → `SettingsView`)
- No changes to `components/mod.rs` (already exports `SettingsView`)

### References

- [Source: docs/planning-artifacts/epics.md#Story 2.1] — Full acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture] — Tailwind CSS, component patterns, signal naming
- [Source: docs/planning-artifacts/architecture.md#Storage Boundaries] — localStorage key conventions (`peach.` prefix)
- [Source: docs/planning-artifacts/ux-design-specification.md#Settings View] — Control types and behavior rules
- [Source: docs/planning-artifacts/ux-design-specification.md#Form Behavior] — Auto-save, bounded controls, no validation errors
- [Source: docs/project-context.md#Leptos Framework Rules] — Component architecture, signal patterns
- [Source: docs/project-context.md#Storage Key Conventions] — `peach.` prefix + `snake_case`
- [Source: web/src/adapters/localstorage_settings.rs] — Existing `set()` method and getter helpers
- [Source: web/src/adapters/default_settings.rs] — Default values
- [Source: domain/src/ports.rs#UserSettings] — Read-only trait (do not modify)
- [Source: domain/src/types/midi.rs#MIDINote::name()] — Note name display
- [Source: domain/src/types/note_range.rs] — NoteRange validation
- [Source: domain/src/types/duration.rs] — NoteDuration 0.3-3.0 clamping
- [Source: domain/src/tuning.rs#TuningSystem] — Enum with serde camelCase serialization

### Previous Story Intelligence (from Story 1.10)

**Patterns established in Epic 1:**
- `LocalStorageSettings::set()` prepared for this story — marked `#[allow(dead_code)]` with comment "Planned for Settings view (Epic 2, story 2.1)"
- `NoteRange` type introduced in story 1.10 code review — validates `min <= max` by construction
- `try_new()` pattern on all domain value types — use for user input validation
- All `Rc<RefCell<>>` borrow scopes verified safe — this story does NOT interact with session state
- `SendWrapper` required for Leptos 0.8 contexts — not relevant for this story (no context providers)

**Code review learnings from Epic 1:**
- Keep web-sys feature lists explicit and minimal — this story does NOT need new web-sys features
- Pattern: implementation commit → code review fixes → story marked done
- Use existing adapter methods rather than direct `web_sys::window()` chains

### Git Intelligence

Recent commit pattern: alternating "Implement story X" → "Apply code review fixes for story X and mark as done". Follow same pattern.

Files most relevant to this story:
- `web/src/components/settings_view.rs` — THE file to modify (replace stub)
- `web/src/adapters/localstorage_settings.rs` — Remove `#[allow(dead_code)]` from `set()`
- All other files remain unchanged

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

No debug issues encountered.

### Completion Notes List

- Replaced `SettingsView` stub with full settings form implementing all 7 controls (note range min/max, note duration, reference pitch, sound source, loudness variation, tuning system)
- Used `UserSettings` trait methods on `LocalStorageSettings` to read initial values — avoids exposing private helpers
- Note range dropdowns use reactive signal-driven option lists: lower bound options constrained to 21..=upper, upper bound options constrained to lower..=108
- Initial note range values clamped to practical 21-108 range for robustness
- Range sliders use `on:input` for live feedback; selects use `on:change`
- Helper function `target_value()` extracts `.value` from event targets using `js_sys::Reflect` — no new web-sys features needed
- Duration values rounded to 1 decimal to avoid floating-point drift
- Loudness stored as 0.0-1.0 float in localStorage, displayed as 0-100% integer
- Sound source defaults to "oscillator:sine" (single option, future Epic 5 extensibility)
- Removed `#[allow(dead_code)]` from `LocalStorageSettings::set()` since it's now actively used
- All Tailwind classes follow established patterns: dark mode variants, 44px touch targets, focus rings, consistent spacing
- All labels use `<label for="...">` + matching `id` for accessibility — no extra `aria-label` needed since labels are visually adjacent to controls

### File List

- `web/src/components/settings_view.rs` — Replaced stub with full settings form (new implementation)
- `web/src/adapters/localstorage_settings.rs` — Removed `#[allow(dead_code)]` from `set()` method
- `docs/implementation-artifacts/sprint-status.yaml` — Updated story status
- `docs/implementation-artifacts/2-1-settings-view-core-training-parameters.md` — Updated tasks, status, dev agent record

## Change Log

- 2026-03-03: Implemented Settings View with all core training parameter controls (story 2.1)
