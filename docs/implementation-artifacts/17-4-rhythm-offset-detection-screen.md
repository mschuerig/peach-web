# Story 17.4: Rhythm Offset Detection Screen UI

Status: draft

## Story

As a user,
I want to practice rhythm offset detection with a visual metronome, Early/Late buttons, and feedback display,
so that I can train my ability to detect timing deviations.

## Context

Prerequisite: Story 17.3 (session state machine), 17.2 (scheduler).

This story replaces the placeholder screen from story 15.3 with the full training UI. The screen shows 4 dots that light up with each click, Early/Late answer buttons, and feedback.

### iOS Reference (Epic 48 + 56 refinements)

- 4 dots light up non-positionally as a visual metronome
- First dot is accented (larger/bolder)
- Early/Late buttons side-by-side
- Feedback: checkmark/cross + current difficulty as percentage (e.g., "4%")
- The offset note (3rd) has an overlapping-circle visualization

## Acceptance Criteria

1. **AC1 — Replaces placeholder:** The `/training/rhythm-offset-detection` route now shows the full training screen.

2. **AC2 — Visual metronome:** 4 dots in a row. Each dot lights up when its corresponding click plays. First dot is visually accented (larger or bolder).

3. **AC3 — Answer buttons:** "Early" and "Late" buttons, side-by-side. Enabled only in `AwaitingAnswer` state. Disabled during playback and feedback.

4. **AC4 — Feedback display:** After answering: checkmark (correct) or cross (incorrect) + current difficulty percentage. Displayed for ~400ms, then auto-advances to next trial.

5. **AC5 — Auto-play:** Trials start automatically after feedback completes. No manual "Play" button needed (the session is a continuous loop of trials).

6. **AC6 — NavBar:** Back button to start page, title "Compare Timing".

7. **AC7 — Session wiring:** Component creates `RhythmOffsetDetectionSession`, `RhythmScheduler`, and connects port trait implementations (profile, store, timeline).

8. **AC8 — Tempo display:** Current tempo shown somewhere on screen (e.g., "80 BPM" in a subtle label).

9. **AC9 — Accessibility:** Early/Late buttons have aria-labels. Screen reader announces feedback. Dot animation uses `aria-hidden`.

10. **AC10 — Localization:** All strings in `en/main.ftl` and `de/main.ftl`.

11. **AC11 — Builds and works:** `trunk build` succeeds. Full training flow works end-to-end.

## Tasks / Subtasks

- [ ] Task 1: Replace placeholder with full component structure
- [ ] Task 2: Implement dot visualization with light-up animation
- [ ] Task 3: Implement Early/Late buttons with state-dependent enable/disable
- [ ] Task 4: Implement feedback display with auto-advance timer
- [ ] Task 5: Wire session + scheduler + port traits
- [ ] Task 6: Add localization strings
- [ ] Task 7: Accessibility pass
- [ ] Task 8: End-to-end smoke test

## Dev Notes

- The dot light-up can be driven by the scheduler's step callback (or a reactive signal set on each step)
- Use `spawn_local_scoped_with_cancellation` for the feedback timer (NOT `Timeout.forget()` — see memory about disposed signal panics)
- The scheduler should start playing as soon as the component mounts (after a brief delay for the audio context to be ready)
- Keep the visual design simple — circles with color transitions, similar to the pitch training feedback indicators
