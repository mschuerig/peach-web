# Story 7.7: Help Content

Status: done

## Story

As a musician,
I want contextual help text available in the app,
so that I can understand how each training mode works and what the statistics mean.

## Context

The iOS app added help buttons (?) on the Settings screen, both training screens, and the Info screen. Each opens a modal sheet with `HelpContentView` rendering an array of titled markdown sections.

peach-web currently has no help system. The Info view shows only app metadata.

**iOS reference files:**
- `Peach/App/HelpContentView.swift` — reusable component rendering `[HelpSection]` (title + markdown body)
- `Peach/Settings/SettingsScreen.swift` — `helpSections` array with 5 sections (Training Range, Intervals, Sound, Difficulty, Data)
- `Peach/PitchComparison/PitchComparisonScreen.swift` — `helpSections` with 5 sections (Goal, Controls, Feedback, Difficulty, Intervals)
- `Peach/PitchMatching/PitchMatchingScreen.swift` — `helpSections` with 4 sections (Goal, Controls, Feedback, Intervals)
- `Peach/Info/InfoScreen.swift` — `helpSections` with 3 sections (What is Peach?, Training Modes, Getting Started) + acknowledgments

Depends on: None (can be implemented in parallel with other stories).

## Acceptance Criteria

1. **AC1 — HelpContent component:** A reusable `HelpContent` component renders a list of help sections, each with a title (headline font) and body text
2. **AC2 — Markdown in body text:** Body text supports basic inline markdown: **bold**, *italic*. Full markdown parsing is not required — a simple regex replacement for `**text**` → `<strong>` and `*text*` → `<em>` suffices.
3. **AC3 — Help modal pattern:** Each view with help has a "?" button in the header/nav area. Clicking it opens a modal (dialog or overlay) with the relevant help sections. A "Done" or close button dismisses it.
4. **AC4 — Settings help:** Settings view gets a help button with sections: Training Range, Intervals, Sound, Difficulty, Data. Content matches iOS text (adapted for web where needed, e.g. no mention of iOS-specific features).
5. **AC5 — Comparison training help:** Comparison view gets a help button with sections: Goal, Controls, Feedback, Difficulty, Intervals.
6. **AC6 — Pitch matching training help:** Pitch matching view gets a help button with sections: Goal, Controls, Feedback, Intervals.
7. **AC7 — Info view help:** Info view integrates help sections inline (not in a modal): "What is Peach?", "Training Modes", "Getting Started", plus Acknowledgments. This replaces/augments the current minimal info content.
8. **AC8 — Training screen help pauses training:** When help opens on a training screen, training pauses (session stops). When help closes, training resumes (session restarts). This matches iOS behavior where help sheet triggers `onAppear`/`onDisappear`.
9. **AC9 — Accessibility:** Help modal has `role="dialog"`, `aria-modal="true"`, focus traps to the modal, Escape key closes it. Help sections use semantic headings.
10. **AC10 — Visual styling:** Help sections have comfortable spacing (20px between sections, 8px between title and body). Title is headline weight. Body is regular weight.

## Tasks / Subtasks

- [x] Task 1: Create HelpContent component (AC: 1, 2, 10)
  - [x] New component in `web/src/components/help_content.rs`
  - [x] Props: sections as `&[HelpSection]` where `HelpSection { title: &'static str, body: &'static str }`
  - [x] Render each section: `<h3>` for title, processed body text for content
  - [x] Simple markdown processing: replace `**text**` with `<strong>text</strong>`, `*text*` with `<em>text</em>`, `\n\n` with `<br><br>`. Use `inner_html` for the processed body (content is static/trusted, not user input).

- [x] Task 2: Create help modal component (AC: 3, 9)
  - [x] New component `HelpModal` (or use HTML `<dialog>` element)
  - [x] Props: `title: &str`, `sections: &[HelpSection]`, `is_open: RwSignal<bool>`
  - [x] Uses `<dialog>` element with `showModal()`/`close()` for native modal behavior (provides backdrop, focus trapping, Escape to close)
  - [x] Header with title and "Done" button
  - [x] Scrollable content area with HelpContent inside

- [x] Task 3: Define help section content (AC: 4, 5, 6, 7)
  - [x] Create `web/src/help_sections.rs` (or const arrays in each view)
  - [x] Settings help: 5 sections matching iOS text, adapted for web
  - [x] Comparison help: 5 sections
  - [x] Pitch matching help: 4 sections
  - [x] Info help: 3 sections + acknowledgments
  - [x] All text in English (no localization)

    **Settings help text (adapted from iOS):**
    - Training Range: "Set the **lowest** and **highest note** for your training. A wider range is more challenging. If you're just starting out, try a smaller range and expand it as your ear improves."
    - Intervals: "Intervals are the distance between two notes. Choose which intervals you want to practice. Start with a few and add more as you gain confidence."
    - Sound: "Pick the **sound** you want to train with — each instrument has a different character.\n\n**Duration** controls how long each note plays.\n\n**Concert Pitch** sets the reference tuning. Most musicians use 440 Hz. Some orchestras tune to 442 Hz.\n\n**Tuning System** determines how intervals are calculated. Equal Temperament divides the octave into 12 equal steps and is standard for most Western music. Just Intonation uses pure frequency ratios and sounds smoother for some intervals."
    - Difficulty: "**Vary Loudness** changes the volume of notes randomly. This makes training harder but more realistic — in real music, notes are rarely played at the same volume."
    - Data: "**Export** saves your training data as a file you can keep as a backup or transfer to another device.\n\n**Import** loads training data from a file. You can replace your current data or merge it with existing records.\n\n**Reset** permanently deletes all training data and resets your profile. This cannot be undone."

    **Comparison training help text:**
    - Goal: "Two notes play one after the other. Your job is to decide: was the **second note higher or lower** than the first? The closer the notes are, the harder it gets — and the sharper your ear becomes."
    - Controls: "After both notes have played, the **Higher** and **Lower** buttons become active. Tap the one that matches what you heard. You can also use keyboard shortcuts: **Arrow Up** or **H** for higher, **Arrow Down** or **L** for lower."
    - Feedback: "After each answer you'll see a brief **checkmark** (correct) or **X** (incorrect). Use this to calibrate your listening — over time, you'll notice patterns in what you get right."
    - Difficulty: "The difference between the two notes is measured in **cents** (1/100 of a semitone). A smaller number means a harder comparison. The app adapts difficulty to your skill level automatically."
    - Intervals: "In interval mode, the two notes are separated by a specific **musical interval** (like a fifth or an octave) instead of a small pitch difference. You still decide which note is higher — but now you're training your sense of musical distance."

    **Pitch matching help text:**
    - Goal: "You'll hear a **reference note**. Your goal is to match its pitch by sliding to the exact same frequency. The closer you get, the better your ear is becoming."
    - Controls: "**Touch** the slider to hear your note, then **drag** up or down to adjust the pitch. When you think you've matched the reference, **release** the slider to lock in your answer. You can also press **Enter** or **Space** to commit."
    - Feedback: "After each attempt, you'll see how many **cents** off you were. A smaller number means a closer match — zero would be perfect. Use the feedback to fine-tune your listening."
    - Intervals: "In interval mode, your target pitch is a specific **musical interval** away from the reference note. Instead of matching the same note, you're matching a note that's a certain distance above or below it."

    **Info view help text:**
    - What is Peach?: "Peach helps you train your ear for music. Practice hearing the difference between notes and learn to match pitches accurately."
    - Training Modes: "**Hear & Compare — Single Notes** — Listen to two notes and decide which one is higher.\n\n**Hear & Compare — Intervals** — The same idea, but with musical intervals between notes.\n\n**Tune & Match — Single Notes** — Hear a note and slide to match its pitch.\n\n**Tune & Match — Intervals** — Match pitches using musical intervals."
    - Getting Started: "Just pick any training mode on the home screen and start practicing. Peach adapts to your skill level automatically."
    - Acknowledgments: "Sounds provided by GeneralUser GS by S. Christian Collins."

- [x] Task 4: Integrate help into Settings view (AC: 4)
  - [x] Add help button ("?") to settings nav/header area
  - [x] Wire to HelpModal with settings help sections

- [x] Task 5: Integrate help into comparison_view.rs (AC: 5, 8)
  - [x] Add help button to comparison view header
  - [x] Wire to HelpModal with comparison help sections
  - [x] On help open: stop comparison session
  - [x] On help close: restart comparison session with same intervals

- [x] Task 6: Integrate help into pitch_matching_view.rs (AC: 6, 8)
  - [x] Add help button to pitch matching view header
  - [x] Wire to HelpModal with pitch matching help sections
  - [x] On help open: stop pitch matching session
  - [x] On help close: restart pitch matching session with same intervals

- [x] Task 7: Update info_view.rs (AC: 7)
  - [x] Add HelpContent inline (not modal) with info help sections
  - [x] Add acknowledgments section
  - [x] Keep existing app metadata (name, version, developer, license, GitHub link)
  - [x] Reorder: header → help content → acknowledgments → metadata

- [x] Task 8: Verify
  - [x] Manual test: open help on each screen, verify content renders
  - [x] Manual test: on training screens, verify training pauses/resumes with help
  - [x] Manual test: Escape closes help modal
  - [x] Manual test: info view shows inline help content
  - [x] Run `cargo clippy`

## Dev Notes

### iOS to Web Mapping

| iOS Element | peach-web Equivalent |
|---|---|
| `HelpContentView` (SwiftUI) | `HelpContent` Leptos component |
| `HelpSection` struct | `HelpSection` struct (`title: &'static str, body: &'static str`) |
| `.sheet(isPresented:)` | HTML `<dialog>` element with `showModal()` |
| `Button("Done")` dismiss | Button calling `dialog.close()` |
| `AttributedString(markdown:)` | Simple regex: `**bold**` → `<strong>`, `*italic*` → `<em>` |
| `.onChange(of: showHelpSheet)` stop/start | Leptos effect watching `is_help_open` signal |

### Design Decisions

- **HTML `<dialog>` element:** Native modal behavior with backdrop, focus trapping, and Escape-to-close. No custom modal implementation needed. Well-supported in all modern browsers.
- **Simple markdown processing:** The help text is static and trusted, so `inner_html` with simple regex replacements is safe and avoids pulling in a markdown parser dependency.
- **Static help text:** All help content is English-only, compile-time constant. No localization infrastructure needed.
- **Training pause/resume on help:** This matches iOS behavior and prevents confusing audio playing behind the modal.

### Architecture Compliance

- **Web crate only:** All components and text content live in the web crate.
- **No domain changes:** Help is purely UI.
- **Accessibility:** `<dialog>` provides native accessibility semantics. Additional ARIA attributes ensure screen reader compatibility.

## Dev Agent Record

### Implementation Notes

- Created `HelpSection` struct and `HelpContent` component with simple inline markdown processing (`**bold**` → `<strong>`, `*italic*` → `<em>`, `\n\n` → `<br><br>`)
- Created `HelpModal` component using native HTML `<dialog>` element with `showModal()`/`close()` — provides native focus trapping, backdrop, and Escape-to-close
- Help modal watches an `is_open: RwSignal<bool>` and syncs dialog state via a Leptos `Effect`
- Native `close` event on `<dialog>` handles Escape key dismissal, syncing the signal back
- Help content defined as static slices in `web/src/help_sections.rs` — all text matches iOS app content adapted for web
- Settings view: added "?" button next to heading, opens `HelpModal` with 5 sections
- Training views (comparison, pitch matching): "?" button stops the session (sets `cancelled`, calls `session.stop()`, stops audio), opens help modal. On close, re-navigates to the same route to remount the component and restart training fresh — matches iOS behavior where help sheet triggers `onDisappear`/`onAppear`
- Info view: `HelpContent` rendered inline (not modal) with 3 help sections + acknowledgments, followed by existing developer/project metadata
- Added 6 unit tests for `process_markdown` function covering bold, italic, combined, newlines, multiple occurrences, and plain text

### Completion Notes

All 8 tasks completed. All 10 acceptance criteria satisfied:
- AC1: HelpContent component renders sections with titles and body
- AC2: Simple markdown processing for bold, italic
- AC3: "?" button pattern on Settings, Comparison, Pitch Matching views opening modal
- AC4: Settings help with 5 sections
- AC5: Comparison help with 5 sections
- AC6: Pitch matching help with 4 sections
- AC7: Info view inline help with 3 sections + acknowledgments
- AC8: Training pauses on help open, restarts on close (via re-navigation)
- AC9: `<dialog>` with `role="dialog"`, `aria-modal="true"`, native focus trapping, Escape closes
- AC10: `space-y-5` (20px), `mt-2` (8px title-body gap), `font-semibold` titles, regular body

## File List

- `web/src/components/help_content.rs` (new) — HelpSection struct, HelpContent component, HelpModal component, process_markdown function, unit tests
- `web/src/help_sections.rs` (new) — Static help text content for all views
- `web/src/components/mod.rs` (modified) — Added `pub mod help_content`
- `web/src/main.rs` (modified) — Added `mod help_sections`
- `web/src/components/settings_view.rs` (modified) — Added help button and HelpModal
- `web/src/components/pitch_comparison_view.rs` (modified) — Added help button, HelpModal, pause/resume logic
- `web/src/components/pitch_matching_view.rs` (modified) — Added help button, HelpModal, pause/resume logic
- `web/src/components/info_view.rs` (modified) — Added inline HelpContent, reordered sections
- `docs/implementation-artifacts/sprint-status.yaml` (modified) — Status updated
- `docs/implementation-artifacts/7-7-help-content.md` (modified) — Story file updated

## Change Log

- 2026-03-06: Implemented help content system — reusable HelpContent/HelpModal components, help text for all views, training pause/resume on help, inline help on Info view
- 2026-03-06: Code review fixes: fix help close to re-navigate to training route instead of start page (AC8), fix HelpModal double on_close callback on Done click, fix heading level skip in info view (a11y), and mark done
