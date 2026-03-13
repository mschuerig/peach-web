---
title: Profile Screen Specification
description: Complete specification for recreating the Peach profile screen in a web app
author: Paige (Technical Writer Agent)
date: 2026-03-13
---

# Profile Screen Specification

This document specifies the Peach profile screen — layout, chart rendering, statistical calculations, interaction behavior, accessibility, and help content. The sibling web app already implements training modes, metrics, help overlays, and shared UI patterns (headlines, pills, help sheets). This spec covers only what is new for the profile screen.

## Training Mode Configuration

The web app already implements the four training modes and their metrics. The profile screen adds per-mode chart parameters:

| Mode | Optimal Baseline | EWMA Half-Life | Session Gap |
|------|-------------------|----------------|-------------|
| Unison Comparison | 8 ¢ | 7 days (604800 s) | 30 min (1800 s) |
| Interval Comparison | 12 ¢ | 7 days | 30 min |
| Unison Matching | 5 ¢ | 7 days | 30 min |
| Interval Matching | 8 ¢ | 7 days | 30 min |

- **Optimal Baseline** — expert-level accuracy target, shown as green dashed line on the chart
- **EWMA Half-Life** — decay constant for exponentially weighted moving average
- **Session Gap** — maximum gap between consecutive records that still counts as one training session

## Screen Layout

### Overall Structure

```text
┌─────────────────────────────────────┐
│ Navigation Bar: "Profile"  [?]      │
├─────────────────────────────────────┤
│ ScrollView (vertical)               │
│ ┌─────────────────────────────────┐ │
│ │ ProgressChartCard (Mode 1)      │ │
│ └─────────────────────────────────┘ │
│         16px gap                    │
│ ┌─────────────────────────────────┐ │
│ │ ProgressChartCard (Mode 2)      │ │
│ └─────────────────────────────────┘ │
│         16px gap                    │
│ │ ... (Mode 3, Mode 4 if active)  │ │
│                                     │
│    padding on all sides             │
└─────────────────────────────────────┘
```

- Navigation bar title: "Profile", inline display mode
- Toolbar trailing button: help icon (`questionmark.circle`), opens help sheet
- Cards appear only for modes that have at least one training record
- Card order follows the `TrainingMode` enum order: unison comparison, interval comparison, unison matching, interval matching
- VStack spacing between cards: **16px**
- Outer padding: system default on all sides

### Chart Card Structure

Each card is a self-contained component:

```text
┌─────────────────────────────────────────────┐
│  Hear & Compare – Single Notes    25.3 ±4.2 ↘│
│                                               │
│  ┌───────────────────────────────────────┐    │
│  │         Chart Area                    │    │
│  │   (scrollable or static)              │    │
│  │                                       │    │
│  │                                       │    │
│  └───────────────────────────────────────┘    │
│    2025                    2026               │
└───────────────────────────────────────────────┘
```

- **Card background**: frosted glass material (`regularMaterial`) with 12px corner radius
- **Card padding**: system default on all sides
- **Internal spacing**: 12px between headline row and chart area
- **Chart height**: 180px (compact/mobile) or 240px (regular/tablet/desktop)

### Headline Row

A horizontal row with baseline-aligned items:

| Position | Content | Style |
|----------|---------|-------|
| Left | Mode display name | Headline font |
| Right | EWMA value (e.g. "25.3") | Title 2, bold |
| Right | StdDev of last bucket (e.g. "±4.2") | Caption, secondary color |
| Right | Trend arrow icon | Colored by trend direction |

The stddev shown in the headline is the **per-bucket stddev of the most recent multi-granularity bucket** (not the running population stddev used for trend computation).

**Number formatting**: one decimal place, locale-aware (e.g. "25.3" in English, "25,3" in German). Use a decimal number formatter with `minimumFractionDigits: 1` and `maximumFractionDigits: 1`.

**Trend indicators**:

| Trend | Icon | Color | Condition |
|-------|------|-------|-----------|
| Improving | arrow pointing down-right (↘) | Green | Latest value < EWMA |
| Stable | arrow pointing right (→) | Secondary/gray | Latest value ≥ EWMA AND ≤ mean + stddev |
| Declining | arrow pointing up-right (↗) | Orange | Latest value > mean + stddev |

Trend requires at least 2 records. With fewer, no trend icon is shown.

**Important**: "Improving" means the value is going *down* (smaller error = better pitch perception). The arrow points down-right to convey this. The stddev used for trend computation is the *running population stddev* across all individual records (not per-bucket).

## Data Pipeline

### Metric Extraction

From raw training records, extract metric points per mode:

```pseudocode
for each record:
  if PitchComparisonRecord:
    if interval == 0 → unisonComparison: abs(centOffset)
    if interval != 0 → intervalComparison: abs(centOffset)
  if PitchMatchingRecord:
    if interval == 0 → unisonMatching: abs(userCentError)
    if interval != 0 → intervalMatching: abs(userCentError)

Each metric point = (timestamp, absoluteValue)
```

### Multi-Granularity Time Bucketing

The chart uses three granularity zones, with boundaries snapped to calendar days:

```text
              dayStart              sessionStart           now
              (midnight,            (midnight               │
               7 days ago)           today)                 ▼
◄── Month zone ──►│◄── Day zone (7d) ──►│◄── Session zone ──►│
 Oct  Nov  Dec Jan│ Fr Sa Su Mo Tu We Th│  session1  session2│
```

**Zone boundary computation**:

```pseudocode
sessionStart = startOfDay(now)           // midnight today, local time
dayStart     = sessionStart - 7 days     // 7 calendar days before today
monthStart   = (no explicit bound)       // everything older
```

**Bucketing rules**:

1. **Session zone** (`timestamp >= sessionStart`): Group records into sessions. Two consecutive records belong to the same session if their timestamps are less than `sessionGap` (30 minutes) apart. Each session becomes one bucket.

2. **Day zone** (`dayStart <= timestamp < sessionStart`): Group records by calendar day. Each day with records becomes one bucket.

3. **Month zone** (`timestamp < dayStart`): Group records by calendar month. The last monthly bucket's end date is truncated to `dayStart` to prevent overlap with the day zone.

**Weekly granularity is intentionally omitted.**

All buckets are concatenated in chronological order (months first, then days, then sessions) into a single flat array.

### Bucket Aggregation

For each bucket, compute:

```pseudocode
mean   = sum(values) / count
stddev = sqrt(sum((value - mean)²) / count)    // population stddev, NOT sample
```

- If only 1 record in bucket: `stddev = 0`
- `recordCount` = number of individual training attempts in the bucket

### EWMA Computation

The EWMA is computed over a separate set of **session-level buckets** (not the multi-granularity display buckets). These session-level buckets are the raw timeline of training sessions — every bucket is tagged as session granularity and grouped purely by the session gap rule (consecutive records within 30 minutes merge into one bucket). This is the internal representation used by the analytics engine, distinct from the multi-granularity buckets used for chart rendering.

The formula:

```pseudocode
ewma[0] = bucket[0].mean

for i in 1..<buckets.count:
  dt    = bucket[i].periodStart - bucket[i-1].periodStart    // in seconds
  alpha = 1.0 - exp(-ln(2) × dt / halflife)                  // halflife = 604800s
  ewma[i] = alpha × bucket[i].mean + (1 - alpha) × ewma[i-1]

currentEWMA = ewma[last]
```

The current EWMA value is displayed in the headline row and used for trend computation.

**Important**: The EWMA uses session-level buckets where age determines the granularity: records < 24h stay as sessions, 24h–7d become daily, 7d–30d become weekly, > 30d become monthly. This is a different bucketing than the display pipeline. The key takeaway: the EWMA formula itself is the same regardless — iterate chronological buckets and apply the exponential decay. What matters is the formula above, not which bucket array feeds it.

### Trend Computation

Requires: at least 2 total records across all time.

Uses three values computed over *all individual records* (not buckets):

- `runningMean` — cumulative mean of all metric values (Welford's online algorithm)
- `runningStddev` — population standard deviation of all metric values: `sqrt(M2 / count)`
- `latestValue` — the most recent individual metric point value

```pseudocode
if latestValue > runningMean + runningStddev → Declining
else if latestValue >= currentEWMA           → Stable
else                                         → Improving
```

## Chart Rendering

### Coordinate System

The chart uses an **index-based X-axis**, not a date-based one. Each bucket gets an integer index (0, 1, 2, ...). This gives every data point equal visual width regardless of the time span it represents.

- **X domain**: `-0.5` to `bucketCount - 0.5`
- **Y domain**: `0` to `max(1, max(bucket.mean + bucket.stddev for each bucket))`
- **Y-axis label**: the mode's unit label ("cents")

### Scrolling Behavior

- If **≤ 8 buckets**: static layout, all data visible, no scrolling
- If **> 8 buckets**: horizontally scrollable, 8 buckets visible at a time
- Initial scroll position: rightmost data visible (most recent data at right edge)
- Scroll position formula: `max(0, bucketCount - 8)`
- Scrolling dismisses any active selection annotation

### Rendering Layers (Z-Order)

The chart draws elements in this exact order (bottom to top):

#### Layer 1: Zone Background Tints

Colored rectangles spanning each granularity zone. Only rendered when more than one zone exists (if all data falls in a single zone, no backgrounds or dividers are drawn).

| Zone | Background Color | Opacity (normal) | Opacity (increased contrast) |
|------|-----------------|-------------------|------------------------------|
| Monthly | System background color | 6% | 12% |
| Daily | Secondary system background | 6% | 12% |
| Session | System background color | 6% | 12% |

Each rectangle spans from `startIndex - 0.5` to `endIndex + 0.5` on X, and the full Y domain.

#### Layer 2: Zone Dividers

Vertical lines at zone transitions and year boundaries.

- **Zone transition dividers**: at each index where granularity changes (drawn at `index - 0.5`)
- **Year boundary dividers**: within the monthly zone, at each index where the calendar year changes
- **Deduplication**: year boundaries within 1 index of a zone transition are suppressed
- **Line style**: solid, 1px width
- **Color**: secondary color (normal contrast) or primary color (increased contrast)

#### Layer 3: Standard Deviation Band

A shaded area between `mean - stddev` and `mean + stddev`, clamped to `max(0, mean - stddev)` on the lower bound.

**Critical rendering rule**: The band connects only non-session buckets via a continuous line. Session dots are disconnected — no band is drawn through the session zone.

**Session bridge**: The line/band extends from the last non-session bucket to the zone separator position (`firstSessionIndex - 0.5`) using a weighted average of all session bucket values. This creates a smooth visual transition at the zone boundary without connecting individual sessions.

```pseudocode
totalRecords = sum(sessionBucket.recordCount for each sessionBucket)
bridgeMean   = sum(sessionBucket.mean × sessionBucket.recordCount) / totalRecords
bridgeVar    = sum(sessionBucket.stddev² × sessionBucket.recordCount) / totalRecords
bridgeStddev = sqrt(bridgeVar)
bridgeX      = firstSessionIndex - 0.5
```

The bridge is only added when both non-session buckets AND session buckets exist. If all data is from today (session zone only), no line or band is drawn — only disconnected session dots appear.

- **Color**: blue, 15% opacity (normal) / 30% opacity (increased contrast)

#### Layer 4: Mean Trend Line

A connected line through the **bucket mean values** of non-session buckets, plus the session bridge point. Note: this line shows per-bucket means, not the EWMA values (the EWMA is only shown as the number in the headline).

- Same data points as the stddev band (Layer 3)
- **Color**: blue, full opacity
- Session buckets are excluded from the line (they appear as dots instead)

#### Layer 5: Session Dots

Individual disconnected points for each session bucket.

- Rendered only for buckets where `bucketSize == .session`
- **Symbol**: circle (point mark)
- **Symbol size**: 20 (area units)
- **Color**: blue
- No connecting lines, no band

#### Layer 6: Baseline

A horizontal dashed line at the mode's `optimalBaseline` value.

- **Line style**: dashed, pattern `[5, 3]` (5px dash, 3px gap), 1px width
- **Color**: green, 60% opacity (normal) / 90% opacity (increased contrast)

#### Layer 7: Selection Indicator

When a bucket is tapped, a vertical dashed line with an annotation popover appears.

- **Line style**: dashed, pattern `[5, 3]`, 1px width
- **Color**: gray, 50% opacity (normal) / 80% opacity (increased contrast)
- **Annotation position**: top of chart, with overflow resolution that keeps the popover within the chart bounds on both axes

### X-Axis Labels

Labels depend on the zone:

| Zone | Format | Example |
|------|--------|---------|
| Monthly | Short month name | Jan, Feb, Mar |
| Daily | Short weekday name | Mon, Tue, Wed |
| Session | "Today" for the first session bucket; empty string for all subsequent session buckets | Today |

**Localization rule**: Strip trailing dots from abbreviated month/day names in languages that add them (e.g., German: "Dez." → "Dez", "Mo." → "Mo").

### Year Labels

Year labels appear below the X-axis, only within the monthly zone.

- Position: horizontally centered between the first and last bucket index of each calendar year span
- Vertical position: below the full chart area (including axis labels), dynamically adapting to text size
- Font: caption2 size, secondary color
- Vertical offset below chart: 8px baseline (scales with caption2 text size)
- Extra bottom padding added to the chart when year labels exist: 16px baseline (scales with caption2 text size)
- When no year labels exist: no extra bottom padding

### Annotation Popover

When a user taps a data point, a popover appears showing bucket details:

```text
┌──────────────────┐
│ Jan 2026         │  ← date label (caption2, secondary)
│ 25.3             │  ← bucket mean value (caption, bold)
│ ±4.2             │  ← bucket stddev (caption2, secondary)
│ 47 records       │  ← record count (caption2, secondary)
└──────────────────┘
```

- **Background**: frosted glass material, 6px corner radius
- **Padding**: 6px on all sides
- **VStack spacing**: 2px

**Date label formatting by zone**:

| Zone | Format | Example |
|------|--------|---------|
| Monthly | Month + Year (template "MMM yyyy") | Jan 2026 |
| Daily | Weekday + Month + Day (template "E MMM d") | Mon, Mar 5 |
| Session | Hours:minutes (template "HH:mm") | 14:30 |

Note: the annotation date formatter for sessions uses a fixed "HH:mm" template. The X-axis label for sessions does not use a time format — it shows "Today" for the first session bucket and nothing for the rest (see X-Axis Labels above).

### Tap Interaction

- **Gesture**: spatial tap (not drag, which would conflict with scroll)
- Tap resolves to the nearest bucket index by rounding the X coordinate
- Tapping the same area again or scrolling dismisses the annotation
- On scrollable charts: scroll changes automatically dismiss the selection

## Zone Configuration Reference

| Property | Monthly | Daily | Session |
|----------|---------|-------|---------|
| Point width | 30px | 40px | 50px |
| Axis label format | "MMM" | "EEE" | (see Session label rules) |

## Increased Contrast Mode

When the user enables the system "Increase Contrast" setting, adjust these opacity values:

| Element | Normal | Increased |
|---------|--------|-----------|
| Stddev band | 0.15 | 0.30 |
| Baseline | 0.60 | 0.90 |
| Zone backgrounds | 0.06 | 0.12 |
| Selection indicator | 0.50 | 0.80 |
| Zone dividers | Secondary color | Primary color |

## Help Content

The profile screen's help overlay (triggered by the `?` toolbar button) uses the same overlay mechanism as all other screens. The content consists of five sections:

1. **Your Progress Chart** — "This chart shows how your pitch perception is developing over time"
2. **Trend Line** — "The blue line shows your smoothed average — it filters out random ups and downs to reveal your real progress"
3. **Variability Band** — "The shaded area around the line shows how consistent you are — a narrower band means more reliable results"
4. **Target Baseline** — "The green dashed line is your goal — as the trend line approaches it, your ear is getting sharper"
5. **Time Zones** — "The chart groups your data by time: months on the left, recent days in the middle, and today's sessions on the right"

## Accessibility

### Card-Level

- Each card is an accessibility container (children individually navigable)
- Accessibility label: "Progress chart for {mode display name}"
- Accessibility value: "Current: {EWMA} {unit}, trend: {trend label}"

### Zone-Level VoiceOver

Each granularity zone gets an invisible accessibility element overlaying its area. The summary includes:

- Zone name (Monthly/Daily/Session)
- Date range (first to last bucket date)
- Value range (first to last bucket mean)
- Number of buckets in the zone (referred to as "data points" in the spoken text)

Single-bucket zones: "{zoneName} zone: {date}, pitch trend {value} {unit}, {count} data points"
Multi-bucket zones: "{zoneName} zone: {dateFirst} through {dateLast}, pitch trend from {valueFirst} to {valueLast} {unit}, {count} data points"

### Screen-Level

- The outer scroll view has an accessibility label summarizing which modes have data: "Profile showing progress for: {comma-separated mode names}"
- If no data: "Profile. No training data available."

### Dynamic Type

- All fonts use semantic text styles (headline, title2, caption, caption2, body) — no hardcoded sizes
- Year label padding scales proportionally with caption2 text size
- Annotation popover text scales automatically

### Reduce Motion

- No animations exist that need suppressing, but scroll position changes are applied without animation as a precaution

## Empty State

If a training mode has zero records, its card is not rendered at all. If no mode has data, the screen shows an empty scroll view with no cards (the navigation bar and help button are still visible).

## Web Implementation Notes

The iOS implementation uses Apple-specific frameworks (SwiftUI Charts, TipKit). For a web implementation:

- **Charts**: use a capable charting library that supports area marks, line marks, point marks, rule marks, mixed chart types, and horizontal scrolling (e.g., D3.js, Observable Plot, Recharts, or Chart.js with plugins)
- **Frosted glass material**: use CSS `backdrop-filter: blur()` with semi-transparent backgrounds
- **System colors**: map iOS system colors to appropriate CSS custom properties or design tokens that respect light/dark mode
- **Spatial tap gesture**: use click/tap handlers on the chart area with coordinate-to-index resolution
- **Horizontal scrolling**: CSS `overflow-x: auto` on the chart container, or a virtual scroll implementation
- **Increased contrast**: detect via `@media (prefers-contrast: more)` CSS media query
- **Reduce motion**: detect via `@media (prefers-reduced-motion: reduce)`
- **Dynamic Type**: use relative font units (`rem`, `em`) and respect user font-size preferences
- **Session bridge computation**: this is a data-layer concern, compute the weighted average bridge point before passing data to the chart renderer
