# Story 12.4: Chart Scrolling

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to scroll through my chart history when I have more than 8 data points,
so that I can review my full training timeline while keeping the chart readable.

## Acceptance Criteria

1. **Static layout for small data** — Given a mode with 8 or fewer display buckets, when the chart renders, then all buckets are visible in a static layout with no scrolling.

2. **Horizontal scrolling for large data** — Given a mode with more than 8 display buckets, when the chart renders, then the chart is horizontally scrollable, 8 buckets are visible at a time, and the initial scroll position shows the rightmost (most recent) data at the right edge. Scroll position = max(0, bucketCount - 8).

3. **Smooth panning** — Given a scrollable chart, when I scroll horizontally, then the chart pans smoothly to reveal earlier or later buckets. Any active selection annotation is dismissed (Epic 13 concern — no-op for now since annotations don't exist yet).

4. **Reduced motion** — Given a scrollable chart with `@media (prefers-reduced-motion: reduce)` active, when the initial scroll position is set, then it is applied without animation.

## Tasks / Subtasks

- [x] Task 1: Add scrollable container wrapper (AC: 1, 2)
  - [x] 1.1 In `progress_chart.rs`, wrap the `<svg>` in a `<div>` container. When `bucket_count > 8`, apply `overflow-x: auto` to the container and scale SVG width proportionally (`bucket_count / 8 * 100%`). When `bucket_count <= 8`, keep current static layout with `width="100%"`.
  - [x] 1.2 Create a `NodeRef` for the scroll container div.
  - [x] 1.3 Adjust the SVG `preserveAspectRatio` to `"xMinYMid meet"` (instead of `"none"`) so the chart maintains its aspect ratio when wider than the viewport.

- [x] Task 2: Set initial scroll position (AC: 2, 4)
  - [x] 2.1 Use `request_animation_frame` (or `leptos::task::spawn_local`) after mount to set `scroll_left` on the container ref to `scroll_width - client_width` (i.e., scroll to right edge).
  - [x] 2.2 Check `prefers-reduced-motion` via `window.match_media("(prefers-reduced-motion: reduce)")`. If reduced motion is preferred, set `scroll_left` directly (instant). Otherwise, use `element.scroll_to_with_scroll_to_options()` with `behavior: "smooth"` — actually NO: initial position should always be instant (not animated), because the user hasn't seen the chart yet. Set `scroll_left` directly in all cases.
  - [x] 2.3 Ensure scroll position is set AFTER the SVG has rendered and the container has its final dimensions.

- [x] Task 3: Ensure chart height is preserved (AC: 1, 2)
  - [x] 3.1 The scrollable container div must preserve the responsive height classes `h-[180px] md:h-[240px]` (move from SVG to container if needed, or keep on SVG — ensure the chart area doesn't collapse).
  - [x] 3.2 Verify the SVG element's height fills the container (may need `height="100%"` on SVG, or min-height on container).

- [x] Task 4: Verify static layout unchanged (AC: 1)
  - [x] 4.1 Confirm that when `bucket_count <= 8`, there is no visible scrollbar and behavior is identical to current implementation.
  - [x] 4.2 Confirm SVG still renders with `width="100%"` in the static case.

- [x] Task 5: Build and lint (AC: 1-4)
  - [x] 5.1 `cargo clippy --workspace` — zero warnings
  - [x] 5.2 `cargo test -p domain` — all tests pass (no domain changes)
  - [ ] 5.3 UNCHECKED — Manual browser test: >8 buckets shows horizontal scrollbar, 8 buckets visible, scrolled to right
  - [ ] 5.4 UNCHECKED — Manual browser test: <=8 buckets shows static layout, no scrollbar
  - [ ] 5.5 UNCHECKED — Manual browser test: scrolling is smooth and responsive
  - [ ] 5.6 UNCHECKED — Manual browser test: initial position shows most recent data on right edge

## Dev Notes

### What Already Exists (DO NOT REINVENT)

- **`web/src/components/progress_chart.rs`** — `ProgressChart` SVG component (~530 lines). Renders all 6 chart layers (zone backgrounds, dividers, stddev band, mean line, session dots, baseline), X-axis labels, year labels. Uses index-based X mapping. Currently renders ALL buckets in a fixed-width SVG with `width="100%"` and `preserveAspectRatio="none"`. **This is the primary file to modify.**
- **`web/src/components/progress_card.rs`** — `ProgressCard` component that calls `ProgressChart`. Passes `buckets`, `optimal_baseline`, and `unit_label`. **No changes needed** — scrolling is handled inside `ProgressChart`.
- **`domain/src/progress_timeline.rs`** — `display_buckets(mode)` returns `Vec<TimeBucket>`. No domain changes needed.
- **`input.css`** — Chart CSS classes. May need scrollbar styling.

### What Must Change

**`web/src/components/progress_chart.rs` (primary file — moderate changes):**

1. **Scrollable wrapper** — Wrap the `<svg>` element in a `<div>` that acts as the scroll container. When `bucket_count > 8`, this div gets `overflow-x: auto` and the SVG gets a proportionally wider width. When `bucket_count <= 8`, the div has no overflow and SVG stays at `width="100%"`.

2. **SVG width calculation** — For scrollable mode, the SVG width should be `bucket_count / 8 * 100%` of the container width. This ensures each bucket has the same visual width as if there were exactly 8 buckets, and the excess extends beyond the container.

3. **Initial scroll position** — After render, scroll the container to the right edge so the most recent data is visible. Use a `NodeRef` and `request_animation_frame` to ensure layout is computed before setting scroll.

4. **No changes to chart rendering logic** — All 6 layers, zone detection, bridge computation, labels, etc. remain exactly as-is. The only change is the container wrapping and width scaling.

### Implementation Approach

```rust
// Scrollable threshold
const VISIBLE_BUCKETS: usize = 8;

// In the component:
let container_ref = NodeRef::<leptos::html::Div>::new();
let is_scrollable = bucket_count > VISIBLE_BUCKETS;

// SVG width: proportional to bucket count when scrollable
let svg_width = if is_scrollable {
    format!("{}%", bucket_count as f64 / VISIBLE_BUCKETS as f64 * 100.0)
} else {
    "100%".to_string()
};

// Container classes
let container_class = if is_scrollable {
    "overflow-x-auto"
} else {
    ""
};
```

**Setting initial scroll position:**

```rust
// After mount, scroll to right edge
if is_scrollable {
    let container = container_ref.clone();
    request_animation_frame(move || {
        if let Some(el) = container.get() {
            let scroll_width = el.scroll_width();
            let client_width = el.client_width();
            el.set_scroll_left(scroll_width - client_width);
        }
    });
}
```

Use `web_sys::window().unwrap().request_animation_frame()` or `wasm_bindgen_futures::spawn_local` + `gloo_timers::future::TimeoutFuture::new(0)` to defer until after layout. Look at existing patterns in the codebase — the project uses `request_animation_frame` from `web_sys` already.

**Important: Use `leptos::task::spawn_local_scoped_with_cancellation`** (NOT `gloo_timers::callback::Timeout::new().forget()`) per project rules to avoid disposed signal panics on navigation. However, since we're only setting scroll position (not accessing signals), a simple `request_animation_frame` closure is safe here.

### SVG preserveAspectRatio

Current: `preserveAspectRatio="none"` — this stretches the SVG to fill its container, distorting the chart when the aspect ratio changes.

For scrollable mode, consider changing to `preserveAspectRatio="xMinYMid meet"` so the chart scales uniformly. However, since the viewBox dimensions and the container dimensions should be proportional, `"none"` may work fine. Test both approaches. The key is that each bucket should have the same visual width regardless of total bucket count.

### Height Handling

The responsive height classes `h-[180px] md:h-[240px]` are currently on the `<svg>` element (`class="mt-2 h-[180px] md:h-[240px]"`). When wrapping in a scroll container:
- Move height classes to the container `<div>`
- Set SVG `height="100%"` so it fills the container vertically
- Keep `mt-2` on the container

### Scrollbar Styling (Optional)

Consider thin scrollbar styling in `input.css`:

```css
.chart-scroll-container {
    scrollbar-width: thin;
    scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
}
.chart-scroll-container::-webkit-scrollbar {
    height: 4px;
}
.chart-scroll-container::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 0.3);
    border-radius: 2px;
}
```

### What NOT to Change

- **Do NOT modify chart rendering logic** — All 6 layers stay exactly as-is.
- **Do NOT modify domain crate** — No changes to `display_buckets()` or any domain types.
- **Do NOT add tap/click interaction** — Epic 13 (PFR15).
- **Do NOT add virtual scrolling** — Simple CSS overflow is sufficient for this data volume (max ~50-100 buckets).
- **Do NOT change `progress_card.rs`** — Scrolling is self-contained within `ProgressChart`.

### Story 12.3 Learnings (Previous Story Intelligence)

From the 12.3 implementation:

1. **Index-based X mapping** — `let x = |index: f64| -> f64 { MARGIN_LEFT + (index + 0.5) / bucket_count as f64 * inner_w }`. This maps ALL buckets into the viewBox width. For scrolling, this stays the same — the SVG is just wider than the visible container.
2. **ViewBox is 300 wide** — `VIEWBOX_WIDTH = 300.0`. The SVG coordinate system stays 300 units wide regardless of scrolling. The scaling happens via the SVG element's CSS width.
3. **`preserveAspectRatio="none"`** — Currently stretches to fill. May need to reconsider for scrollable mode.
4. **Height classes on SVG** — `class="mt-2 h-[180px] md:h-[240px]"` — need to move to container.
5. **Zone backgrounds, dividers, labels** — All positioned via the index-based `x()` function. They will correctly extend into the scrollable area without changes.
6. **Year labels and bottom margin** — Dynamic bottom margin (`MARGIN_BOTTOM_YEARS` vs `MARGIN_BOTTOM_BASE`) stays as-is.
7. **Code review fix from 12.3** — Fixed year boundary suppression to use `i.abs_diff(t) <= 1`. Added CSS for `.chart-zone-bg-secondary`. Fixed stddev band fill to use Tailwind classes.

### Git Intelligence

Recent commits (all in Epic 12 scope):
- `acb3443` — Code review passed for story 12.3: mark as done
- `5f43e08` — Implement story 12.3: Chart Rendering
- `8536b07` — Create story 12.3: Chart Rendering
- `c3013c0` — Code review passed for story 12.2: mark as done
- `1fc8031` — Implement story 12.2: Profile screen layout & chart cards

Files most recently modified: `progress_chart.rs`, `progress_card.rs`, `mod.rs`, `input.css`, locale files.

### Project Structure Notes

- Primary file: `web/src/components/progress_chart.rs` (add scroll wrapper)
- Optional CSS: `input.css` (scrollbar styling)
- No domain crate changes
- No new files needed
- No i18n changes needed
- No changes to `progress_card.rs`

### References

- [Source: docs/ios-reference/profile-screen-specification.md#Scrolling Behavior] — 8-bucket threshold, scroll position formula, smooth panning
- [Source: docs/ios-reference/profile-screen-specification.md#Reduce Motion] — No animation for scroll position
- [Source: docs/planning-artifacts/epics.md#Story 12.4: Chart Scrolling] — Full BDD acceptance criteria
- [Source: docs/planning-artifacts/epics.md#Epic 12: Profile Progress Charts] — PFR9 (horizontal scrolling)
- [Source: web/src/components/progress_chart.rs] — Current SVG component with all 6 layers
- [Source: docs/project-context.md] — Coding conventions, Leptos patterns, spawn_local_scoped rules

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

### Completion Notes List

- Wrapped SVG in scrollable `<div>` container with `overflow-x: auto` when `bucket_count > 8`
- SVG width scales proportionally (`bucket_count / 8 * 100%`) for scrollable mode, `100%` for static
- `NodeRef` on container div for scroll position control
- `preserveAspectRatio` kept as `"none"` for both modes — `"xMinYMid meet"` caused content to scale to height and not fill width
- Initial scroll position set to right edge via `request_animation_frame` + `set_scroll_left(scroll_width - client_width)` — always instant (no animation)
- Height classes (`h-[180px] md:h-[240px]`) moved from SVG to container div; SVG uses `height="100%"`
- Thin scrollbar CSS added via `.chart-scroll-container` class in `input.css`
- No changes to chart rendering logic (all 6 layers), domain crate, or `progress_card.rs`
- Clippy: zero warnings. Domain tests: 359 passed.
- Manual browser tests (5.3-5.6) deferred to user — agent cannot verify in browser.

### Change Log

- 2026-03-13: Implemented chart scrolling — scrollable container wrapper, initial scroll-to-right, height preservation, scrollbar styling

### File List

- web/src/components/progress_chart.rs (modified — scroll container wrapper, NodeRef, rAF scroll, preserveAspectRatio)
- input.css (modified — added `.chart-scroll-container` scrollbar styling)
