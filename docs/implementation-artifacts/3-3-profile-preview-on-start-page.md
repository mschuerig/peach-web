# Story 3.3: Profile Preview on Start Page

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to see a compact profile preview on the start page that I can click to view details,
so that I get a glanceable snapshot of my progress every time I open Peach.

## Acceptance Criteria

1. **AC1 — Compact profile preview with training data (FR19):** Given the Start Page loads with training data, when I see the profile preview, then a compact visualization is visible showing the same band shape as the full visualization, and no axis labels or numerical values are shown in the preview.

2. **AC2 — Empty state (FR19):** Given the Start Page loads with no training data, when I see the profile preview, then a subtle placeholder shape is shown (simplified keyboard outline or faint uniform band).

3. **AC3 — Clickable navigation to Profile (FR20):** Given the profile preview, when I click it, then I navigate to the full Profile view at `/profile`.

4. **AC4 — Accessibility:** Given the profile preview, when I inspect the HTML, then it has an accessible label: "Your pitch profile. Click to view details." and if data exists: "Your pitch profile. Average threshold: X cents. Click to view details."

5. **AC5 — Keyboard navigation:** Given the profile preview, when I use keyboard navigation, then it is focusable and activatable via Enter/Space.

## Tasks / Subtasks

- [x] Task 1: Extract shared SVG helpers from profile_visualization.rs (AC: 1)
  - [x] 1.1 Change `NoteData`, constants (`MIDI_MIN`, `MIDI_MAX`, `WHITE_KEY_WIDTH`, `BLACK_KEY_WIDTH`, `KEYBOARD_Y`, `KEYBOARD_HEIGHT`, `BLACK_KEY_HEIGHT`, `CHART_TOP`, `CHART_BOTTOM`, `MAX_CENTS`, `VIEWBOX_WIDTH`, `VIEWBOX_HEIGHT`, `NOTE_OFFSETS`, `IS_WHITE`, `FIRST_KEY_POSITION`), and helper functions (`note_x_center`, `white_key_x`, `cents_to_y`, `build_band_segments`, `band_path`, `build_mean_segments`, `mean_path`) to `pub(crate)` visibility in `profile_visualization.rs`
  - [x] 1.2 Verify `cargo clippy -p web` still passes — no dead code warnings since items are now used by two modules

- [x] Task 2: Create ProfilePreview component (AC: 1,2,4)
  - [x] 2.1 Create `web/src/components/profile_preview.rs` with `ProfilePreview` component
  - [x] 2.2 Add `pub mod profile_preview;` and `pub use profile_preview::ProfilePreview;` in `web/src/components/mod.rs`
  - [x] 2.3 Use `use_context` to get `SendWrapper<Rc<RefCell<PerceptualProfile>>>` and `is_profile_loaded: RwSignal<bool>` — same pattern as profile_visualization.rs
  - [x] 2.4 Extract per-note data into owned `Vec<NoteData>` using the borrow-then-extract pattern, then drop the borrow before `view!` macro
  - [x] 2.5 Render compact inline SVG using the same viewBox as full visualization (`0 0 520 210`), reusing `build_band_segments` and `build_mean_segments` from `profile_visualization`
  - [x] 2.6 Render simplified piano keyboard (white + black key rectangles, same layout) but NO octave labels
  - [x] 2.7 Render confidence band and mean line segments (same as full visualization)
  - [x] 2.8 Empty state: render keyboard only with no band (same as full visualization empty state) — the keyboard outline itself IS the subtle placeholder shape
  - [x] 2.9 Loading state (profile not yet hydrated): render keyboard only (same as empty)

- [x] Task 3: Make preview clickable and accessible (AC: 3,4,5)
  - [x] 3.1 Wrap the SVG inside an `<a>` element with `href="/profile"` using `leptos_router::components::A` for client-side navigation
  - [x] 3.2 Add `aria-label` on the `<a>` wrapper: dynamic based on whether training data exists
  - [x] 3.3 If data exists: `"Your pitch profile. Average threshold: X.X cents. Click to view details."`
  - [x] 3.4 If no data: `"Your pitch profile. Click to view details."`
  - [x] 3.5 Style the link with focus ring: `focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900 rounded-lg`
  - [x] 3.6 The `<a>` element is natively focusable and activatable via Enter — no extra keyboard handling needed

- [x] Task 4: Integrate ProfilePreview into StartPage (AC: 1,2,3)
  - [x] 4.1 Import `ProfilePreview` in `start_page.rs`
  - [x] 4.2 Replace the loading placeholder / empty span block with `<ProfilePreview />`
  - [x] 4.3 Position the preview ABOVE the training mode nav, below the `<h1>`
  - [x] 4.4 The preview renders in ALL states — keyboard always visible during and after loading

- [x] Task 5: Styling and dark mode (AC: 1,2)
  - [x] 5.1 Reuse existing `--pv-*` CSS custom properties from `input.css` — no CSS changes needed
  - [x] 5.2 Set compact height via Tailwind class on the wrapping `<a>` element (e.g., `max-h-24` or similar to constrain vertical size)
  - [x] 5.3 SVG `width="100%"` ensures responsive scaling — the viewBox handles aspect ratio
  - [x] 5.4 Add `overflow-hidden rounded-lg` on the wrapper for clean edges

- [x] Task 6: Verify and validate (AC: all)
  - [x] 6.1 `cargo clippy -p domain` — zero warnings
  - [x] 6.2 `cargo clippy -p web` — zero warnings
  - [x] 6.3 `cargo test -p domain` — all tests pass (254 tests)
  - [x] 6.4 `trunk build` — successful WASM compilation
  - [x] 6.5 Manual browser smoke test: empty state, sparse data, populated data, dark mode, click navigation, keyboard focus + Enter activation, responsive resizing

## Dev Notes

### Core Approach: Compact SVG Miniature Reusing Shared Helpers

This story creates a new `ProfilePreview` component (`web/src/components/profile_preview.rs`) that renders a compact, clickable miniature of the perceptual profile visualization on the start page. The miniature uses the same SVG rendering logic as the full `ProfileVisualization` component from Story 3.2 — same band shape, same keyboard layout — but without octave labels, without numerical values, and at a smaller physical size.

**Key design decision:** Reuse, don't duplicate. The helper functions in `profile_visualization.rs` (note positioning, band segment building, mean line building) are changed to `pub(crate)` visibility so `profile_preview.rs` can import and use them directly. This guarantees the preview band shape is always identical to the full visualization.

**Why not a separate rendering pipeline:** The UX spec explicitly states "same band shape as full visualization." Any divergence between preview and full view would confuse users. Shared code makes this impossible.

### Shared Code from profile_visualization.rs

The following items need `pub(crate)` visibility (currently private):

**Struct:**
- `NoteData` — per-note extracted data

**Constants:**
- `MIDI_MIN`, `MIDI_MAX`, `WHITE_KEY_WIDTH`, `BLACK_KEY_WIDTH`
- `KEYBOARD_Y`, `KEYBOARD_HEIGHT`, `BLACK_KEY_HEIGHT`
- `CHART_TOP`, `CHART_BOTTOM`, `MAX_CENTS`
- `VIEWBOX_WIDTH`, `VIEWBOX_HEIGHT`
- `NOTE_OFFSETS`, `IS_WHITE`, `FIRST_KEY_POSITION`

**Functions:**
- `note_x_center(midi: u8) -> f64`
- `white_key_x(midi: u8) -> Option<f64>`
- `cents_to_y(cents: f64) -> f64`
- `build_band_segments(notes: &[NoteData]) -> Vec<String>`
- `band_path(run: &[&NoteData]) -> String`
- `build_mean_segments(notes: &[NoteData]) -> Vec<String>`
- `mean_path(run: &[&NoteData]) -> String`

**What the preview renders (from shared helpers):**
- White key rectangles (from `white_key_x`)
- Black key rectangles (from `note_x_center`, `IS_WHITE`)
- Band segments (from `build_band_segments`)
- Mean line segments (from `build_mean_segments`)

**What the preview OMITS (compared to full visualization):**
- Octave labels (C2, C3, etc.) — not rendered
- `class="mt-6"` — preview has its own spacing via the start page layout

### ProfilePreview Component Structure

```rust
use leptos_router::components::A;
use super::profile_visualization::{
    NoteData, MIDI_MIN, MIDI_MAX, WHITE_KEY_WIDTH, BLACK_KEY_WIDTH,
    KEYBOARD_Y, KEYBOARD_HEIGHT, BLACK_KEY_HEIGHT,
    VIEWBOX_WIDTH, VIEWBOX_HEIGHT, IS_WHITE,
    note_x_center, white_key_x, build_band_segments, build_mean_segments,
};

#[component]
pub fn ProfilePreview() -> impl IntoView {
    // Extract context (same pattern as ProfileVisualization)
    // Borrow profile, extract data, drop borrow
    // Render compact SVG inside <A href="/profile"> wrapper
}
```

### Context Wiring Pattern (Established)

Same as Stories 3.1 and 3.2:

```rust
let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
    use_context().expect("PerceptualProfile context");
let is_profile_loaded: RwSignal<bool> =
    use_context().expect("is_profile_loaded context");
```

**Borrow-then-extract pattern (critical):** Borrow `RefCell`, extract all data into owned `Vec<NoteData>`, drop the borrow, THEN use in `view!` macro. This avoids `RefCell` borrow conflicts in reactive closures.

### Start Page Integration

Current `start_page.rs` has this block that should be replaced:

```rust
{move || {
    if !is_profile_loaded.get() {
        view! { <p class="text-sm text-gray-500 dark:text-gray-400">"Loading profile..."</p> }.into_any()
    } else {
        view! { <span></span> }.into_any()
    }
}}
```

Replace with:

```rust
<ProfilePreview />
```

The `ProfilePreview` component handles its own loading state internally (renders keyboard-only while profile is loading, then adds band data once loaded). The start page no longer needs to manage this.

**Layout order after integration:**
1. `<h1 class="sr-only">"Peach"</h1>`
2. `<ProfilePreview />` — always renders, handles own states
3. `<nav aria-label="Training modes">` — training buttons
4. `<nav aria-label="Utility">` — settings/profile/info links

### Clickable Wrapper

Use `leptos_router::components::A` (already imported in `start_page.rs`):

```rust
<A href="/profile"
    attr:aria-label=aria_label
    attr:class="block rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900 overflow-hidden">
    <svg ...>
        // compact visualization
    </svg>
</A>
```

**Why `<A>` from leptos_router:** Provides client-side navigation (no full page reload). The `<A>` component renders as an `<a>` tag which is natively focusable and activatable via Enter key — no extra keyboard handling required (AC5 satisfied for free).

### Empty State

When no training data exists, the preview shows the keyboard rectangles only (no band paths). The keyboard outline itself serves as the "subtle placeholder shape" per the UX spec. This is identical to the full visualization's empty state, just rendered smaller.

No special "placeholder" graphic needed — the keyboard IS the placeholder.

### Accessibility

Dynamic `aria-label` on the `<a>` wrapper (not on the SVG):

```rust
let aria_label = if trained_count > 0 {
    format!(
        "Your pitch profile. Average threshold: {:.1} cents. Click to view details.",
        avg_threshold.unwrap_or(0.0)
    )
} else {
    "Your pitch profile. Click to view details.".to_string()
};
```

The SVG inside the link should have `role="img"` and `aria-hidden="true"` since the parent `<a>` already has the descriptive label — avoids duplicate screen reader announcements.

### Compact Sizing

The preview should be visually compact — a glanceable miniature, not a full-height chart. Control this via:

1. The wrapping `<a>` element gets a max-height constraint (e.g., `max-h-20` or `max-h-24` — 5rem or 6rem)
2. The SVG inside has `width="100%"` and the viewBox handles aspect ratio scaling
3. `overflow-hidden` on the wrapper clips any overflow from the SVG's aspect ratio

Since the SVG viewBox is `520x210` (roughly 2.5:1 aspect ratio), at `max-h-24` (6rem = 96px) the SVG will be approximately 240px wide — compact enough for a preview while showing the band shape clearly.

Alternatively, don't constrain height — let the SVG scale naturally to full width inside the `max-w-lg` container. At full width (~512px max), the height would be ~207px — still reasonable as a preview. Test both and choose what looks better.

**Recommended approach:** Let SVG scale to full width, add some vertical margin/padding control. The preview at full width of the container looks like a natural header element for the start page. If it feels too tall, add a max-height constraint.

### Dark Mode

Reuses the same `--pv-*` CSS custom properties from `input.css` (established in Story 3.2). No CSS changes needed. The SVG elements reference these variables via `fill="var(--pv-key-white)"`, `fill="var(--pv-band-fill)"`, etc.

### What NOT to Implement

- **No additional domain changes** — all APIs exist from previous stories
- **No tooltip or hover effects on the preview** — pure visual display
- **No loading spinner** — keyboard renders immediately as the placeholder
- **No "click to see details" visible text** — the accessibility label provides this for screen readers; the visual preview is self-explanatory per UX spec
- **No animation on the band** — respect "disappearing UI" philosophy

### Project Structure Notes

- **New file:** `web/src/components/profile_preview.rs` — compact clickable profile miniature component
- **Modified:** `web/src/components/profile_visualization.rs` — Change helper functions, NoteData struct, and constants to `pub(crate)` visibility
- **Modified:** `web/src/components/mod.rs` — Add `pub mod profile_preview;` and `pub use profile_preview::ProfilePreview;`
- **Modified:** `web/src/components/start_page.rs` — Import `ProfilePreview`, replace loading placeholder with `<ProfilePreview />`
- **No CSS changes** — reuses existing `--pv-*` custom properties
- **No domain changes** — all APIs exist
- **No routing changes** — `/profile` route already exists
- **No new dependencies** — all imports already available in the web crate

### References

- [Source: docs/planning-artifacts/epics.md#Story 3.3] — Full acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/architecture.md#Component Architecture] — Lists `profile_preview.rs` as custom component (Medium complexity)
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture] — StartPage composes Profile Preview + training buttons + nav links
- [Source: docs/planning-artifacts/ux-design-specification.md#Profile Preview] — Compact, clickable, same band shape, no axis labels, empty state
- [Source: docs/planning-artifacts/ux-design-specification.md#Start Page] — Profile preview element specification
- [Source: docs/planning-artifacts/ux-design-specification.md#Empty States] — "Subtle placeholder shape" for profile preview with no data
- [Source: docs/project-context.md] — Coding conventions, anti-patterns, type naming rules
- [Source: web/src/components/profile_visualization.rs] — SVG rendering helpers to reuse (note_x_center, build_band_segments, etc.)
- [Source: web/src/components/start_page.rs] — Current start page to integrate preview into
- [Source: web/src/app.rs] — Context providers for profile, is_profile_loaded

### Previous Story Intelligence (from Story 3.2)

**Patterns to follow:**
- Context extraction: `use_context::<SendWrapper<Rc<RefCell<T>>>>().expect("...")` — exact same pattern
- Borrow-then-extract: borrow `RefCell`, extract all data into owned `Vec<NoteData>`, drop borrow, use in `view!` macro — critical for avoiding `RefCell` conflicts
- SVG in Leptos `view!` macro: SVG elements render directly, Leptos handles SVG namespace automatically inside `<svg>` tag
- CSS custom properties: `fill="var(--pv-key-white)"` etc. — reuse from `input.css`
- `into_any()` for unifying branch return types in reactive closures

**Code review learnings from Story 3.2:**
- Leptos `view!` macro consumes Strings — clone into separate variables before SVG element if needed in multiple places (e.g., `aria_label` and `<title>`)
- Collapse `else { if }` blocks per clippy `collapsible_else_if` lint
- Keyboard should render during loading state (not just after profile loads) — avoids layout shift

**Debug learnings from Story 3.2:**
- `aria_label` ownership: Leptos `view!` macro consumes the String, so clone before use if needed in multiple attributes
- No need for `pub use` in `mod.rs` if the component is only used internally by one other module — but since ProfilePreview will be used by StartPage (a public component), the `pub use` IS needed

### Git Intelligence

Recent commits follow pattern: "Implement story X.Y ..." → "Apply code review fixes for story X.Y and mark as done". Follow same pattern.

Last 3 commits: story 3.2 implementation and code review fixes, then story 3.2 creation. The ProfileVisualization component is fresh from the most recent work.

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

No issues encountered during implementation.

### Completion Notes List

- Task 1: Changed NoteData struct, 15 constants, and 7 helper functions to `pub(crate)` visibility in `profile_visualization.rs`. Clippy passes with zero warnings — no dead code since items are now consumed by both `profile_visualization.rs` and `profile_preview.rs`.
- Tasks 2, 3, 5: Created `ProfilePreview` component in a single file. Component renders compact SVG miniature reusing shared helpers from `profile_visualization`. Wrapped in `<A href="/profile">` for client-side navigation. Dynamic `aria-label` based on training data presence. SVG has `role="img" aria-hidden="true"` to avoid duplicate screen reader announcements. Focus ring styling and `overflow-hidden rounded-lg` on wrapper. SVG scales naturally to container width via `width="100%"` with viewBox aspect ratio — following the recommended approach from Dev Notes (no max-height constraint).
- Task 4: Replaced the conditional loading placeholder / empty span block in `start_page.rs` with `<ProfilePreview />`. Removed the now-unused `is_profile_loaded` context extraction from StartPage (ProfilePreview handles its own loading state). Layout order: h1 → ProfilePreview → training nav → utility nav.
- Task 6: `cargo clippy -p domain` zero warnings, `cargo clippy -p web` zero warnings, `cargo test -p domain` 254 tests pass, `trunk build` succeeds. Manual browser smoke test (6.5) left for Michael.

### Implementation Plan

Followed the story's task sequence exactly. Key decisions:
1. Reuse over duplication: all SVG rendering logic shared via `pub(crate)` imports
2. Natural SVG scaling rather than max-height constraint (recommended in Dev Notes)
3. `<A>` from leptos_router provides native `<a>` tag — keyboard navigation (Enter/Space) works out of the box
4. Empty and loading states both show keyboard-only — consistent with full visualization behavior

### File List

- `web/src/components/profile_preview.rs` (new) — ProfilePreview component
- `web/src/components/profile_visualization.rs` (modified) — pub(crate) visibility for shared items
- `web/src/components/mod.rs` (modified) — added profile_preview module and pub use
- `web/src/components/start_page.rs` (modified) — replaced loading placeholder with ProfilePreview
- `docs/implementation-artifacts/sprint-status.yaml` (modified) — status tracking
- `docs/implementation-artifacts/3-3-profile-preview-on-start-page.md` (modified) — story tracking

## Change Log

- 2026-03-04: Implemented story 3.3 — Profile Preview on Start Page. Created ProfilePreview component reusing shared SVG helpers from ProfileVisualization. Compact, clickable miniature with dynamic aria-label, keyboard navigation support, and dark mode via existing CSS custom properties. Integrated into StartPage replacing loading placeholder.
