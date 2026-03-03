# Story 1.4: App Shell & Routing

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to see a start page with a Comparison training button and navigate between views,
so that I can access the app's features from a clean, simple hub.

## Acceptance Criteria

1. **AC1 — Start Page renders at `/`:** Given the app is loaded, when I navigate to `/`, then I see the Start Page with a "Comparison" button as the primary action, navigation links for Settings, Profile, and Info, and no onboarding/tutorial/welcome message.

2. **AC2 — Settings navigation:** Given I am on the Start Page, when I click the Settings link, then I navigate to `/settings` and see a placeholder Settings view with a way to navigate back to the Start Page.

3. **AC3 — Profile navigation:** Given I am on the Start Page, when I click the Profile link, then I navigate to `/profile` and see a placeholder Profile view.

4. **AC4 — Hub-and-spoke back navigation:** Given I am on any secondary view (Settings, Profile, Info), when I navigate back, then I return to the Start Page.

5. **AC5 — Training route exists:** Given the routes are configured, when I access `/training/comparison`, then a placeholder Comparison Training view loads.

6. **AC6 — Accessibility shell:** Given the app is rendered, when I inspect the HTML, then a "Skip to main content" link is present at the top of the page, and semantic HTML elements (`<nav>`, `<main>`) are used for navigation and main content areas.

7. **AC7 — Responsive layout:** Given the layout, when viewed on any viewport width, then content is centered at a comfortable maximum width on desktop, and single-column layout works on mobile without horizontal scrolling.

## Tasks / Subtasks

- [ ] Task 1: Create `web/src/app.rs` with Router, Routes, and all route definitions (AC: 1,2,3,4,5)
  - [ ] 1.1 Set up `<Router>` wrapping the app
  - [ ] 1.2 Define all routes: `/`, `/training/comparison`, `/training/pitch-matching`, `/profile`, `/settings`, `/info`
  - [ ] 1.3 Add 404 fallback route
  - [ ] 1.4 Update `main.rs` to mount `App` from `app.rs`

- [ ] Task 2: Create `web/src/components/mod.rs` and view component files (AC: 1,2,3,4,5)
  - [ ] 2.1 Create `components/mod.rs` with module declarations
  - [ ] 2.2 Create `components/start_page.rs` — `StartPage` component with Comparison button (primary), navigation links (Settings, Profile, Info)
  - [ ] 2.3 Create `components/comparison_view.rs` — placeholder `ComparisonView`
  - [ ] 2.4 Create `components/pitch_matching_view.rs` — placeholder `PitchMatchingView`
  - [ ] 2.5 Create `components/profile_view.rs` — placeholder `ProfileView`
  - [ ] 2.6 Create `components/settings_view.rs` — placeholder `SettingsView`
  - [ ] 2.7 Create `components/info_view.rs` — placeholder `InfoView`

- [ ] Task 3: Implement app shell layout with accessibility (AC: 6,7)
  - [ ] 3.1 Add "Skip to main content" link at top of page
  - [ ] 3.2 Use `<nav>` for navigation, `<main id="main-content">` for primary content
  - [ ] 3.3 Wrap content in responsive container: centered, max-width, single-column

- [ ] Task 4: Style with Tailwind CSS (AC: 1,7)
  - [ ] 4.1 Start Page layout: Comparison button prominent, secondary links below
  - [ ] 4.2 Responsive container: `max-w-lg mx-auto px-4` or similar
  - [ ] 4.3 Dark mode support via `prefers-color-scheme` / Tailwind `dark:` utilities

- [ ] Task 5: Verify build and manual test (AC: all)
  - [ ] 5.1 `trunk serve` compiles without errors
  - [ ] 5.2 All routes navigate correctly
  - [ ] 5.3 Back navigation returns to Start Page from all views
  - [ ] 5.4 Skip link works
  - [ ] 5.5 `cargo clippy -p web` passes

## Dev Notes

### Architecture Compliance

**File Structure (from architecture.md):**

```
web/src/
├── main.rs          # Leptos mount, composition root
├── app.rs           # Top-level App component, router setup
└── components/
    ├── mod.rs
    ├── start_page.rs
    ├── comparison_view.rs
    ├── pitch_matching_view.rs
    ├── profile_view.rs
    ├── settings_view.rs
    └── info_view.rs
```

Only create the files listed above. Do NOT create `adapters/`, `bridge.rs`, or `signals.rs` yet — those belong to later stories.

**Router Setup (Leptos 0.8 / leptos_router 0.8):**

```rust
use leptos_router::{
    components::{Router, Route, Routes, A},
};
use leptos_router_macro::path;
```

- `<Router>` wraps the entire app
- `<Routes fallback=|| view! { <p>"Page not found."</p> }>` contains route definitions
- `<Route path=path!("/") view=StartPage />`
- `<A href="/settings">"Settings"</A>` for client-side navigation (NOT `<a>`)

**Route Definitions:**

| Route | Component | Notes |
|---|---|---|
| `/` | `StartPage` | Hub page |
| `/training/comparison` | `ComparisonView` | Placeholder for now |
| `/training/pitch-matching` | `PitchMatchingView` | Placeholder for now |
| `/profile` | `ProfileView` | Placeholder |
| `/settings` | `SettingsView` | Placeholder |
| `/info` | `InfoView` | Placeholder |

Interval mode (`?interval=true`) is NOT needed in this story — it will be handled when training views are implemented.

**Component Naming Convention:**

- `PascalCase` function names: `fn StartPage()`, `fn ComparisonView()`, etc.
- One component per file, file names `snake_case`: `start_page.rs`, `comparison_view.rs`
- All return `impl IntoView`

### Start Page Layout

Per UX specification, the Start Page is the hub with this hierarchy:

1. **Comparison button** — primary, most prominent (hero action)
2. **Pitch Matching button** — secondary, below comparison
3. **Visual separator** (subtle divider)
4. **Interval Comparison button** — secondary, below separator
5. **Interval Pitch Matching button** — secondary, below separator
6. **Settings, Profile, Info** — tertiary links (text links at bottom)

**Behavior rules:**
- No onboarding, tutorial, or welcome message — ever
- Identical on every visit regardless of time elapsed
- No "welcome back," streak, or activity summary

For this story, all training buttons navigate to their routes. The buttons are non-functional (no audio/sessions yet) but the routing must work.

### Accessibility Requirements

1. **Skip link:** `<a href="#main-content" class="sr-only focus:not-sr-only ...">Skip to main content</a>` at the very top of the `<body>` output
2. **Semantic HTML:** `<nav>` for navigation links, `<main id="main-content">` for content
3. **Focus indicators:** Tailwind's `focus:ring` utilities on all interactive elements
4. **Keyboard navigation:** All links/buttons reachable via Tab
5. **Dark mode:** Use `dark:` Tailwind variants for all color utilities

### Responsive Layout

- Mobile-first: base styles are mobile
- Content container: centered, max-width ~`max-w-lg` or `max-w-xl`, horizontal padding
- Single-column layout throughout — no breakpoint-dependent layout changes needed for this story
- Touch targets: minimum 44x44px on all buttons/links (use `min-h-11 min-w-11` or `p-3` etc.)

### Tailwind CSS 4.x Notes

The project uses **Tailwind CSS 4.x** (configured via Trunk with version 4.2.1). The `input.css` uses the v4 syntax:
```css
@import 'tailwindcss';
@source "./web/src/**/*.rs";
```

Tailwind 4 auto-detects utility classes from source files. No `tailwind.config.js` needed — configuration is done via CSS `@theme` directives if customization is required.

### Placeholder View Pattern

All placeholder views (except StartPage) should follow this pattern:

```rust
#[component]
fn SettingsView() -> impl IntoView {
    view! {
        <div>
            <h1>"Settings"</h1>
            <A href="/">"Back to Start"</A>
        </div>
    }
}
```

Keep placeholders minimal — just a title and a back link. No lorem ipsum, no fake content.

### Current Codebase State

- `web/src/main.rs`: Currently has a simple `App` component rendering `<h1>"Peach"</h1>`. This will be refactored to import and mount the `App` from `app.rs`.
- `web/Cargo.toml`: Already includes `leptos` (0.8, csr), `leptos_router` (0.8), `console_log`, `log`, `console_error_panic_hook`, `getrandom`
- `index.html`: Viewport meta tag already present, Tailwind CSS linked via Trunk
- `input.css`: Tailwind 4 directives configured, scanning `web/src/**/*.rs`
- Domain crate: 180 tests passing, all types from stories 1.1-1.3 available

### Previous Story Intelligence (Story 1.3)

Key learnings that apply to this story:

- **Clippy compliance:** Run `cargo clippy -p web` early and fix warnings immediately
- **NaN guards pattern:** Not directly applicable to UI, but keep the defensive coding mindset
- **Module structure:** Follow the same `mod.rs` + individual files pattern used in `domain/src/training/`
- **Code review caught:** Missing validation, wrong data structure choices — keep code clean from the start

### Git History Context

Recent commits show sequential story completion (1.1 → 1.2 → 1.3), all following the pattern:
1. Create story file (ready-for-dev)
2. Implement story
3. Code review + fixes → done

The web crate is untouched since story 1.1 scaffold. This is the FIRST story that modifies `web/src/` beyond `main.rs`.

### Project Structure Notes

- Alignment: This story establishes the `web/src/components/` directory structure that all subsequent UI stories will build upon
- The `app.rs` file becomes the routing hub — future stories add functionality to existing view components
- No conflicts with domain crate — this story is purely `web` crate changes

### References

- [Source: docs/planning-artifacts/epics.md#Story 1.4: App Shell & Routing]
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: docs/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/planning-artifacts/ux-design-specification.md#Start Page]
- [Source: docs/planning-artifacts/ux-design-specification.md#Hub-and-Spoke Model]
- [Source: docs/planning-artifacts/ux-design-specification.md#Responsive Design]
- [Source: docs/planning-artifacts/ux-design-specification.md#Accessibility]
- [Source: docs/project-context.md#Leptos Framework Rules]
- [Source: docs/project-context.md#Routing]
- [Leptos Router 0.8 API: https://docs.rs/leptos_router/latest/leptos_router/]
- [Leptos Router Example: https://github.com/leptos-rs/leptos/blob/main/examples/router/src/lib.rs]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
