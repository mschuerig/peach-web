# Story 8.6: Pitch Comparison Feedback Icons

Status: ready-for-dev

## Story

As a user,
I want to see a brief checkmark (correct) or X (incorrect) in a colored circle after each pitch comparison answer,
so that the feedback is clean, consistent with the help text, and positioned like the pitch matching screen.

## Acceptance Criteria

1. Correct answers show a white checkmark inside a green circle in the top-right area of the header
2. Incorrect answers show a white "X" inside a red circle in the top-right area of the header
3. Feedback position mirrors the pitch matching view layout: stats left, feedback indicator right
4. The help text already says "checkmark (correct) or X (incorrect)" and must remain accurate
5. The old thumbs-up/thumbs-down emoji feedback in the center row is removed
6. Feedback timing and duration remain unchanged (`FEEDBACK_DURATION_SECS`)
7. Dark mode preserved, no regressions
8. Screen reader announcement behavior unchanged

## Tasks / Subtasks

- [ ] Task 1: Move feedback indicator from center row to header top-right (AC: #3, #5)
  - [ ] 1.1 Remove the standalone `<div class="flex items-center justify-center h-12">` feedback block between stats and buttons in `pitch_comparison_view.rs` (~line 717-730)
  - [ ] 1.2 Add feedback indicator to the right side of the existing compact header row (similar to pitch matching view's `<div class="text-right">` pattern at ~line 746-772 of `pitch_matching_view.rs`)
- [ ] Task 2: Replace thumbs emoji with checkmark/X in colored circles (AC: #1, #2)
  - [ ] 2.1 Replace `\u{1F44D}` (thumbs up) with a checkmark character (e.g. `\u{2713}` or `\u{2714}`) rendered as white text inside a green circle (`bg-green-500 rounded-full w-8 h-8 flex items-center justify-center text-white font-bold`)
  - [ ] 2.2 Replace `\u{1F44E}` (thumbs down) with an X character (e.g. `\u{2717}` or plain "X") rendered as white text inside a red circle (`bg-red-500 rounded-full w-8 h-8 flex items-center justify-center text-white font-bold`)
  - [ ] 2.3 Verify dark mode appearance — circle colors should look good in both modes
- [ ] Task 3: Verify help text accuracy (AC: #4)
  - [ ] 3.1 Confirm `help_sections.rs` line 36-37 already says "checkmark (correct) or X (incorrect)" — no change needed
- [ ] Task 4: Verify no regressions (AC: #6, #7, #8)
  - [ ] 4.1 Confirm `show_feedback` and `is_last_correct` signals still drive the indicator correctly
  - [ ] 4.2 Confirm `sr_announcement` still fires for screen readers
  - [ ] 4.3 Test both light and dark modes

## Dev Notes

### Current Implementation

The feedback indicator in `pitch_comparison_view.rs` (~line 717-730) currently renders thumbs-up/thumbs-down emoji centered between the stats row and answer buttons:

```rust
<div class="flex items-center justify-center h-12" aria-hidden="true">
    {move || {
        if show_feedback.get() {
            if is_last_correct.get() {
                view! { <span class="text-4xl text-green-600 dark:text-green-400">{"\u{1F44D}"}</span> }.into_any()
            } else {
                view! { <span class="text-4xl text-red-600 dark:text-red-400">{"\u{1F44E}"}</span> }.into_any()
            }
        } else {
            view! { <span></span> }.into_any()
        }
    }}
</div>
```

### Target Layout (Pitch Matching Reference)

The pitch matching view (`pitch_matching_view.rs` ~line 746-772) uses a header with stats left, feedback right:

```rust
<div class="flex items-start justify-between mb-2">
    <TrainingStats ... />
    <div class="text-right" aria-hidden="true">
        {move || { /* feedback rendering */ }}
    </div>
</div>
```

Follow this same pattern for the pitch comparison header — place the circle icon in a `<div class="text-right">` on the right side of the existing header/stats row.

### Icon Rendering Approach

Use Tailwind utility classes to create colored circles with white text inside. No SVG or external icon library needed:

```rust
// Correct: white checkmark in green circle
view! { <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-green-500 text-white font-bold text-lg">{"\u{2713}"}</span> }

// Incorrect: white X in red circle
view! { <span class="inline-flex items-center justify-center w-8 h-8 rounded-full bg-red-500 text-white font-bold text-lg">{"X"}</span> }
```

When feedback is not showing, render a placeholder of the same dimensions to prevent layout shift:

```rust
view! { <div class="w-8 h-8"></div> }.into_any()
```

### Signals (Unchanged)

- `show_feedback: RwSignal<bool>` — controls visibility
- `is_last_correct: RwSignal<bool>` — controls which icon to show
- No new signals needed; the existing two signals are sufficient

### Project Structure Notes

- All changes are in `web/src/components/pitch_comparison_view.rs`
- No domain crate changes needed
- No changes to `help_sections.rs` needed (text already matches)
- Follow established dark mode pattern from story 8.5

### References

- [Source: web/src/components/pitch_comparison_view.rs#L717-730] Current feedback rendering
- [Source: web/src/components/pitch_matching_view.rs#L746-772] Target layout reference
- [Source: web/src/help_sections.rs#L36-37] Help text (already correct)
- [Source: docs/implementation-artifacts/8-5-settings-page-ios-alignment.md] Previous story patterns

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
