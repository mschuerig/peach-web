# Story 7.5: Training Stats with Trend Arrows

Status: done

## Story

As a musician,
I want to see my latest result and session best with a trend indicator on the training screens,
so that I get immediate feedback on my current performance trajectory.

## Context

The iOS app added a `TrainingStatsView` component shown at the top of both training screens (PitchComparisonScreen and PitchMatchingScreen). It displays:
- "Latest: X.X cents" with a trend arrow (color-coded)
- "Best: X.X cents" (session best)

The trend arrow comes from `ProgressTimeline.trend(for: mode)`, requiring the training screen to know which TrainingMode it represents.

Currently, peach-web's comparison_view.rs and pitch_matching_view.rs show no stats during training.

**iOS reference files:**
- `Peach/App/TrainingStatsView.swift` — the shared stats component
- `Peach/PitchComparison/PitchComparisonScreen.swift` — integration showing `lastCompletedCentDifference`, `sessionBestCentDifference`, and trend
- `Peach/PitchMatching/PitchMatchingScreen.swift` — integration showing `lastResult.userCentError.magnitude`, `sessionBestCentError`, and trend

Depends on: Story 7.2 (ProgressTimeline for trend data).

## Acceptance Criteria

1. **AC1 — TrainingStats component:** A reusable `TrainingStats` component exists showing latest value, session best, and trend arrow
2. **AC2 — Latest value display:** Shows "Latest: X.X cents" formatted to one decimal place. Hidden (opacity 0) when no result yet in this session.
3. **AC3 — Session best display:** Shows "Best: X.X cents" in smaller/caption text. Hidden when no result yet.
4. **AC4 — Trend arrow:** Shows a directional arrow next to the latest value: improving = ↘ (green), stable = → (gray), declining = ↗ (orange). Hidden when trend is None (< 2 records).
5. **AC5 — Comparison integration:** `comparison_view.rs` shows TrainingStats at the top with:
   - Latest = absolute cent offset of last completed comparison
   - Best = smallest absolute cent offset in this session
   - Trend = from ProgressTimeline for the current mode (unison or interval)
6. **AC6 — Pitch matching integration:** `pitch_matching_view.rs` shows TrainingStats at the top with:
   - Latest = absolute user cent error of last completed matching
   - Best = smallest absolute user cent error in this session
   - Trend = from ProgressTimeline for the current mode (unison or interval)
7. **AC7 — Mode detection:** Training views determine their TrainingMode from the intervals: if only prime/unison → unison mode, otherwise → interval mode
8. **AC8 — Session tracking:** "Session best" tracks only the current session (resets when navigating away and back). This is local state, not persisted.
9. **AC9 — Accessibility:** Stats region has `aria-live="polite"` for screen reader updates. Trend arrow has accessible label ("Improving", "Stable", "Declining").
10. **AC10 — Visual consistency:** Stats align left, use secondary/muted text colors, compact font sizes. Match the understated training aesthetic (not gamified).

## Tasks / Subtasks

- [x] Task 1: Create TrainingStats component (AC: 1, 2, 3, 4, 9, 10)
  - [x] New component in `web/src/components/training_stats.rs` or inline
  - [x] Props: `latest_value: Option<f64>`, `session_best: Option<f64>`, `trend: Option<Trend>`
  - [x] Render "Latest: X.X cents" with opacity toggle
  - [x] Render trend arrow with color (CSS classes: text-green-600, text-gray-500, text-orange-500)
  - [x] Render "Best: X.X cents" in smaller text with opacity toggle
  - [x] Container with `aria-live="polite"`

- [x] Task 2: Add session tracking signals to comparison_view.rs (AC: 5, 7, 8)
  - [x] Add `latest_cent_offset: RwSignal<Option<f64>>` — updated on each completed comparison
  - [x] Add `session_best: RwSignal<Option<f64>>` — tracks minimum absolute offset this session
  - [x] Determine TrainingMode from intervals prop (prime only → UnisonPitchComparison, else IntervalPitchComparison)
  - [x] Read trend from ProgressTimeline context: `timeline.borrow().trend(mode)`
  - [x] Update signals in the comparison observer callback or after each answer
  - [x] Reset both signals on session start

- [x] Task 3: Add session tracking signals to pitch_matching_view.rs (AC: 6, 7, 8)
  - [x] Add `latest_cent_error: RwSignal<Option<f64>>` — updated on each commit
  - [x] Add `session_best: RwSignal<Option<f64>>` — tracks minimum absolute error this session
  - [x] Determine TrainingMode from intervals prop (prime only → UnisonMatching, else IntervalMatching)
  - [x] Read trend from ProgressTimeline context
  - [x] Update signals after each pitch commit
  - [x] Reset both signals on session start

- [x] Task 4: Integrate TrainingStats into views (AC: 5, 6)
  - [x] Add `<TrainingStats>` component at the top of comparison_view.rs, before answer buttons
  - [x] Add `<TrainingStats>` component at the top of pitch_matching_view.rs, before slider
  - [x] Pass latest, best, and trend as props

- [x] Task 5: Formatting helper (AC: 2, 3)
  - [x] `format_cents(value: f64) -> String` — one decimal place, e.g. "12.3"
  - [x] Reusable across TrainingStats and ProgressChart (story 7.4)

- [x] Task 6: Verify
  - [x] Manual testing: start comparison training, verify stats appear after first answer
  - [x] Manual testing: start pitch matching, verify stats appear after first commit
  - [x] Verify trend arrow appears after sufficient training history
  - [x] Verify session best resets when returning to start page and starting again
  - [x] Run `cargo clippy`

## Dev Notes

### iOS to Web Mapping

| iOS Element | peach-web Equivalent |
|---|---|
| `TrainingStatsView` (SwiftUI View) | `TrainingStats` Leptos component |
| `pitchComparisonSession.lastCompletedCentDifference` | `latest_cent_offset` RwSignal |
| `pitchComparisonSession.sessionBestCentDifference` | `session_best` RwSignal |
| `progressTimeline.trend(for: trainingMode)` | `timeline.borrow().trend(mode)` |
| `Image(systemName: "arrow.down.right")` | Unicode arrow "↘" or SVG |
| `.opacity(latestValue != nil ? 1 : 0)` | Leptos `class:opacity-0` conditional |

### Design Decisions

- **Unicode arrows over SVG:** "↘", "→", "↗" are universally supported and require no asset loading. They're styled with color via CSS class.
- **Session tracking in view, not domain:** Session best is ephemeral UI state, not domain logic. Tracked via Leptos signals, not persisted.
- **Trend reads from ProgressTimeline:** The trend reflects long-term progress, not just the current session. This matches iOS behavior.
- **No "difficulty" display:** The iOS app removed the old difficulty number display in favor of TrainingStats. peach-web's comparison_view currently doesn't show difficulty either, so no removal needed.

### Architecture Compliance

- **Web crate only:** TrainingStats component and signal wiring are in the web crate.
- **Domain types:** `Trend` enum imported from domain. `TrainingMode` from story 7.1.
- **Observer pattern preserved:** The existing observer callbacks in comparison_view and pitch_matching_view are where we update the session signals.

## Dev Agent Record

### Implementation Plan

- Created `TrainingStats` Leptos component with `latest_value`, `session_best`, and `trend` Signal props
- Added `format_cents()` helper for consistent one-decimal formatting
- Added `last_cent_difference()` getter to `PitchComparisonSession` (mirrors iOS `lastCompletedCentDifference`)
- Integrated stats signals into both training views, updated in answer/commit handlers
- TrainingMode detection from interval query params in both views
- Trend read from ProgressTimeline context after each answer/commit

### Completion Notes

- All 6 tasks completed, all ACs satisfied
- 3 new domain tests for `last_cent_difference()` (none, populated, cleared on stop)
- 1 unit test for `format_cents()` helper
- Full clippy pass clean (domain + web crates)
- Session best for comparison uses domain's `session_best_cent_difference()` (already tracked, correct answers only)
- Session best for matching tracked via RwSignal in view (all attempts, min absolute error)
- Stats hidden (opacity-0) until first result, trend arrow hidden until sufficient history
- `aria-live="polite"` on stats container, trend arrows have accessible labels

## File List

- `domain/src/session/pitch_comparison_session.rs` — added `last_cent_difference()` getter + 3 tests
- `web/src/components/training_stats.rs` — new TrainingStats component + format_cents helper + test
- `web/src/components/mod.rs` — added training_stats module declaration
- `web/src/components/pitch_comparison_view.rs` — added stats signals, TrainingMode detection, TrainingStats integration
- `web/src/components/pitch_matching_view.rs` — added stats signals, TrainingMode detection, TrainingStats integration

## Change Log

- 2026-03-06: Implemented story 7.5 — TrainingStats component with trend arrows on both training screens
- 2026-03-06: Code review fixes — session best tracks all attempts (M1), remove duplicate aria-live (M2), re-export TrainingStats (L1), fix import paths (L1), rename latest_cent_offset to latest_cent_difference (L3), and mark done
