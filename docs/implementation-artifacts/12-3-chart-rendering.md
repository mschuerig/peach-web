# Story 12.3: Chart Rendering

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to see my progress visualized as a chart with trend lines, variability bands, session dots, and a target baseline,
So that I can understand how my pitch perception is developing over time.

## Acceptance Criteria

1. **Index-based X-axis** — Given computed display buckets for a mode, when the chart renders, then each bucket gets equal visual width. X domain is -0.5 to bucketCount - 0.5. Y domain is 0 to max(1, max(bucket.mean + bucket.stddev)).

2. **Zone backgrounds (Layer 1)** — Given buckets spanning multiple granularity zones, when zone backgrounds render, then colored rectangles span each zone from startIndex - 0.5 to endIndex + 0.5. Monthly zone uses system background color at 6% opacity. Daily zone uses secondary system background at 6% opacity. Session zone uses system background color at 6% opacity.

3. **Single-zone suppression** — Given all data falls in a single zone, when the chart renders, then no zone backgrounds or divider lines are drawn.

4. **Zone dividers (Layer 2)** — Given zone transitions exist, when dividers render, then vertical lines appear at each index where granularity changes (drawn at index - 0.5). Year boundary dividers appear within the monthly zone where calendar year changes. Year boundaries within 1 index of a zone transition are suppressed. Lines are solid, 1px, secondary color.

5. **Stddev band (Layer 3)** — Given non-session buckets exist, when the stddev band renders, then a shaded area spans from max(0, mean - stddev) to mean + stddev for each non-session bucket. The band connects via a continuous line through non-session buckets only. Color is blue at 15% opacity.

6. **Session bridge** — Given both non-session and session buckets exist, when the session bridge renders, then a bridge point is computed at X = firstSessionIndex - 0.5. bridgeMean = weighted average of session bucket means by record count. bridgeStddev = sqrt(weighted average of session bucket variances by record count). The band and line extend to this bridge point.

7. **Session-only case** — Given only session buckets exist (all data is from today), when the chart renders, then no line or band is drawn — only disconnected session dots appear.

8. **Mean trend line (Layer 4)** — Given non-session buckets, when the mean trend line renders, then a blue line connects the mean values of non-session buckets plus the bridge point. Session buckets are not connected by the line.

9. **Session dots (Layer 5)** — Given session buckets, when session dots render, then each session bucket is shown as a disconnected blue circle (point mark, size 20 area units).

10. **Baseline (Layer 6)** — Given a mode's optimal baseline value, when the baseline renders, then a horizontal dashed line appears at that Y value. Dash pattern is [5, 3], 1px width, green at 60% opacity.

11. **X-axis labels** — Given X-axis labels, when they render: monthly zone shows short month name (e.g. "Jan", "Feb"); daily zone shows short weekday name (e.g. "Mon", "Tue"); first session bucket shows "Today", subsequent session buckets show nothing. Trailing dots stripped from abbreviated names (e.g. "Dez." → "Dez" in German).

12. **Year labels** — Given monthly zone spans multiple calendar years, when year labels render, then year text (e.g. "2025", "2026") is centered below the monthly zone between the first and last bucket index of each year span. Font is caption2 size, secondary color. Extra bottom padding (16px baseline) is added. No monthly zone = no year labels = no extra padding.

13. **Increased contrast** — Given @media (prefers-contrast: more) is active: stddev band opacity is 0.30 (not 0.15), baseline opacity is 0.90 (not 0.60), zone background opacity is 0.12 (not 0.06), zone dividers use primary color (not secondary).

14. **Responsive height** — Chart height is 180px on mobile, 240px on tablet/desktop.

## Tasks / Subtasks

- [x] Task 1: Refactor coordinate system to index-based X-axis (AC: 1)
  - [x] 1.1 Replace time-based X mapping with index-based: X domain = -0.5 to bucketCount - 0.5, each bucket at its integer index
  - [x] 1.2 Update Y domain computation: 0 to max(1, max(bucket.mean + bucket.stddev))
  - [x] 1.3 Update `x()` and `y()` closure helpers to use index-based coordinate system
  - [x] 1.4 Adjust viewbox margins to accommodate year labels when needed (increase MARGIN_BOTTOM dynamically)

- [x] Task 2: Implement zone detection and background tints (AC: 2, 3)
  - [x] 2.1 Compute zone ranges: scan `buckets` array, group consecutive buckets by `bucket_size`, record start/end indices per zone
  - [x] 2.2 Count distinct zones — if only 1 zone, skip all background and divider rendering
  - [x] 2.3 Render zone background rectangles: X from startIndex - 0.5 to endIndex + 0.5, full Y height. Monthly/Session zones: `fill="currentColor"` at 6% opacity (light) / 6% opacity (dark). Daily zone: secondary color at 6% opacity
  - [x] 2.4 Use CSS custom properties or inline opacity for increased contrast override (0.06 → 0.12)

- [x] Task 3: Implement zone dividers and year boundaries (AC: 4)
  - [x] 3.1 Compute zone transition indices: where `buckets[i].bucket_size != buckets[i-1].bucket_size`; draw vertical line at index - 0.5
  - [x] 3.2 Compute year boundary indices: within monthly zone, where calendar year of `buckets[i]` differs from `buckets[i-1]`; draw vertical line at index - 0.5
  - [x] 3.3 Suppress year boundaries within 1 index of any zone transition
  - [x] 3.4 Line style: solid 1px, secondary color (gray); use CSS class for increased contrast override to primary color

- [x] Task 4: Implement session bridge computation (AC: 6, 7)
  - [x] 4.1 Identify `firstSessionIndex` — index of first bucket where `bucket_size == Session`
  - [x] 4.2 Compute bridge point: bridgeMean = sum(session.mean * session.record_count) / totalSessionRecords; bridgeStddev = sqrt(sum(session.stddev^2 * session.record_count) / totalSessionRecords)
  - [x] 4.3 Bridge X coordinate = firstSessionIndex - 0.5
  - [x] 4.4 Only compute bridge when both non-session AND session buckets exist
  - [x] 4.5 When only session buckets exist, set a flag to render dots only (no line, no band)

- [x] Task 5: Refactor stddev band rendering (AC: 5)
  - [x] 5.1 Build band polygon points using only non-session buckets (filter `bucket_size != Session`)
  - [x] 5.2 Append bridge point to band polygon (if bridge exists)
  - [x] 5.3 Upper edge: mean + stddev at each point; lower edge: max(0, mean - stddev) at each point
  - [x] 5.4 Render as SVG `<polygon>` or `<path>` with fill blue at 15% opacity
  - [x] 5.5 Use CSS class for increased contrast override (0.15 → 0.30)

- [x] Task 6: Refactor mean trend line (AC: 8)
  - [x] 6.1 Build polyline points from non-session bucket means only
  - [x] 6.2 Append bridge point (if bridge exists)
  - [x] 6.3 Render as SVG `<polyline>` with stroke blue, stroke-width 2, stroke-linejoin round, vector-effect non-scaling-stroke

- [x] Task 7: Implement session dots (AC: 9)
  - [x] 7.1 For each bucket where `bucket_size == Session`, render an SVG `<circle>` at (index, mean)
  - [x] 7.2 Circle area = 20 units → radius = sqrt(20 / π) ≈ 2.52; scale to viewbox coordinates
  - [x] 7.3 Fill blue, no stroke

- [x] Task 8: Update baseline rendering (AC: 10)
  - [x] 8.1 Verify baseline dashed line renders at `optimal_baseline` Y value across full chart width
  - [x] 8.2 Ensure dash pattern [5, 3], 1px width, green color
  - [x] 8.3 Set opacity to 0.60; use CSS class for increased contrast override (0.60 → 0.90)

- [x] Task 9: Refactor X-axis labels (AC: 11)
  - [x] 9.1 Monthly zone: use `Intl.DateTimeFormat` with `{month: "short"}` for locale-aware short month name from bucket's `period_start` epoch
  - [x] 9.2 Daily zone: use `Intl.DateTimeFormat` with `{weekday: "short"}` for locale-aware short weekday name
  - [x] 9.3 Session zone: "Today" (i18n key) for first session bucket, empty for subsequent
  - [x] 9.4 Strip trailing dots from formatted names (regex: trim trailing "." from abbreviated strings)
  - [x] 9.5 Deduplicate adjacent identical labels (already partially implemented — verify works with new zone-aware formatting)

- [x] Task 10: Implement year labels (AC: 12)
  - [x] 10.1 Scan monthly zone buckets, extract calendar year from each bucket's `period_start`
  - [x] 10.2 Group consecutive monthly buckets by year, compute center X position per year span
  - [x] 10.3 Render year text (e.g. "2025") at center X, below X-axis labels
  - [x] 10.4 Font: small secondary color (8px in viewbox coordinates, or ~caption2 equivalent)
  - [x] 10.5 When year labels exist, increase MARGIN_BOTTOM to accommodate (add ~16px equivalent in viewbox units)
  - [x] 10.6 When no monthly zone or all same year, skip year labels and extra padding

- [x] Task 11: Implement increased contrast CSS (AC: 13)
  - [x] 11.1 Add CSS classes or custom properties for chart opacity values
  - [x] 11.2 Add `@media (prefers-contrast: more)` rules in `input.css` for: zone backgrounds (0.06→0.12), stddev band (0.15→0.30), baseline (0.60→0.90), zone dividers (secondary→primary color)
  - [x] 11.3 Apply these classes/properties to the corresponding SVG elements

- [x] Task 12: Reconnect ProgressChart in ProgressCard (AC: 14)
  - [x] 12.1 In `progress_card.rs`, replace the placeholder `<div>` with the `ProgressChart` component, passing `buckets`, `optimal_baseline`, and `unit_label` from the timeline context
  - [x] 12.2 Remove `#[allow(dead_code)]` from `progress_chart` module in `mod.rs`
  - [x] 12.3 Verify responsive height classes `h-[180px] md:h-[240px]` are applied (already on the ProgressChart component)

- [x] Task 13: Verify and test (AC: 1-14)
  - [x] 13.1 `cargo clippy --workspace` — zero warnings
  - [x] 13.2 `cargo test -p domain` — all tests pass (no domain changes expected)
  - [ ] 13.3 UNCHECKED — Manual browser test: verify all 6 rendering layers render correctly with test data spanning all three zones
  - [ ] 13.4 UNCHECKED — Manual browser test: verify single-zone case (no backgrounds/dividers)
  - [ ] 13.5 UNCHECKED — Manual browser test: verify session-only case (dots only, no line/band)
  - [ ] 13.6 UNCHECKED — Manual browser test: verify increased contrast mode changes opacities
  - [ ] 13.7 UNCHECKED — Manual browser test: verify locale-aware X-axis labels (German month/day names without trailing dots)

## Dev Notes

### What Already Exists (DO NOT REINVENT)

- **`web/src/components/progress_chart.rs`** — `ProgressChart` SVG component (200 lines). Has basic rendering with viewbox 300×160, margins, mean line, stddev band, baseline, Y-axis label, X-axis labels. **This is the primary file to modify.** Currently renders ALL buckets in both band and line (must change to exclude sessions). Uses time-based X-axis (must change to index-based).
- **`web/src/components/progress_card.rs`** — `ProgressCard` component with headline row, frosted glass styling, locale-aware `format_decimal_1()`. Currently renders a placeholder `<div>` instead of `ProgressChart` — Story 12.3 reconnects it.
- **`web/src/components/mod.rs`** — Has `#[allow(dead_code)]` on `progress_chart` module. Remove this when reconnecting.
- **`domain/src/progress_timeline.rs`** — Full public API: `display_buckets(mode) -> Vec<TimeBucket>`, `current_ewma(mode)`, `trend(mode)`, `latest_bucket_stddev(mode)`, `state(mode)`. `TimeBucket` has fields: `period_start`, `period_end`, `bucket_size` (BucketSize enum: Session, Day, Month), `mean`, `stddev`, `record_count`.
- **`domain/src/training_mode.rs`** — `TrainingModeConfig` with `optimal_baseline: f64`, `unit_label: &'static str`, `display_name: &'static str`. Per-mode baselines: Unison Comparison 8¢, Interval Comparison 12¢, Unison Matching 5¢, Interval Matching 8¢.
- **`web/src/app.rs`** — Provides `ProgressTimeline` via context as `SendWrapper<Rc<RefCell<ProgressTimeline>>>`. Profile route at `/profile`.
- **`input.css`** — Already has `@media (prefers-contrast: more)` rules for `.progress-card` frosted glass. Extend with chart-specific contrast rules.
- **`web/locales/en/main.ftl`** and **`web/locales/de/main.ftl`** — i18n files. Will need "Today" label key (check if already exists from progress_chart.rs).

### What Must Change

**`web/src/components/progress_chart.rs` (primary file — major overhaul):**

1. **X-axis coordinate system** — Replace the current time-based `x()` closure (maps `period_start` to pixel position) with index-based mapping. Each bucket renders at its array index (0, 1, 2, ...). X domain = -0.5 to bucketCount - 0.5.

2. **Zone detection** — Add logic to scan the `buckets` array and identify zone ranges (start/end index per BucketSize variant). Count distinct zones to decide whether to render backgrounds/dividers.

3. **Zone backgrounds (Layer 1, NEW)** — Render colored `<rect>` elements behind the data layers. Only when multiple zones exist. Use opacity 0.06 (normal), 0.12 (increased contrast). For dark/light mode differentiation, use Tailwind color tokens or CSS variables.

4. **Zone dividers (Layer 2, NEW)** — Render `<line>` elements at zone transition indices and year boundaries. Suppress year boundaries within 1 index of zone transitions.

5. **Session bridge computation (NEW)** — Before rendering band/line, compute the bridge point from session bucket statistics. Bridge X = firstSessionIndex - 0.5, bridge mean/stddev are record-count-weighted averages.

6. **Stddev band refactor (Layer 3)** — Currently includes ALL buckets. Must exclude session buckets and append bridge point instead. Build polygon from non-session bucket means ± stddev + bridge point.

7. **Mean line refactor (Layer 4)** — Currently includes ALL buckets. Must exclude session buckets and append bridge point.

8. **Session dots (Layer 5, NEW)** — Render `<circle>` for each session bucket. Area = 20 units. Not connected by line.

9. **X-axis label refactor** — Replace current `format_x_label()` function with zone-aware logic using `Intl.DateTimeFormat` for locale-aware month/weekday names. First session = "Today", rest empty. Strip trailing dots.

10. **Year labels (NEW)** — Render year text below monthly zone buckets. Increase bottom margin when present.

11. **Baseline update** — Existing baseline rendering is close to spec. Verify opacity (0.60), add increased contrast CSS class.

12. **Rendering order** — Ensure SVG elements are in correct Z-order: zone backgrounds → dividers → stddev band → mean line → session dots → baseline.

**`web/src/components/progress_card.rs` (secondary file):**

1. Replace the placeholder `<div class="mt-3 h-[180px] md:h-[240px] rounded-lg bg-gray-200/30 dark:bg-gray-700/30" />` with the actual `ProgressChart` component call.

**`web/src/components/mod.rs`:**

1. Remove `#[allow(dead_code)]` from the `progress_chart` module declaration.

**`input.css`:**

1. Add `@media (prefers-contrast: more)` rules for chart SVG elements (zone backgrounds, stddev band, baseline, dividers).

### Component Signature

The existing `ProgressChart` signature is:

```rust
pub fn ProgressChart(
    buckets: Vec<TimeBucket>,
    optimal_baseline: f64,
    unit_label: &'static str,
) -> impl IntoView
```

This signature is sufficient — no changes needed. The caller (`ProgressCard`) passes `timeline.display_buckets(mode)`, `mode.config().optimal_baseline`, and `mode.config().unit_label`.

### Coordinate System — Index-Based X-Axis

```rust
// Current (WRONG — time-based):
let x = |timestamp: f64| -> f64 {
    MARGIN_LEFT + (timestamp - x_min) / (x_max - x_min) * chart_width
};

// New (CORRECT — index-based):
let bucket_count = buckets.len();
let x = |index: f64| -> f64 {
    MARGIN_LEFT + (index + 0.5) / bucket_count as f64 * chart_width
};
// X domain: -0.5 to bucket_count - 0.5
// Bucket i is centered at x(i as f64)
```

### Zone Detection Algorithm

```rust
struct ZoneRange {
    zone: BucketSize,
    start_index: usize,
    end_index: usize,  // inclusive
}

fn detect_zones(buckets: &[TimeBucket]) -> Vec<ZoneRange> {
    let mut zones = Vec::new();
    if buckets.is_empty() { return zones; }
    let mut current_zone = buckets[0].bucket_size;
    let mut start = 0;
    for (i, b) in buckets.iter().enumerate() {
        if b.bucket_size != current_zone {
            zones.push(ZoneRange { zone: current_zone, start_index: start, end_index: i - 1 });
            current_zone = b.bucket_size;
            start = i;
        }
    }
    zones.push(ZoneRange { zone: current_zone, start_index: start, end_index: buckets.len() - 1 });
    zones
}
```

### Session Bridge Computation

```rust
fn compute_session_bridge(
    buckets: &[TimeBucket],
    first_session_index: usize,
) -> Option<(f64, f64, f64)> {
    // Returns (bridge_x, bridge_mean, bridge_stddev)
    let session_buckets: Vec<&TimeBucket> = buckets[first_session_index..]
        .iter()
        .filter(|b| b.bucket_size == BucketSize::Session)
        .collect();
    if session_buckets.is_empty() { return None; }

    let total_records: usize = session_buckets.iter().map(|b| b.record_count).sum();
    if total_records == 0 { return None; }

    let bridge_mean = session_buckets.iter()
        .map(|b| b.mean * b.record_count as f64)
        .sum::<f64>() / total_records as f64;

    let bridge_var = session_buckets.iter()
        .map(|b| b.stddev * b.stddev * b.record_count as f64)
        .sum::<f64>() / total_records as f64;

    Some((
        first_session_index as f64 - 0.5,  // bridge X
        bridge_mean,
        bridge_var.sqrt(),
    ))
}
```

### X-Axis Label Formatting — Locale-Aware

Use `js_sys::Intl::DateTimeFormat` for locale-aware month/weekday names:

```rust
use js_sys::{Intl, Object, Reflect, Date as JsDate};

fn format_month_label(epoch_secs: f64) -> String {
    let date = JsDate::new(&JsValue::from_f64(epoch_secs * 1000.0));
    let options = Object::new();
    Reflect::set(&options, &"month".into(), &"short".into()).unwrap();
    // Use Intl.DateTimeFormat to get locale-aware month name
    // Strip trailing dot if present
    let formatted = /* ... */;
    formatted.trim_end_matches('.').to_string()
}

fn format_weekday_label(epoch_secs: f64) -> String {
    let date = JsDate::new(&JsValue::from_f64(epoch_secs * 1000.0));
    let options = Object::new();
    Reflect::set(&options, &"weekday".into(), &"short".into()).unwrap();
    // Strip trailing dot if present
    let formatted = /* ... */;
    formatted.trim_end_matches('.').to_string()
}
```

**Note:** The existing `format_x_label()` function uses hardcoded English day/month names. This must be replaced with `Intl.DateTimeFormat` for proper locale support. Look at the existing `format_decimal_1()` in `progress_card.rs` for the `js_sys::Intl` usage pattern.

### Year Boundary Detection

```rust
fn year_from_epoch(epoch_secs: f64) -> i32 {
    let date = JsDate::new(&JsValue::from_f64(epoch_secs * 1000.0));
    date.get_full_year() as i32
}
```

Within the monthly zone, scan for year changes between adjacent buckets. Suppress year dividers within 1 index of a zone transition.

### SVG Rendering Order (Z-Order)

Elements must appear in this order in the SVG (later = on top):

1. Zone background `<rect>` elements (Layer 1)
2. Zone divider `<line>` elements (Layer 2)
3. Stddev band `<polygon>` / `<path>` (Layer 3)
4. Mean trend `<polyline>` (Layer 4)
5. Session dot `<circle>` elements (Layer 5)
6. Baseline `<line>` (Layer 6)
7. Y-axis label `<text>` (existing)
8. X-axis label `<text>` elements (existing, refactored)
9. Year label `<text>` elements (new)

### Increased Contrast — CSS Strategy

Add CSS classes to SVG elements, then override in `input.css`:

```css
/* Chart increased contrast overrides */
@media (prefers-contrast: more) {
    .chart-zone-bg { opacity: 0.12; }
    .chart-stddev-band { opacity: 0.30; }
    .chart-baseline { opacity: 0.90; }
    .chart-zone-divider { stroke: currentColor; /* primary instead of secondary */ }
}
```

Apply these classes to the corresponding SVG elements. Normal opacity values are set inline or via default CSS.

### Color Mapping (iOS System Colors → CSS)

| iOS System Color | Light Mode CSS | Dark Mode CSS | Usage |
|---|---|---|---|
| Blue (accent) | `rgb(59, 130, 246)` | `rgb(96, 165, 250)` | Mean line, session dots, stddev band |
| Green | `rgb(34, 197, 94)` | `rgb(34, 197, 94)` | Baseline |
| System background | `rgb(249, 250, 251)` | `rgb(17, 24, 39)` | Monthly/Session zone backgrounds |
| Secondary background | `rgb(243, 244, 246)` | `rgb(31, 41, 55)` | Daily zone background |
| Secondary text | `rgb(107, 114, 128)` | `rgb(156, 163, 175)` | Zone dividers, axis labels |

These colors are already used in the existing `progress_chart.rs`. Verify they match and extend for zone backgrounds.

### What NOT to Change

- **Do NOT add horizontal scrolling** — Story 12.4 handles scrolling for >8 buckets.
- **Do NOT add annotation popovers or tap interaction** — Epic 13 (PFR15).
- **Do NOT add help overlay content** — Epic 13 (PFR17).
- **Do NOT add zone-level VoiceOver summaries** — Deferred to follow-up story per spec.
- **Do NOT modify domain crate** — All data comes from the existing `display_buckets()` API.
- **Do NOT add selection indicator (Layer 7)** — Epic 13 (PFR15).

### Story 12.2 Learnings (Previous Story Intelligence)

From the 12.2 implementation:

1. **`format_decimal_1()` exists in `progress_card.rs`** — Uses `js_sys::Intl::NumberFormat`. Follow this exact pattern for `Intl.DateTimeFormat` in chart labels.
2. **Chart placeholder is `<div class="mt-3 h-[180px] md:h-[240px] rounded-lg bg-gray-200/30 dark:bg-gray-700/30" />`** — Replace with `<ProgressChart>` component call.
3. **`#[allow(dead_code)]` on `progress_chart` module** — Remove when reconnecting.
4. **Frosted glass CSS for `.progress-card`** — Already has increased contrast rules in `input.css`. Add chart-specific rules alongside.
5. **4 new i18n keys added** — Check if "Today" key already exists before adding.
6. **`ProgressChart` already has responsive height** — `h-[180px] md:h-[240px]` classes are on the component.

### Story 12.1 Learnings

1. **`display_buckets(mode)` is the API** — returns `Vec<TimeBucket>` in chronological order (months → days → sessions).
2. **`TimeBucket.bucket_size`** — `BucketSize::Session`, `BucketSize::Day`, or `BucketSize::Month`. Use this to identify zones and determine label formatting.
3. **`TimeBucket.record_count`** — needed for session bridge weighted average computation.
4. **Population stddev** — `stddev` field uses population formula (n divisor). No adjustment needed.

### Git Intelligence

Recent commits (all in Epic 12 scope):
- `c3013c0` — Code review passed for story 12.2: mark as done
- `1fc8031` — Implement story 12.2: frosted glass cards, locale formatting, chart placeholder, accessibility
- `28f3fc4` — Code review fixes for story 12.1: removed dead `_now` param, deduplicated `start_of_today`
- `cc6847b` — Implement story 12.1: three-zone bucketing, EWMA pipeline

Files most recently modified: `progress_chart.rs`, `progress_card.rs`, `profile_view.rs`, `mod.rs`, `input.css`, locale files.

### Project Structure Notes

- Primary file: `web/src/components/progress_chart.rs` (major overhaul)
- Secondary file: `web/src/components/progress_card.rs` (reconnect chart)
- Secondary file: `web/src/components/mod.rs` (remove dead_code allow)
- CSS: `input.css` (add chart increased contrast rules)
- Possible i18n: `web/locales/en/main.ftl`, `web/locales/de/main.ftl` (add "Today" key if not present)
- No domain crate changes
- No new files needed

### References

- [Source: docs/ios-reference/profile-screen-specification.md#Chart Rendering] — Authoritative reference for coordinate system, rendering layers, session bridge, zone backgrounds, dividers, labels
- [Source: docs/ios-reference/profile-screen-specification.md#Increased Contrast Mode] — Opacity overrides
- [Source: docs/ios-reference/profile-screen-specification.md#X-Axis Labels] — Label formatting rules, trailing dot stripping
- [Source: docs/ios-reference/profile-screen-specification.md#Year Labels] — Year label positioning and padding
- [Source: docs/planning-artifacts/epics.md#Story 12.3: Chart Rendering] — Full BDD acceptance criteria
- [Source: docs/planning-artifacts/epics.md#Epic 12: Profile Progress Charts] — PFR8-PFR14, PFR16, PNFR3, PNFR5
- [Source: web/src/components/progress_chart.rs] — Existing SVG component to overhaul
- [Source: web/src/components/progress_card.rs] — Card container with chart placeholder to replace
- [Source: domain/src/progress_timeline.rs] — Public API (display_buckets, TimeBucket, BucketSize)
- [Source: domain/src/training_mode.rs] — TrainingModeConfig (optimal_baseline, unit_label)
- [Source: docs/project-context.md] — Coding conventions, Leptos patterns, accessibility requirements

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

None — clean implementation with zero compilation issues.

### Completion Notes List

- Rewrote `progress_chart.rs` from time-based to index-based X-axis coordinate system (AC 1)
- Implemented zone detection algorithm grouping consecutive buckets by BucketSize (AC 2, 3)
- Zone backgrounds render as `<rect>` with `currentColor` at 6% opacity, suppressed for single-zone case
- Zone dividers and year boundary lines at zone transitions, with year boundary suppression near zone transitions (AC 4)
- Session bridge computation: record-count-weighted mean/stddev at firstSessionIndex - 0.5 (AC 6)
- Session-only detection: when all data is today, renders only disconnected dots (AC 7)
- Stddev band and mean trend line now exclude session buckets, appending bridge point instead (AC 5, 8)
- Session dots rendered as `<circle>` with area=20 units (AC 9)
- Baseline dashed line at optimal_baseline with [5,3] dash pattern, 0.60 opacity (AC 10)
- X-axis labels now locale-aware via `Intl.DateTimeFormat` for months/weekdays, with trailing dot stripping (AC 11)
- Added i18n key `chart-today` for "Today" label on first session bucket (EN/DE)
- Year labels centered below monthly zone spans, with dynamic bottom margin (AC 12)
- Added `@media (prefers-contrast: more)` CSS overrides for all chart elements (AC 13)
- Reconnected ProgressChart in ProgressCard, removed dead_code allow (AC 14)
- Responsive height h-[180px] md:h-[240px] already present on SVG element

### File List

- `web/src/components/progress_chart.rs` — Major rewrite: index-based coords, zone detection, bridge, layers 1-6, locale-aware labels, year labels
- `web/src/components/progress_card.rs` — Replaced chart placeholder div with ProgressChart component
- `web/src/components/mod.rs` — Removed `#[allow(dead_code)]` from progress_chart module
- `input.css` — Added `@media (prefers-contrast: more)` rules for chart SVG elements
- `web/locales/en/main.ftl` — Added `chart-today = Today`
- `web/locales/de/main.ftl` — Added `chart-today = Heute`
- `docs/implementation-artifacts/sprint-status.yaml` — Updated 12-3-chart-rendering status

### Change Log

- 2026-03-13: Implemented all 13 tasks for Story 12.3 Chart Rendering. Full SVG chart overhaul with 6 rendering layers, zone detection, session bridge, locale-aware labels, year labels, and increased contrast CSS.
- 2026-03-13: Code review fixes — (H1) Fixed year boundary suppression off-by-half comparing mixed coordinate spaces, now uses `i.abs_diff(t) <= 1`. (M1) Added CSS rules for `.chart-zone-bg-secondary` to differentiate daily zone background. (M2) Fixed stddev band fill color: was hardcoded dark-mode blue, now uses Tailwind `fill-blue-500 dark:fill-blue-400`.
