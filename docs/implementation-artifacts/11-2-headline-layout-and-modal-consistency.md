# Story 11.2: Headline Layout and Modal Consistency

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want consistent headline layouts and modal behavior across all pages,
so that the app feels polished and predictable.

## Acceptance Criteria

1. On all pages (start page, training pages, settings, profile, info), the headline layout follows this structure: left pill flush left, title centered in the remaining space, right pill flush right.
2. The help modal is closed by a "Done" button (no decoration) rendered inside a pill in the top left corner of the modal, replacing the current top-right "Done" button.
3. The info view is closed by a "Done" button (no decoration) rendered inside a pill in the top left corner, replacing the current back-arrow pill.
4. Help modals display without a grey backdrop overlay — matching the info view's no-overlay appearance.

## Tasks / Subtasks

- [x] Task 1: Fix headline layout to center title between pills (AC: #1)
  - [x] 1.1 In `NavBar` (`web/src/components/nav_bar.rs`), remove the `title_left` prop and always center the title. The current layout uses `w-11 shrink-0` for the left slot, `flex-1 text-center` for the title, and `flex shrink-0` for the right slot. The problem is that the right slot's width isn't balanced against the left slot — the title is centered within its flex-1 space, not centered on the page. To truly center: the left and right containers must have equal width, or use absolute positioning for the title, or use a CSS grid with three columns where the center column is auto-sized.
  - [x] 1.2 Recommended approach: use a three-column CSS grid layout — `grid grid-cols-[auto_1fr_auto]` with the left slot `justify-self-start`, title `justify-self-center`, and right slot `justify-self-end`. This gives true visual centering regardless of left/right content widths.
  - [ ] 1.3 Verify on all pages: start page (info pill left, profile+settings right), training pages (back pill left, help+settings+profile right), settings page (back pill left, no right), profile page (back pill left, no right), info page (back or Done pill left, no right). *(deferred to user — agent cannot verify in browser)*

- [x] Task 2: Update help modal — "Done" pill in top left, no overlay (AC: #2, #4)
  - [x] 2.1 In `HelpModal` (`web/src/components/help_content.rs`, lines 122-145), move the "Done" button from the top-right position to the top-left position. Currently the layout is `<h2>` left + `<button>Done</button>` right. Reverse to `<button>Done</button>` left + `<h2>` right, or put Done first in the flex container.
  - [x] 2.2 Style the "Done" button as a pill: use the same pill styling as the NavBar back button (`min-h-11 min-w-11 flex items-center justify-center rounded-full bg-gray-100 dark:bg-gray-800`). The text should have no extra decoration — just "Done" in a pill shape.
  - [x] 2.3 Remove the grey backdrop overlay. Currently uses `backdrop:bg-black/50` on the `<dialog>` element. Remove or change to `backdrop:bg-transparent`. Note: the `<dialog>` element shown via `show_modal()` always creates a backdrop — set it transparent.
  - [x] 2.4 Since without a backdrop overlay the help modal needs to visually separate from the page behind it, ensure the dialog itself has sufficient visual weight (it already has `bg-white dark:bg-gray-800` and `rounded-lg` which should be sufficient).

- [x] Task 3: Update info view — "Done" pill in top left corner (AC: #3)
  - [x] 3.1 In `InfoView` (`web/src/components/info_view.rs`), replace the NavBar back button (which shows `‹` back arrow) with a "Done" text pill. Use the `left_content` prop on NavBar to render a custom pill button with "Done" text instead of the default back arrow.
  - [x] 3.2 The "Done" pill should navigate back to `/` (same as the current back button). Use the same pill styling as the NavBar back button.
  - [x] 3.3 Since info view has no right-side icons, the title "Info" should still be centered in the full remaining space.

## Dev Notes

### Architecture & Patterns

- **Tailwind CSS only** — no custom CSS files. All styling via utility classes.
- **NavBar component** (`web/src/components/nav_bar.rs`): Current flexbox layout with left (`w-11 shrink-0`), center (`flex-1 text-center`), and right (`flex shrink-0`) sections. Title centering is only visual within the flex-1 space, not truly centered on the page when left and right have different widths.
- **HelpModal** (`web/src/components/help_content.rs`, lines 84-146): Uses native `<dialog>` element with `show_modal()`. Current layout has title left, "Done" button right. `backdrop:bg-black/50` creates grey overlay.
- **InfoView** (`web/src/components/info_view.rs`): Full page view (not a dialog). Uses standard NavBar with back button. No overlay.

### Key Implementation Details

**NavBar centering approach**: The cleanest solution for true centering is CSS Grid with `grid grid-cols-[auto_1fr_auto]`. The left and right cells are `auto` width, and the title in the center cell is `text-center`. With grid, the center cell spans the full middle area and the title is centered within it regardless of the widths of left/right cells.

Alternative: keep flexbox but make left and right containers equal width by setting both to `min-w-11` and using `flex-1` on both sides of the title. However, this constrains the right pill group width.

**Dialog backdrop**: The `<dialog>` element opened via `show_modal()` always creates a `::backdrop` pseudo-element. Setting `backdrop:bg-transparent` (Tailwind) will make it invisible. Alternatively, use `dialog.show()` instead of `dialog.show_modal()` which doesn't create a backdrop at all — but this loses the native Escape-to-close and top-layer behavior. Best to keep `show_modal()` and just set the backdrop transparent.

### Key Files to Modify

| File | Changes |
|---|---|
| `web/src/components/nav_bar.rs` | Restructure layout for true title centering; remove `title_left` prop |
| `web/src/components/help_content.rs` | Move "Done" to top-left pill; remove backdrop overlay |
| `web/src/components/info_view.rs` | Replace back arrow with "Done" pill text |
| `web/src/components/pitch_comparison_view.rs` | Remove `title_left=true` prop from NavBar (if prop is removed) |
| `web/src/components/pitch_matching_view.rs` | Remove `title_left=true` prop from NavBar (if prop is removed) |

### NavBar Current Props (from story 11.1)

- `title: &'static str` — page title
- `back_href: Option<String>` — back button destination
- `on_back: Option<Callback<()>>` — back button click handler
- `title_left: bool` — left-align title (to be removed)
- `pill_group: bool` — wrap right icons in pill container
- `left_content: Option<ViewFn>` — custom left content (replaces back button)
- `children: Option<Children>` — right-side icon buttons

### Unicode Characters in Use

| Icon | Character | Usage |
|---|---|---|
| Info | `\u{24D8}` ⓘ | Start page left pill |
| Help | `"?"` | Training page right pill |
| Settings | `\u{2699}\u{FE0F}` | Training/start page right pill |
| Profile/Chart | `\u{1F4CA}` | Training/start page right pill |
| Back | `\u{2039}` ‹ | Sub-pages left pill |

### Testing

- `cargo clippy --workspace` — lint check
- Manual browser testing: verify all pages have properly centered titles
- Check training pages with 3 right icons vs start page with 2 right icons — title should be centered in both
- Check pages with only left pill (settings, profile) — title still centered
- Check help modal opens without grey overlay, "Done" is top-left in a pill
- Check info view has "Done" pill instead of back arrow
- Test Escape key still closes help modal and navigates back from info view
- Check dark mode for all changes

### Project Structure Notes

- All changes are in `web/` crate — no domain changes needed
- No new files — all modifications to existing components
- Follows existing Tailwind CSS patterns throughout

### References

- [Source: web/src/components/nav_bar.rs] — NavBar and NavIconButton components
- [Source: web/src/components/help_content.rs#L84-146] — HelpModal component
- [Source: web/src/components/info_view.rs] — InfoView page component
- [Source: web/src/components/pitch_comparison_view.rs#L688-692] — Comparison training NavBar usage
- [Source: web/src/components/pitch_matching_view.rs#L717-721] — Pitch matching NavBar usage
- [Source: docs/implementation-artifacts/11-1-ui-consistency-and-sound-level.md] — Previous story context
- [Source: docs/project-context.md] — Project conventions and rules

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

None — clean implementation, no debugging needed.

### Completion Notes List

- Converted NavBar from flexbox to CSS grid (`grid grid-cols-[auto_1fr_auto]`) for true title centering regardless of left/right content widths.
- Removed `title_left` prop from NavBar and all call sites (pitch_comparison_view, pitch_matching_view).
- HelpModal: moved "Done" button to top-left, styled as pill (`rounded-full bg-gray-100`), removed grey backdrop overlay (`backdrop:bg-transparent`).
- InfoView: replaced back arrow with "Done" pill button using `left_content` prop and `navigate()` (per project rules, `navigate()` resolves base internally — no `base_href` needed).
- Removed unused `base_href` import from info_view.rs.
- All 357 tests pass (326 domain + 31 web), `cargo clippy` clean, no regressions.
- Task 1.3 (manual browser verification) deferred to user — agent cannot verify in browser.

### Change Log

- 2026-03-08: Implemented all tasks — NavBar grid layout, help modal pill/no-overlay, info view Done pill.

### File List

- web/src/components/nav_bar.rs (modified — flexbox→grid layout, removed title_left prop)
- web/src/components/help_content.rs (modified — Done pill top-left, transparent backdrop)
- web/src/components/info_view.rs (modified — Done pill replaces back arrow, removed base_href import)
- web/src/components/pitch_comparison_view.rs (modified — removed title_left=true)
- web/src/components/pitch_matching_view.rs (modified — removed title_left=true)
- docs/implementation-artifacts/sprint-status.yaml (modified — story status)
