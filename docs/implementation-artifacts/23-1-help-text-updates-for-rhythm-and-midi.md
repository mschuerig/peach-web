# Story 23.1: Help Text Updates for Rhythm and MIDI

Status: review

## Story

As a user,
I want help text that covers all training disciplines and input methods,
so that I can understand all available training modes and how to use MIDI controllers.

## Acceptance Criteria

1. **Info view lists all 6 disciplines**: The "Training Modes" help section on the info page includes rhythm disciplines "Compare Timing" (Rhythm Offset Detection) and "Fill the Gap" (Continuous Rhythm Matching) alongside the existing 4 pitch disciplines.
2. **Pitch matching help mentions MIDI pitch bend**: The pitch matching controls help text explains that a MIDI pitch bend wheel can be used as an alternative to the on-screen slider — deflect to adjust pitch, return to center to commit.
3. **Fill the Gap help mentions MIDI input**: The Fill the Gap controls help text explains that a connected MIDI controller's keys can also be used to tap (any MIDI note-on triggers a tap).
4. **Rhythm offset detection keyboard keys localized**: The keyboard shortcut letters in the rhythm offset detection help text use Fluent variables (e.g. `{ $earlyKey }` / `{ $lateKey }`) so they can differ per locale, rather than hardcoding "E" and "L". The actual key bindings in the Rust code must also be read from the locale (or at minimum, the German locale uses locale-appropriate keys like "F" for "Früh" and "S" for "Spät").
5. **German locale updated**: All help text changes are mirrored in `web/locales/de/main.ftl`.
6. **Pitch discrimination keyboard keys localized**: Apply the same localization pattern as AC4 to pitch discrimination shortcuts (currently hardcoded "H" for higher, "L" for lower in English; German could use "H" for "Höher" and "T" for "Tiefer" — which already exists in the code but is not documented in help text).

## Tasks / Subtasks

- [x] Task 1: Extend "Training Modes" help text in info view (AC: 1)
  - [x] 1.1 Update `help-info-modes-body` in `web/locales/en/main.ftl` to add two rhythm entries: "Compare Timing" and "Fill the Gap" with brief descriptions
  - [x] 1.2 Update `help-info-modes-body` in `web/locales/de/main.ftl` with equivalent German text

- [x] Task 2: Add MIDI pitch bend to pitch matching help (AC: 2, 5)
  - [x] 2.1 Update `help-matching-controls-body` in `web/locales/en/main.ftl` to mention MIDI pitch bend wheel as alternative input (deflect to adjust, return to center to commit)
  - [x] 2.2 Update `help-matching-controls-body` in `web/locales/de/main.ftl` with equivalent German text

- [x] Task 3: Add MIDI input to Fill the Gap help (AC: 3, 5)
  - [x] 3.1 Update `help-fill-the-gap-controls-body` in `web/locales/en/main.ftl` to mention MIDI note-on as alternative tap trigger
  - [x] 3.2 Update `help-fill-the-gap-controls-body` in `web/locales/de/main.ftl` with equivalent German text

- [x] Task 4: Localize keyboard shortcuts for rhythm offset detection (AC: 4, 5)
  - [x] 4.1 Add Fluent keys for shortcut letters in both locales: e.g. `rhythm-offset-early-key = E` (en), `rhythm-offset-early-key = F` (de), `rhythm-offset-late-key = L` (en), `rhythm-offset-late-key = S` (de)
  - [x] 4.2 Update `help-rhythm-offset-controls-body` in both locales to use Fluent message references for the letter keys instead of hardcoded letters
  - [x] 4.3 Update the keyboard handler in `rhythm_offset_detection_view.rs` to read the shortcut keys from the locale (use `untrack(|| tr!(...))` to get the current key letters) instead of hardcoded `"e"` / `"l"`
  - [x] 4.4 Verify the key match is case-insensitive (match both lower and upper case of the localized key)

- [x] Task 5: Localize keyboard shortcuts for pitch discrimination (AC: 6, 5)
  - [x] 5.1 Add Fluent keys for shortcut letters: `discrimination-higher-key = H` (en/de), `discrimination-lower-key = L` (en), `discrimination-lower-key = T` (de)
  - [x] 5.2 Update `help-discrimination-controls-body` in both locales to use Fluent message references for the letter keys
  - [x] 5.3 Update the keyboard handler in `pitch_discrimination_view.rs` to read the shortcut keys from the locale using `untrack(|| tr!(...))`
  - [x] 5.4 Verify the key match is case-insensitive

## Dev Notes

### Files to Modify

| File | Changes |
|------|---------|
| `web/locales/en/main.ftl` | Update 4 help body keys, add 4 shortcut key entries |
| `web/locales/de/main.ftl` | Mirror all English changes with German text |
| `web/src/components/rhythm_offset_detection_view.rs` | Replace hardcoded key letters with locale-driven keys |
| `web/src/components/pitch_discrimination_view.rs` | Replace hardcoded key letters with locale-driven keys |

### Localization Strategy for Keyboard Shortcuts

The keyboard handlers currently use hardcoded string literals:

```rust
// rhythm_offset_detection_view.rs:281
"ArrowLeft" | "e" | "E" => { on_answer(true); }  // early
"ArrowRight" | "l" | "L" => { on_answer(false); } // late

// pitch_discrimination_view.rs:329
"ArrowUp" | "h" | "H" => { /* higher */ }
"ArrowDown" | "l" | "L" | "t" | "T" => { /* lower */ }
```

Arrow keys are universal and stay hardcoded. The letter keys need to come from the locale. Pattern:

1. At component mount (outside the reactive loop), capture the locale keys once:
   ```rust
   let early_key = untrack(|| tr!("rhythm-offset-early-key")).to_lowercase();
   let late_key = untrack(|| tr!("rhythm-offset-late-key")).to_lowercase();
   ```
2. In the keydown handler, compare `ev.key().to_lowercase()` against the captured keys.
3. Arrow keys remain as hardcoded fallbacks (always work regardless of locale).

**Important**: Use `untrack(|| tr!(...))` to avoid reactive tracking warnings in event handler setup. The keys are captured once at mount — language switching mid-training is not a supported scenario.

### MIDI Help Text Guidance

Keep MIDI mentions brief and non-technical. The help text should say something like:

- **Pitch matching**: "If you have a MIDI controller connected, you can also use the **pitch bend wheel** to adjust the pitch. Return the wheel to center to lock in your answer."
- **Fill the Gap**: "You can also tap using a connected **MIDI controller** — any key press will register as a tap."

### Existing Patterns

- Help text uses `{"\u000A\u000A"}` for paragraph breaks in Fluent
- Bold markup: `**text**` (processed by `HelpContent` component's simple markdown renderer)
- The German locale already has the same structure as English — mirror changes 1:1
- Pitch discrimination already accepts "t"/"T" for German "Tiefer" (lower) in code but this is not documented in help text

### What NOT to Change

- No changes to `help_sections.rs` — no new help sections are added, only existing body text is updated
- No changes to `help_content.rs` — the markdown renderer already supports the needed formatting
- No domain crate changes
- Arrow key bindings remain hardcoded (universal, not locale-dependent)
- No changes to MIDI adapter code — only help text describes existing functionality

### Project Structure Notes

- Locale files: `web/locales/{en,de}/main.ftl` — Fluent format
- Help section registry: `web/src/help_sections.rs` — static arrays of `HelpSection` with i18n keys
- Help component: `web/src/components/help_content.rs` — renders sections with simple markdown
- View components own their keyboard handlers inline

### References

- [Source: web/locales/en/main.ftl#Help sections] — all help text keys
- [Source: web/locales/de/main.ftl#Help sections] — German translations
- [Source: web/src/components/rhythm_offset_detection_view.rs:279-288] — keyboard handler
- [Source: web/src/components/pitch_discrimination_view.rs:328-335] — keyboard handler
- [Source: web/src/components/continuous_rhythm_matching_view.rs:431-446] — MIDI note-on wiring
- [Source: web/src/components/pitch_matching_view.rs:634-692] — MIDI pitch bend wiring
- [Source: docs/implementation-artifacts/22-2-*.md] — MIDI note-on story (Fill the Gap)
- [Source: docs/implementation-artifacts/22-3-*.md] — MIDI pitch bend story (Pitch Matching)
- [Source: docs/project-context.md#Anti-Patterns] — `untrack(|| tr!(...))` for one-time captures

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

None — clean implementation with no issues.

### Completion Notes List

- Task 1: Added "Compare Timing" and "Fill the Gap" entries to `help-info-modes-body` in both EN and DE locales.
- Task 2: Added MIDI pitch bend wheel paragraph to `help-matching-controls-body` in both locales.
- Task 3: Added MIDI controller tap paragraph to `help-fill-the-gap-controls-body` in both locales.
- Task 4: Added `rhythm-offset-early-key` / `rhythm-offset-late-key` Fluent keys (EN: E/L, DE: F/S). Updated help text to use Fluent message references. Replaced hardcoded letter keys in `rhythm_offset_detection_view.rs` with `untrack(|| tr!(...))` locale-driven keys with case-insensitive matching.
- Task 5: Added `discrimination-higher-key` / `discrimination-lower-key` Fluent keys (EN: H/L, DE: H/T). Updated help text to use Fluent message references. Replaced hardcoded letter keys in `pitch_discrimination_view.rs` with `untrack(|| tr!(...))` locale-driven keys with case-insensitive matching.
- Design note: Used Fluent **message references** (`{ rhythm-offset-early-key }`) instead of Fluent variables (`{ $earlyKey }`) because the help rendering pipeline (`help_content.rs`) does not pass variables — message references resolve automatically without code changes.

### File List

- `web/locales/en/main.ftl` — Updated 4 help body keys, added 4 shortcut key entries
- `web/locales/de/main.ftl` — Mirrored all English changes with German text
- `web/src/components/rhythm_offset_detection_view.rs` — Replaced hardcoded key letters with locale-driven keys
- `web/src/components/pitch_discrimination_view.rs` — Replaced hardcoded key letters with locale-driven keys
- `docs/implementation-artifacts/23-1-help-text-updates-for-rhythm-and-midi.md` — Story file updated
- `docs/implementation-artifacts/sprint-status.yaml` — Status updated

### Change Log

- 2026-03-26: Implemented all 5 tasks — help text updates for rhythm/MIDI and keyboard shortcut localization
