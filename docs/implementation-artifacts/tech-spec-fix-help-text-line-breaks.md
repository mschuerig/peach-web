---
title: 'Fix help text line breaks using Fluent multiline syntax'
slug: 'fix-help-text-line-breaks'
created: '2026-03-11'
status: 'ready-for-dev'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['Rust', 'Leptos 0.8', 'Project Fluent FTL (i18n)', 'leptos_fluent']
files_to_modify:
  - 'web/locales/en/main.ftl'
  - 'web/locales/de/main.ftl'
code_patterns: ['Fluent multiline block syntax (indented continuation, blank lines preserved)', 'process_markdown inner_html rendering']
test_patterns: ['inline #[cfg(test)] mod tests in help_content.rs']
---

# Tech-Spec: Fix help text line breaks using Fluent multiline syntax

**Created:** 2026-03-11

## Overview

### Problem Statement

Help texts on the start page (info) and settings page display literal `\n\n` instead of line breaks. The FTL locale files use `{"\\n\\n"}` (Fluent string literal syntax) which outputs the literal characters `\n\n`. The `process_markdown()` function tries to replace actual newline characters but never matches these literals.

### Solution

Replace the `{"\\n\\n"}` hacks in FTL files with proper Fluent multiline block syntax. Fluent multiline preserves blank lines between content as real `\n\n` characters, so `process_markdown()`'s existing `\n\n` → `<br><br>` replacement will work correctly without any code changes.

### Scope

**In Scope:**

- Fix 3 translation keys in both `en/main.ftl` and `de/main.ftl`: `help-sound-body`, `help-data-body`, `help-info-modes-body`

**Out of Scope:**

- Other help text keys (they don't use line breaks)
- Other markdown processing features (bold, italic)
- Help rendering component changes

## Context for Development

### Codebase Patterns

- Fluent `.ftl` files in `web/locales/{lang}/main.ftl` for i18n
- Help text rendered via `inner_html` with `process_markdown()` for inline formatting (`**bold**` → `<strong>`, `*italic*` → `<em>`, `\n\n` → `<br><br>`)
- `HelpContent` and `HelpModal` components in `help_content.rs`
- Help section definitions in `help_sections.rs` (static arrays of `HelpSection` structs with i18n keys)
- All current help body texts are single-line in FTL; only 3 keys use `{"\\n\\n"}` for paragraph breaks

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `web/locales/en/main.ftl:146,150,178` | English translations — 3 keys with `{"\\n\\n"}` |
| `web/locales/de/main.ftl:146,150,178` | German translations — 3 keys with `{"\\n\\n"}` |
| `web/src/components/help_content.rs:13-54` | `process_markdown()` — no changes needed |
| `web/src/help_sections.rs` | Help section definitions — no changes needed |

### Technical Decisions

- Use Fluent's native multiline block syntax instead of `{"\\n\\n"}` escape hacks
- Fluent multiline: start value on new line, indent all continuation lines by at least one space. Blank lines between content are preserved as real `\n\n` in the output ([Fluent Multiline Guide](https://projectfluent.org/fluent/guide/multiline.html))
- `process_markdown()` remains unchanged — its `\n\n` → `<br><br>` replacement now actually works since Fluent multiline delivers real newline characters (previously it never matched the literal `\n\n` text)

## Implementation Plan

### Tasks

- [ ] Task 1: Convert `help-sound-body` in `web/locales/en/main.ftl`
  - File: `web/locales/en/main.ftl` (line 146)
  - Action: Replace the single-line value containing `{"\\n\\n"}` with a Fluent multiline block. Start value on next line, indent each paragraph by 4 spaces, separate paragraphs with a blank line.
  - Before: `help-sound-body = Pick the **sound**...{"\\n\\n"}**Duration** controls...{"\\n\\n"}**Concert Pitch**...{"\\n\\n"}**Tuning System**...`
  - After:
    ```
    help-sound-body =
        Pick the **sound** you want to train with — each instrument has a different character.

        **Duration** controls how long each note plays.

        **Concert Pitch** sets the reference tuning. Most musicians use 440 Hz. Some orchestras tune to 442 Hz.

        **Tuning System** determines how intervals are calculated. Equal Temperament divides the octave into 12 equal steps and is standard for most Western music. Just Intonation uses pure frequency ratios and sounds smoother for some intervals.
    ```

- [ ] Task 2: Convert `help-data-body` in `web/locales/en/main.ftl`
  - File: `web/locales/en/main.ftl` (line 150)
  - Action: Same multiline conversion. 3 paragraphs (Export, Import, Reset).

- [ ] Task 3: Convert `help-info-modes-body` in `web/locales/en/main.ftl`
  - File: `web/locales/en/main.ftl` (line 178)
  - Action: Same multiline conversion. 4 paragraphs (one per training mode).

- [ ] Task 4: Convert `help-sound-body` in `web/locales/de/main.ftl`
  - File: `web/locales/de/main.ftl` (line 146)
  - Action: Same multiline conversion as Task 1, German text.

- [ ] Task 5: Convert `help-data-body` in `web/locales/de/main.ftl`
  - File: `web/locales/de/main.ftl` (line 150)
  - Action: Same multiline conversion as Task 2, German text.

- [ ] Task 6: Convert `help-info-modes-body` in `web/locales/de/main.ftl`
  - File: `web/locales/de/main.ftl` (line 178)
  - Action: Same multiline conversion as Task 3, German text.

- [ ] Task 7: Verify build and tests
  - Run: `cargo clippy --workspace` and `cargo test -p web`
  - Confirm no regressions.

### Acceptance Criteria

- [ ] AC 1: Given the start page info help is displayed, when viewing `help-info-modes-body`, then paragraph breaks appear between training modes (no literal `\n\n` visible)
- [ ] AC 2: Given the settings help modal is open, when viewing `help-sound-body`, then paragraph breaks appear between Sound, Duration, Concert Pitch, and Tuning System sections
- [ ] AC 3: Given the settings help modal is open, when viewing `help-data-body`, then paragraph breaks appear between Export, Import, and Reset sections
- [ ] AC 4: Given the same pages in German locale, when viewing the same help texts, then identical paragraph break behavior
- [ ] AC 5: Given `process_markdown()` receives text with `\n\n`, when processing, then newlines are converted to `<br><br>` as before
- [ ] AC 6: Given `cargo test -p web`, when running tests, then all existing tests pass

## Additional Context

### Dependencies

None — pure locale file fix.

### Testing Strategy

- Unit tests: existing `test_process_markdown_newlines` in `help_content.rs` validates `\n\n` → `<br><br>` — remains valid, no changes needed
- Manual: check all 3 help texts in both English and German in browser (deferred to user)

### Notes

- All other help body texts are single-line and don't use any line break mechanism — no adaptation needed
- `process_markdown()` is unchanged — bold, italic, and newline processing all remain as-is
- The `{"\\n\\n"}` Fluent syntax outputs the literal 4-character string `\n\n` — it does NOT produce newline characters. This was the root cause of the bug.
