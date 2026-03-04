# Story 6.2: Screen Reader Accessibility Polish

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician using a screen reader,
I want all training feedback and state changes announced,
so that I can train my pitch discrimination with full accessibility.

## Acceptance Criteria

1. **Given** comparison training (FR46) **When** I answer a comparison **Then** the screen reader announces "Correct" or "Incorrect".

2. **Given** pitch matching training (FR46) **When** I commit a pitch **Then** the screen reader announces the result (e.g. "4 cents sharp", "27 cents flat", "Dead center").

3. **Given** any training mode **When** training starts **Then** the screen reader announces "Training started".

4. **Given** any training mode **When** training stops **Then** the screen reader announces "Training stopped".

5. **Given** interval training mode **When** the interval changes between challenges **Then** the screen reader announces the target interval (e.g. "Target interval: Perfect Fifth Up").

6. **Given** all `aria-live` regions **When** reviewed for completeness **Then** all training feedback events across comparison, pitch matching, and interval modes have announcements **And** announcements use `aria-live="polite"` to avoid interrupting the user.

7. **Given** all interactive elements across all views **When** audited for accessibility **Then** visible focus indicators are present on every interactive element **And** tab order follows visual order in every view **And** all custom components (profile visualization, pitch slider, feedback indicators) have appropriate ARIA roles and labels.

## Tasks / Subtasks

- [ ] Task 1: Add "Training started" announcements (AC: #3)
  - [ ] 1.1 In `comparison_view.rs`, set `sr_announcement` to "Training started" when the training loop begins (after first session state is set)
  - [ ] 1.2 In `pitch_matching_view.rs`, set `sr_announcement` to "Training started" when the training loop begins
  - [ ] 1.3 Verify announcements fire only once on initial load, not on every loop iteration

- [ ] Task 2: Add "Training stopped" announcements (AC: #4)
  - [ ] 2.1 In `comparison_view.rs`, announce "Training stopped" when the user navigates away (Escape, nav link click, or tab visibility change triggers `on_nav_away()`)
  - [ ] 2.2 In `pitch_matching_view.rs`, announce "Training stopped" on same triggers
  - [ ] 2.3 Ensure the announcement fires before the component unmounts (use `on_cleanup` or announce before navigation)

- [ ] Task 3: Add `aria-atomic="true"` to all `aria-live` regions (AC: #6)
  - [ ] 3.1 In `comparison_view.rs` line ~555, add `aria-atomic="true"` to the `aria-live="polite"` div
  - [ ] 3.2 In `pitch_matching_view.rs` line ~611, add `aria-atomic="true"` to the `aria-live="polite"` div

- [ ] Task 4: Audit and fix focus indicators across all views (AC: #7)
  - [ ] 4.1 Verify all `<A>` navigation links in `page_nav.rs` have visible focus ring classes
  - [ ] 4.2 Verify all buttons in `start_page.rs` have focus ring classes
  - [ ] 4.3 Verify "Back to Start" links in settings, profile, and info views have focus ring classes
  - [ ] 4.4 Verify the reset confirmation dialog's buttons have focus ring classes
  - [ ] 4.5 Fix any missing or inconsistent focus indicators found

- [ ] Task 5: Audit and fix tab order across all views (AC: #7)
  - [ ] 5.1 Verify tab order matches visual order in: start page, comparison view, pitch matching view, profile view, settings view, info view
  - [ ] 5.2 Ensure no `tabindex` values greater than 0 exist (only `0` or `-1` allowed)
  - [ ] 5.3 Verify that `<dialog>` in settings view traps focus correctly when open

- [ ] Task 6: Audit ARIA on custom components (AC: #7)
  - [ ] 6.1 Verify `VerticalPitchSlider` has complete ARIA: `role="slider"`, `aria-label`, `aria-orientation`, `aria-valuemin`, `aria-valuemax`, `aria-valuenow` — ALREADY DONE, verify correct
  - [ ] 6.2 Verify `ProfileVisualization` has `role="img"` and descriptive `aria-label` — ALREADY DONE, verify text is appropriate
  - [ ] 6.3 Verify `ProfilePreview` has descriptive `aria-label` on the clickable link — ALREADY DONE, verify
  - [ ] 6.4 Add `aria-label` to any remaining unlabeled interactive elements found during audit
  - [ ] 6.5 Verify feedback indicator divs are properly marked `aria-hidden="true"` to prevent duplicate announcements (visual + live region) — ALREADY DONE, verify

- [ ] Task 7: Verify existing announcements are correct (AC: #1, #2, #5)
  - [ ] 7.1 Verify comparison feedback announces "Correct" or "Incorrect" — ALREADY IMPLEMENTED
  - [ ] 7.2 Verify pitch matching feedback announces results (e.g. "4 cents sharp") — ALREADY IMPLEMENTED
  - [ ] 7.3 Verify interval changes announce the target interval — ALREADY IMPLEMENTED
  - [ ] 7.4 Manual test with VoiceOver (macOS) to confirm all announcements are heard

## Dev Notes

### Architecture & Pattern Compliance

- **Crate boundary**: This story only touches the `web` crate (`web/src/components/`). No domain crate changes needed.
- **Signal pattern**: Screen reader announcements use `RwSignal<String>` signals (`sr_announcement`) that feed into `aria-live="polite"` regions. This pattern is already established in both training views.
- **UIObserver bridge**: The `sr_announcement` signal is set from within `sync_session_to_signals()` functions, not directly from components. New "Training started"/"Training stopped" announcements should follow this pattern where possible, or set the signal at the appropriate lifecycle point.

### What Already Exists (from codebase audit)

The following accessibility features are ALREADY IMPLEMENTED and need only verification, not reimplementation:

| Feature | Location | Status |
|---|---|---|
| "Correct"/"Incorrect" announcements | `comparison_view.rs` line ~126 | Done |
| Pitch matching result announcements | `pitch_matching_view.rs` line ~170 | Done |
| Interval change announcements | Both views, line ~137 / ~178 | Done |
| `aria-live="polite"` regions | Both views, line ~555 / ~611 | Done |
| `role="slider"` on pitch slider | `pitch_slider.rs` line ~142 | Done |
| `role="img"` on profile viz | `profile_visualization.rs` line ~251 | Done |
| Descriptive `aria-label` on profile preview | `profile_preview.rs` line ~76 | Done |
| Skip-to-content link | `app.rs` line ~163 | Done |
| Focus rings via Tailwind | All component files | Done |
| Keyboard shortcuts (arrows, H/L, Escape) | Both training views | Done |
| `aria-hidden="true"` on visual feedback | Both training views | Done |
| Dialog `aria-labelledby` | `settings_view.rs` line ~429 | Done |

### What Needs to Be Added

1. **"Training started" announcement**: Set `sr_announcement.set("Training started".into())` once at the beginning of the training loop in both views. In `comparison_view.rs`, this should fire after the first call to `sync_session_to_signals()`. In `pitch_matching_view.rs`, similarly after initial state setup. Use a flag to ensure it only fires once.

2. **"Training stopped" announcement**: This is trickier because the component is about to unmount when the user navigates away. Options:
   - Use Leptos `on_cleanup()` to set the announcement before teardown — but the DOM may already be unmounting
   - Announce in the `on_nav_away()` handler before triggering navigation
   - Best approach: set announcement in `on_nav_away()` before calling the cancellation/navigation logic

3. **`aria-atomic="true"`**: Simple attribute addition to existing `aria-live` divs. Ensures the entire region content is announced as a single unit, not just the changed text.

### Timing Considerations for Announcements

- "Training started" should fire after the `sr_announcement` signal is created and the live region is mounted in the DOM. If set too early (before mount), the screen reader won't catch it.
- "Training stopped" must fire before the component unmounts. Use `on_cleanup` or announce in the nav-away handler with a small delay before navigation.
- Between feedback announcements, the `sr_announcement` signal should be briefly cleared (set to empty string) before setting the next value, to ensure the screen reader re-announces identical consecutive values (e.g., two "Correct" in a row). Verify this is already handled.

### Testing Approach

- **Primary**: Manual testing with VoiceOver on macOS (Cmd+F5 to toggle)
- **Verify**: Each announcement type fires exactly once at the right time
- **Verify**: No duplicate announcements (visual feedback is `aria-hidden`, live region is `sr-only`)
- **Verify**: Tab through all views, confirm focus ring visible on every interactive element
- **Verify**: Tab order matches visual layout in every view
- **Tool**: Use browser accessibility inspector (Chrome DevTools > Accessibility tab) to verify ARIA tree

### Project Structure Notes

- All changes are in `web/src/components/` — no new files needed
- No domain crate changes
- No routing changes
- No new dependencies

### Files to Modify

| File | Action | Purpose |
|---|---|---|
| `web/src/components/comparison_view.rs` | **Modify** | Add "Training started"/"stopped" announcements, add `aria-atomic` |
| `web/src/components/pitch_matching_view.rs` | **Modify** | Add "Training started"/"stopped" announcements, add `aria-atomic` |
| (any files with focus/ARIA gaps found during audit) | **Modify** | Fix accessibility gaps |

### What NOT to Do

- Do NOT add announcements to training views for every note played — this would be overwhelming and disrupt the training flow
- Do NOT use `aria-live="assertive"` — polite is correct for training feedback that shouldn't interrupt
- Do NOT add visible text labels for screen reader content — use `sr-only` class for announcements
- Do NOT change the visual feedback design — only add/verify the parallel screen reader announcement layer
- Do NOT add ARIA to training views that would duplicate existing semantic HTML (buttons already have labels)
- Do NOT add "Training started" on component mount if the audio context hasn't been established — only announce when training actually begins

### Previous Story Intelligence (Story 6.1)

- Story 6.1 created `PageNav` component with `aria-label="Page navigation"` — good pattern to verify
- Story 6.1 added PageNav to settings, profile, and info views — these need focus indicator verification
- Clean implementation, all tests pass, `cargo clippy` clean
- Pattern: modify existing files, minimal new code

### Git Intelligence

Recent commits show a pattern of clean, focused implementations:
- `b11a2cf` Implement story 6.1 Info View & Complete Navigation
- `ba36c3a` Implement story 5.2 SoundFont Audio
- All recent stories follow: implement, clippy clean, tests pass, then mark as review

### References

- [Source: docs/planning-artifacts/epics.md#Epic 6, Story 6.2]
- [Source: docs/planning-artifacts/ux-design-specification.md#Accessibility]
- [Source: docs/planning-artifacts/ux-design-specification.md#Screen Reader Announcements]
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture — Component Architecture]
- [Source: docs/project-context.md#Leptos Framework Rules]
- [Source: docs/planning-artifacts/prd.md#Input & Accessibility — FR46]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
