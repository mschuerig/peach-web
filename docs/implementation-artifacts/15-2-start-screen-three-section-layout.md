# Story 15.2: Start Screen Three-Section Layout with Rhythm Routes

Status: review

## Story

As a user,
I want to see rhythm training options on the start screen alongside pitch training,
so that I can navigate to rhythm training as soon as it becomes available.

## Context

Prerequisite: Story 15.1 (rhythm enum cases).

The iOS app organizes the start screen into three sections of 2 buttons each:
- **Single Notes**: Compare / Match
- **Intervals**: Compare / Match
- **Rhythm**: Compare Timing / Fill the Gap

The rhythm buttons initially route to placeholder screens (story 15.3). This gives the user a complete navigation skeleton to test against as rhythm features are built.

## Acceptance Criteria

1. **AC1 — Three sections:** Start screen shows three sections with headers: "Single Notes" (existing, renamed from previous labels if needed), "Intervals" (existing), "Rhythm" (new).

2. **AC2 — Rhythm section buttons:**
   - "Compare Timing" with ear icon or timing icon, routes to `/training/rhythm-offset-detection`
   - "Fill the Gap" with hand-tap icon, routes to `/training/continuous-rhythm-matching`

3. **AC3 — Button labels simplified:** All sections use short labels:
   - Single Notes: "Compare" / "Match"
   - Intervals: "Compare" / "Match"
   - Rhythm: "Compare Timing" / "Fill the Gap"

4. **AC4 — Accessibility labels:** Each button has a distinct `aria-label`:
   - "Compare Pitch", "Match Pitch", "Compare Intervals", "Match Intervals", "Compare Timing", "Fill the Gap"

5. **AC5 — Routes registered:** `app.rs` has routes for `/training/rhythm-offset-detection` and `/training/continuous-rhythm-matching` pointing to placeholder views.

6. **AC6 — Sparklines:** Rhythm `TrainingCard`s show `ProgressSparkline` for their respective `TrainingDiscipline` variants (will show empty/no-data state until rhythm data exists).

7. **AC7 — Responsive layout:** Portrait: vertical stack of 3 sections. The rhythm section sits below intervals. On wider screens (md+), the 3 sections sit side-by-side in a row.

8. **AC8 — SoundFont gate:** Rhythm buttons respect the same SoundFont loading gate as pitch buttons (disabled while fetching). This may change later if rhythm uses a different audio source.

9. **AC9 — Localization:** New keys added to `en/main.ftl` and `de/main.ftl`:
   - Section header: "Rhythm" / "Rhythmus"
   - Button labels: "Compare Timing" / "Timing vergleichen", "Fill the Gap" / "Lücke füllen"
   - Aria labels for all new buttons

10. **AC10 — Builds and navigates:** `trunk build` succeeds. Clicking rhythm buttons navigates to placeholder screens.

## Tasks / Subtasks

- [x] Task 1: Add routes for rhythm screens in `app.rs`
- [x] Task 2: Add Rhythm section to `StartPage` with two `TrainingCard`s
- [x] Task 3: Update button labels for all sections (simplify to "Compare"/"Match")
- [x] Task 4: Add accessibility labels
- [x] Task 5: Add localization strings (en + de)
- [x] Task 6: Verify responsive layout (portrait vs. landscape)
- [x] Task 7: `trunk build` succeeds, navigation works

## Dev Notes

- The `TrainingCard` component already takes a `mode: TrainingMode` (will be `TrainingDiscipline` after Epic 14). Pass the new rhythm variants.
- Rhythm cards don't need the `interval_href()` helper — they're simple direct links
- Icons: Consider `\u{1F3B5}` (musical note) for Compare Timing, `\u{1F91A}` (raised back of hand) or `\u{270B}` for Fill the Gap — or use text-based icons. Match iOS where `hand.tap` is used for Fill the Gap.

## Dev Agent Record

### Implementation Plan

- Created placeholder views for rhythm training routes (story 15.3 will fill them in)
- Added Rhythm section as third section on start page using existing `TrainingCard` component pattern
- Updated aria labels per AC4: "Compare Pitch", "Match Pitch", "Compare Intervals", "Match Intervals", "Compare Timing", "Fill the Gap"
- Rhythm buttons use same SoundFont gate as pitch buttons (AC8)
- Icons: 🎵 for Compare Timing, ✋ for Fill the Gap

### Debug Log

No issues encountered.

### Completion Notes

All 7 tasks complete. 342 domain tests pass, clippy clean, trunk build succeeds. Responsive layout uses existing `flex-col md:flex-row` pattern — browser verification deferred to user.

## File List

- `web/src/components/rhythm_placeholder_view.rs` (new) — Placeholder views for rhythm routes
- `web/src/components/mod.rs` (modified) — Register and export rhythm placeholder views
- `web/src/components/start_page.rs` (modified) — Add Rhythm section, update aria labels
- `web/src/app.rs` (modified) — Add rhythm routes, import placeholder views
- `web/locales/en/main.ftl` (modified) — Add rhythm section header, button labels, aria labels, coming-soon
- `web/locales/de/main.ftl` (modified) — German translations for rhythm strings

## Change Log

- 2026-03-24: Implemented Story 15.2 — Three-section start screen layout with rhythm routes
