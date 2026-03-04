# Story 6.1: Info View & Complete Navigation

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to see basic app information and navigate smoothly between all views,
so that I know what I'm using and can access every part of the app.

## Acceptance Criteria

1. **Given** I navigate to `/info` **When** the Info view loads (FR47) **Then** I see the app name (Peach), developer name, copyright notice, version number, and acknowledgments **And** the content is minimal and static.

2. **Given** the Info view **When** I look at the top of the page **Then** I see navigation links to Start Page, Settings, and Profile.

3. **Given** any non-start-page view (Settings, Profile, Info) **When** I look at the top of the page **Then** I see navigation links to Start Page, Settings, and Profile **And** the current page's link is visually distinguished or absent.

4. **Given** the Info view **When** I navigate back **Then** I return to the Start Page.

5. **Given** all routes are implemented **When** I navigate between any views **Then** client-side routing handles all transitions without full page reloads **And** hub-and-spoke model is maintained (all views one level deep from start page).

## Tasks / Subtasks

- [ ] Task 1: Create shared navigation component (AC: #2, #3)
  - [ ] 1.1 Create `web/src/components/page_nav.rs` — a reusable `PageNav` component that renders links to Start Page, Settings, and Profile at the top of utility views
  - [ ] 1.2 Use `<A>` from `leptos_router` with the existing indigo link styling pattern
  - [ ] 1.3 Accept an optional `current` prop to omit or visually distinguish the active page link
  - [ ] 1.4 Register in `web/src/components/mod.rs`

- [ ] Task 2: Implement InfoView content (AC: #1, #2)
  - [ ] 2.1 Replace the skeleton in `web/src/components/info_view.rs` with full content
  - [ ] 2.2 Add `PageNav` at the top of the view
  - [ ] 2.3 Add content sections matching iOS InfoScreen:
    - App title: "Peach" (large heading)
    - Version: "0.1.0" (read from a constant or hardcoded for now)
    - Developer: "Michael Schürig"
    - Email: link to `mailto:michael@schuerig.de`
    - GitHub: link to `https://github.com/mschuerig/peach-web`
    - License: "MIT"
    - Copyright: "© 2026 Michael Schürig"
    - Acknowledgment: "GeneralUser GS" SoundFont by S. Christian Collins with link to `http://www.schristiancollins.com`
  - [ ] 2.4 Keep existing "Back to Start" link at bottom (pattern consistent with Settings/Profile)
  - [ ] 2.5 Use semantic HTML: `<section>`, `<address>` or `<dl>` for info groups, `<a>` for external links

- [ ] Task 3: Add PageNav to Settings view (AC: #3)
  - [ ] 3.1 Import and add `PageNav` at the top of `web/src/components/settings_view.rs`
  - [ ] 3.2 Pass `current="settings"` to mark active page
  - [ ] 3.3 Keep the existing "Back to Start" link at bottom

- [ ] Task 4: Add PageNav to Profile view (AC: #3)
  - [ ] 4.1 Import and add `PageNav` at the top of `web/src/components/profile_view.rs`
  - [ ] 4.2 Pass `current="profile"` to mark active page
  - [ ] 4.3 Keep the existing "Back to Start" link at bottom

- [ ] Task 5: Verify navigation completeness (AC: #4, #5)
  - [ ] 5.1 Verify all routes are accessible: `/`, `/training/comparison`, `/training/pitch-matching`, `/profile`, `/settings`, `/info`
  - [ ] 5.2 Verify hub-and-spoke model: all views return to start page
  - [ ] 5.3 Test keyboard navigation (Tab through links, Enter to activate)

## Dev Notes

### Architecture & Pattern Compliance

- **Crate boundary**: This story only touches the `web` crate (`web/src/components/`). No domain crate changes needed.
- **Component pattern**: All views use `<div class="py-12">` as outer container and `<h1 class="text-2xl font-bold dark:text-white">` for headings.
- **Navigation links**: Use `<A>` from `leptos_router::components::A` with `attr:class` for styling. The standard indigo link classes are:
  ```
  "min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:text-indigo-400 dark:hover:text-indigo-300"
  ```
- **"Back to Start" pattern**: Settings and Profile views use `mt-8 inline-block` prefix before the indigo classes. InfoView currently uses `mt-4` — should be updated to `mt-8` for consistency.
- **External links**: Use standard `<a href="..." target="_blank" rel="noopener noreferrer">` (not `<A>` from leptos_router, which is for internal routing).
- **Dark mode**: All text and background classes must include `dark:` variants. Follow the existing pattern in settings_view.rs and profile_view.rs.

### PageNav Component Design

The `PageNav` component should be a simple `<nav>` element with links styled consistently with the start page utility nav (gray text, not indigo — matching `start_page.rs` lines 92-105):
```
"min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200"
```

The component takes a `current: &'static str` prop (values: `"settings"`, `"profile"`, `"info"`) and renders:
- "Start" link → `/`  (always shown)
- "Settings" link → `/settings` (omit when `current == "settings"`)
- "Profile" link → `/profile` (omit when `current == "profile"`)

Wrap in `<nav aria-label="Page navigation" class="flex gap-6 text-sm mb-6">`.

### InfoView Content Structure

Follow the iOS InfoScreen layout adapted for web. Use grouped sections with spacing:

```
PageNav (top)

"Peach" (h1, large)
"Version 0.1.0" (caption/muted text)

Developer section:
  "Michael Schürig"
  michael@schuerig.de (mailto link)

Project section:
  GitHub (external link)
  License: MIT
  © 2026 Michael Schürig

Acknowledgments section:
  "GeneralUser GS" SoundFont by S. Christian Collins (external link)

"Back to Start" (bottom link)
```

Use `VStack` equivalent spacing: `space-y-8` or `gap-8` between sections. Muted/secondary text: `text-gray-500 dark:text-gray-400` or `text-sm text-gray-600 dark:text-gray-300`.

### Files to Create/Modify

| File | Action | Purpose |
|---|---|---|
| `web/src/components/page_nav.rs` | **Create** | Shared PageNav component |
| `web/src/components/mod.rs` | **Modify** | Export `PageNav` |
| `web/src/components/info_view.rs` | **Modify** | Full info content + PageNav |
| `web/src/components/settings_view.rs` | **Modify** | Add PageNav at top |
| `web/src/components/profile_view.rs` | **Modify** | Add PageNav at top |

### What NOT to Do

- Do NOT add PageNav to training views (ComparisonView, PitchMatchingView) — they have their own in-training navigation that stops the session via `on_nav_away()`
- Do NOT add PageNav to StartPage — it IS the hub; the utility nav at the bottom already serves this purpose
- Do NOT use JavaScript interop for version detection — hardcode `"0.1.0"` matching `web/Cargo.toml`
- Do NOT create a modal/overlay for Info — it's a separate routed page (route already exists in `app.rs`)
- Do NOT remove the existing "Back to Start" links at the bottom of views — keep them for users who scroll down

### Previous Work Patterns (from Epic 5)

- Story 5.2 SoundFont established the AudioWorklet pattern with `synth-processor.js` and separate `synth-worklet` crate
- The SoundFont acknowledgment in the Info view references the GeneralUser GS file used by story 5.2
- All recent stories follow: modify existing files over creating new ones where possible; one new component file is justified here for `PageNav` since it's shared across 3 views

### Project Structure Notes

- All routes already registered in `web/src/app.rs` (line 189 for `/info`)
- `InfoView` already imported in `app.rs` — no router changes needed
- `web/src/components/mod.rs` already exports `InfoView` — just needs new `PageNav` export

### References

- [Source: docs/planning-artifacts/epics.md#Epic 6, Story 6.1]
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture — Routing]
- [Source: docs/planning-artifacts/architecture.md#Component Architecture]
- [Source: docs/planning-artifacts/ux-design-specification.md#Info View]
- [Source: docs/planning-artifacts/ux-design-specification.md#Navigation Rules]
- [Source: docs/project-context.md#Leptos Framework Rules]
- [Source: iOS reference — Peach/Info/InfoScreen.swift]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
