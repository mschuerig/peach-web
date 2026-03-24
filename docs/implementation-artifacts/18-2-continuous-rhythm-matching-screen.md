# Story 18.2: Continuous Rhythm Matching Screen UI

Status: draft

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

- [ ] Task 1: Replace placeholder with full component structure
- [ ] Task 2: Implement cycling dot visualization
- [ ] Task 3: Implement tap button with `pointerdown` handler
- [ ] Task 4: Implement tap audio feedback (play click on tap)
- [ ] Task 5: Implement timing feedback display
- [ ] Task 6: Implement trial completion indicator
- [ ] Task 7: Wire session + scheduler (loop mode) + port traits
- [ ] Task 8: Handle navigation away (stop sequencer, discard incomplete trial)
- [ ] Task 9: Localization strings
- [ ] Task 10: Accessibility pass
- [ ] Task 11: End-to-end smoke test

## Dev Notes

- Use `pointerdown` event (not `click`) for the tap button — `click` fires on pointer-up, adding 50-200ms latency
- Read `audioContext.currentTime` in the `pointerdown` handler for tap measurement
- The sequencer loop continues during feedback display — no pausing between trials
- Handle component cleanup carefully: stop the scheduler's interval timer and the sequencer when the component is disposed (use Leptos `on_cleanup`)
- Dot animation timing: the scheduler can emit a signal/callback on each step, driving the dot highlight
