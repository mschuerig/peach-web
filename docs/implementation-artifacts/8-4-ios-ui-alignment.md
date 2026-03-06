# Story 8.4: iOS UI Alignment

Status: ready-for-dev

## Story

As a user,
I want the web app's layout and navigation to look and feel closer to the native iOS version,
so that the experience is consistent across platforms and feels polished on both mobile and desktop.

## Acceptance Criteria

1. **Navigation bar** on all pages follows the iOS pattern: back arrow (left) returns to previous/start page, page title centered, icon buttons for contextual actions (help, settings, profile) on the right — replacing the current text-link nav
2. **PageNav component** is refactored into a reusable `NavBar` that accepts title, optional back href, and optional right-side action slots
3. **Start page** header shows "Peach" title (left-aligned on mobile, centered on wider screens) with profile icon (chart) and settings icon (gear) on the right, and info icon on the left — matching iOS layout
4. **Pitch Comparison view** buttons are large blue rounded-rectangle cards filling the available width, each containing a circle with an arrow icon and the label text ("Higher" / "Lower") — stacked vertically on portrait, side-by-side on landscape/wide screens
5. **Pitch Comparison view** shows the interval name and tuning system label centered between the nav bar and the answer buttons (matching iOS: "Perfect Fifth ascending" / "12-TET")
6. **Pitch Matching view** header area (below nav bar) shows: left side has "Latest: X cents" with trend arrow and "Best: X cents" below; right side shows the signed cent deviation feedback (color-coded: green/yellow/red with direction arrow); interval name and tuning label centered below stats — matching the user's layout sketch
7. **Pitch Matching view** vertical slider fills all remaining vertical space below the header area
8. All navigation changes maintain keyboard accessibility (focus management, arrow key support) and screen reader announcements
9. Dark mode styling is preserved across all changed components
10. No regressions in training functionality, audio playback, or data persistence

## Tasks / Subtasks

- [ ] Task 1: Create shared `NavBar` component (AC: 1, 2, 8, 9)
  - [ ] Create `web/src/components/nav_bar.rs` with a `NavBar` component
  - [ ] Props: `title: &'static str`, `back_href: Option<&'static str>`, `children` slot for right-side action icons
  - [ ] Layout: flex row — back arrow button (left), centered title, right-side slot
  - [ ] Back arrow: use `<` or unicode left arrow character, styled as a rounded button with gray background circle (matching iOS back button pattern)
  - [ ] Title: centered via `flex-1 text-center`, bold, truncated with ellipsis on narrow screens
  - [ ] Right slot: flex row with gap for icon buttons
  - [ ] All buttons: `min-h-11 min-w-11` touch targets, proper focus rings, aria-labels
  - [ ] Dark mode: ensure all text/bg colors have dark: variants
  - [ ] Export from `components/mod.rs`

- [ ] Task 2: Create shared icon button helpers (AC: 1, 8)
  - [ ] Create small `NavIconButton` component or helper for the recurring icon-button pattern (help "?", settings gear, profile chart, info "i")
  - [ ] Props: `href: Option<&'static str>`, `on_click: Option<Callback>`, `label: &'static str`, `icon: &'static str`
  - [ ] Renders as `<a>` when href provided, `<button>` when on_click provided
  - [ ] Consistent styling: `min-h-11 min-w-11 flex items-center justify-center rounded-full` with hover/focus states
  - [ ] Use existing text/unicode icons: "?" for help, gear unicode for settings, chart unicode for profile, "i" for info

- [ ] Task 3: Update Start Page navigation (AC: 3)
  - [ ] Replace `<PageNav current="start" />` with `<NavBar>` — no back arrow on start page
  - [ ] Title: "Peach" — left-aligned on mobile (text-left), centered on md+ if preferred
  - [ ] Left side: info icon button linking to `/info`
  - [ ] Right side: profile icon (linking to `/profile`) + settings icon (linking to `/settings`)
  - [ ] Remove `<h1 class="sr-only">"Peach"</h1>` since the NavBar title replaces it (ensure the title is still an h1 or has appropriate heading role)

- [ ] Task 4: Update Settings, Profile, Info views navigation (AC: 1, 2)
  - [ ] Settings: `<NavBar title="Settings" back_href="/">` with help "?" button in right slot
  - [ ] Profile: `<NavBar title="Profile" back_href="/">` — no right-side actions (or help if applicable)
  - [ ] Info: `<NavBar title="Peach" back_href="/">` with version subtitle if needed
  - [ ] Remove all `<PageNav>` usage from these views

- [ ] Task 5: Update Pitch Comparison view layout (AC: 1, 4, 5, 8, 9)
  - [ ] Replace inline nav with `<NavBar>` — back_href="/", title based on mode ("Comparison Training" / "Interval Comparison"), right slot: help + settings + profile icons
  - [ ] Remove the old text-link navigation block
  - [ ] Add interval info display between nav and buttons: interval name (e.g. "Perfect Fifth ascending") centered, tuning system label below in smaller gray text
  - [ ] Redesign answer buttons as large blue rounded-rectangle cards:
    - Full width within the content area, generous vertical padding (py-8 or more)
    - Blue background (`bg-blue-500` / `bg-blue-600`), white text, rounded-2xl corners
    - Each button contains: circle icon (white circle with arrow inside) + label text below
    - Arrow icons: up arrow (unicode ↑ in a circle) for "Higher", down arrow (↓ in a circle) for "Lower"
    - Stacked vertically by default (flex-col gap-4), side-by-side on wide screens (md:flex-row)
    - Disabled state: lighter blue/gray, reduced opacity, cursor-not-allowed
  - [ ] Keep feedback indicator (thumbs up/down emoji) positioned between the buttons or as a brief overlay
  - [ ] Ensure keyboard shortcuts still work (ArrowUp/ArrowDown, H/L)

- [ ] Task 6: Update Pitch Matching view layout (AC: 1, 6, 7, 8, 9)
  - [ ] Replace inline nav with `<NavBar>` — same pattern as comparison view
  - [ ] Redesign the area below nav bar as a compact header section:
    - Left column: "Latest: X.X cents" with trend arrow on first line, "Best: X.X cents" on second line (reuse/adapt `TrainingStats` component)
    - Right column: signed cent deviation display — large text showing "+12 cents" or "-14 cents" with direction arrow, color-coded (green < 10, yellow 10-30, red > 30)
    - Center/below: interval name + tuning system label (for interval mode)
  - [ ] Vertical pitch slider: change container from `mt-4` with implicit height to `flex-1` or `flex-grow` filling all remaining viewport height below the header
  - [ ] The slider should expand to fill the screen, making it easy to use on mobile — this is the primary interaction element

- [ ] Task 7: Remove old PageNav component (AC: 2)
  - [ ] After all views are migrated, remove `web/src/components/page_nav.rs`
  - [ ] Remove export from `components/mod.rs`
  - [ ] Verify no remaining references

- [ ] Task 8: Responsive and accessibility testing (AC: 8, 9, 10)
  - [ ] Test all views at mobile (375px), tablet (768px), and desktop (1024px) widths
  - [ ] Verify comparison buttons stack/unstack correctly at breakpoint
  - [ ] Verify pitch slider fills available height on all screen sizes
  - [ ] Test keyboard navigation: Tab through nav icons, Enter/Space to activate, Escape for help modal
  - [ ] Test screen reader: nav landmarks, button labels, live regions for feedback
  - [ ] Test dark mode across all changed components
  - [ ] Verify no regressions in training session flow, audio, and persistence

## Dev Notes

### Current Navigation Pattern (to be replaced)

The existing `PageNav` component (`web/src/components/page_nav.rs`) renders text links ("Start", "Settings", "Profile", "Info") in a horizontal flex row, conditionally hiding the current page link. Training views duplicate this pattern inline with raw `<a>` tags.

The iOS app uses a consistent nav bar pattern across all screens: back arrow left, centered title, icon buttons right. This story replaces the text-link approach with that pattern.

### iOS Reference (from screenshots)

**Nav bar pattern:**
- Back: circular gray background button with `<` chevron
- Title: centered, bold, sometimes truncated on narrow screens
- Right icons: help (?), settings (gear), profile (chart) — not all present on every screen
- Start page: info (i) left, title left-aligned, chart + gear right

**Pitch Comparison (Horen & Vergleichen):**
- Interval name centered: "Reine Quinte aufwarts" with "Gleichstufige Stimmung" subtitle
- Two large blue cards filling the width, rounded corners (~16-20px radius)
- Each card: white circle with arrow icon (up/down) + label text ("Hoher"/"Tiefer")
- Portrait: stacked vertically with gap
- Landscape: side by side

**Pitch Matching (Stimmen & Treffen):**
- Stats top-left: "Zuletzt: 4,3 cents" with trend arrow, "Bestmarke: 4,3 cents"
- Deviation top-right: colored "+4 Cent" / "-14 Cent" with direction arrow (green/yellow)
- No interval name visible when in unison mode (only in interval mode)
- Draggable element centered, fills remaining space

### Key Code Locations

| File | Changes |
|---|---|
| `web/src/components/nav_bar.rs` | NEW — shared NavBar component |
| `web/src/components/page_nav.rs` | DELETE after migration |
| `web/src/components/start_page.rs` | Replace PageNav with NavBar, rearrange header |
| `web/src/components/pitch_comparison_view.rs` | Replace nav, redesign buttons as blue cards, add interval info |
| `web/src/components/pitch_matching_view.rs` | Replace nav, redesign header layout, make slider fill height |
| `web/src/components/settings_view.rs` | Replace PageNav with NavBar |
| `web/src/components/profile_view.rs` | Replace PageNav with NavBar |
| `web/src/components/info_view.rs` | Replace PageNav with NavBar |
| `web/src/components/training_stats.rs` | May need layout adjustments for new pitch matching header |
| `web/src/components/mod.rs` | Add nav_bar export, remove page_nav export |

### Tailwind Classes Reference

**Blue card buttons (comparison):**
```
// Enabled
"flex flex-col items-center justify-center gap-3 w-full rounded-2xl bg-blue-500 px-6 py-8 text-white text-xl font-semibold shadow-md hover:bg-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:ring-offset-2 dark:bg-blue-600 dark:hover:bg-blue-500"

// Disabled
"flex flex-col items-center justify-center gap-3 w-full rounded-2xl bg-gray-300 px-6 py-8 text-gray-500 text-xl font-semibold cursor-not-allowed dark:bg-gray-700 dark:text-gray-500"
```

**Circle arrow icon:**
```
"flex items-center justify-center w-14 h-14 rounded-full bg-white/30 text-white text-2xl"
```

**Slider fill remaining height:**
```
// Parent container should be flex-col with h-[calc(100vh-navheight)] or min-h-screen
// Slider wrapper: "flex-1 flex justify-center items-stretch"
```

### Architecture Constraints

- All components use Leptos 0.8.x `#[component]` macro with `view!` macro
- Navigation uses `leptos_router` `<A>` component for client-side routing (not raw `<a>`)
- `on:click` handlers for navigation must call `ev.prevent_default()` and use `leptos_router::hooks::use_navigate()`  when needed for programmatic nav — but `<A href>` is preferred for simple links
- Touch targets: minimum 44x44px (`min-h-11 min-w-11`)
- Dark mode: every color class needs a `dark:` variant
- Accessibility: `aria-label` on icon buttons, `role="navigation"` on nav, focus rings on all interactive elements

### Previous Story Intelligence (8.3)

Story 8.3 added disabled state handling to `TrainingCard` on the start page and SF2 loading indicators. The start page changes in this story must preserve the SF2 loading gate behavior — disabled cards when `SoundFontLoadStatus::Fetching`.

### Git Intelligence

Recent commits use imperative mood with story reference: "Fix audio playback reliability: ... (story 8.2)". Follow same pattern.

### Project Structure Notes

- All styling is Tailwind utility classes inline in `.rs` files — no separate CSS files
- `input.css` only contains Tailwind directives (`@source`, `@import`)
- Main content container: `<div class="mx-auto max-w-lg px-4">` — 512px max width. This may need to be relaxed for the comparison buttons to have more room on wider screens
- The `py-12` top padding on view containers may need adjustment since the NavBar provides its own spacing

### References

- [Source: web/src/components/page_nav.rs] — Current navigation component to be replaced
- [Source: web/src/components/pitch_comparison_view.rs] — Current comparison button markup (lines ~771-797)
- [Source: web/src/components/pitch_matching_view.rs] — Current pitch matching layout (lines ~732-845)
- [Source: web/src/components/start_page.rs] — Current start page header (lines ~95-163)
- [Source: web/src/components/training_stats.rs] — Current stats display component
- [Source: web/src/app.rs] — Router and main layout container (max-w-lg constraint)
- [Source: docs/project-context.md] — Coding conventions, component architecture rules
- iOS screenshots: Start, PitchComparison, PitchMatching, Settings, Profile views

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
