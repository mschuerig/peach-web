# Story 7.4: Profile Screen with Progress Charts

Status: ready-for-dev

## Story

As a musician,
I want to see per-mode progress charts showing EWMA trends with stddev bands,
so that I can understand how my pitch discrimination is evolving in each training mode.

## Context

The current peach-web profile view shows a piano keyboard SVG visualization (per-note detection thresholds) plus summary statistics (mean, stddev, trend) and matching stats. The iOS app replaced this entirely with per-mode progress charts.

Each chart shows:
- A headline row with mode name, current EWMA value, stddev, and trend arrow
- An area chart: x-axis = time, y-axis = metric (cents). Shaded band = mean +/- stddev per bucket. Line = bucket mean. Dashed baseline = optimal target.
- Only modes with data are displayed (NoData modes hidden)

**iOS reference files:**
- `Peach/Profile/ProfileScreen.swift` — iterates TrainingMode.allCases, shows ProgressChartView per active mode
- `Peach/Profile/ProgressChartView.swift` — chart card with headline row + Swift Charts (LineMark + AreaMark)

Since peach-web runs in the browser without Swift Charts, we need to render charts using SVG or Canvas. SVG is the natural choice given the existing profile_visualization.rs SVG infrastructure.

Depends on: Story 7.2 (ProgressTimeline with buckets, EWMA, trend).

## Acceptance Criteria

1. **AC1 — Per-mode display:** Profile page shows one progress card per TrainingMode that has data (state == Active). Modes with no data are hidden.
2. **AC2 — Card headline:** Each card shows: mode display name (left), EWMA value formatted as "X.X" (right), "+/-X.X" stddev (right, secondary), trend arrow icon (right, color-coded)
3. **AC3 — Chart area:** Each card shows an SVG chart below the headline:
   - X-axis: time (bucket period_start values)
   - Y-axis: metric values (cents)
   - Shaded area: mean +/- stddev per bucket (light blue fill)
   - Line: bucket means connected (blue stroke)
   - Dashed baseline: optimal_baseline from TrainingModeConfig (green, dashed)
4. **AC4 — X-axis labels:** Bucket labels adapt to bucket_size: Session → relative time ("2h ago"), Day → weekday abbreviation ("Mon"), Week → "Mar 3", Month → "Mar"
5. **AC5 — Y-axis label:** Shows the unit label from config ("cents")
6. **AC6 — Responsive chart height:** Compact viewport → 180px, regular → 240px
7. **AC7 — Trend indicators:** Arrow icons: improving = down-right arrow (green), stable = right arrow (gray), declining = up-right arrow (orange). Match iOS `TrainingStatsView.trendSymbol/trendColor`.
8. **AC8 — Card styling:** Rounded corners, subtle background (matching start page cards), padding, spacing between cards
9. **AC9 — Old profile view removed:** The piano keyboard SVG visualization (`ProfileVisualization`) and the old summary statistics section are replaced. The `profile_visualization.rs` and `profile_preview.rs` components can be removed if no longer referenced.
10. **AC10 — Empty state:** When no training data exists for any mode, show a brief message (e.g. "No training data yet. Start a training session to see your progress.")
11. **AC11 — Accessibility:** Chart cards have aria-label with mode name, EWMA, and trend. Charts themselves are decorative (aria-hidden) with the headline providing the key data.
12. **AC12 — ProgressTimeline in context:** ProgressTimeline must be available via Leptos context (set up in story 7.2). Profile view reads from it.

## Tasks / Subtasks

- [ ] Task 1: Create progress chart SVG component (AC: 3, 4, 5, 6)
  - [ ] New component `ProgressChart` in `web/src/components/progress_chart.rs`
  - [ ] Props: `buckets: Vec<TimeBucket>`, `optimal_baseline: f64`, `unit_label: &str`, `chart_height: f64`
  - [ ] SVG layout: chart area with margins for axes. X scales linearly over bucket period_start range. Y scales from 0 to max(mean + stddev) with some padding.
  - [ ] Render stddev band as `<path>` with fill (upper = mean + stddev, lower = mean - stddev, clamped to 0)
  - [ ] Render mean line as `<polyline>` connecting bucket means
  - [ ] Render baseline as dashed `<line>` at optimal_baseline y-coordinate
  - [ ] X-axis tick labels formatted by bucket_size
  - [ ] Y-axis label text

- [ ] Task 2: Create progress card component (AC: 2, 7, 8)
  - [ ] New component `ProgressCard` in `web/src/components/progress_card.rs` (or inline in profile_view.rs)
  - [ ] Props: `mode: TrainingMode`
  - [ ] Reads ProgressTimeline from Leptos context
  - [ ] Headline row: mode display name, formatted EWMA, formatted stddev, trend arrow
  - [ ] Trend arrows: Unicode arrows (↘ improving green, → stable gray, ↗ declining orange) or SVG

- [ ] Task 3: Rewrite profile_view.rs (AC: 1, 9, 10, 12)
  - [ ] Replace current content with: iterate TrainingMode::ALL, for each where state == Active, render ProgressCard
  - [ ] Empty state when all modes are NoData
  - [ ] Remove imports/usage of PerceptualProfile summary stats, TrendAnalyzer, ThresholdTimeline from this view
  - [ ] ProgressTimeline accessed via Leptos context

- [ ] Task 4: Remove old visualization components (AC: 9)
  - [ ] If `profile_visualization.rs` is no longer imported anywhere, delete it
  - [ ] If `profile_preview.rs` is no longer imported anywhere (removed from start page in 7.3), delete it
  - [ ] Update `components/mod.rs` to remove the module declarations
  - [ ] Remove unused CSS variables for piano keyboard visualization if any

- [ ] Task 5: Accessibility (AC: 11)
  - [ ] Card wrapper: `aria-label="Progress chart for {mode_name}"`
  - [ ] Card value: `aria-label` with EWMA and trend text
  - [ ] SVG chart: `aria-hidden="true"` (decorative, data conveyed in headline)

- [ ] Task 6: Formatting helpers
  - [ ] EWMA formatting: one decimal place (e.g. "12.3")
  - [ ] Stddev formatting: "+/-" prefix, one decimal (e.g. "+/-4.1")
  - [ ] X-axis label formatting per bucket size (relative for Session, weekday for Day, "Mon DD" for Week, "Mon" for Month)
  - [ ] Relative time formatting: if within last hour "Xm ago", else "Xh ago" (keep simple)

- [ ] Task 7: Verify
  - [ ] `trunk serve` and manual testing with training data
  - [ ] Verify chart renders correctly with 1, 5, 20, 50 buckets
  - [ ] Verify empty state displays when no data
  - [ ] Run `cargo clippy` on web crate

## Dev Notes

### iOS to Web Mapping

| iOS Element | peach-web Equivalent |
|---|---|
| Swift Charts `LineMark` | SVG `<polyline>` |
| Swift Charts `AreaMark` | SVG `<path>` (closed area between upper and lower bounds) |
| Swift Charts `RuleMark` | SVG `<line>` with `stroke-dasharray` |
| `Chart` with `chartXAxis` | Custom SVG with tick marks and labels |
| `.background(.regularMaterial, in: RoundedRectangle)` | Tailwind card: `bg-surface rounded-xl p-4` |
| `@Environment(\.horizontalSizeClass)` | CSS media query or viewport width check |

### SVG Chart Approach

Rather than pulling in a charting library, build the SVG chart directly (same approach as the existing profile_visualization.rs). The chart is simple enough:

```
SVG viewBox = "0 0 {width} {height}"
├── <rect> background
├── <path> stddev band (filled, semi-transparent blue)
├── <polyline> mean line (blue stroke)
├── <line> baseline (green dashed)
├── <text> x-axis labels
└── <text> y-axis label
```

### Architecture Compliance

- **Web crate:** Chart components live in `web/src/components/`. Domain types (`TimeBucket`, `BucketSize`, `TrainingMode`) are imported from domain crate.
- **No domain changes:** This story is purely UI. ProgressTimeline API was defined in story 7.2.
- **Signal pattern:** ProgressTimeline wrapped in `Rc<RefCell<ProgressTimeline>>`, read via Leptos context. The bridge observer updates it and triggers re-renders via a signal (e.g. a version counter signal that increments on each update).
