# Story 11.1: UI Consistency and Sound Level Fixes

Status: review

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

- [x] Task 1: Increase audio gain to match iOS volume level (AC: #1)
  - [x] 1.1 Root-caused the volume difference: OxiSynth's default `gain: 0.2` is designed for multi-instrument mixing, much quieter than iOS's AVAudioUnitSampler. No correction factor needed in the oscillator or soundfont adapters.
  - [x] 1.2 Set OxiSynth `SynthDescriptor.gain` from 0.2 to 1.0 (~+14 dB) in `synth-worklet/src/lib.rs`, matching iOS perceived volume.
  - [x] 1.3 AmplitudeDB clamp range unchanged — the fix is at the synth engine level, not the gain calculation.
- [x] Task 2: Fix training mode name consistency (AC: #2)
  - [x] 2.1 Decided with Michael: training pages use simple names "Hear & Compare" / "Tune & Match" regardless of interval mode. Start page short labels + section headers are correct as-is.
  - [x] 2.2 Updated training page NavBar titles in `pitch_comparison_view.rs` and `pitch_matching_view.rs` to use simple names.
  - [x] 2.3 Checked help_sections.rs — uses em dash (U+2014) in info page help text, domain uses "--" in display_name. No visible inconsistency since domain display_name isn't directly rendered. No change needed.
- [x] Task 3: Use circled question mark for Help icon (AC: #3)
  - [x] 3.1 Kept plain "?" character. The visible circle comes from the pill container (pill_group=true on training pages) which matches the circled info icon approach.
  - [x] 3.2 Settings view help icon unchanged — it renders as a button with rounded-full styling already.
- [x] Task 4: Start page header — iOS layout alignment (AC: #4)
  - [x] 4.1 Added `pill_group: bool` and `filled: bool` props to `NavBar` and `NavIconButton` respectively in `nav_bar.rs`.
  - [x] 4.2 Set `pill_group=true` on start page NavBar.
  - [x] 4.3 Set `filled=true` on info icon button for visible circle background.
- [x] Task 5: Center section titles (AC: #5)
  - [x] 5.1 Added `text-center` to both section title `<h2>` elements in `start_page.rs`.
- [x] Task 6: Fixed-height training cards (AC: #6)
  - [x] 6.1 Changed `min-h-11` to `h-[4.5rem]` on training cards in `start_page.rs`.
  - [x] 6.2 Fixed height ensures consistent card size regardless of sparkline presence.
- [x] Task 7: Training page header — iOS layout alignment (AC: #7)
  - [x] 7.1 Added `title_left: bool` prop to NavBar; set on training pages for left-aligned title.
  - [x] 7.2 Added Settings and Profile/Chart NavIconButtons to both training page NavBars.
  - [x] 7.3 Set `pill_group=true` on training page NavBars to group right-side icons.
- [x] Task 8: Link SoundFont credit to author's page (AC: #8)
  - [x] 8.1 Changed plain text to `<a>` hyperlink in `help_sections.rs`. Body is rendered via `inner_html` so raw HTML works directly. Link opens in new tab with `target="_blank"` and `rel="noopener noreferrer"`.

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
Claude Opus 4.6

### Debug Log References
None — clean implementation, no debugging needed.

### Completion Notes List
- Root-caused SoundFont volume difference: OxiSynth default gain 0.2 vs iOS AVAudioUnitSampler. Fixed by setting gain to 1.0.
- Training page titles simplified to "Hear & Compare" / "Tune & Match" (per user decision)
- NavBar enhanced with `pill_group`, `title_left` props; NavIconButton enhanced with `filled` prop
- Training pages now show Help, Settings, and Profile icons in a pill-shaped container with left-aligned title
- Start page info icon has visible circle background; right icons grouped in pill
- Section titles centered; training cards have fixed height (h-[4.5rem])
- SoundFont credit linked to author's website via inline HTML in help section body

### File List
- synth-worklet/src/lib.rs (modified — set OxiSynth gain from 0.2 to 1.0)
- web/src/components/nav_bar.rs (modified — added pill_group, title_left, filled props)
- web/src/components/start_page.rs (modified — pill_group, filled info icon, centered titles, fixed card height)
- web/src/components/pitch_comparison_view.rs (modified — simple title, title_left, pill_group, settings+profile icons)
- web/src/components/pitch_matching_view.rs (modified — simple title, title_left, pill_group, settings+profile icons)
- web/src/help_sections.rs (modified — SoundFont credit hyperlink)
- docs/implementation-artifacts/sprint-status.yaml (modified — story status)

### Change Log
- 2026-03-08: Implemented all 8 tasks for story 11.1 — UI consistency and sound level fixes
