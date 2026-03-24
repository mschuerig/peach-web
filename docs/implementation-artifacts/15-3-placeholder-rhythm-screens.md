# Story 15.3: Placeholder Rhythm Training Screens

Status: draft

## Story

As a user,
I want to see informative placeholder screens when navigating to rhythm training,
so that I know the features are coming and can test the navigation flow.

## Context

Prerequisite: Story 15.2 (routes exist).

Two placeholder screens that will later be replaced with full training UIs. Each screen should have the standard NavBar with back navigation, a title, and a brief description of what the training mode will do.

## Acceptance Criteria

1. **AC1 — Rhythm Offset Detection placeholder:**
   - Route: `/training/rhythm-offset-detection`
   - NavBar with back button to start page
   - Title: "Compare Timing"
   - Description: brief text explaining the exercise (hear 4 clicks, judge if the odd one is early or late)
   - Visual: centered layout, muted styling indicating "coming soon"

2. **AC2 — Continuous Rhythm Matching placeholder:**
   - Route: `/training/continuous-rhythm-matching`
   - NavBar with back button to start page
   - Title: "Fill the Gap"
   - Description: brief text explaining the exercise (continuous beat loop with a gap, tap to fill it)
   - Visual: same centered layout

3. **AC3 — Localization:** Title and description strings in `en/main.ftl` and `de/main.ftl`.

4. **AC4 — Accessibility:** Screen reader can navigate title and description. ARIA landmark structure consistent with pitch training screens.

5. **AC5 — Component files:**
   - `web/src/components/rhythm_offset_detection_view.rs`
   - `web/src/components/continuous_rhythm_matching_view.rs`
   - Registered in `components/mod.rs`

6. **AC6 — Builds:** `trunk build` succeeds. Full navigation round-trip works: start → rhythm screen → back.

## Tasks / Subtasks

- [ ] Task 1: Create `RhythmOffsetDetectionView` component with NavBar + placeholder content
- [ ] Task 2: Create `ContinuousRhythmMatchingView` component with NavBar + placeholder content
- [ ] Task 3: Register components in `mod.rs`, wire into routes in `app.rs`
- [ ] Task 4: Add localization strings
- [ ] Task 5: Smoke test navigation

## Dev Notes

- Keep these minimal — they'll be substantially rewritten when implementing the actual training logic
- Follow the same component structure as `PitchDiscriminationView` (after Epic 14 rename): NavBar at top, content area below
- No session state machines, no audio, no timers — pure static content for now
