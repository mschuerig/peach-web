# Story 3.2: Perceptual Profile Visualization

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to see my pitch discrimination ability visualized across the piano keyboard,
so that I can see where my hearing is strong and where it needs work.

## Acceptance Criteria

1. **AC1 — Piano keyboard with confidence band (FR16):** Given the Profile view with training data, when the visualization renders, then I see a horizontal piano keyboard strip with stylized key rectangles and a confidence band area chart overlaid above the keyboard, with the Y-axis inverted (lower = better discrimination) and note names at octave boundaries (C2, C3, C4, etc.).

2. **AC2 — Sparse data state:** Given training data exists for some notes, when the visualization renders (sparse state), then the band renders where data exists, the band fades or is absent where no data exists, and there is no interpolation across large gaps.

3. **AC3 — Populated data state:** Given extensive training data (populated state), when the visualization renders, then a continuous confidence band is visible across the trained range.

4. **AC4 — Empty state (no data):** Given no training data, when the visualization renders, then the piano keyboard renders fully, and the band is absent or shown as a faint uniform placeholder at 100 cents.

5. **AC5 — Responsive width:** Given the visualization is implemented in Canvas or SVG, when the view is resized, then the visualization uses the full available width.

6. **AC6 — Accessibility:** Given the visualization, when I inspect the HTML, then it has an ARIA role and label on the container, and a text alternative: "Perceptual profile: average detection threshold X cents across Y trained notes."

7. **AC7 — Dark mode:** Given `prefers-color-scheme: dark` is set, when the visualization renders, then colors adapt appropriately for dark mode.

## Tasks / Subtasks

- [ ] Task 1: Create ProfileVisualization component with piano keyboard (AC: 1,4,5)
  - [ ] 1.1 Create `web/src/components/profile_visualization.rs` with `ProfileVisualization` component
  - [ ] 1.2 Add `pub mod profile_visualization;` and re-export `ProfileVisualization` in `web/src/components/mod.rs`
  - [ ] 1.3 Render inline `<svg>` with `viewBox` and `width="100%"` for responsive scaling
  - [ ] 1.4 Implement piano keyboard rendering: white keys as rectangles, black keys as smaller overlapping rectangles, standard proportions
  - [ ] 1.5 Render octave boundary labels (C2, C3, C4, C5, C6, C7) below or on the keyboard
  - [ ] 1.6 Handle empty state: keyboard renders fully with no band (or faint uniform placeholder at 100 cents)

- [ ] Task 2: Implement confidence band rendering (AC: 1,2,3)
  - [ ] 2.1 Extract per-note data from `PerceptualProfile` via `note_stats(MIDINote::new(i))` for MIDI 21-108, collecting mean, std_dev, and is_trained into owned `Vec`
  - [ ] 2.2 Render confidence band as SVG `<path>` filled area: upper edge = mean - std_dev, lower edge = mean + std_dev (inverted Y-axis)
  - [ ] 2.3 Map each MIDI note to its x-center position on the keyboard layout
  - [ ] 2.4 Break band into separate segments at untrained notes (no interpolation across gaps)
  - [ ] 2.5 Handle sparse state: band appears only where consecutive trained notes exist
  - [ ] 2.6 Handle populated state: continuous band across trained range
  - [ ] 2.7 Render mean line (center of band) as a subtle stroke for clarity

- [ ] Task 3: Integrate into ProfileView (AC: 1,4)
  - [ ] 3.1 Import and render `<ProfileVisualization />` inside `profile_view.rs`, above the statistics sections
  - [ ] 3.2 Visualization renders in ALL states (cold start, sparse, populated) — keyboard always visible
  - [ ] 3.3 Cold-start message "Start training to build your profile." remains below the visualization (existing behavior)

- [ ] Task 4: Dark mode and styling (AC: 5,7)
  - [ ] 4.1 White keys: light fill in light mode, darker fill in dark mode
  - [ ] 4.2 Black keys: dark fill in light mode, lighter dark fill in dark mode
  - [ ] 4.3 Band fill: semi-transparent indigo/blue, adjusted for dark mode
  - [ ] 4.4 Mean line: solid indigo stroke with dark mode variant
  - [ ] 4.5 Octave labels: `text-gray-500 dark:text-gray-400` equivalent in SVG
  - [ ] 4.6 Use CSS `prefers-color-scheme` media query or Tailwind's `dark:` class on parent to control SVG colors via CSS custom properties or conditional rendering

- [ ] Task 5: Accessibility (AC: 6)
  - [ ] 5.1 Add `role="img"` on the `<svg>` element
  - [ ] 5.2 Add dynamic `aria-label`: "Perceptual profile: average detection threshold X cents across Y trained notes" (computed from profile data), or "Perceptual profile: no training data yet" for cold start
  - [ ] 5.3 Add `<title>` element inside SVG for screen reader support

- [ ] Task 6: Verify and validate (AC: all)
  - [ ] 6.1 `cargo clippy -p domain` — zero warnings
  - [ ] 6.2 `cargo clippy -p web` — zero warnings
  - [ ] 6.3 `cargo test -p domain` — all tests pass
  - [ ] 6.4 `trunk build` — successful WASM compilation
  - [ ] 6.5 Manual browser smoke test: empty state (keyboard only), sparse data (partial band), populated data (continuous band), dark mode, responsive resizing

## Dev Notes

### Core Approach: Inline SVG Piano Keyboard + Confidence Band

This story creates a new `ProfileVisualization` component (`web/src/components/profile_visualization.rs`) that renders an inline SVG visualization. The SVG contains two layers:

1. **Piano keyboard strip** — stylized rectangles for 88 keys (MIDI 21-108), standard proportions
2. **Confidence band** — area chart above the keyboard showing per-note detection threshold (mean ± std_dev)

**Why SVG over Canvas:**
- **Declarative:** SVG elements render naturally inside Leptos `view!` macros — no imperative drawing code, no `NodeRef` + `Effect` pattern needed
- **Dark mode:** CSS styling via `prefers-color-scheme` or Tailwind `dark:` class on parent container; SVG fills/strokes use `currentColor` or CSS custom properties
- **Responsive:** SVG `viewBox` + `width="100%"` scales automatically to container width — no resize observers needed
- **Accessibility:** SVG elements support `role`, `aria-label`, `<title>` — better than Canvas which is a black box to screen readers
- **No additional web-sys features:** SVG is DOM — no need to add `HtmlCanvasElement` or `CanvasRenderingContext2d` to `web/Cargo.toml`

**Scope boundary:** This story is the visualization ONLY. The profile preview on the start page is Story 3.3. Do NOT touch `start_page.rs` or create a `profile_preview.rs` component.

### Existing Infrastructure (DO NOT recreate)

All per-note data access already exists in the domain crate:

| Method | Location | Returns | Purpose |
|---|---|---|---|
| `PerceptualProfile::note_stats(note)` | `domain/src/profile.rs:238` | `&PerceptualNote` | Get stats for a specific MIDI note |
| `PerceptualNote::mean()` | `domain/src/profile.rs:54` | `f64` | Mean detection threshold in cents |
| `PerceptualNote::std_dev()` | `domain/src/profile.rs:58` | `f64` | Standard deviation in cents |
| `PerceptualNote::is_trained()` | `domain/src/profile.rs:70` | `bool` | True if sample_count > 0 |
| `PerceptualNote::sample_count()` | `domain/src/profile.rs:62` | `u32` | Number of training samples |
| `PerceptualProfile::overall_mean()` | `domain/src/profile.rs:151` | `Option<f64>` | Average threshold across all trained notes |
| `MIDINote::new(raw_value)` | `domain/src/types/midi.rs` | `MIDINote` | Create from u8 (panics if > 127) |
| `MIDINote::name()` | `domain/src/types/midi.rs` | `String` | Note name e.g. "C4", "A4" |

**No domain changes needed.** All required methods exist. Loop MIDI 21-108 calling `note_stats(MIDINote::new(i))` for each note.

### Context Wiring Pattern (from `app.rs`)

Same pattern as Story 3.1 — extract contexts inside the component:

```rust
let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
    use_context().expect("PerceptualProfile context");
let is_profile_loaded: RwSignal<bool> =
    use_context().expect("is_profile_loaded context");
```

**Critical:** Borrow the profile, extract ALL per-note data into owned types (a `Vec<NoteData>` or similar), then drop the borrow BEFORE the `view!` macro. This avoids `RefCell` borrow conflicts in the reactive closure.

```rust
struct NoteData {
    midi: u8,
    mean: f64,
    std_dev: f64,
    is_trained: bool,
    sample_count: u32,
}
```

Extract data in the reactive closure:
```rust
{move || {
    let prof = profile_rc.borrow();
    let notes: Vec<NoteData> = (21..=108).map(|i| {
        let stat = prof.note_stats(MIDINote::new(i));
        NoteData {
            midi: i,
            mean: stat.mean(),
            std_dev: stat.std_dev(),
            is_trained: stat.is_trained(),
            sample_count: stat.sample_count(),
        }
    }).collect();
    let trained_count = notes.iter().filter(|n| n.is_trained).count();
    let avg_threshold = prof.overall_mean();
    drop(prof);
    // Now render SVG using `notes`, `trained_count`, `avg_threshold`
}}
```

### Piano Keyboard Layout Algorithm

**Range:** MIDI 21 (A0) to MIDI 108 (C8) — 88 keys, standard piano.

**Key classification by note-within-octave (MIDI % 12):**

| Remainder | Note | Type |
|---|---|---|
| 0 | C | White |
| 1 | C# | Black |
| 2 | D | White |
| 3 | D# | Black |
| 4 | E | White |
| 5 | F | White |
| 6 | F# | Black |
| 7 | G | White |
| 8 | G# | Black |
| 9 | A | White |
| 10 | A# | Black |
| 11 | B | White |

**White keys:** 52 total in the 88-key range. Each white key has equal width. Total keyboard width = 52 white key widths.

**Black keys:** 36 total. Each black key is ~60% the width of a white key and ~65% the height. Centered between the two white keys it sits above.

**X-position mapping for the confidence band:** Each MIDI note (white or black) maps to the horizontal center of its key. This ensures the band data points align with the keys they represent.

**Suggested coordinate system (SVG viewBox):**
- `viewBox="0 0 520 200"` — where each white key is 10 units wide, keyboard height is 60 units, chart area above is 140 units
- White keys: x = `white_key_index * 10`, width = 10, y = 140, height = 60
- Black keys: x = computed center - 3, width = 6, y = 140, height = 39
- Chart area: y = 0 (top, best threshold 0 cents) to y = 140 (bottom, worst threshold ~200 cents)

**Computing white key index for a MIDI note:**
```rust
fn white_key_index(midi: u8) -> Option<u32> {
    // Returns the white key position index, or None for black keys
    let octave = midi / 12;
    let note = midi % 12;
    let white_notes = [0, 2, 4, 5, 7, 9, 11]; // C D E F G A B
    white_notes.iter().position(|&n| n == note).map(|pos| {
        octave as u32 * 7 + pos as u32
    })
}
```

**Computing x-center for ANY MIDI note (needed for band data points):**
```rust
fn note_x_center(midi: u8) -> f64 {
    // Map MIDI note to x position on the keyboard
    // White keys: center of the key rectangle
    // Black keys: center between the two adjacent white keys
    let octave = (midi / 12) as f64;
    let note = midi % 12;
    let white_key_width = 10.0;
    // x positions of note centers within an octave (relative to octave start)
    // C=0, C#=0.5, D=1, D#=1.5, E=2, F=3, F#=3.5, G=4, G#=4.5, A=5, A#=5.5, B=6
    let note_offsets: [f64; 12] = [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0];
    // Subtract the offset for MIDI 21 (A0) which is the first key
    let first_white_x = /* compute offset for MIDI 21 */;
    (octave * 7.0 + note_offsets[note as usize]) * white_key_width + white_key_width / 2.0 - first_white_x
}
```

**Note:** The exact positioning arithmetic should be computed relative to MIDI 21 (A0) being the leftmost key. The dev agent should work out the precise offsets. The key insight is that note_offsets maps each chromatic note to its position in "white key units" — black keys fall at 0.5 unit offsets between white keys.

### Confidence Band Rendering Algorithm

**Y-axis mapping (inverted — lower cents = better = higher on screen):**
- Define a Y scale: e.g., 0 cents maps to y=10 (near top), 200 cents maps to y=130 (near bottom of chart area)
- `y = chart_top + (value / max_cents) * chart_height`
- Clamp values to the visible range

**Band segments (no interpolation across gaps):**
1. Iterate through notes left to right (MIDI 21-108)
2. Collect consecutive runs of trained notes
3. For each run of 1+ consecutive trained notes, create a filled SVG `<path>`:
   - Upper edge: `mean - std_dev` (clamped to min 0)
   - Lower edge: `mean + std_dev` (clamped to max_cents)
   - Single trained note: render as a narrow rectangle or diamond
4. Also render the mean line as a polyline/path for each segment

**SVG path construction for a band segment:**
```
M x1,y1_upper L x2,y2_upper ... L xN,yN_upper  (upper edge left to right)
L xN,yN_lower L x(N-1),y(N-1)_lower ... L x1,y1_lower  (lower edge right to left)
Z  (close path)
```

**Gap definition:** Any untrained note breaks the band. Adjacent segments are rendered as separate paths.

### Empty State Logic

| Condition | Keyboard | Band | Aria-label |
|---|---|---|---|
| No trained notes at all | Full 88-key keyboard | Absent (no paths) or faint placeholder line at 100 cents | "Perceptual profile: no training data yet" |
| Some trained notes (sparse) | Full keyboard | Band segments where trained, gaps where not | "Perceptual profile: average detection threshold X.X cents across Y trained notes" |
| Many trained notes (populated) | Full keyboard | Continuous or near-continuous band | "Perceptual profile: average detection threshold X.X cents across Y trained notes" |

### Integration into ProfileView

Modify `web/src/components/profile_view.rs` to include the visualization:

```rust
use super::profile_visualization::ProfileVisualization;

// In the view, BEFORE the stats/cold-start conditional:
<ProfileVisualization />
```

**Layout:** The visualization renders above the statistics sections. It is always visible regardless of cold-start state. The existing cold-start message and stats sections remain conditional (as implemented in Story 3.1).

**Suggested order in ProfileView:**
1. `<h1>` "Profile"
2. `<ProfileVisualization />` — always renders
3. Cold-start message OR statistics sections (existing conditional)
4. Back link

### Dark Mode Implementation

SVG colors must adapt to dark mode. Two approaches (choose one):

**Approach A — CSS custom properties (recommended):**
Set CSS custom properties on the SVG parent based on dark mode, reference them in SVG `fill`/`stroke` attributes:
```css
:root {
    --key-white: #ffffff;
    --key-black: #1a1a1a;
    --key-border: #cccccc;
    --band-fill: rgba(99, 102, 241, 0.3);  /* indigo-500 */
    --band-stroke: rgb(99, 102, 241);
    --label-color: #6b7280;  /* gray-500 */
}
@media (prefers-color-scheme: dark) {
    :root {
        --key-white: #374151;  /* gray-700 */
        --key-black: #111827;  /* gray-900 */
        --key-border: #4b5563;  /* gray-600 */
        --band-fill: rgba(129, 140, 248, 0.3);  /* indigo-400 */
        --band-stroke: rgb(129, 140, 248);
        --label-color: #9ca3af;  /* gray-400 */
    }
}
```

**Approach B — Conditional rendering in Rust:**
Detect dark mode via `window().match_media("(prefers-color-scheme: dark)")` and render different SVG color values. More complex but avoids CSS file changes.

**Recommendation:** Approach A is simpler and matches the Tailwind philosophy. Add the CSS custom properties to `input.css` (the Tailwind directives file). SVG elements reference them: `fill="var(--key-white)"`.

**Alternatively:** If CSS custom properties feel like too much infrastructure, hard-code light mode colors in SVG and use Tailwind's `dark:` class on the parent container with CSS selectors targeting SVG elements. Example:
```css
.dark .profile-viz-white-key { fill: #374151; }
.dark .profile-viz-black-key { fill: #111827; }
```

### Accessibility Requirements

- `<svg>` element: `role="img"` + `aria-label="Perceptual profile: ..."` (dynamic)
- `<title>` element inside SVG as first child (for screen readers that read SVG title)
- The visualization is decorative in the sense that all data is also available in the text statistics below — the SVG provides a visual overview, not the sole data access
- No interactive elements inside the SVG — it is purely visual
- Keyboard users access profile data through the text statistics (from Story 3.1)

### What NOT to Implement (Separate Stories)

- **Profile preview on start page** — Story 3.3 (`profile_preview.rs`). Do NOT touch `start_page.rs`.
- **Interactive elements in visualization** — No click-to-see-note-details, no tooltips, no hover effects. Pure visual display.
- **Pitch matching data in visualization** — The visualization shows comparison training thresholds only. Pitch matching stats are in the text statistics section (Story 3.1).
- **Timeline/history view** — ThresholdTimeline exists but is not used in this story.
- **Gamification** — No celebratory colors, no "great improvement!" annotations.

### UX Requirements (from UX Spec)

- **"Disappearing UI" philosophy:** The visualization should be calm and factual. No decorative elements, no fancy gradients.
- **System colors:** Use standard color palette (indigo for the band matches the app's indigo accent for links).
- **Dark mode:** All colors must work in both modes.
- **Responsive:** SVG fills available width. No fixed pixel widths.
- **Empty state:** Keyboard visible even with no data. Inviting, not urgent.

### Project Structure Notes

- **New file:** `web/src/components/profile_visualization.rs` — SVG-based piano keyboard + confidence band component
- **Modified:** `web/src/components/mod.rs` — Add `pub mod profile_visualization;` and `pub use profile_visualization::ProfileVisualization;`
- **Modified:** `web/src/components/profile_view.rs` — Import and render `<ProfileVisualization />` above stats sections
- **Possibly modified:** `input.css` — Add CSS custom properties for SVG colors (if using Approach A for dark mode)
- **No domain changes needed** — All APIs exist
- **No new dependencies** — SVG is native DOM, no Canvas web-sys features needed
- **No routing changes** — ProfileView already exists at `/profile`

### References

- [Source: docs/planning-artifacts/epics.md#Story 3.2] — Full acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/architecture.md#Component Architecture] — Lists `profile_visualization.rs` as custom component
- [Source: docs/planning-artifacts/architecture.md#Crate Boundary] — Domain crate pure Rust, web crate has Leptos UI
- [Source: docs/planning-artifacts/ux-design-specification.md#Profile View] — Visualization spec: piano keyboard + confidence band, empty states
- [Source: docs/planning-artifacts/ux-design-specification.md#Perceptual Profile Visualization] — Implementation details: Canvas/SVG, states, accessibility
- [Source: docs/planning-artifacts/ux-design-specification.md#Empty States] — Cold start: keyboard renders, band absent or faint placeholder
- [Source: docs/project-context.md] — Coding conventions, anti-patterns, type naming rules
- [Source: domain/src/profile.rs] — PerceptualProfile API: note_stats(), overall_mean(), PerceptualNote type
- [Source: domain/src/types/midi.rs] — MIDINote type: new(), name(), raw_value()
- [Source: web/src/app.rs#provide_context] — Context providers for profile, is_profile_loaded
- [Source: web/src/components/profile_view.rs] — Current ProfileView to integrate visualization into
- [Source: web/src/components/settings_view.rs] — MIDI range 21-108 (A0-C8) as default boundaries

### Previous Story Intelligence (from Story 3.1)

**Patterns established:**
- Context extraction: `use_context::<SendWrapper<Rc<RefCell<T>>>>().expect("...")` — same pattern needed here
- Borrow-then-extract pattern: borrow `RefCell`, extract all data into owned types, drop borrow, then use in `view!` macro — critical for avoiding borrow conflicts
- `format_cents()` helper for Option<f64> formatting — can reuse for aria-label
- Tailwind dark mode: every color utility needs `dark:` variant
- `into_any()` for unifying branch return types in reactive closures

**Code review learnings from Stories 2.x and 3.1:**
- Use semantic HTML grouping elements
- Handle all `Option` types explicitly — never unwrap, always show fallback UI
- Values extracted from `RefCell` borrows must be fully owned before `view!` macro
- Dark mode ring offset: `dark:ring-offset-gray-900`

**Debug learnings from Story 3.1:**
- Leptos 0.8.x `Show` component requires `Send + Sync` for children closures; `Rc<RefCell<T>>` doesn't satisfy this. Use `{move || { ... }}` closure pattern with `.into_any()` for branch unification, and clone `SendWrapper` (not unwrap to `Rc`) to preserve `Send`.
- `Signal::derive` also requires `Send + Sync` — precompute values into owned types before the `view!` macro.

### Git Intelligence

Recent commit pattern: "Implement story X.Y ..." → "Apply code review fixes for story X.Y and mark as done". Follow same pattern.

Most recent commits show story 3.1 completed, meaning the ProfileView component is fresh and the dev agent should review the current implementation before modifying.

### Latest Tech Information

**Leptos 0.8.16** (released 2026-02-16) — current stable. SVG rendering in Leptos `view!` macro works by including SVG elements directly. Leptos handles the SVG namespace automatically when elements are inside an `<svg>` tag.

**web-sys Canvas API** (version 0.3.88+) — available if needed, but SVG approach avoids the need for `HtmlCanvasElement` and `CanvasRenderingContext2d` features. The current `web/Cargo.toml` does not include these features and they should NOT be added for this story.

**SVG in Leptos pattern:**
```rust
view! {
    <svg viewBox="0 0 520 200" width="100%" role="img" aria-label="...">
        <title>"..."</title>
        <rect x="0" y="140" width="10" height="60" fill="var(--key-white)" stroke="var(--key-border)" />
        // ... more SVG elements
    </svg>
}
```

SVG elements are first-class in Leptos's `view!` macro — no special handling needed.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
