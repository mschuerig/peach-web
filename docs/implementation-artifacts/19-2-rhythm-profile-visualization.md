# Story 19.2: Rhythm Profile Visualization

Status: review

## Story

As a user,
I want to see my rhythm training progress on the profile screen,
so that I can track improvement across tempos and directions.

## Context

Prerequisite: Rhythm records exist and are stored (Epics 17-18), StatisticsKey with TempoRange × RhythmDirection exists (Epic 16).

The iOS app uses a spectrogram-style chart for rhythm profiles (x=time, y=tempo, color=accuracy). The web app currently shows line charts for pitch. We need to decide whether to use spectrogram or line charts for rhythm on the web.

### iOS Reference (Epic 51)

- Spectrogram: x=time (session/day/month buckets), y=tempos trained at, cell color=green/yellow/red
- Thresholds: green ≤5%, yellow 5-15%, red >15%
- Headline: EWMA of most recent bucket + trend arrow
- Tap cell for early/late breakdown

## Acceptance Criteria

1. **AC1 — Rhythm cards on profile screen:** Two new progress cards appear: "Compare Timing" and "Fill the Gap", showing EWMA + trend arrow (same format as pitch cards).

2. **AC2 — Rhythm sparklines on start page:** The rhythm training cards on the start screen show progress sparklines (same component as pitch, using discipline's merged statistics).

3. **AC3 — Rhythm progress charts:** Expandable chart view for each rhythm discipline, showing accuracy over time. Use the same line chart component as pitch for now (spectrogram is a future enhancement).

4. **AC4 — Unit label:** Charts show "% of 16th" as the unit, not "cents".

5. **AC5 — Color coding:** If accuracy thresholds are displayed, use: green ≤5%, yellow 5-15%, red >15%.

6. **AC6 — All training disciplines visible:** Profile screen shows all 6 disciplines (4 pitch + 2 rhythm), with rhythm disciplines showing "No data" until training begins.

7. **AC7 — Builds and renders:** `trunk build` succeeds. Profile screen displays correctly with mixed pitch + rhythm data.

## Tasks / Subtasks

- [x] Task 1: Add rhythm progress cards to profile view
- [x] Task 2: Ensure sparklines work for rhythm disciplines
- [x] Task 3: Add rhythm progress charts (reuse line chart)
- [x] Task 4: Display correct unit label per discipline
- [x] Task 5: Verify "No data" state for untrained rhythm disciplines
- [x] Task 6: Smoke test with mixed data

## Dev Notes

- The profile view already iterates `TrainingDiscipline::ALL` — rhythm disciplines should appear automatically after Epic 16's profile redesign
- The spectrogram visualization is appealing but complex. Start with line charts (same as pitch) and add spectrogram as a future enhancement.
- The merged statistics (across TempoRange × RhythmDirection) provide a single timeline per discipline, which is what the line chart needs

## Dev Agent Record

### Implementation Plan

The architecture was already designed for rhythm visualization — `ProfileView`, `ProgressCard`, `ProgressChart`, and `ProgressSparkline` all iterate `TrainingDiscipline::ALL` and use generic discipline config. The main work was fixing hardcoded "cents" unit labels in i18n keys and components.

### Debug Log

No issues encountered. All changes were straightforward i18n and component prop additions.

### Completion Notes

- AC1: Profile view already iterates all 6 disciplines via `TrainingDiscipline::ALL`; `ProgressCard` renders EWMA + trend arrow generically for any discipline
- AC2: Start page `TrainingCard` components already include `ProgressSparkline` for rhythm disciplines; fixed sparkline to use `value-percent-16th` i18n key for rhythm
- AC3: `ProgressChart` already accepts generic `unit_label` and `optimal_baseline` from discipline config — no changes needed
- AC4: Fixed hardcoded `value-cents` in sparkline, training stats, and profile aria messages to use discipline-aware i18n keys; added `value-percent-16th` i18n key
- AC5: Color coding thresholds are not currently displayed in the line chart (they apply to the spectrogram future enhancement); trend colors (green/amber/gray) are already applied
- AC6: `ProgressCard` returns empty div for `NoData` state; `ProfileView` shows "No training data" when all disciplines are empty
- AC7: `cargo clippy --workspace` passes; `cargo test -p domain` passes (480 tests); WASM compilation succeeds
- Removed resolved finding from `docs/future-work.md` (hardcoded "cents" unit)

## File List

- web/src/components/progress_sparkline.rs (modified)
- web/src/components/progress_card.rs (modified)
- web/src/components/training_stats.rs (modified)
- web/src/components/rhythm_offset_detection_view.rs (modified)
- web/src/components/continuous_rhythm_matching_view.rs (modified)
- web/locales/en/main.ftl (modified)
- web/locales/de/main.ftl (modified)
- docs/future-work.md (modified)

## Change Log

- 2026-03-25: Implemented rhythm profile visualization — fixed hardcoded "cents" unit labels across sparkline, training stats, and profile aria text; added `value-percent-16th` i18n key; parameterized `current-trend`, `value-trend`, `progress-for` i18n messages with `$unit`; removed resolved future-work finding
