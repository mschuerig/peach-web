# Story 7.6: Start Page Sparklines

Status: ready-for-dev

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

- [ ] Task 1: Create ProgressSparkline component (AC: 1, 2, 3, 4, 5, 6)
  - [ ] New component in `web/src/components/progress_sparkline.rs`
  - [ ] Props: `mode: TrainingMode`
  - [ ] Read ProgressTimeline from Leptos context
  - [ ] Get state, buckets, EWMA, trend for the mode
  - [ ] If NoData, render nothing (empty fragment)
  - [ ] If Active, render HStack of SVG sparkline + EWMA text
  - [ ] SVG: viewBox="0 0 60 24", `<polyline>` with computed points
  - [ ] Point computation: x = index / (count-1) * 60, y = 24 * (1 - (value - min) / range), handle range < 0.1
  - [ ] Stroke color: CSS class based on trend

- [ ] Task 2: Integrate into training cards (AC: 7, 8)
  - [ ] In start_page.rs, add `<ProgressSparkline mode=.../>` inside each card below the label
  - [ ] Map: Single Notes Hear & Compare → UnisonPitchComparison, etc.

- [ ] Task 3: Accessibility (AC: 10)
  - [ ] SVG: `aria-hidden="true"`
  - [ ] Container div: `aria-label` with mode name, EWMA value, and trend text when data exists
  - [ ] Container div: no aria-label when no data (nothing to announce)

- [ ] Task 4: Styling
  - [ ] Sparkline stroke-width: 1.5px, no fill
  - [ ] EWMA text: caption/small size, secondary color, "X.X cents" or "X.X ¢"
  - [ ] HStack: 6px gap between sparkline and text
  - [ ] Sparkline container: inline-flex, vertically centered with text

- [ ] Task 5: Verify
  - [ ] Manual test: train in one mode, return to start page, verify sparkline appears
  - [ ] Manual test: verify modes with no data show no sparkline
  - [ ] Verify sparkline scales correctly with varying data ranges
  - [ ] Run `cargo clippy`

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
