# Story 12.2: Profile Screen Layout & Chart Cards

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to see a profile screen with a card for each training mode I've used, showing my current EWMA and trend at a glance,
So that I can quickly see how I'm doing in each mode.

## Acceptance Criteria

1. **Profile screen with mode cards** — Given I navigate to `/profile` when training data exists for at least one mode, then I see a scrollable list of chart cards, one per mode with data, in TrainingMode enum order (unison comparison, interval comparison, unison matching, interval matching). Modes with zero records show no card. 16px gap between cards.

2. **Empty state** — Given the profile screen when no mode has any training data, then I see the navigation bar with "Profile" title and help button, and the scroll area is empty (no cards).

3. **Card styling** — Given a chart card for a mode, when it renders, then the card has a frosted glass background (`backdrop-filter: blur()` with semi-transparent background) and 12px corner radius. Internal padding follows the existing app's system default pattern.

4. **Headline row with EWMA** — Given a chart card headline row when it renders with sufficient data, then: left side shows the mode display name in headline font; right side shows the EWMA value in title 2 bold (e.g. "25.3"), the stddev of the most recent display bucket in caption secondary color (e.g. "±4.2"), and a trend arrow icon colored by direction: green down-right for improving, gray right for stable, orange up-right for declining.

5. **Headline with insufficient data** — Given a mode with fewer than 2 records, when the headline renders, then the EWMA value is shown (computed from the single record) and no trend arrow is displayed.

6. **Locale-aware number formatting** — Given the EWMA value "25.3" in an English locale, it shows "25.3" with a decimal point. In a German locale, it shows "25,3" with a comma. Use `Intl.NumberFormat` with `minimumFractionDigits: 1, maximumFractionDigits: 1`.

7. **Chart placeholder** — Given a chart card when the chart area renders (before Story 12.3), then a placeholder area is shown at the correct height: 180px on mobile, 240px on tablet/desktop.

8. **Scroll view accessibility** — Given the profile screen scroll view, it has an `aria-label` summarizing which modes have data (e.g. "Profile showing progress for: Hear & Compare – Single Notes, Tune & Match – Single Notes").

9. **Card accessibility** — Given a chart card, it has `role` and `aria-label` "Progress chart for {mode display name}" and `aria-valuenow` or equivalent conveying "Current: {EWMA} cents, trend: {trend label}".

10. **Semantic text sizes** — Given the profile screen on any viewport, all text uses semantic sizes (rem/em) with no hardcoded pixel font sizes.

## Tasks / Subtasks

- [x] Task 1: Refactor ProfileView to frosted-glass card layout (AC: 1, 2, 3)
  - [x] 1.1 Replace current `bg-gray-100 dark:bg-gray-800 rounded-xl` card style with frosted glass: `backdrop-blur-md bg-white/60 dark:bg-gray-900/60 rounded-xl` (CSS `backdrop-filter: blur(12px)`)
  - [x] 1.2 Ensure 16px gap between cards (`gap-4` or `space-y-4` in Tailwind)
  - [x] 1.3 Verify empty state: nav bar + help button visible, no cards when zero modes have data
  - [x] 1.4 Verify card ordering matches `TrainingMode::ALL` (unison comparison first, interval matching last)

- [x] Task 2: Implement locale-aware number formatting (AC: 6)
  - [x] 2.1 Create a `format_decimal_1` helper in `web/src/` (e.g. in a utils module or inline in progress_card.rs) that calls `Intl.NumberFormat` via `js_sys`/`web_sys` with `{minimumFractionDigits: 1, maximumFractionDigits: 1}` using the browser's current locale
  - [x] 2.2 Use this helper for EWMA and stddev display values in the headline

- [x] Task 3: Update ProgressCard headline row (AC: 4, 5, 10)
  - [x] 3.1 Left side: mode display name using `move_tr!()` with the i18n key from `mode.config().display_name`; use headline-appropriate Tailwind class (`text-base font-semibold` or similar with `rem` units)
  - [x] 3.2 Right side: EWMA value formatted via `format_decimal_1`, styled as title 2 bold (`text-xl font-bold`)
  - [x] 3.3 Right side: stddev of most recent bucket via `timeline.latest_bucket_stddev(mode)`, formatted as "±{value}" in caption secondary color (`text-xs text-gray-500 dark:text-gray-400`)
  - [x] 3.4 Right side: trend arrow — use existing unicode arrows (↘ improving green, → stable gray, ↗ declining orange) with color classes; hide arrow entirely when `timeline.trend(mode)` returns `None`
  - [x] 3.5 Ensure all font sizes use rem/em, no hardcoded px for text

- [x] Task 4: Add chart placeholder area (AC: 7)
  - [x] 4.1 Below the headline row, add a placeholder `<div>` with responsive height: `h-[180px] md:h-[240px]`
  - [x] 4.2 Style placeholder with subtle background to indicate chart area (e.g. `bg-gray-200/30 dark:bg-gray-700/30 rounded-lg`)
  - [x] 4.3 12px spacing between headline row and chart area (`mt-3` or equivalent)

- [x] Task 5: Add accessibility attributes (AC: 8, 9)
  - [x] 5.1 Scroll view container: compute `aria-label` dynamically based on which modes have `Active` state — format: "Profile showing progress for: {comma-separated mode display names}"
  - [x] 5.2 Each card: `role="group"` and `aria-label="Progress chart for {mode display name}"`
  - [x] 5.3 Each card: `aria-valuenow` or `aria-description` conveying "Current: {EWMA} cents, trend: {trend label}"
  - [x] 5.4 Empty state: aria-label "Profile. No training data available."

- [x] Task 6: Increased contrast support (AC: 3, derived from PNFR3)
  - [x] 6.1 Add CSS for `@media (prefers-contrast: more)` that doubles frosted glass background opacity (e.g. `bg-white/60` → `bg-white/90` under high contrast)
  - [x] 6.2 Verify card borders/outlines remain visible under high contrast

- [x] Task 7: Verify and test (AC: 1-10)
  - [x] 7.1 `cargo clippy --workspace` — zero warnings
  - [x] 7.2 `cargo test -p domain` — all tests pass (no domain changes expected, but verify)
  - [ ] 7.3 UNCHECKED — Manual browser test: verify frosted glass rendering, card layout, responsive height, locale formatting (deferred to user — agent cannot verify in browser)
  - [ ] 7.4 UNCHECKED — Manual browser test: verify screen reader announces correct aria-labels (deferred to user)

## Dev Notes

### What Already Exists (DO NOT REINVENT)

The following are **already implemented and working** — extend, don't recreate:

- **`web/src/components/profile_view.rs`** — `ProfileView` component. Iterates `TrainingMode::ALL`, checks `state(mode) == Active`, renders `ProgressCard` per mode. Uses `SendWrapper<Rc<RefCell<ProgressTimeline>>>` from context. Currently uses `space-y-4` for card spacing.
- **`web/src/components/progress_card.rs`** — `ProgressCard` component with headline row (mode name, EWMA, stddev, trend arrow). Currently styled with `rounded-xl bg-gray-100 p-4 dark:bg-gray-800`. Already has unicode arrows (↘/→/↗) with color classes. Already reads `timeline.current_ewma(mode)`, `timeline.latest_bucket_stddev(mode)`, `timeline.trend(mode)`.
- **`web/src/components/progress_chart.rs`** — `ProgressChart` SVG component, already has responsive height `h-[180px] md:h-[240px]`. **Not modified in this story** — Story 12.3 handles chart rendering changes.
- **`web/src/app.rs`** — Profile route at `/profile`, provides `ProgressTimeline` via context with `SendWrapper`, calls `rebuild()` with `start_of_today` during hydration.
- **`web/src/bridge.rs`** — `compute_start_of_today()` helper, `ProgressTimelineObserver` for incremental updates.
- **`domain/src/progress_timeline.rs`** — Full public API: `display_buckets(mode)`, `current_ewma(mode)`, `trend(mode)`, `latest_bucket_stddev(mode)`, `state(mode)`.
- **`domain/src/training_mode.rs`** — `TrainingMode` enum with `ALL` array, `TrainingModeConfig` with `display_name` (i18n key), `unit_label`, `optimal_baseline`, etc.

### What Must Change

**`web/src/components/progress_card.rs` (primary file):**

1. **Replace card background** — Change from `bg-gray-100 dark:bg-gray-800` to frosted glass: `backdrop-blur-md bg-white/60 dark:bg-gray-900/60 border border-white/20 dark:border-gray-700/30`. Keep `rounded-xl` (12px) and `p-4`.

2. **Add locale-aware number formatting** — Current code uses Rust `format!("{:.1}", value)` which always produces a decimal point. Replace with a JS `Intl.NumberFormat` call via `js_sys`/`web_sys` for locale-aware formatting (comma in German, period in English).

3. **Add chart placeholder** — Below headline, add a placeholder div at responsive height `h-[180px] md:h-[240px]` with subtle background. This replaces the current `ProgressChart` rendering in the card — the real chart comes back in Story 12.3.

4. **Enhance accessibility** — Add `role="group"` and computed `aria-label`/`aria-description` to each card.

**`web/src/components/profile_view.rs` (secondary file):**

1. **Add scroll view aria-label** — Compute a dynamic `aria-label` on the outer container listing active modes.
2. **Empty state aria-label** — When no modes are active, set "Profile. No training data available."

### Locale-Aware Number Formatting — Implementation Guide

The `Intl.NumberFormat` API must be called from Rust via `js_sys`/`web_sys`. Pattern:

```rust
use js_sys::{Intl, Object, Reflect};
use wasm_bindgen::JsValue;

fn format_decimal_1(value: f64) -> String {
    let options = Object::new();
    Reflect::set(&options, &"minimumFractionDigits".into(), &1.into()).unwrap();
    Reflect::set(&options, &"maximumFractionDigits".into(), &1.into()).unwrap();
    let formatter = Intl::NumberFormat::new(
        &js_sys::Array::new(), // empty = browser default locale
        &options,
    );
    formatter.format().call1(&formatter, &JsValue::from_f64(value))
        .unwrap()
        .as_string()
        .unwrap_or_else(|| format!("{:.1}", value))
}
```

**Note:** Research the exact `js_sys::Intl` API before implementing — the above is a sketch. The `web_sys` `Intl.NumberFormat` bindings may require specific feature flags. Check existing codebase for any prior `Intl` usage patterns. If `js_sys::Intl::NumberFormat` is not available, use `js_sys::eval` or a small JS snippet via `wasm_bindgen`.

### Frosted Glass CSS

```css
/* Frosted glass card */
.card-frosted {
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    background: rgba(255, 255, 255, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 12px;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
    .card-frosted {
        background: rgba(17, 24, 39, 0.6);
        border: 1px solid rgba(75, 85, 99, 0.3);
    }
}

/* Increased contrast */
@media (prefers-contrast: more) {
    .card-frosted {
        background: rgba(255, 255, 255, 0.9);
        border: 1px solid rgba(0, 0, 0, 0.3);
    }
}
```

Since the project uses Tailwind, prefer Tailwind utilities:
- `backdrop-blur-md bg-white/60 dark:bg-gray-900/60 border border-white/20 dark:border-gray-700/30 rounded-xl`
- For increased contrast, add a CSS rule in `input.css` targeting `@media (prefers-contrast: more)`.

### Accessibility Details

**Scroll view label** — Dynamically built from active modes:
```rust
let active_modes: Vec<String> = TrainingMode::ALL
    .iter()
    .filter(|m| timeline.borrow().state(**m) == TrainingModeState::Active)
    .map(|m| tr!(m.config().display_name))  // resolved i18n name
    .collect();
let aria_label = if active_modes.is_empty() {
    tr!("profile-no-data")  // "Profile. No training data available."
} else {
    format!("{}: {}", tr!("profile-showing-progress"), active_modes.join(", "))
};
```

**Card label** — Per card:
- `role="group"`
- `aria-label` = `"Progress chart for {mode_name}"`
- `aria-description` = `"Current: {ewma} cents, trend: {trend_label}"`

### What NOT to Change

- **Do NOT modify `progress_chart.rs` chart rendering logic** — Story 12.3 handles chart rendering (index-based X-axis, zone backgrounds, layers, etc.)
- **Do NOT modify domain crate** — The data pipeline from Story 12.1 provides everything needed.
- **Do NOT add scrolling behavior to charts** — Story 12.4 handles horizontal scrolling.
- **Do NOT add annotation popovers** — Epic 13 (PFR15).
- **Do NOT add help overlay content** — Epic 13 (PFR17).

### Story 12.1 Learnings (Previous Story Intelligence)

From the 12.1 code review:

1. **`compute_start_of_today()` is `pub(crate)` in `bridge.rs`** — reuse it, don't recreate.
2. **Cross-midnight stale zones** — `add_point()` doesn't reclassify after midnight. Call `rebuild()` to recategorize. Not relevant for this story but good to know.
3. **Dead parameters were cleaned** — `_now` parameter removed from domain API.
4. **`display_buckets()` is the renamed API** (was `buckets()`). Use `timeline.display_buckets(mode)`.
5. **Population stddev** — All stddev is `sqrt(m2/n)`, matches `latest_bucket_stddev()` return.

### Git Intelligence

Recent commits (all in Epic 12 scope):
- `28f3fc4` — Code review fixes: removed dead `_now` param, deduplicated `start_of_today`
- `cc6847b` — Story 12.1 implementation: three-zone bucketing, EWMA pipeline
- Files modified: `domain/src/progress_timeline.rs`, `web/src/app.rs`, `web/src/bridge.rs`, `web/src/components/progress_chart.rs`, `progress_card.rs`, `progress_sparkline.rs`

### Project Structure Notes

- All changes in `web/src/components/` (profile_view.rs, progress_card.rs)
- Possible new utility function in `web/src/` for locale formatting (or inline)
- Possible CSS additions in `input.css` for increased contrast media query
- No new files strictly required — extend existing components
- No domain crate changes

### References

- [Source: docs/planning-artifacts/epics.md#Story 12.2: Profile Screen Layout & Chart Cards] — Full BDD acceptance criteria
- [Source: docs/ios-reference/profile-screen-specification.md] — Authoritative reference for layout, headline, accessibility
- [Source: docs/planning-artifacts/epics.md#Epic 12: Profile Progress Charts] — PFR1-PFR2, PNFR1-PNFR5
- [Source: web/src/components/progress_card.rs] — Existing card implementation to modify
- [Source: web/src/components/profile_view.rs] — Existing profile view to modify
- [Source: domain/src/progress_timeline.rs] — Public API (display_buckets, current_ewma, trend, latest_bucket_stddev, state)
- [Source: domain/src/training_mode.rs] — TrainingMode enum, config, display_name i18n keys
- [Source: docs/project-context.md] — Coding conventions, Leptos patterns, accessibility requirements

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

None — clean implementation with no debugging needed.

### Completion Notes List

- Replaced solid card backgrounds with frosted glass styling using `backdrop-blur-md bg-white/60 dark:bg-gray-900/60` plus subtle border
- Implemented `format_decimal_1()` using `js_sys::Intl::NumberFormat` for locale-aware number formatting (respects browser locale — comma in German, period in English)
- Updated headline row: left side uses `text-base font-semibold` (rem-based), right side keeps `text-xl font-bold` for EWMA, `text-xs` for stddev, existing unicode trend arrows
- Replaced `ProgressChart` SVG component with a chart placeholder div at responsive height `h-[180px] md:h-[240px]` — Story 12.3 will reconnect the real chart
- Added `role="group"`, `aria-label` (localized "Progress chart for {name}"), and `aria-description` (localized "Current: {ewma} cents, trend: {trend}") to each card
- Added dynamic `aria-label` on scroll view container listing active mode names
- Added empty state `aria-label` "Profile. No training data available."
- Added `@media (prefers-contrast: more)` CSS rules in `input.css` for increased background opacity (0.9) in both light and dark modes
- Suppressed dead_code warnings on `progress_chart` module (temporarily unused until Story 12.3)
- Added 4 new i18n keys to both en and de locale files
- `cargo clippy --workspace` — zero warnings; `cargo test -p domain` — all 359 tests pass

### Change Log

- 2026-03-13: Story 12.2 implementation — frosted glass cards, locale formatting, chart placeholder, accessibility, high contrast support

### File List

- web/src/components/progress_card.rs (modified — frosted glass, locale formatting, chart placeholder, accessibility)
- web/src/components/profile_view.rs (modified — scroll view and empty state aria-labels)
- web/src/components/mod.rs (modified — #[allow(dead_code)] on progress_chart module)
- input.css (modified — high contrast media queries for frosted glass cards)
- web/locales/en/main.ftl (modified — 4 new i18n keys)
- web/locales/de/main.ftl (modified — 4 new i18n keys)
- docs/implementation-artifacts/sprint-status.yaml (modified — status update)
