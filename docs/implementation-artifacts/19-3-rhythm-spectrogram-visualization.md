# Story 19.3: Rhythm Spectrogram Visualization

Status: backlog

## Story

As a user,
I want to see my rhythm accuracy as a heatmap across tempo ranges and time,
so that I can identify which tempos I'm precise at and where I need work.

## Context

Prerequisite: Story 19.2 (rhythm profile visualization with line charts exists), Epics 17-18 (all rhythm record types and training views exist).

The iOS app uses a spectrogram-style heatmap for rhythm profiles (`SpectrogramData.swift` + `RhythmSpectrogramView.swift`). During Story 19.2, the web implementation substituted generic line charts for rhythm — merging all tempo ranges and directions into a single timeline. This loses the key dimension that makes rhythm progress visible: accuracy *per tempo range*.

The web app already has the domain building blocks: `TempoRange` (Slow/Medium/Fast), `RhythmDirection` (Early/OnBeat/Late), `StatisticsKey` with tempo-range x direction keying, and `ProgressTimeline` with three-zone time bucketing. What's missing is the spectrogram computation and a heatmap chart component.

### iOS Reference

- `SpectrogramData.swift`: Grid computation — one cell per (TempoRange x TimeBucket), with combined mean accuracy and separate early/late `SpectrogramCellStats` (mean%, stddev%, count)
- `RhythmSpectrogramView.swift`: SwiftUI grid with color-coded cells, tap-to-detail popover showing early/late breakdown, VoiceOver per-column summaries
- `SpectrogramThresholds`: Hybrid floor/ceiling model — base percentage of 16th note, clamped to absolute ms bounds. Default: precise = 8% (12-30ms), moderate = 20% (25-50ms)
- Only tempo ranges with training data are shown on the y-axis (`trainedRanges`)

## Acceptance Criteria

1. **AC1 — Domain SpectrogramData type:** A `SpectrogramData` struct in the domain crate computes the spectrogram grid from `ProgressTimeline` data. Each cell represents one `TempoRange` x one `TimeBucket` with: `mean_accuracy_percent: Option<f64>`, `early_stats: Option<CellStats>`, `late_stats: Option<CellStats>`. Only tempo ranges with data appear in the grid.

2. **AC2 — SpectrogramThresholds:** Accuracy levels (precise/moderate/erratic) use the iOS hybrid floor/ceiling model: base percentage of 16th-note duration, clamped to absolute ms bounds. Default thresholds match iOS: precise = 8% base (12-30ms clamp), moderate = 20% base (25-50ms clamp).

3. **AC3 — Heatmap chart component:** A new `RhythmSpectrogramChart` web component renders the grid as an SVG heatmap. X-axis = time buckets with zone backgrounds matching `ProgressChart` conventions (session/day/month). Y-axis = trained tempo ranges. Cell color: green (precise), yellow (moderate), red (erratic), gray (no data).

4. **AC4 — Tap-to-detail:** Clicking a cell shows an annotation popover (matching the `ProgressChart` pattern) with: tempo range, date/period, mean accuracy %, early stats (mean%, count), late stats (mean%, count).

5. **AC5 — Profile integration:** The rhythm disciplines on the profile screen use the spectrogram chart instead of (or in addition to) the line chart. The existing EWMA headline + trend arrow on the `ProgressCard` remain unchanged.

6. **AC6 — Scrollable for large datasets:** When bucket count exceeds the visible area, the chart scrolls horizontally, matching `ProgressChart` behavior.

7. **AC7 — Accessibility:** Chart has `aria-label`. Selected cell detail is announced via `aria-live="polite"` region. Cells are keyboard-navigable.

8. **AC8 — Responsive:** Desktop `h-[240px]`, mobile `h-[180px]`, matching existing chart sizing.

9. **AC9 — Empty state:** When no rhythm data exists, the card shows "No data" (existing behavior). When data exists for only one tempo range, the grid renders a single row.

10. **AC10 — All tests pass:** Domain unit tests for `SpectrogramData` computation, threshold clamping, and edge cases (single bucket, single range, empty data). `cargo test -p domain` and `cargo clippy --workspace` pass.

## Tasks / Subtasks

- [ ] Task 1: Add `SpectrogramThresholds`, `SpectrogramAccuracyLevel`, `SpectrogramCellStats`, `SpectrogramCell`, `SpectrogramColumn`, `SpectrogramData` to domain crate
- [ ] Task 2: Implement `SpectrogramData::compute()` using `ProgressTimeline` buckets and `StatisticsKey::rhythm(mode, range, direction)` lookups
- [ ] Task 3: Add domain unit tests (threshold clamping across tempos, cell computation, empty/single-range/multi-range grids, bucket filtering)
- [ ] Task 4: Create `RhythmSpectrogramChart` web component — SVG heatmap grid with zone backgrounds, tempo-range y-axis labels, time x-axis labels
- [ ] Task 5: Implement cell color mapping (green/yellow/red/gray) using `SpectrogramThresholds::accuracy_level()`
- [ ] Task 6: Add click-to-detail popover with early/late breakdown (follow `ProgressChart` annotation pattern)
- [ ] Task 7: Add horizontal scroll for large datasets
- [ ] Task 8: Integrate into `ProfileView` — rhythm disciplines use spectrogram chart
- [ ] Task 9: Add i18n keys for spectrogram labels (tempo range names, accuracy levels, detail labels)
- [ ] Task 10: Accessibility — aria-label, aria-live announcements, keyboard navigation
- [ ] Task 11: Verify responsive sizing and dark mode
- [ ] Task 12: `cargo test -p domain`, `cargo clippy --workspace`, `trunk build`

## Dev Notes

- The iOS `SpectrogramData.compute()` takes `timeBuckets: [TimeBucket]` and `profile: TrainingProfile` — the web equivalent is `ProgressTimeline::display_buckets()` for time buckets, and the profile's `Statistics` keyed by `StatisticsKey::rhythm(discipline, range, direction)` for per-cell data
- The iOS approach filters raw `MetricPoint` arrays into buckets. The web may need a similar approach — check whether `ProgressTimeline` already provides per-key bucketed data or if spectrogram computation needs raw record access
- `TempoRange::midpoint_tempo()` and sixteenth-note-duration conversion are needed for threshold clamping — check if these exist in domain or need adding
- The `ProgressChart` component's SVG architecture (viewport, scroll container, zone backgrounds, annotation popover, accessibility live region) is the reference implementation for the chart infrastructure, but the spectrogram is a grid of rectangles, not a line chart — it's a new component, not a variant
- Consider whether to replace the rhythm line chart entirely or offer both views (heatmap as primary, line chart as secondary expandable). The iOS app shows only the spectrogram for rhythm.
- Story 19.2's dev notes confirm the line chart "just works" because `ProfileView` iterates `TrainingDiscipline::ALL` generically — the spectrogram needs discipline-type branching in profile rendering
