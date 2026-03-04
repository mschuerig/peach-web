# Story 3.1: Profile View with Summary Statistics

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to view my detection threshold statistics and pitch matching accuracy,
so that I can understand my pitch discrimination ability in concrete numbers.

## Acceptance Criteria

1. **AC1 — Profile view replaces stub (FR16):** Given I navigate to `/profile`, when the Profile view loads, then it replaces the placeholder from Epic 1 with the full profile view and I see a back navigation link to the Start Page.

2. **AC2 — Comparison training statistics (FR17):** Given comparison training data exists, when I view summary statistics, then I see the overall mean detection threshold in cents, the standard deviation, and a trend indicator (improving/stable/declining).

3. **AC3 — Trend indicator threshold:** Given fewer than 20 comparison records exist, when I view the trend indicator, then the trend indicator is hidden (insufficient data).

4. **AC4 — Pitch matching statistics (FR18):** Given pitch matching data exists, when I view pitch matching statistics, then I see the mean absolute error in cents, the standard deviation, and the sample count.

5. **AC5 — No pitch matching data:** Given no pitch matching data exists, when I view the Profile view, then the pitch matching statistics section is hidden or shows dashes.

6. **AC6 — Cold start (no data at all):** Given no training data exists, when I view the Profile view, then statistics show dashes ("—") instead of numbers, the trend indicator is hidden, and text reads "Start training to build your profile."

7. **AC7 — Semantic HTML and accessibility:** Given the Profile view, when I inspect the HTML, then statistics use semantic HTML with appropriate headings and the view is keyboard-navigable.

## Tasks / Subtasks

- [x] Task 1: Add `matching_count()` getter to PerceptualProfile (AC: 4)
  - [x] 1.1 Add `pub fn matching_count(&self) -> u32` to `PerceptualProfile` in `domain/src/profile.rs` — returns the private `matching_count` field
  - [x] 1.2 Add unit test `test_matching_count_returns_sample_count` verifying count increments after `update_matching()` calls

- [x] Task 2: Replace ProfileView stub with statistics layout (AC: 1,6,7)
  - [x] 2.1 In `web/src/components/profile_view.rs`, extract contexts: `PerceptualProfile`, `TrendAnalyzer`, `is_profile_loaded` signal
  - [x] 2.2 Show loading state while `is_profile_loaded` is false
  - [x] 2.3 Add page heading "Profile" with back navigation link to `/`
  - [x] 2.4 Add cold-start message "Start training to build your profile." when `overall_mean()` returns `None` and `matching_mean()` returns `None`
  - [x] 2.5 Use semantic HTML: `<section>` with `<h2>` headings for each statistics group, `<dl>`/`<dt>`/`<dd>` for stat key-value pairs

- [x] Task 3: Comparison training statistics section (AC: 2,3,6)
  - [x] 3.1 Add `<section>` with heading "Comparison Training"
  - [x] 3.2 Display mean detection threshold: `profile.overall_mean()` formatted to 1 decimal place with " cents" suffix, or "—" if `None`
  - [x] 3.3 Display standard deviation: `profile.overall_std_dev()` formatted to 1 decimal place with " cents" suffix, or "—" if `None`
  - [x] 3.4 Display trend indicator: `trend_analyzer.trend()` as text "Improving" / "Stable" / "Declining", hidden if `None` (< 20 records)

- [x] Task 4: Pitch matching statistics section (AC: 4,5)
  - [x] 4.1 Add `<section>` with heading "Pitch Matching"
  - [x] 4.2 Display mean absolute error: `profile.matching_mean()` formatted to 1 decimal place with " cents" suffix, or "—" if `None`
  - [x] 4.3 Display standard deviation: `profile.matching_std_dev()` formatted to 1 decimal place with " cents" suffix, or "—" if `None`
  - [x] 4.4 Display sample count: `profile.matching_count()`, or "—" if 0
  - [x] 4.5 Hide entire section OR show all dashes when no pitch matching data exists (`matching_mean()` is `None`)

- [x] Task 5: Tailwind styling and dark mode (AC: 1,7)
  - [x] 5.1 Follow existing Tailwind patterns from SettingsView: `dark:` variants on all colors, 44px min touch targets on links, focus rings on interactive elements
  - [x] 5.2 Use consistent spacing: `py-12` for page padding, `gap-6`/`gap-8` between sections
  - [x] 5.3 Style stat values with emphasis (e.g., `text-2xl font-bold`) and labels with muted text (`text-sm text-gray-500 dark:text-gray-400`)
  - [x] 5.4 Style trend indicator with contextual color: green for Improving, gray for Stable, red/amber for Declining

- [x] Task 6: Verify and validate (AC: all)
  - [x] 6.1 `cargo clippy -p domain` — zero warnings
  - [x] 6.2 `cargo clippy -p web` — zero warnings
  - [x] 6.3 `cargo test -p domain` — all tests pass
  - [x] 6.4 `trunk build` — successful WASM compilation
  - [x] 6.5 Manual browser smoke test: profile with data shows stats, empty profile shows dashes + cold-start message, trend hidden with < 20 records

## Dev Notes

### Core Approach: Replace ProfileView Stub + Add matching_count() Getter

This story replaces the existing `ProfileView` placeholder component (`web/src/components/profile_view.rs`) with a full statistics display. The domain crate already has all necessary statistics methods on `PerceptualProfile` and `TrendAnalyzer` — the only domain change is adding a `matching_count()` getter since the field is private.

**Scope boundary:** This story is summary statistics ONLY. The piano keyboard visualization is Story 3.2. The start page profile preview is Story 3.3. Do NOT implement Canvas/SVG rendering or any visualization component in this story.

### Existing Infrastructure (DO NOT recreate)

All statistics computation already exists in the domain crate:

| Method | Location | Returns | Purpose |
|---|---|---|---|
| `PerceptualProfile::overall_mean()` | `domain/src/profile.rs` | `Option<f64>` | Average detection threshold across all trained notes |
| `PerceptualProfile::overall_std_dev()` | `domain/src/profile.rs` | `Option<f64>` | Std dev of trained note means (None if < 2 notes) |
| `PerceptualProfile::matching_mean()` | `domain/src/profile.rs` | `Option<f64>` | Mean absolute pitch matching error |
| `PerceptualProfile::matching_std_dev()` | `domain/src/profile.rs` | `Option<f64>` | Std dev of matching errors (None if < 2 samples) |
| `TrendAnalyzer::trend()` | `domain/src/trend.rs` | `Option<Trend>` | Improving/Stable/Declining (None if < 20 data points) |
| `Trend` enum | `domain/src/trend.rs` | `Improving \| Stable \| Declining` | Trend variants |

**Missing:** `PerceptualProfile` has no public getter for `matching_count` (private `u32` field). Add `pub fn matching_count(&self) -> u32` to expose the pitch matching sample count (AC4 requires displaying it).

### Context Wiring Pattern (from `app.rs`)

All required state is provided via `provide_context` in `App` (lines 32-37 of `web/src/app.rs`):

| Context Type | How to Access | What to Read |
|---|---|---|
| `SendWrapper<Rc<RefCell<PerceptualProfile>>>` | `use_context::<SendWrapper<Rc<RefCell<PerceptualProfile>>>>()` | `.borrow()` then call stats methods |
| `SendWrapper<Rc<RefCell<TrendAnalyzer>>>` | `use_context::<SendWrapper<Rc<RefCell<TrendAnalyzer>>>>()` | `.borrow().trend()` |
| `RwSignal<bool>` (is_profile_loaded) | `use_context::<RwSignal<bool>>()` | `.get()` — true when hydration complete |

**Note:** `SendWrapper` dereferences to the inner type via `Deref`, so `profile.borrow()` works directly. Follow the same context extraction pattern as `settings_view.rs` and `comparison_view.rs`.

**Important:** This view only READS data — use `.borrow()` (immutable), never `.borrow_mut()`. Do NOT write to domain signals from components.

### Profile View Component Structure

```rust
#[component]
pub fn ProfileView() -> impl IntoView {
    let profile = use_context::<SendWrapper<Rc<RefCell<PerceptualProfile>>>>()
        .expect("PerceptualProfile context");
    let trend_analyzer = use_context::<SendWrapper<Rc<RefCell<TrendAnalyzer>>>>()
        .expect("TrendAnalyzer context");
    let is_profile_loaded = use_context::<RwSignal<bool>>()
        .expect("is_profile_loaded context");

    view! {
        <div class="py-12">
            <h1 ...>"Profile"</h1>
            <Show when=move || is_profile_loaded.get() fallback=|| view! { /* loading */ }>
                {move || {
                    let prof = profile.borrow();
                    let trend = trend_analyzer.borrow();
                    // Determine state: cold-start vs has-data
                    // Render comparison stats section
                    // Render pitch matching stats section
                }}
            </Show>
            <A href="/">..."Back to Start"</A>
        </div>
    }
}
```

### Empty State Logic

| Condition | Display |
|---|---|
| `overall_mean()` is `None` AND `matching_mean()` is `None` | Cold start: "Start training to build your profile." All stats show "—" |
| `overall_mean()` is `Some` | Show comparison stats with values |
| `overall_std_dev()` is `None` (only 1 trained note) | Std dev shows "—" |
| `trend_analyzer.trend()` is `None` (< 20 records) | Trend indicator hidden |
| `matching_mean()` is `None` | Pitch matching section shows dashes or is hidden |

### Formatting

- All cent values: 1 decimal place (e.g., "12.4 cents") using `format!("{:.1}", value)`
- Trend text: "Improving" / "Stable" / "Declining" — plain text, no emoji
- Dashes: use em dash "—" (Unicode `\u{2014}`) for missing values
- Sample count: integer, no decimal (e.g., "47")

### Tailwind Styling Patterns

Follow existing patterns from `settings_view.rs`:

```
// Page container
class="py-12"

// Page heading
class="text-2xl font-bold dark:text-white"

// Back link (matches existing pattern from current stub)
class="mt-4 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600
       hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400
       focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"

// Section headings
class="text-lg font-semibold dark:text-white"

// Stat label (dt)
class="text-sm text-gray-500 dark:text-gray-400"

// Stat value (dd)
class="text-2xl font-bold dark:text-white"

// Trend indicator colors
Improving: class="text-green-600 dark:text-green-400"
Stable:    class="text-gray-600 dark:text-gray-400"
Declining: class="text-amber-600 dark:text-amber-400"

// Cold start message
class="text-gray-500 dark:text-gray-400"
```

### Accessibility Requirements

- Use `<section>` with `aria-labelledby` pointing to each section's `<h2>` id
- Use `<dl>`, `<dt>`, `<dd>` for statistics (semantic description list)
- All text has sufficient contrast in both light and dark modes
- Back link is keyboard-focusable with visible focus ring
- No custom interactive elements — just semantic HTML and links
- Trend indicator: include `aria-label` with full context (e.g., "Trend: Improving")

### What NOT to Implement (Separate Stories)

- **Piano keyboard visualization** — Story 3.2 (`profile_visualization.rs`). Do NOT create Canvas/SVG rendering.
- **Profile preview on start page** — Story 3.3 (`profile_preview.rs`). Do NOT touch `start_page.rs`.
- **Weak spots display** — Not in any AC for 3.1. `weak_spots()` exists but is not needed here.
- **Timeline/history display** — ThresholdTimeline is available but not required by 3.1 ACs.
- **Gamification** — No scores, badges, streaks, or celebratory language. Factual presentation only.

### UX Requirements (from UX Spec)

- **"Disappearing UI" philosophy:** Minimal, factual presentation. No decorative elements.
- **Calm focus:** Statistics presented as calm, factual numbers. No celebratory animations or judgment language.
- **Dark mode:** All Tailwind classes include `dark:` variants.
- **Touch targets:** Back link minimum 44x44px (`min-h-11 min-w-11`).
- **Responsive:** Single-column layout works at all viewport sizes. No breakpoints needed for stats display.

### Project Structure Notes

- Modified: `domain/src/profile.rs` — Add `matching_count()` getter method + unit test
- Modified: `web/src/components/profile_view.rs` — Replace stub with full statistics display
- No new files needed
- No routing changes needed (`/profile` route already exists in `app.rs` line 127)
- No changes to `components/mod.rs` (ProfileView already exported)
- New imports in `profile_view.rs`: `SendWrapper`, `Rc`, `RefCell`, `domain::PerceptualProfile`, `domain::TrendAnalyzer`, `domain::Trend`

### References

- [Source: docs/planning-artifacts/epics.md#Story 3.1] — Full acceptance criteria with BDD scenarios
- [Source: docs/planning-artifacts/architecture.md#Component Architecture] — ProfileView component, routing at `/profile`
- [Source: docs/planning-artifacts/architecture.md#Crate Boundary] — Domain crate pure Rust, web crate has Leptos UI
- [Source: docs/planning-artifacts/ux-design-specification.md#Profile View] — Summary statistics, empty states, back navigation
- [Source: docs/planning-artifacts/ux-design-specification.md#Empty States] — Dashes for no data, "Start training to build your profile"
- [Source: docs/project-context.md] — Coding conventions, anti-patterns, type naming rules
- [Source: domain/src/profile.rs] — PerceptualProfile public API: overall_mean(), overall_std_dev(), matching_mean(), matching_std_dev(), note_stats()
- [Source: domain/src/trend.rs] — TrendAnalyzer::trend() returns Option<Trend>, requires 20+ data points
- [Source: web/src/app.rs#provide_context] — Context providers for profile, trend_analyzer, is_profile_loaded (lines 32-37)
- [Source: web/src/components/profile_view.rs] — Current stub to replace
- [Source: web/src/components/settings_view.rs] — Reference patterns for context extraction, Tailwind styling, dark mode

### Previous Story Intelligence (from Stories 2.1-2.3)

**Patterns established:**
- Context extraction: `use_context::<SendWrapper<Rc<RefCell<T>>>>().expect("...")` — same pattern needed here
- Tailwind dark mode: every color utility needs `dark:` variant, ring offsets need `dark:ring-offset-gray-900`
- Touch targets: `min-h-11` (44px) on all interactive elements
- `<fieldset>`/`<legend>` for grouped sections — consider for stats groups
- Focus rings: `focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2`

**Code review learnings from Epic 2:**
- Use semantic HTML grouping elements (`<fieldset>`, `<section>`)
- Add `aria-labelledby` for accessibility on grouped content
- Handle all `Option` types explicitly — never unwrap, always show fallback UI
- Dark mode ring offset: `dark:ring-offset-gray-900`

### Git Intelligence

Recent commit pattern: "Implement story X.Y ..." → "Apply code review fixes for story X.Y and mark as done". Follow same pattern.

Files to modify:
- `domain/src/profile.rs` — Add `matching_count()` getter + test
- `web/src/components/profile_view.rs` — Replace stub with stats display

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Leptos 0.8.x `Show` component requires `Send + Sync` for children closures; `Rc<RefCell<T>>` doesn't satisfy this. Solved by using direct `{move || { ... }}` closure with `.into_any()` for branch unification, and cloning `SendWrapper` (not unwrapping to `Rc`) to preserve `Send`.
- `Signal::derive` also requires `Send + Sync` — cannot be used with `Rc<RefCell<T>>`. Precomputing values into owned types before the `view!` macro is the correct pattern.

### Completion Notes List

- Added `pub fn matching_count(&self) -> u32` getter to `PerceptualProfile` exposing the private `matching_count` field
- Added `test_matching_count_returns_sample_count` unit test verifying count increments across multiple `update_matching()` calls
- Replaced `ProfileView` stub with full statistics display: comparison training (mean, std dev, trend) and pitch matching (mean, std dev, count)
- Implemented cold-start state with "Start training to build your profile." message and em dashes for all stats
- Loading state shown while `is_profile_loaded` signal is false
- Trend indicator hidden when `TrendAnalyzer::trend()` returns `None` (< 20 records)
- Pitch matching section shows dashes when no data exists
- Semantic HTML: `<section>` with `aria-labelledby`, `<dl>`/`<dt>`/`<dd>` for statistics
- Trend `aria-label` provides screen reader context (e.g., "Trend: Improving")
- Tailwind dark mode variants on all colors, 44px touch targets on back link, focus rings
- Trend indicator uses contextual colors: green/gray/amber for Improving/Stable/Declining
- Helper function `format_cents()` reduces repetition for Option<f64> → String formatting
- All values extracted from `RefCell` borrows before `view!` macro to avoid borrow conflicts

### Change Log

- 2026-03-04: Implemented story 3.1 — Profile View with Summary Statistics
- 2026-03-04: Code review fixes — removed redundant "Trend:" prefix from aria-label (M1), hid stats sections during cold-start to show only the message (M2)

### File List

- `domain/src/profile.rs` — Added `matching_count()` getter method + unit test
- `web/src/components/profile_view.rs` — Replaced stub with full statistics display
- `docs/implementation-artifacts/sprint-status.yaml` — Updated status to review
- `docs/implementation-artifacts/3-1-profile-view-with-summary-statistics.md` — Updated tasks, status, dev agent record
