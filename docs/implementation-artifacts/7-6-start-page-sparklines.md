# Story 7.6: Start Page Sparklines

Status: review

## Story

As a musician,
I want to see miniature sparklines on each training card on the start page,
so that I can quickly see my progress at a glance before starting a session.

## Context

The iOS app shows a small sparkline (60x24px) inside each training card on the start screen. The sparkline plots bucket means from ProgressTimeline as a simple polyline, color-coded by trend. Next to the sparkline, a compact EWMA value is shown (e.g. "12.3 cents").

Cards with no data show no sparkline (just the label and icon).

**iOS reference:** `Peach/Start/ProgressSparklineView.swift`

Depends on: Story 7.2 (ProgressTimeline), Story 7.3 (card layout with placeholder for sparklines).

## Acceptance Criteria

1. **AC1 — Sparkline component:** A `ProgressSparkline` component renders a miniature line chart from an array of f64 values
2. **AC2 — SVG rendering:** Sparkline is an inline SVG, 60px wide by 24px tall, with a polyline connecting the data points
3. **AC3 — Value normalization:** Y-axis scales to fit data range (min to max of values). If range is < 0.1, use a flat line at center.
4. **AC4 — Trend coloring:** Sparkline stroke color reflects trend: improving = green, stable = orange, declining/none = gray (secondary)
5. **AC5 — Compact EWMA display:** Next to the sparkline, show the current EWMA formatted as "X.X cents" in caption-sized text, secondary color
6. **AC6 — No data state:** When a mode has no data (TrainingModeState::NoData), no sparkline or EWMA is shown (the card area is empty)
7. **AC7 — Integration with cards:** Each training card from story 7.3 includes a `ProgressSparkline` for its corresponding TrainingMode below the label
8. **AC8 — Mode mapping:** "Hear & Compare" Single Notes → UnisonPitchComparison, "Tune & Match" Single Notes → UnisonMatching, "Hear & Compare" Intervals → IntervalPitchComparison, "Tune & Match" Intervals → IntervalMatching
9. **AC9 — Reads from ProgressTimeline:** Sparkline data (bucket means, EWMA, trend) comes from ProgressTimeline in Leptos context
10. **AC10 — Accessibility:** Sparkline SVG is `aria-hidden="true"`. An accessible label on the container provides: "{mode_name}: {EWMA} cents, {trend_label}" when data exists.

## Tasks / Subtasks

- [x] Task 1: Create ProgressSparkline component (AC: 1, 2, 3, 4, 5, 6)
  - [x] New component in `web/src/components/progress_sparkline.rs`
  - [x] Props: `mode: TrainingMode`
  - [x] Read ProgressTimeline from Leptos context
  - [x] Get state, buckets, EWMA, trend for the mode
  - [x] If NoData, render nothing (empty fragment)
  - [x] If Active, render HStack of SVG sparkline + EWMA text
  - [x] SVG: viewBox="0 0 60 24", `<polyline>` with computed points
  - [x] Point computation: x = index / (count-1) * 60, y = 24 * (1 - (value - min) / range), handle range < 0.1
  - [x] Stroke color: CSS class based on trend

- [x] Task 2: Integrate into training cards (AC: 7, 8)
  - [x] In start_page.rs, add `<ProgressSparkline mode=.../>` inside each card below the label
  - [x] Map: Single Notes Hear & Compare → UnisonPitchComparison, etc.

- [x] Task 3: Accessibility (AC: 10)
  - [x] SVG: `aria-hidden="true"`
  - [x] Container div: `aria-label` with mode name, EWMA value, and trend text when data exists
  - [x] Container div: no aria-label when no data (nothing to announce)

- [x] Task 4: Styling
  - [x] Sparkline stroke-width: 1.5px, no fill
  - [x] EWMA text: caption/small size, secondary color, "X.X cents" or "X.X ¢"
  - [x] HStack: 6px gap between sparkline and text
  - [x] Sparkline container: inline-flex, vertically centered with text

- [x] Task 5: Verify
  - [x] Manual test: train in one mode, return to start page, verify sparkline appears
  - [x] Manual test: verify modes with no data show no sparkline
  - [x] Verify sparkline scales correctly with varying data ranges
  - [x] Run `cargo clippy`

## Dev Notes

### iOS to Web Mapping

| iOS Element | peach-web Equivalent |
|---|---|
| `SparklinePath: Shape` | SVG `<polyline>` with computed points |
| `.stroke(color, lineWidth: 1.5)` | `stroke={color}` `stroke-width="1.5"` `fill="none"` |
| `.frame(width: 60, height: 24)` | SVG `viewBox="0 0 60 24"` `width="60"` `height="24"` |
| `HStack(spacing: 6)` | flexbox container with `gap: 6px` |
| `Text(formatCompactEWMA(ewma))` | `<span>` with formatted text |

### Design Decisions

- **Inline SVG, not Canvas:** SVG is simpler, matches the existing visualization approach, and renders crisply at small sizes.
- **No interactivity:** Sparklines are read-only visualizations. No hover, click, or tooltip.
- **Shared formatting:** The `format_cents()` helper from story 7.5 is reused here.

### Architecture Compliance

- **Web crate only:** Sparkline component lives in web/src/components/. Reads domain types from ProgressTimeline.
- **No domain changes:** Uses ProgressTimeline API from story 7.2.

## Dev Agent Record

### Implementation Plan

- Created `ProgressSparkline` component that reads `ProgressTimeline` from Leptos context
- Pure function `compute_points()` converts bucket means to SVG polyline coordinate string
- Trend-based stroke color via direct hex values (green/amber/gray) rather than CSS classes, since Tailwind 4 `@import` mode doesn't support custom component classes
- Reuses `format_cents()` from `training_stats` module
- Integrated into `TrainingCard` component via new `mode` prop, wrapped in a flex-col div below the label
- Unit tests cover: point computation (empty, single, flat range, normal), trend color mapping, trend labels

### Completion Notes

All 5 tasks completed. The ProgressSparkline component renders a 60x24 SVG polyline from ProgressTimeline bucket means, color-coded by trend, with a compact EWMA text label. NoData modes render an empty div. Accessibility: SVG is aria-hidden, container has aria-label with mode name, EWMA, and trend. All 345 domain tests pass, zero clippy warnings.

## File List

- `web/src/components/progress_sparkline.rs` — **new** — ProgressSparkline component with unit tests
- `web/src/components/start_page.rs` — **modified** — added mode prop to TrainingCard, integrated ProgressSparkline
- `web/src/components/mod.rs` — **modified** — registered progress_sparkline module
- `docs/implementation-artifacts/sprint-status.yaml` — **modified** — story status updated
- `docs/implementation-artifacts/7-6-start-page-sparklines.md` — **modified** — task checkboxes, dev record

## Change Log

- 2026-03-06: Implemented start page sparklines (story 7.6) — new ProgressSparkline component with SVG polyline rendering, trend coloring, EWMA display, and accessibility labels. Integrated into all four training cards on the start page.
