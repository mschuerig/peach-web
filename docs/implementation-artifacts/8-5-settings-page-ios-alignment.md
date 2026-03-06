# Story 8.5: Settings Page iOS Alignment

Status: done

## Story

As a user,
I want the settings page to use grouped card sections with iOS-style controls,
so that the settings experience matches the polished look of the rest of the app.

## Acceptance Criteria

1. Settings are organized into visually distinct grouped card sections with section headers above each card, matching the iOS "Einstellungen" layout: **Pitch Range**, **Intervals**, **Sound**, **Difficulty**, **Data**
2. Each section is rendered as a rounded card (light gray background) containing its related controls, separated by thin dividers between rows — matching iOS grouped table style
3. Pitch range (lower/upper bound) uses +/- stepper buttons showing the current note name (e.g. "Lowest Note: C2  [- | +]") instead of dropdown selects
4. Interval selection uses a compact grid layout with column headers (P1, m2, M2, ..., P8), an ascending row, and a descending row — toggle buttons instead of a vertical checkbox list
5. Sound settings (instrument, duration, concert pitch, tuning system) are grouped in a single "Sound" card with rows separated by dividers
6. Duration and concert pitch use +/- stepper controls instead of range sliders, showing the current value inline (e.g. "Duration: 1.3s [- | +]")
7. Loudness variation keeps a slider control but is in its own "Difficulty" section card
8. Data management (export/import/delete) remains in its own "Data" section card, keeping existing functionality
9. All controls maintain keyboard accessibility (Tab navigation, Enter/Space activation, arrow keys for steppers)
10. Dark mode styling preserved across all changed sections
11. No regressions in settings persistence or training behavior

## Tasks / Subtasks

- [x] Task 1: Create reusable card section components (AC: 1, 2)
  - [x] Create a `SettingsSection` component with props: `title: &'static str`, `children` slot
  - [x] Renders: section header in gray/muted text above, rounded card container (`rounded-xl bg-gray-100 dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700`) wrapping children
  - [x] Create a `SettingsRow` component with props: `label: &'static str`, `children` slot for the control
  - [x] Renders: flex row with label left, control right, consistent padding (px-4 py-3), minimum 44px height
  - [x] These can be inline in settings_view.rs or a small helper file

- [x] Task 2: Implement +/- stepper component (AC: 3, 6, 9)
  - [x] Create a `Stepper` component with props: `value: Signal<String>` (display text), `on_decrement: Callback`, `on_increment: Callback`, `decrement_disabled: Signal<bool>`, `increment_disabled: Signal<bool>`
  - [x] Renders: paired buttons `[- | +]` with a rounded pill/segmented-control style matching iOS stepper
  - [x] Styling: gray background, rounded, divider between - and +, min touch targets
  - [x] Keyboard: buttons focusable, Enter/Space activates, aria-labels "Decrease {label}" / "Increase {label}"

- [x] Task 3: Refactor Pitch Range section (AC: 1, 3)
  - [x] Replace the two `<select>` dropdowns with `SettingsSection` "Pitch Range" containing two `SettingsRow`s
  - [x] "Lowest Note: {note_name}" with Stepper that decrements/increments the MIDI note value
  - [x] "Highest Note: {note_name}" with Stepper that decrements/increments the MIDI note value
  - [x] Stepper increments by semitone (1 MIDI note)
  - [x] Decrement disabled when at range minimum (21 for low, current min+1 for high)
  - [x] Increment disabled when at range maximum (current max-1 for low, 108 for high)
  - [x] Persist to localStorage on each change (existing `peach.note_range_min`, `peach.note_range_max` keys)

- [x] Task 4: Refactor Interval Selection as compact grid (AC: 1, 4, 9)
  - [x] Replace vertical checkbox list with `SettingsSection` "Intervals"
  - [x] Render a grid with column headers: P1, m2, M2, m3, M3, P4, d5, P5, m6, M6, m7, M7, P8
  - [x] Row 1 (ascending, marked with up arrow): toggle buttons for each interval ascending
  - [x] Row 2 (descending, marked with down arrow): toggle buttons for each interval descending
  - [x] Active intervals: blue/indigo highlight background; inactive: gray
  - [x] P1 descending is always disabled/hidden (same as P1 ascending)
  - [x] At least one interval must remain active — last active toggle is disabled
  - [x] Grid should scroll horizontally on narrow screens if needed (`overflow-x-auto`)
  - [x] Persist to localStorage using existing serialization pattern
  - [x] Add hint text below grid: "Select the intervals you want to practice. At least one must remain active."

- [x] Task 5: Refactor Sound section (AC: 1, 5, 6)
  - [x] Create `SettingsSection` "Sound" containing grouped rows:
    - [x] "Sound" row: keep dropdown/select for instrument picker (instrument list is dynamic from SF2 presets) — show current value on the right like iOS
    - [x] "Duration: {value}s" row: replace range slider with Stepper (step 0.1s, range 0.3-3.0)
    - [x] "Concert Pitch: {value} Hz" row: replace dropdown with Stepper (step 1 Hz, range 415-450 or similar reasonable bounds)
    - [x] "Tuning" row: keep dropdown for tuning system (only 2 options) — show current value on right
  - [x] Add hint text below tuning selector matching iOS: "Select the tuning for intervals. Equal temperament divides the octave into 12 equal steps. Just intonation uses pure frequency ratios."

- [x] Task 6: Refactor Difficulty section (AC: 1, 7)
  - [x] Create `SettingsSection` "Difficulty"
  - [x] "Loudness Variation" row with the existing range slider (Off — Max labels on left/right)
  - [x] Keep the existing slider implementation, just wrap it in the card layout

- [x] Task 7: Refactor Data section (AC: 1, 8)
  - [x] Wrap existing export/import/delete controls in `SettingsSection` "Data"
  - [x] Keep "Export Data" and "Import Data" as rows within the card (styled as tappable rows rather than buttons, or keep as buttons within the card)
  - [x] "Delete All Training Data" as a red-text row at the bottom (matching iOS destructive action pattern)
  - [x] All existing functionality (export, import dialog, reset confirmation) preserved exactly

- [x] Task 8: Clean up and test (AC: 9, 10, 11)
  - [x] Remove old layout code (standalone labels, bare divs, fieldset/legend patterns)
  - [x] Test all settings controls function correctly (values persist, steppers increment/decrement properly)
  - [x] Test responsive behavior at mobile/tablet/desktop widths
  - [x] Test keyboard navigation through all controls
  - [x] Test dark mode
  - [x] Verify no regressions: change a setting, start training, confirm it takes effect

## Dev Notes

### Current Settings Layout (to be replaced)

The settings view (`web/src/components/settings_view.rs`) currently uses a flat vertical list of form controls:
- Individual `<label>` + `<select>` or `<input type="range">` blocks with `space-y-6` gap
- Interval selection: vertical checkbox list with `min-h-[44px]` per item
- Data management / Danger zone: separated by `border-t` dividers with `fieldset`/`legend`

This works but doesn't match the iOS grouped-card visual pattern.

### iOS Reference (from screenshots)

**Settings sections visible in iOS:**
1. **Tonumfang (Pitch Range)**: Card with "Tiefster Ton: C2 [- | +]" and "Hochster Ton: C6 [- | +]" rows separated by divider
2. **Intervalle (Intervals)**: Compact grid — header row (P1 m2 M2 m3 M3 P4 d5 P5 m6 M6 m7 M7 P8), ascending row with up-arrow prefix, descending row with down-arrow prefix. Active = blue highlight, inactive = light gray
3. **Klang (Sound)**: Card with "Klang: Cello" dropdown, "Dauer: 1,3s [- | +]", "Kammerton: 440 Hz [- | +]", "Stimmung: Gleichstufige Stimmung" dropdown. Hint text below about tuning systems
4. **Schwierigkeit (Difficulty)**: "Lautstärke variieren" with Off-Max slider
5. **Daten (Data)**: "Trainingsdaten exportieren" (with share icon), "Trainingsdaten importieren" (blue text), "Alle Trainingsdaten loschen" (red text)

**Key visual patterns:**
- Section headers: muted/gray text, slightly smaller font, above the card
- Cards: rounded corners, light gray fill, rows separated by 1px dividers
- Steppers: paired `[- | +]` segmented control, gray background pill
- Dropdowns: show current value on right side with chevron indicator
- Destructive actions: red text

### Key Code Locations

| File | Changes |
|---|---|
| `web/src/components/settings_view.rs` | Major refactor — restructure all controls into grouped cards |
| `web/src/components/mod.rs` | Add new component exports if extracted |

### Architecture Constraints

- Stepper for note range must respect the constraint that min < max (existing validation logic)
- Stepper for concert pitch: iOS shows discrete values (415, 432, 440, 442). Consider whether to use +/- 1Hz free stepping or keep as a small set of presets. The iOS version uses +/- with 1Hz steps. Current web version uses a dropdown with 4 options. Recommend: Stepper with 1Hz steps, range 400-460 Hz
- Interval grid: the existing `all_directed_intervals()` function returns all 25 directed intervals. The grid layout maps these to a 13-column (P1 through P8) x 2-row (asc/desc) grid
- Sound source dropdown must remain dynamic (populated from `sf2_presets` signal) — it cannot be a stepper
- Tuning system has only 2 options — could be a stepper toggling between them or keep as dropdown. iOS uses a dropdown with chevron. Recommend: keep dropdown

### Stepper Value Ranges

| Setting | Min | Max | Step | Display |
|---|---|---|---|---|
| Note Range Lower | MIDI 21 (A0) | current upper - 1 | 1 semitone | Note name (e.g. "C2") |
| Note Range Upper | current lower + 1 | MIDI 108 (C8) | 1 semitone | Note name (e.g. "C6") |
| Duration | 0.3 | 3.0 | 0.1 | "{value}s" |
| Concert Pitch | 400 | 460 | 1 | "{value} Hz" |

### Existing Patterns to Reuse

- `MIDINote::new(val).name()` — converts MIDI number to note name string
- `LocalStorageSettings::set(key, value)` — persists settings
- `target_value(&ev)` / `target_checked(&ev)` — event value extraction helpers
- Existing dark mode pattern: every `bg-gray-100` has `dark:bg-gray-800`, every `text-gray-700` has `dark:text-gray-300`

### Previous Story Intelligence (8.4)

Story 8.4 introduces the `NavBar` component which replaces `PageNav` on the settings page. This story's changes to the settings view body are independent of the nav bar change but should use the new nav pattern. If implementing after 8.4, the `<PageNav current="settings" />` will already be replaced with `<NavBar>`.

If implementing before 8.4, keep the existing `PageNav` — 8.4 will replace it.

### References

- [Source: web/src/components/settings_view.rs] — Current settings implementation (all sections)
- [Source: docs/project-context.md] — Coding conventions, component architecture
- iOS screenshots: Settings-portrait-1.png, Settings-portrait-2.png, Settings-landscape-1.png, Settings-landscape-2.png, Settings-landscape-3.png

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Compilation error: type mismatch in descending interval row — P1 empty `<td>` vs button `<td>` return different Leptos view types. Fixed with `.into_any()` on both branches.
- Removed unused `target_checked` helper (no longer needed after switching from checkboxes to toggle buttons).

### Completion Notes List

- Created `SettingsSection` component: muted uppercase section header + rounded card container with divide-y dividers
- Created `SettingsRow` component: flex row with label left, control right, 44px min height
- Created `SettingsRowDynamic` component: same as SettingsRow but with reactive `Signal<String>` label
- Created `Stepper` component: iOS-style [- | +] paired buttons with aria-labels, disabled states, gray pill styling
- Refactored Pitch Range: replaced `<select>` dropdowns with stepper controls showing note names (e.g., "Lowest Note: C2")
- Refactored Intervals: replaced vertical checkbox list with compact 13-column grid (P1-P8) x 2 rows (ascending/descending), toggle buttons with indigo active state
- Refactored Sound: grouped instrument dropdown, duration stepper (0.3-3.0s), concert pitch stepper (400-460 Hz), tuning dropdown into single card with hint text
- Refactored Difficulty: wrapped loudness slider in card with Off/Max labels
- Refactored Data: export/import as tappable text rows (indigo), delete as red text row (iOS destructive pattern), all functionality preserved
- Removed old layout code (standalone labels, bare divs, fieldset/legend patterns)
- All dark mode classes preserved across all sections
- All existing persistence logic unchanged (localStorage keys, serialization patterns)
- Concert pitch range expanded from 4 fixed options (415/432/440/442) to continuous stepper (400-460 Hz, 1 Hz steps) matching iOS behavior

### Senior Developer Review (AI)

**Reviewer:** Michael (via adversarial code review) | **Date:** 2026-03-06 | **Outcome:** Approved with fixes applied

**Issues found:** 0 Critical, 3 Medium, 2 Low — all fixed inline.

| # | Severity | Issue | Fix |
|---|---|---|---|
| 1 | MEDIUM | Hint text rendered inside card (iOS puts it below) | Moved intervals and tuning hints outside `SettingsSection` cards |
| 2 | MEDIUM | No chevron indicator on dropdown selects (`appearance-none` removed native arrow) | Added `›` (U+203A) chevron span after Sound and Tuning selects |
| 3 | MEDIUM | Duplicated ~150-char button class strings in interval grid (4 copies) | Extracted to `TOGGLE_ACTIVE` / `TOGGLE_INACTIVE` constants |
| 4 | LOW | `as i32` truncation for concert pitch display | Changed to `.round() as i32` |
| 5 | LOW | Pre-existing: Sine Oscillator option vanishes when SF2 loads | Not fixed (pre-existing, out of scope) |

### Change Log

- 2026-03-06: Code review fixes — hint text below cards, dropdown chevrons, extracted toggle constants, safe pitch rounding
- 2026-03-06: Complete iOS-style settings page refactor — grouped card sections, stepper controls, compact interval grid, data management as tappable rows

### File List

- `web/src/components/settings_view.rs` — Major refactor: all settings reorganized into iOS-style grouped cards with SettingsSection/SettingsRow/Stepper components
- `docs/implementation-artifacts/sprint-status.yaml` — Status updated: ready-for-dev → in-progress → review
- `docs/implementation-artifacts/8-5-settings-page-ios-alignment.md` — Story file updated with completion status
