# Story 13.1: Chart Tap Annotation

Status: review

## Story

As a musician,
I want to tap a data point on the chart and see its details in a popover,
so that I can explore specific time periods and see exactly how I performed.

## Acceptance Criteria

1. **Given** a rendered chart, **When** I tap/click on the chart area, **Then** the tap resolves to the nearest bucket index by rounding the X coordinate, **And** a vertical dashed selection line appears at that bucket's X position, **And** an annotation popover appears at the top of the chart.

2. **Given** the selection line, **When** rendered, **Then** it is dashed with pattern `[5, 3]`, 1px width, **And** gray at 50% opacity (80% in increased contrast mode).

3. **Given** the annotation popover for a monthly zone bucket, **When** displayed, **Then** it shows the date as "MMM yyyy" (e.g. "Jan 2026"), **And** the bucket mean value in caption bold (e.g. "25.3"), **And** the bucket stddev in caption2 secondary (e.g. "±4.2"), **And** the record count in caption2 secondary (e.g. "47 records").

4. **Given** the annotation popover for a daily zone bucket, **When** displayed, **Then** it shows the date as "E MMM d" (e.g. "Mon, Mar 5").

5. **Given** the annotation popover for a session zone bucket, **When** displayed, **Then** it shows the time as "HH:mm" (e.g. "14:30").

6. **Given** the popover, **When** rendered, **Then** it has a frosted glass background with 6px corner radius, **And** 6px padding on all sides, 2px VStack spacing, **And** overflow resolution keeps the popover within chart bounds on both axes.

7. **Given** a bucket is selected, **When** I tap the same area again, **Then** the annotation and selection line are dismissed.

8. **Given** a bucket is selected on a scrollable chart, **When** I scroll the chart, **Then** the annotation and selection line are dismissed.

9. **Given** number values in the popover, **When** formatted, **Then** they use locale-aware formatting with 1 decimal place (matching the headline row).

10. **Given** a bucket is selected, **When** the annotation popover appears, **Then** screen readers announce the popover content via an `aria-live="polite"` region, **And** the popover container has `role="status"` so it is treated as a live region update, not a focus trap.

11. **Given** the chart SVG, **When** rendered, **Then** it has `role="img"` with a descriptive `aria-label`, **And** the chart container is not keyboard-focusable (annotation is a visual enhancement, not a primary interaction).

## Tasks / Subtasks

- [x] Task 1: Add selection state signal (AC: #1, #7)
  - [x] 1.1 Add `selected_bucket: RwSignal<Option<usize>>` signal in `ProgressChart`
  - [x] 1.2 Signal stores `Some(index)` when selected, `None` when dismissed

- [x] Task 2: Implement tap/click handler (AC: #1, #7)
  - [x] 2.1 Add `on:click` handler to the main chart SVG element
  - [x] 2.2 Convert click clientX to SVG coordinate space using `getBoundingClientRect()` and viewBox scaling
  - [x] 2.3 Map SVG X coordinate to bucket index by inverting the `x()` coordinate function and rounding to nearest integer, clamping to `[0, bucket_count - 1]`
  - [x] 2.4 Toggle logic: if `selected_bucket == Some(clicked_index)` → set to `None`, else set to `Some(clicked_index)`

- [x] Task 3: Render selection line (AC: #2)
  - [x] 3.1 When `selected_bucket.get()` is `Some(idx)`, render a vertical `<line>` from `MARGIN_TOP` to `MARGIN_TOP + inner_h` at `x(idx as f64)`
  - [x] 3.2 Style: `stroke="currentColor"`, `stroke-dasharray="5 3"`, `stroke-width="1"`, CSS class `.chart-selection-line` with `opacity: 0.5`
  - [x] 3.3 Add increased contrast CSS: `.chart-selection-line { opacity: 0.8 }` under `@media (prefers-contrast: more)`

- [x] Task 4: Build annotation popover (AC: #3, #4, #5, #6, #9)
  - [x] 4.1 Create popover as an SVG `<foreignObject>` containing an HTML `<div>` with frosted glass styling
  - [x] 4.2 Frosted glass: `backdrop-blur-md bg-white/60 dark:bg-gray-900/60 border border-white/20 dark:border-gray-700/30 rounded-[6px]` (reuse existing pattern from `progress_card.rs`)
  - [x] 4.3 Inner layout: `p-[6px] space-y-[2px]` for 6px padding and 2px VStack spacing
  - [x] 4.4 Date label row (caption2, secondary color) — formatted per zone type (see Task 5)
  - [x] 4.5 Mean value row (caption, bold) — `format_decimal_1(bucket.mean)` using existing locale-aware formatter from `progress_card.rs`
  - [x] 4.6 Stddev row (caption2, secondary) — `±{format_decimal_1(bucket.stddev)}`
  - [x] 4.7 Record count row (caption2, secondary) — `"{count} records"` (i18n key)

- [x] Task 5: Zone-specific date formatting (AC: #3, #4, #5)
  - [x] 5.1 Monthly zone: format `period_start` as "MMM yyyy" using `Intl.DateTimeFormat` with `{month: "short", year: "numeric"}`
  - [x] 5.2 Daily zone: format `period_start` as "E MMM d" using `Intl.DateTimeFormat` with `{weekday: "short", month: "short", day: "numeric"}`
  - [x] 5.3 Session zone: format `period_start` as "HH:mm" using `Intl.DateTimeFormat` with `{hour: "2-digit", minute: "2-digit", hour12: false}`
  - [x] 5.4 Strip trailing dots from abbreviated names (reuse existing pattern from `format_month_label`)

- [x] Task 6: Popover overflow resolution (AC: #6)
  - [x] 6.1 Default position: popover anchored horizontally at the selection line X, vertically at the top of the chart area (just below `MARGIN_TOP`)
  - [x] 6.2 Horizontal overflow: if popover right edge exceeds `MARGIN_LEFT + inner_w`, shift left; if popover left edge goes below `MARGIN_LEFT`, shift right
  - [x] 6.3 Vertical overflow: if popover bottom edge exceeds `MARGIN_TOP + inner_h`, shift upward (unlikely for top-anchored positioning)

- [x] Task 7: Dismiss on scroll (AC: #8)
  - [x] 7.1 Add `scroll` event listener on the `.chart-scroll-container` div (the scrollable container)
  - [x] 7.2 On scroll event, set `selected_bucket` to `None`

- [x] Task 8: i18n keys (AC: #3, #4, #5, #10)
  - [x] 8.1 Add `chart-annotation-records = { $count } records` to `web/locales/en/main.ftl`
  - [x] 8.2 Add `chart-annotation-records = { $count } Einträge` to `web/locales/de/main.ftl`
  - [x] 8.3 Add `chart-annotation-summary = { $date } — { $mean } { $unit }, ±{ $stddev }, { $count } records` to EN
  - [x] 8.4 Add `chart-annotation-summary = { $date } — { $mean } { $unit }, ±{ $stddev }, { $count } Einträge` to DE

- [x] Task 9: Increased contrast support (AC: #2)
  - [x] 9.1 Add `.chart-selection-line` increased contrast rule in `input.css`

- [x] Task 10: Accessibility (AC: #10, #11)
  - [x] 10.1 Add a hidden `<div>` with `role="status"` and `aria-live="polite"` outside the SVG (sibling to the chart container) that mirrors the popover content as text
  - [x] 10.2 When `selected_bucket` changes to `Some(idx)`, update the live region text to: "{date} — {mean} {unit}, ±{stddev}, {count} records" (plain text summary)
  - [x] 10.3 When `selected_bucket` changes to `None`, clear the live region text
  - [x] 10.4 Add i18n key for the screen reader summary template

## Dev Notes

### Architecture & Patterns

- **SVG coordinate system**: Charts use an index-based X-axis. Bucket `i` is centered at `x(i as f64)` where `x = |index: f64| MARGIN_LEFT + (index + 0.5) / bucket_count as f64 * inner_w`. The inverse for hit-testing: `clicked_index = round((svg_x - MARGIN_LEFT) / inner_w * bucket_count - 0.5)`.
- **ViewBox scaling**: Static charts use `viewBox="0 0 300 {height}"`. Scrollable charts scale viewBox width to `300 * bucket_count / 8`. The click-to-SVG coordinate conversion must account for this scaling via `getBoundingClientRect()` ratio.
- **Rendering order**: Selection line and popover should be Layer 7 (topmost), rendered after all existing 6 layers.
- **Signal pattern**: Use `RwSignal<Option<usize>>` for selection state. This is local to the `ProgressChart` component, not shared via context.
- **Frosted glass**: Reuse the exact same Tailwind classes from `progress_card.rs`: `backdrop-blur-md bg-white/60 dark:bg-gray-900/60 border border-white/20 dark:border-gray-700/30`. Just adjust the corner radius to `rounded-[6px]`.
- **Locale-aware number formatting**: Reuse `format_decimal_1()` from `progress_card.rs`. Consider extracting it to a shared utility if not already shared — or just call it directly if the function is accessible.
- **foreignObject for HTML-in-SVG**: The popover needs HTML layout (text wrapping, backdrop-filter). Use `<foreignObject>` inside SVG to embed an HTML `<div>`. This allows using Tailwind classes and `backdrop-blur`. Set `width` and `height` attributes on `foreignObject` to accommodate the popover content. Note: `foreignObject` is well-supported in modern browsers and works in WASM/Leptos.

### Click-to-Bucket Coordinate Mapping

The critical algorithm for tap resolution:

```
1. Get click event clientX, clientY
2. Get SVG element's bounding rect via getBoundingClientRect()
3. Convert to SVG coordinates:
   svg_x = (clientX - rect.left) / rect.width * viewbox_width
4. Invert the x() function:
   raw_index = (svg_x - MARGIN_LEFT) / inner_w * bucket_count - 0.5
5. Round to nearest integer and clamp to [0, bucket_count - 1]
```

For scrollable charts, `rect.width` corresponds to the **visible** portion. The SVG `viewBox` width is larger. Use the SVG element's actual rendered width (which matches the scrollable content width) for proper conversion. The `getBoundingClientRect()` returns the visible container size, so account for scroll offset: `effective_x = clientX - rect.left + scrollLeft`. Then scale by `viewbox_width / svg_element_width`.

### Scroll Dismissal

The scroll container has a `NodeRef` already set up in the current implementation. Add a `scroll` event listener that sets `selected_bucket` to `None`. This cleanly dismisses both the selection line and popover reactively.

### Popover Overflow Resolution

Position the `foreignObject` at:
- X: `x(selected_index) - popover_width / 2` (centered on selection line)
- Y: `MARGIN_TOP + 2` (just below top margin)

Then clamp:
- `fo_x = max(MARGIN_LEFT, min(fo_x, MARGIN_LEFT + inner_w - popover_width))`

The popover dimensions in SVG viewBox units: estimate ~60 units wide, ~50 units tall for 4 lines of text at caption size. Fine-tune based on content.

### Accessibility

- **Live region pattern**: The popover is inside SVG (`foreignObject`), which screen readers handle inconsistently. Instead, mirror the popover content to a hidden HTML `<div>` with `role="status"` and `aria-live="polite"` placed as a sibling to the chart container (outside the SVG). This ensures reliable announcement across screen readers.
- **Hidden live region styling**: Use `sr-only` (Tailwind) or equivalent visually-hidden class. The div is never visible — it only exists for screen reader announcements.
- **No focus management needed**: The annotation is a passive display triggered by click/tap. It should NOT trap focus or become a keyboard-interactive widget. The live region announces the content without moving focus.
- **Existing accessibility pattern**: Story 12.2 established `role="group"` and `aria-label`/`aria-description` on chart cards. The annotation live region complements this — it doesn't replace existing aria attributes.

### Existing Patterns to Reuse

- **Locale date formatting**: `format_month_label()` and `format_weekday_label()` in `progress_chart.rs` — extend the same pattern for annotation-specific formats
- **Frosted glass styling**: from `progress_card.rs` card containers
- **Increased contrast CSS**: pattern established in `input.css` under `@media (prefers-contrast: more)`
- **Scroll container NodeRef**: already exists for initial scroll positioning in `ProgressChart`

### Project Structure Notes

- All changes are in `web/src/components/progress_chart.rs` — no domain crate changes needed
- CSS additions to `input.css` for `.chart-selection-line` increased contrast
- New i18n keys in `web/locales/en/main.ftl` and `web/locales/de/main.ftl`
- No new files needed — this extends the existing chart component

### Testing

- **Domain tests**: No domain changes → existing 359 tests should remain green
- **Clippy**: Run `cargo clippy --workspace` — zero warnings expected
- **Manual browser testing** (deferred to user):
  - Tap bucket → selection line + popover appears
  - Tap same bucket again → dismissed
  - Tap different bucket → popover moves
  - Scroll chart → popover dismissed
  - Monthly bucket: shows "Jan 2026" format
  - Daily bucket: shows "Mon, Mar 5" format
  - Session bucket: shows "14:30" format
  - Popover stays within chart bounds on edge buckets
  - Frosted glass appearance in light/dark mode
  - Increased contrast: selection line at 80% opacity
  - Locale-aware number formatting (comma vs period)
  - Static chart (≤8 buckets): tap works without scroll logic
  - Scrollable chart (>8 buckets): scroll dismisses
  - Screen reader (VoiceOver): tap announces bucket details via live region
  - Screen reader: dismissing clears the announcement

### References

- [Source: docs/ios-reference/profile-screen-specification.md#Layer 7: Selection Indicator]
- [Source: docs/ios-reference/profile-screen-specification.md#Annotation Popover]
- [Source: docs/ios-reference/profile-screen-specification.md#Tap Interaction]
- [Source: docs/ios-reference/profile-screen-specification.md#Increased Contrast Mode]
- [Source: docs/planning-artifacts/epics.md#Story 13.1]
- [Source: docs/implementation-artifacts/12-3-chart-rendering.md] — SVG rendering patterns, coordinate system
- [Source: docs/implementation-artifacts/12-4-chart-scrolling.md] — Scroll container, NodeRef pattern
- [Source: docs/implementation-artifacts/12-2-profile-screen-layout-and-chart-cards.md] — Frosted glass, locale formatting
- [Source: web/src/components/progress_chart.rs] — Current chart implementation (665 lines)
- [Source: web/src/components/progress_card.rs] — format_decimal_1(), frosted glass classes

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Fixed compilation: removed `xmlns` attribute on `<div>` inside `<foreignObject>` (not supported in Leptos html view macro, not needed for browser rendering)
- Fixed compilation: used `ev.current_target()` + `dyn_into` instead of `NodeRef` for SVG element access in click handler (Leptos custom element NodeRef doesn't have direct `as_ref()` to `web_sys::Element`)

### Completion Notes List

- Implemented chart tap annotation with selection line, frosted glass popover, and screen reader live region
- Click handler converts client coordinates to SVG coordinate space, accounting for scroll offset in scrollable charts
- Selection line rendered as dashed vertical line at 50% opacity (80% in increased contrast mode)
- Popover uses `<foreignObject>` for HTML-in-SVG with frosted glass styling matching existing card pattern
- Zone-specific date formatting: monthly (MMM yyyy), daily (E MMM d), session (HH:mm)
- Horizontal overflow resolution clamps popover within chart bounds
- Scroll event listener dismisses annotation on scrollable charts
- Added `format_decimal_1_chart()` local to progress_chart.rs (mirrors `format_decimal_1()` in progress_card.rs)
- SVG element changed from `aria-hidden="true"` to `role="img"` with `aria-label` per AC #11
- i18n keys added for EN and DE

### File List

- web/src/components/progress_chart.rs (modified)
- input.css (modified)
- web/locales/en/main.ftl (modified)
- web/locales/de/main.ftl (modified)
- docs/implementation-artifacts/13-1-chart-tap-annotation.md (modified)
- docs/implementation-artifacts/sprint-status.yaml (modified)

### Change Log

- 2026-03-13: Implemented story 13.1 — Chart Tap Annotation (all 10 tasks)
