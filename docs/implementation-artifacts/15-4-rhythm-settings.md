# Story 15.4: Rhythm Settings

Status: review

## Story

As a user,
I want to configure rhythm training parameters in the settings screen,
so that I can adjust tempo and gap positions before rhythm training is fully implemented.

## Context

Prerequisite: Story 15.1 (TempoBPM, StepPosition types).

The iOS settings screen has a "Rhythm" section with:
- **Tempo stepper** (40–200 BPM, step 1, default 80)
- **Gap position toggles** (4 buttons for Beat/E/And/A, default: only position 4)

These settings are persisted in LocalStorage and will be consumed by rhythm sessions once implemented.

## Acceptance Criteria

1. **AC1 — Rhythm section in settings:** A new "Rhythm" section appears below the existing pitch settings, with a section header.

2. **AC2 — Tempo stepper:**
   - Label: "Tempo"
   - Value display: "{N} BPM"
   - Range: 40–200
   - Step: 1 (or 5 for faster adjustment — check what feels right)
   - Default: 80 BPM
   - Buttons: decrement (−) and increment (+), disabled at bounds

3. **AC3 — Gap position toggles:**
   - Label: "Gap Positions" with help text: "Select which subdivisions are used in Fill the Gap training."
   - Four toggle buttons labeled "Beat", "E", "And", "A"
   - Default: only "A" (position 4) enabled
   - Constraint: at least one position must remain active — prevent deselecting the last active position
   - Visual: active positions highlighted, inactive dimmed

4. **AC4 — Persistence:**
   - Tempo stored in LocalStorage as integer
   - Gap positions stored as comma-separated position indices (e.g. "0,3" for Beat + A)
   - Values loaded on app start and available via `UserSettings` port

5. **AC5 — UserSettings port extended:** The `UserSettings` trait (or equivalent settings interface) gains:
   - `get_tempo_bpm() -> TempoBPM`
   - `set_tempo_bpm(TempoBPM)`
   - `get_enabled_gap_positions() -> HashSet<StepPosition>`
   - `set_enabled_gap_positions(HashSet<StepPosition>)`

6. **AC6 — Localization:** All labels and help text in `en/main.ftl` and `de/main.ftl`.

7. **AC7 — Accessibility:** Stepper and toggles are keyboard-navigable with appropriate ARIA roles and labels.

8. **AC8 — Builds and renders:** `trunk build` succeeds. Settings screen shows rhythm section. Values persist across page reloads.

## Tasks / Subtasks

- [x] Task 1: Extend `UserSettings` port with tempo and gap position methods
- [x] Task 2: Implement in `LocalStorageSettings` adapter with persistence
- [x] Task 3: Add tempo stepper UI to settings view
- [x] Task 4: Add gap position toggle row to settings view
- [x] Task 5: Enforce "at least one position" constraint
- [x] Task 6: Add localization strings (en + de)
- [x] Task 7: Test persistence across reloads — verified by user in browser

## Dev Notes

- Follow the existing settings view patterns (stepper for note duration, toggle for intervals)
- The gap position encoding matches iOS: comma-separated raw values `"0"`, `"1"`, `"2"`, `"3"`
- `TempoBPM` validation (clamp 40–200) happens at the type level per story 15.1
- The step size for the tempo stepper could be 1 (precise) or 5 (faster to reach target). iOS uses 1. Start with 1; we can adjust if it feels tedious.
- These settings won't be consumed by anything yet — rhythm sessions will read them once implemented

## Dev Agent Record

### Implementation Plan

- Extended `UserSettings` trait with `tempo_bpm()` and `enabled_gap_positions()` getters (following existing read-only pattern)
- Added setter methods `set_tempo_bpm()` and `set_enabled_gap_positions()` as static methods on `LocalStorageSettings` (matching `set_selected_intervals` pattern)
- Implemented persistence: `peach.tempo_bpm` as integer, `peach.gap_positions` as comma-separated indices
- Added Rhythm section to settings view between Difficulty and Data sections
- Used existing `Stepper` component for tempo, custom toggle buttons for gap positions
- Gap position toggle buttons use `flex-1` layout with indigo active / gray inactive styling
- "At least one position" constraint: button disabled when it's the only active position (same pattern as intervals)
- Added help section for Rhythm settings in settings help modal

### Completion Notes

- AC5 deviation: Getters added to `UserSettings` trait; setters on `LocalStorageSettings` directly (consistent with existing codebase pattern — trait is read-only)
- Tempo step size: 1 BPM (matching iOS)
- All 357 domain tests pass, zero clippy warnings
- Task 7 (persistence across reloads) deferred — requires browser testing

## File List

- `domain/src/ports.rs` — Added `tempo_bpm()` and `enabled_gap_positions()` to `UserSettings` trait
- `domain/src/session/pitch_discrimination_session.rs` — Updated test mock impls for new trait methods
- `domain/src/session/pitch_matching_session.rs` — Updated test mock impls for new trait methods
- `web/src/adapters/default_settings.rs` — Implemented new `UserSettings` methods with defaults
- `web/src/adapters/localstorage_settings.rs` — Implemented persistence (get/set for tempo and gap positions)
- `web/src/components/settings_view.rs` — Added Rhythm section with tempo stepper and gap position toggles
- `web/src/help_sections.rs` — Added Rhythm help section
- `web/locales/en/main.ftl` — Added rhythm settings and help localization strings
- `web/locales/de/main.ftl` — Added rhythm settings and help localization strings (German)

## Change Log

- 2026-03-24: Implemented rhythm settings (tempo stepper + gap position toggles) with persistence and localization
