# Story 11.1: UI Consistency and Sound Level Fixes

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want the web app to match the iOS app's visual layout and sound level,
so that the experience is consistent across platforms.

## Acceptance Criteria

1. Audio playback is approximately 12 dB louder than current levels, matching the perceived volume of the iOS app.
2. Training mode names on the start page cards match the names used on the training pages and in help sections.
3. The help icon on training pages uses a circled question mark character (matching the circled info icon on the start page).
4. Start page header layout matches the iOS reference screenshot:
   - Info icon (left) has a visible circle background (like the back button on subpages).
   - Chart and Settings icons (right) are grouped in a pill-shaped container.
5. Section titles ("Single Notes", "Intervals") on the start page are centered.
6. Training cards on the start page have a fixed height regardless of whether a sparkline is present.
7. Training page header layout matches the iOS reference screenshot:
   - Title is left-aligned (after the back button), not centered.
   - Right side groups Help, Settings, and Chart icons in a pill-shaped container.
8. In the info/acknowledgements section, "GeneralUser GS by S. Christian Collins" is a hyperlink to `https://schristiancollins.com/generaluser.php`.

## Tasks / Subtasks

- [ ] Task 1: Increase audio gain by ~12 dB (AC: #1)
  - [ ] 1.1 In `web/src/audio_oscillator.rs`, add a constant `MASTER_GAIN_DB: f32 = 12.0` and apply it as an offset when converting `amplitude_db` to linear gain (line ~108). The formula becomes: `10_f32.powf((amplitude_db.raw_value() + MASTER_GAIN_DB) / 20.0)`.
  - [ ] 1.2 In `web/src/audio_soundfont.rs`, implement the `_amplitude_db` parameter that is currently ignored (~line 183). Apply the same `MASTER_GAIN_DB` offset.
  - [ ] 1.3 Verify `AmplitudeDB` clamp range (`-90.0..=+12.0` in `domain/src/types/amplitude.rs`) still makes sense — with the +12 dB offset, a 0.0 dB setting will produce +12 dB linear gain. Consider whether the max should be lowered to 0.0 to prevent clipping when vary_loudness is active.
- [ ] Task 2: Fix training mode name consistency (AC: #2)
  - [ ] 2.1 Decide on canonical short names for the start page cards. Current state: start page shows "Hear & Compare" / "Tune & Match" (no suffix); training pages use full `display_name` like "Hear & Compare -- Single Notes". The section headers ("Single Notes", "Intervals") already provide context, so the short labels may be intentional. **Ask Michael:** Should cards show the full name, or is the current short label + section header approach correct? If short labels are correct, ensure training page titles use the same short form.
  - [ ] 2.2 In `web/src/components/start_page.rs`, update card labels OR training page NavBar titles to be consistent (based on decision above).
  - [ ] 2.3 Check `web/src/help_sections.rs` uses en-dash ("---") vs double-dash ("--") in display names — align with `domain/src/training_mode.rs`.
- [ ] Task 3: Use circled question mark for Help icon (AC: #3)
  - [ ] 3.1 In `web/src/components/pitch_comparison_view.rs` (~line 693) and `web/src/components/pitch_matching_view.rs` (~line 722), change `icon="?".to_string()` to `icon="\u{24E0}".to_string()` (circled latin small letter q) or another appropriate circled question mark character. Test rendering across browsers. Note: There is no standard "circled question mark" in Unicode BMP like there is for info (U+24D8). Alternatives: U+2753 (red question mark ornament), U+003F in a styled circle via CSS, or keeping "?" and relying on the existing `rounded-full` button styling for the circle. **Recommendation:** Use `"\u{003F}"` (plain ?) and ensure the button background circle is always visible (see Task 5 pill styling).
  - [ ] 3.2 Also update `web/src/components/settings_view.rs` help icon if present.
- [ ] Task 4: Start page header — iOS layout alignment (AC: #4)
  - [ ] 4.1 In `web/src/components/nav_bar.rs`, add a new prop `pill_group: bool` (default false) to `NavBar` that wraps the children div in a pill-shaped container: `class="flex items-center gap-1 bg-gray-100 dark:bg-gray-800 rounded-full px-2 py-1"` (or similar). When `pill_group` is true, right-side icons get the pill background.
  - [ ] 4.2 In `web/src/components/start_page.rs` (~line 102-110), set `pill_group=true` on the NavBar.
  - [ ] 4.3 Give the info icon button on the start page a visible circle background (same as back button styling: `bg-gray-100 dark:bg-gray-800`). This may require a new prop on `NavIconButton` like `filled: bool`, or a separate CSS class.
- [ ] Task 5: Center section titles (AC: #5)
  - [ ] 5.1 In `web/src/components/start_page.rs` (~lines 132, 155), add `text-center` to the `<h2>` class.
- [ ] Task 6: Fixed-height training cards (AC: #6)
  - [ ] 6.1 In `web/src/components/start_page.rs` (~line 31), determine the rendered height of a card with a sparkline (likely ~68-72px with the 24px SVG + text + padding). Set a fixed height class (e.g. `h-[4.5rem]`) instead of `min-h-11`.
  - [ ] 6.2 Ensure the sparkline `<div>` and the no-data `<span>` both occupy the same vertical space, or use `overflow-hidden` on the card.
- [ ] Task 7: Training page header — iOS layout alignment (AC: #7)
  - [ ] 7.1 Restructure the `NavBar` layout for training pages. In the iOS screenshot, the title is left-aligned after the back button (not centered). This requires either a new NavBar variant or a prop like `align_title_left: bool`.
  - [ ] 7.2 Add Settings and Chart icons to the training page right side. In `pitch_comparison_view.rs` and `pitch_matching_view.rs`, add `NavIconButton` for settings (href to settings page) and profile/chart (href to profile page) alongside the existing help button.
  - [ ] 7.3 Set `pill_group=true` on training page NavBars so the three right-side icons are grouped in a pill.
- [ ] Task 8: Link SoundFont credit to author's page (AC: #8)
  - [ ] 8.1 In `web/src/help_sections.rs` (~line 85), change the plain text "GeneralUser GS by S. Christian Collins" to include a hyperlink. Since help section bodies are plain strings rendered in Leptos views, check how the body is rendered (likely as text content). May need to change the body field to support inline HTML/view fragments, or split the text and insert an `<a>` tag with `target="_blank"` and `rel="noopener noreferrer"` pointing to `https://schristiancollins.com/generaluser.php`.

## Dev Notes

### Architecture & Patterns

- **Tailwind CSS only** — no custom CSS files. All styling via utility classes.
- **NavBar component** (`web/src/components/nav_bar.rs`): Flexbox layout with left (w-11), center (flex-1 text-center), and right (flex shrink-0) sections. Needs modification for pill grouping and optional left-aligned title.
- **NavIconButton** (`nav_bar.rs:7-35`): Renders circular touch-target buttons (44x44px min). Currently transparent background; needs a `filled` variant for always-visible circle background (matching back button style).
- **Training cards** (`start_page.rs:22-68`): Flex row with icon + column (label + optional sparkline). Current `min-h-11` allows height variation.
- **Audio gain** (`audio_oscillator.rs:108`): Converts `AmplitudeDB` to linear via `10^(dB/20)`. The +12 dB offset is a simple addition before conversion.
- **SoundFont gain** (`audio_soundfont.rs:183`): Currently ignores amplitude parameter — needs implementation.

### Key Files to Modify

| File | Changes |
|---|---|
| `web/src/audio_oscillator.rs` | Add master gain offset constant, apply to gain calculation |
| `web/src/audio_soundfont.rs` | Implement amplitude_db parameter with master gain offset |
| `web/src/components/nav_bar.rs` | Add `pill_group` and `filled` props; optional left-aligned title layout |
| `web/src/components/start_page.rs` | Center section titles, fix card height, set pill_group, update card labels |
| `web/src/components/pitch_comparison_view.rs` | Add settings+chart icons, set pill_group, update help icon |
| `web/src/components/pitch_matching_view.rs` | Same as pitch_comparison_view |
| `web/src/components/settings_view.rs` | Update help icon if needed |
| `domain/src/training_mode.rs` | Possibly update display_name (pending naming decision) |
| `web/src/help_sections.rs` | Make SoundFont credit a hyperlink |

### Unicode Characters in Use

| Icon | Current | Character |
|---|---|---|
| Info | `\u{24D8}` | ⓘ (Circled Latin Small Letter I) |
| Help | `"?"` | Plain question mark |
| Settings | `\u{2699}\u{FE0F}` | Gear |
| Profile/Chart | `\u{1F4CA}` | Bar chart |
| Back | `\u{2039}` | Single left angle quote |

There is no standard Unicode "circled question mark" equivalent to ⓘ. Options: use CSS to ensure the button circle is always visible (recommended), or use a custom approach.

### Testing

- `cargo test -p domain` — verify AmplitudeDB type changes if any
- `cargo clippy --workspace` — lint check
- Manual browser testing: verify all 8 acceptance criteria visually
- Check both light and dark mode for pill styling
- Check training cards with and without sparkline data for consistent height
- Test audio volume with headphones — compare oscillator and SoundFont playback

### Project Structure Notes

- All changes are in `web/` crate except possible `domain/src/types/amplitude.rs` adjustment
- No new files needed — all modifications to existing components
- Follows existing Tailwind CSS patterns throughout

### References

- [Source: web/src/components/nav_bar.rs] — NavBar and NavIconButton components
- [Source: web/src/components/start_page.rs] — Start page layout, training cards, section titles
- [Source: web/src/components/pitch_comparison_view.rs#L690-694] — Comparison training NavBar
- [Source: web/src/components/pitch_matching_view.rs#L721-723] — Pitch matching NavBar
- [Source: web/src/audio_oscillator.rs#L108] — Gain calculation
- [Source: web/src/audio_soundfont.rs#L183] — SoundFont amplitude (ignored)
- [Source: domain/src/training_mode.rs] — Display names for training modes
- [Source: web/src/help_sections.rs#L75] — Help section names
- [Source: docs/project-context.md] — Project conventions and rules

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
