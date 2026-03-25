# Story 17.4: Rhythm Offset Detection Screen UI

Status: done

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

- [x] Task 1: Replace placeholder with full component structure
- [x] Task 2: Implement dot visualization with light-up animation
- [x] Task 3: Implement Early/Late buttons with state-dependent enable/disable
- [x] Task 4: Implement feedback display with auto-advance timer
- [x] Task 5: Wire session + scheduler + port traits
- [x] Task 6: Add localization strings
- [x] Task 7: Accessibility pass
- [ ] Task 8: End-to-end smoke test — deferred to user; agent cannot verify in browser

## Dev Notes

- The dot light-up can be driven by the scheduler's step callback (or a reactive signal set on each step)
- Use `spawn_local_scoped_with_cancellation` for the feedback timer (NOT `Timeout.forget()` — see memory about disposed signal panics)
- The scheduler should start playing as soon as the component mounts (after a brief delay for the audio context to be ready)
- Keep the visual design simple — circles with color transitions, similar to the pitch training feedback indicators

## Dev Agent Record

### Implementation Plan

The view follows the same architectural patterns as PitchDiscriminationView:
- Session + port traits (ProfilePort, RecordPort, TimelinePort) wired in component init
- UI signals bridge domain state to Leptos reactivity via `sync_session_to_signals()`
- `cancelled`/`terminated` flags for lifecycle management
- `spawn_local` async training loop with `'session`/`'training` nested loops
- Event listeners (keydown, visibilitychange, audiocontext state) with cleanup

Key design decisions:
1. **Dot animation driven by audio clock polling**: Instead of spawning separate timeout tasks per dot, the training loop polls at 20ms intervals and computes which dot should be lit based on precomputed beat times vs. current AudioContext time. This avoids spawn_local disposal issues.
2. **Scheduler + manual offset click**: RhythmScheduler plays beats 1, 2, 4 with pattern [Play, Play, Silent, Play]. Beat 3 (the offset click) is scheduled manually via `play_click_at()` at the offset time. This cleanly separates the regular metronome from the training-specific offset.
3. **Beat times computed upfront**: All 4 beat times are calculated before playback starts, enabling both audio scheduling and visual animation from the same source of truth.
4. **3rd dot ring indicator**: The offset dot (index 2) has a subtle ring border to visually distinguish it as the beat being tested, matching the iOS overlapping-circle pattern.

### Debug Log

No issues encountered during implementation.

### Completion Notes

- Replaced placeholder "coming soon" screen with full training UI
- 4-dot visual metronome with accent on first dot and ring on offset dot (3rd)
- Early/Late buttons with state-dependent enable/disable (only in AwaitingAnswer)
- Feedback display: checkmark/X icon + difficulty percentage
- Auto-play loop: Idle → Playing → AwaitingAnswer → ShowingFeedback → repeat
- NavBar with back button, title "Compare Timing", help/settings/profile icons
- Help modal with 3 sections (Goal, Controls, Feedback)
- Keyboard shortcuts: Arrow Left/E for Early, Arrow Right/L for Late
- Tempo display below nav bar
- Screen reader live region for feedback announcements
- All strings localized in en/main.ftl and de/main.ftl
- Visibility change and AudioContext state change handlers for interruption
- Proper cleanup on component unmount (event listeners, session stop)
- Training stats component wired with difficulty percentage tracking

## File List

- `web/src/components/rhythm_offset_detection_view.rs` — Complete rewrite: full training UI
- `web/src/adapters/rhythm_scheduler.rs` — Added `pub play_click_at()`, made `ACCENT_GAIN`/`NORMAL_GAIN` public
- `web/locales/en/main.ftl` — Added: early, late, bpm-label, difficulty-pct, rhythm-offset-title, rhythm-offset-help-title, help-rhythm-offset-* strings
- `web/locales/de/main.ftl` — Added: German translations for all new strings
- `web/src/help_sections.rs` — Added `RHYTHM_OFFSET_DETECTION_HELP` static

## Change Log

- 2026-03-25: Implemented full rhythm offset detection training screen UI (Story 17.4)
- 2026-03-25: Fix review findings — abort trial on offset click failure, use difficulty-pct locale key in view, add difficulty % to screen reader announcement. Cataloged D1 (unscoped spawn_local in AudioContext resume) as PEF-011.
