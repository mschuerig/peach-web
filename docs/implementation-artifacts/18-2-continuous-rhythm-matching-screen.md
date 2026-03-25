# Story 18.2: Continuous Rhythm Matching Screen UI

Status: done

## Story

As a user,
I want to practice continuous rhythm matching with a looping beat, a tap button, and timing feedback,
so that I can train my ability to maintain steady rhythm and fill gaps accurately.

## Context

Prerequisite: Story 18.1 (session + sequencer).

This story replaces the placeholder screen from story 15.3 with the full "Fill the Gap" training UI.

### iOS Reference (Epic 54 + 57 refinements)

- 4 dots cycle continuously; gap dot shown as outline circle
- Beat-1 dot is larger/bolder
- Large tap button (ideally full-screen tap target)
- Tap triggers the same click sound (user "fills" the gap audibly)
- Directional timing indicator: offset in ms with spectrogram color bands
- Sequencer starts immediately on screen load

## Acceptance Criteria

1. **AC1 — Replaces placeholder:** The `/training/continuous-rhythm-matching` route now shows the full training screen.

2. **AC2 — Cycling dots:** 4 dots in a row, lighting up in sequence with the beat. Gap position shown as an outline (empty) circle instead of a filled dot. Beat-1 dot is accented (larger).

3. **AC3 — Tap button:** Large, prominent button (or full-screen tap area). Fires on `pointerdown` (not click/pointerup) for lowest latency. Always enabled while running.

4. **AC4 — Tap audio feedback:** When user taps, play the same click sound at the moment of the tap (user audibly fills the gap).

5. **AC5 — Timing feedback:** After each tap, briefly show the timing offset:
   - Direction arrow (< for early, > for late)
   - Offset in milliseconds
   - Color coding: green (≤5%), yellow (5-15%), red (>15%)

6. **AC6 — Trial completion indicator:** After every 16 cycles, briefly flash a summary (hit rate, mean offset). This doesn't pause the sequencer.

7. **AC7 — Auto-start:** The sequencer starts immediately when the screen loads (after audio context is ready). No start button.

8. **AC8 — NavBar:** Back button (stops sequencer and navigates), title "Fill the Gap".

9. **AC9 — Session wiring:** Component creates session, scheduler in loop mode, connects port traits.

10. **AC10 — Tempo display:** Current tempo shown (e.g., "80 BPM").

11. **AC11 — Accessibility:** Tap button has aria-label. Feedback announced to screen reader. Dot animation uses `aria-hidden`.

12. **AC12 — Localization:** All strings in `en/main.ftl` and `de/main.ftl`.

13. **AC13 — Builds and works:** `trunk build` succeeds. Full training flow works end-to-end.

## Tasks / Subtasks

- [x] Task 1: Replace placeholder with full component structure
- [x] Task 2: Implement cycling dot visualization
- [x] Task 3: Implement tap button with `pointerdown` handler
- [x] Task 4: Implement tap audio feedback (play click on tap)
- [x] Task 5: Implement timing feedback display
- [x] Task 6: Implement trial completion indicator
- [x] Task 7: Wire session + scheduler (loop mode) + port traits
- [x] Task 8: Handle navigation away (stop sequencer, discard incomplete trial)
- [x] Task 9: Localization strings
- [x] Task 10: Accessibility pass
- [x] Task 11: End-to-end smoke test — tested by user on desktop (Mac) and mobile (iPhone). Functional but latency issues noted in `docs/future-work.md`

## Dev Notes

- Use `pointerdown` event (not `click`) for the tap button — `click` fires on pointer-up, adding 50-200ms latency
- Read `audioContext.currentTime` in the `pointerdown` handler for tap measurement
- The sequencer loop continues during feedback display — no pausing between trials
- Handle component cleanup carefully: stop the scheduler's interval timer and the sequencer when the component is disposed (use Leptos `on_cleanup`)
- Dot animation timing: the scheduler can emit a signal/callback on each step, driving the dot highlight

## Dev Agent Record

### Implementation Plan

Replaced placeholder `ContinuousRhythmMatchingView` component with full training screen following the established pattern from `RhythmOffsetDetectionView`. Architecture:

1. **Session wiring:** Creates `ContinuousRhythmMatchingSession` with `ProfilePort`, `RecordPort`, `TimelinePort`, and `RandomGapSelector`
2. **Audio lifecycle:** Eagerly creates AudioContext on render (user gesture), uses `ensure_audio_ready` in async loop, shares `ctx_rc` and `click_buffer` via `Rc<RefCell<Option<...>>>` for synchronous tap handler access
3. **Training loop:** Outer `'session` loop (restarts after help modal), inner `'training` loop (one cycle per iteration). Each cycle: build pattern from gap position, create single-pass scheduler, animate dots, wait for cycle end, call `session.cycle_complete()`
4. **Tap handling:** `pointerdown` handler reads `audioContext.currentTime`, calls `session.handle_tap()`, stores offset for cycle_complete, plays click sound immediately
5. **Timing feedback:** Direction arrow + ms offset with green/yellow/red color coding based on percentage of sixteenth note (≤5%, 5-15%, >15%)
6. **Trial summary:** After 16 cycles, briefly shows hit rate and mean offset in a non-blocking overlay (sequencer continues)
7. **Dot visualization:** 4 dots with beat-1 accent (larger), gap dot rendered as outline circle
8. **Lifecycle management:** `on_cleanup` stops session, clears event listeners; visibility change and AudioContext state change handlers navigate home

### Debug Log

No issues encountered during implementation. Clean compilation with no new warnings.

### Completion Notes

- All 13 ACs addressed in a single implementation pass
- Task 11 (E2E smoke test) deferred to user — agent cannot run browser
- Pre-existing dead_code warnings in `rhythm_scheduler.rs` unchanged
- Help modal, training stats, error notifications follow exact patterns from rhythm offset detection view
- Keyboard support: Space/Enter to tap (matching "large tap area" intent)
- `touch-manipulation` CSS on tap button prevents browser double-tap-to-zoom delay

## File List

- `web/src/components/continuous_rhythm_matching_view.rs` — Full replacement of placeholder
- `web/locales/en/main.ftl` — Added Fill the Gap UI strings and help strings
- `web/locales/de/main.ftl` — Added German translations for Fill the Gap
- `web/src/help_sections.rs` — Added `CONTINUOUS_RHYTHM_MATCHING_HELP`

## Change Log

- 2026-03-25: Implemented full Fill the Gap training screen (Story 18.2) — replaced placeholder with session-wired training UI including cycling dots, tap button, timing feedback, trial summaries, help modal, accessibility, and localization
