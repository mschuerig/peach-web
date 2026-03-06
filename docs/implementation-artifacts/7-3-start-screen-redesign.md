# Story 7.3: Start Screen Redesign

Status: ready-for-dev

## Story

As a musician,
I want the start screen organized into "Single Notes" and "Intervals" sections with descriptive labels ("Hear & Compare", "Tune & Match"),
so that training modes are clearly named and visually grouped.

## Context

The iOS sibling app redesigned the start screen from four flat buttons into two grouped sections with card-style training buttons. The current peach-web start page has four buttons labeled "Comparison", "Pitch Matching", "Interval Comparison", and "Interval Pitch Matching" in a plain vertical list.

The new design groups by note type (single notes vs intervals) and uses verb-based labels that describe the activity rather than the technical term.

**iOS reference:** `Peach/Start/StartScreen.swift`

### iOS Start Screen Layout

```
┌──────────────────────────┐
│       Single Notes       │  ← section header (.secondary)
│ ┌──────────────────────┐ │
│ │ 🎧 Hear & Compare    │ │  ← NavigationLink card
│ │   [sparkline] 12.3 ¢ │ │     (sparkline added in story 7.6)
│ └──────────────────────┘ │
│ ┌──────────────────────┐ │
│ │ 🎯 Tune & Match      │ │
│ │   [sparkline] 8.1 ¢  │ │
│ └──────────────────────┘ │
│                          │
│        Intervals         │  ← section header (.secondary)
│ ┌──────────────────────┐ │
│ │ 🎧 Hear & Compare    │ │
│ │   [sparkline] 15.7 ¢ │ │
│ └──────────────────────┘ │
│ ┌──────────────────────┐ │
│ │ 🎯 Tune & Match      │ │
│ │   [sparkline]         │ │
│ └──────────────────────┘ │
└──────────────────────────┘
```

Sparklines are deferred to story 7.6. This story focuses on the layout, labels, and card styling.

Depends on: Story 7.1 (TrainingMode — needed for card structure, even though sparklines come later).

## Acceptance Criteria

1. **AC1 — Section grouping:** Start page shows two sections: "Single Notes" and "Intervals", each with a secondary-styled header
2. **AC2 — Card labels:** Each section has two cards labeled "Hear & Compare" and "Tune & Match"
3. **AC3 — Card icons:** "Hear & Compare" cards show an ear icon, "Tune & Match" cards show a target icon (use CSS or Unicode approximations since no SF Symbols in web)
4. **AC4 — Card styling:** Cards have rounded corners, subtle background (e.g. surface/muted color), full-width, minimum 44px touch target height, padding
5. **AC5 — Card navigation:** "Hear & Compare" in Single Notes navigates to `/training/comparison` (unison mode). "Tune & Match" in Single Notes navigates to `/training/pitch-matching` (unison mode). Interval section cards navigate with the user's selected intervals as query params (same as current behavior).
6. **AC6 — Press feedback:** Cards show a brief opacity change on click/tap (visual press indication)
7. **AC7 — Responsive layout:** On wide viewports (landscape or large screens), the two sections display side-by-side. On narrow viewports (portrait phone), they stack vertically.
8. **AC8 — Navigation bar:** Toolbar/nav area retains Info, Profile, and Settings links. Consider moving from inline page links to a header bar matching iOS pattern (info left, profile + settings right).
9. **AC9 — ProfilePreview removed from start page:** The old inline profile preview (miniature piano keyboard SVG) is removed from the start page. Profile access is via the nav link. (The ProfilePreview component itself may be removed entirely in story 7.4 when the profile view is revamped.)
10. **AC10 — Accessibility:** Cards have accessible labels (e.g. "Hear and Compare, Single Notes"), section headers are semantically correct (heading elements or aria-label)
11. **AC11 — Existing routes unchanged:** All route paths remain the same (`/training/comparison`, `/training/pitch-matching`, with `?intervals=` params)

## Tasks / Subtasks

- [ ] Task 1: Restructure start_page.rs layout (AC: 1, 2, 7)
  - [ ] Replace the four flat buttons with two sections
  - [ ] Each section: header text + two card components
  - [ ] Use CSS flexbox/grid: vertical stack on narrow, side-by-side on wide (media query or container query)

- [ ] Task 2: Create training card component (AC: 3, 4, 6)
  - [ ] Card component accepting: label (&str), icon (ear/target), href (route string)
  - [ ] Card HTML: anchor tag wrapping a div with icon + label text
  - [ ] Styling: rounded-xl, bg-surface/muted, px-4 py-3, min-h-[44px], full-width
  - [ ] Press feedback: CSS `:active` opacity transition (opacity 0.7 on press, ease-in-out 150ms)
  - [ ] Icon options: use Unicode characters (e.g. "👂" for ear, "🎯" for target) or Tailwind/SVG icons

- [ ] Task 3: Update navigation links (AC: 5, 8)
  - [ ] Wire card hrefs: Single Notes → unison routes, Intervals → interval routes with encoded params
  - [ ] Reuse existing `navigate_with_intervals()` helper for interval encoding
  - [ ] Update nav bar: consider header layout with Info (left), Profile + Settings (right)

- [ ] Task 4: Remove ProfilePreview from start page (AC: 9)
  - [ ] Remove `ProfilePreview` component usage from start_page.rs
  - [ ] Keep the `profile_preview.rs` file for now (story 7.4 will determine if it's still needed)

- [ ] Task 5: Accessibility (AC: 10)
  - [ ] Section headers as `<h2>` elements
  - [ ] Card links with `aria-label` combining mode and section (e.g. "Hear and Compare, Single Notes")
  - [ ] Ensure tab order follows visual order (Single Notes cards first, then Intervals)

- [ ] Task 6: Visual polish
  - [ ] Section spacing: ~28px between sections, ~10px between cards within a section
  - [ ] Header text: secondary color, smaller font (text-sm or similar)
  - [ ] Page padding and centering consistent with other views

- [ ] Task 7: Verify (AC: 11)
  - [ ] All four training routes still work correctly
  - [ ] Interval query params still encoded and decoded correctly
  - [ ] `trunk serve` and manual browser testing

## Dev Notes

### iOS to Web Mapping

| iOS Element | peach-web Equivalent |
|---|---|
| `NavigationLink(value:)` | `<a href="...">` Leptos component |
| `Label("Hear & Compare", systemImage: "ear")` | Unicode icon + text in div |
| `.buttonStyle(TrainingCardButtonStyle())` | CSS `:active` opacity + transition |
| `HStack` (compact) / `VStack` (regular) | CSS flexbox with media query |
| `.font(.title3).foregroundStyle(.secondary)` | Tailwind: `text-sm text-muted-foreground` or similar |

### Design Decisions

- **No SF Symbols:** The web doesn't have SF Symbols. Use Unicode emoji (👂🎯) or simple SVG icons. Unicode is simplest and works everywhere.
- **CSS-only press feedback:** No JavaScript needed — CSS `:active` pseudo-class with opacity transition matches the iOS `TrainingCardButtonStyle`.
- **Card as anchor tag:** Cards are navigation links, so semantically they should be `<a>` elements, not buttons. This also gives free keyboard navigation and right-click context menus.
- **Leave placeholder for sparklines:** Each card should have a container div below the label where story 7.6 will insert sparklines. For now it can be empty or hidden.

### Architecture Compliance

- **Web crate only:** This story changes only `web/src/components/start_page.rs` and CSS. No domain changes.
- **Routes unchanged:** Same paths, same query param encoding.
