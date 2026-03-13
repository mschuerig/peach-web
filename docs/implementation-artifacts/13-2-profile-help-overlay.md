# Story 13.2: Profile Help Overlay

Status: ready-for-dev

## Story

As a musician,
I want a help overlay on the profile screen that explains what each chart element means,
so that I can learn to read my progress charts.

## Acceptance Criteria

1. **Given** the profile screen navigation bar, **When** rendered, **Then** a help button (? icon) is visible in the toolbar trailing position.

2. **Given** I tap the help button, **When** the help overlay opens, **Then** it uses the same `HelpModal` (native `<dialog>`) mechanism as all other screens in the app.

3. **Given** the help overlay content, **When** displayed, **Then** it shows five sections in this order:
   - "Your Progress Chart" — "This chart shows how your pitch perception is developing over time"
   - "Trend Line" — "The blue line shows your smoothed average — it filters out random ups and downs to reveal your real progress"
   - "Variability Band" — "The shaded area around the line shows how consistent you are — a narrower band means more reliable results"
   - "Target Baseline" — "The green dashed line is your goal — as the trend line approaches it, your ear is getting sharper"
   - "Time Zones" — "The chart groups your data by time: months on the left, recent days in the middle, and today's sessions on the right"

4. **Given** the help overlay, **When** I dismiss it (Done button, Escape key, or backdrop click), **Then** I return to the profile screen with chart state preserved (including any active tap annotation from story 13.1).

## Tasks / Subtasks

- [ ] Task 1: Define PROFILE_HELP sections (AC: #3)
  - [ ] 1.1 Add `PROFILE_HELP` static array to `web/src/help_sections.rs` with 5 `HelpSection` entries using i18n keys
  - [ ] 1.2 Add English i18n keys to `web/locales/en/main.ftl`: `profile-help-title`, `help-profile-chart-title`, `help-profile-chart-body`, `help-profile-trend-title`, `help-profile-trend-body`, `help-profile-band-title`, `help-profile-band-body`, `help-profile-baseline-title`, `help-profile-baseline-body`, `help-profile-zones-title`, `help-profile-zones-body`
  - [ ] 1.3 Add German i18n keys to `web/locales/de/main.ftl` with equivalent translations

- [ ] Task 2: Add help button to profile NavBar (AC: #1)
  - [ ] 2.1 Add `is_help_open: RwSignal<bool>` signal in `ProfileView`
  - [ ] 2.2 Add `NavIconButton` as child of `NavBar` with `icon="?"`, `circled=true`, and `on_click` callback that sets `is_help_open` to `true`
  - [ ] 2.3 Import `NavIconButton` from `super::nav_bar`

- [ ] Task 3: Render HelpModal (AC: #2, #4)
  - [ ] 3.1 Add `HelpModal` component after the NavBar, passing `title=move_tr!("profile-help-title")`, `sections=PROFILE_HELP`, `is_open=is_help_open`
  - [ ] 3.2 Import `HelpModal` from `super::help_content` and `PROFILE_HELP` from `crate::help_sections`
  - [ ] 3.3 No `on_close` callback needed — profile view is read-only (no pause/resume like training views)

## Dev Notes

### Architecture & Patterns

- **Exact pattern to follow**: Settings view (`web/src/components/settings_view.rs` lines 259-280) is the closest match. Profile view is read-only like settings — no pause/resume needed.
- **HelpModal component**: Already implemented in `web/src/components/help_content.rs`. Uses native `<dialog>` element with `showModal()`/`close()`. Provides focus trapping, Escape key dismiss, and backdrop click dismiss out of the box.
- **NavIconButton**: In `web/src/components/nav_bar.rs`. Use `circled=true` for the "?" icon with thin border circle. Use `on_click=Callback::new(...)` (not `href`).
- **i18n pattern**: All help text uses i18n keys, never hardcoded strings. Use `move_tr!()` for the modal title. Help section keys follow the pattern `help-{screen}-{topic}-title` / `help-{screen}-{topic}-body`.

### Integration Points

- **Profile view file**: `web/src/components/profile_view.rs` — currently 74 lines. NavBar at line 27 has no children (no help button yet).
- **Help sections file**: `web/src/help_sections.rs` — add `PROFILE_HELP` array following the exact same pattern as `SETTINGS_HELP`.
- **Imports needed in profile_view.rs**:
  - `use super::nav_bar::NavIconButton;`
  - `use super::help_content::HelpModal;`
  - `use crate::help_sections::PROFILE_HELP;`
  - `leptos_fluent::tr` is already imported

### Chart State Preservation (AC: #4)

- The `HelpModal` uses a native `<dialog>` which overlays without unmounting the profile view content. All chart signals, scroll positions, and tap annotation state (from story 13.1) are preserved because the underlying DOM is untouched.
- No special handling needed — this is inherent to the native dialog approach.

### Help Content (from iOS spec)

The five sections with exact text from `docs/ios-reference/profile-screen-specification.md`:

| Section | Title | Body |
|---------|-------|------|
| 1 | Your Progress Chart | This chart shows how your pitch perception is developing over time |
| 2 | Trend Line | The blue line shows your smoothed average — it filters out random ups and downs to reveal your real progress |
| 3 | Variability Band | The shaded area around the line shows how consistent you are — a narrower band means more reliable results |
| 4 | Target Baseline | The green dashed line is your goal — as the trend line approaches it, your ear is getting sharper |
| 5 | Time Zones | The chart groups your data by time: months on the left, recent days in the middle, and today's sessions on the right |

### German Translations

Translate the help content naturally. The existing German help text in the app uses informal "du/dein" form consistently.

### Previous Story Intelligence (13.1)

- Story 13.1 added chart tap annotation with selection line and popover in `progress_chart.rs`
- The annotation uses `RwSignal<Option<usize>>` local to each `ProgressChart` component
- Frosted glass popover via `<foreignObject>` inside SVG
- Code review fixes included: scroll listener cleanup, spawn_local lifecycle, popover dimensions
- **Key insight**: The help modal opens above the chart SVG via native `<dialog>` — no interaction between annotation state and help modal

### Project Structure Notes

- All changes are in `web/` crate — no domain changes
- Files to modify:
  - `web/src/help_sections.rs` — add `PROFILE_HELP` array
  - `web/src/components/profile_view.rs` — add help button + modal
  - `web/locales/en/main.ftl` — add English i18n keys
  - `web/locales/de/main.ftl` — add German i18n keys
- No new files needed
- No CSS changes needed (HelpModal and NavIconButton styling already exist)

### Testing

- **Domain tests**: No domain changes → existing tests must remain green (`cargo test -p domain`)
- **Clippy**: `cargo clippy --workspace` — zero warnings expected
- **Manual browser testing** (deferred to user):
  - Profile screen shows ? help button in NavBar trailing position
  - Tapping ? opens help overlay with 5 sections in correct order
  - Help text matches spec exactly
  - Escape key closes the overlay
  - Backdrop click closes the overlay
  - Done button closes the overlay
  - Chart state (scroll position, tap annotation) preserved after closing help
  - Help button visible even when no training data exists (empty state)
  - Dark mode: overlay styling correct
  - Screen reader: dialog announced with correct title and content
  - Language switch (EN ↔ DE): help text updates correctly

### References

- [Source: docs/ios-reference/profile-screen-specification.md#Help Content]
- [Source: docs/ios-reference/profile-screen-specification.md#Navigation Bar]
- [Source: docs/planning-artifacts/epics.md#Story 13.2]
- [Source: web/src/components/settings_view.rs] — Reference implementation for help button + modal integration
- [Source: web/src/components/help_content.rs] — HelpModal and HelpContent components
- [Source: web/src/help_sections.rs] — Existing help section definitions
- [Source: web/src/components/nav_bar.rs] — NavIconButton component
- [Source: web/src/components/profile_view.rs] — Current profile view (no help button)
- [Source: docs/implementation-artifacts/13-1-chart-tap-annotation.md] — Previous story context

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
