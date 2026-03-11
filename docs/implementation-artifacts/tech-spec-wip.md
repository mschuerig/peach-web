---
title: 'Fix help text line breaks using Fluent multiline syntax'
slug: 'fix-help-text-line-breaks'
created: '2026-03-11'
status: 'in-progress'
stepsCompleted: [1]
tech_stack: ['Rust', 'Leptos', 'Fluent (i18n)']
files_to_modify:
  - 'web/locales/en/main.ftl'
  - 'web/locales/de/main.ftl'
  - 'web/src/components/help_content.rs'
code_patterns: ['Fluent multiline syntax', 'process_markdown']
test_patterns: ['unit tests in help_content.rs']
---

# Tech-Spec: Fix help text line breaks using Fluent multiline syntax

**Created:** 2026-03-11

## Overview

### Problem Statement

Help texts on the start page (info) and settings page display literal `\n\n` instead of line breaks. The FTL locale files use `{"\\n\\n"}` (Fluent string literal syntax) which outputs the literal characters `\n\n`. The `process_markdown()` function tries to replace actual newline characters but never matches these literals.

### Solution

Replace the `{"\\n\\n"}` hacks in FTL files with proper Fluent multiline syntax (indented continuation lines with blank lines for paragraph breaks). Remove the now-unnecessary `\n\n` → `<br><br>` replacement from `process_markdown()` and update its test.

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
- Help text rendered via `inner_html` with `process_markdown()` for inline formatting
- `HelpContent` and `HelpModal` components in `help_content.rs`
- Help section definitions in `help_sections.rs`

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `web/locales/en/main.ftl` | English translations — 3 keys to fix |
| `web/locales/de/main.ftl` | German translations — 3 keys to fix |
| `web/src/components/help_content.rs` | `process_markdown()` function and tests |

### Technical Decisions

- Use Fluent's native multiline syntax instead of `{"\\n\\n"}` escape hacks — produces real newline characters in output
- Keep `process_markdown()`'s `\n\n` → `<br><br>` replacement — it's needed because `inner_html` ignores raw newlines in HTML; the replacement now actually works since Fluent multiline delivers real `\n` characters

## Implementation Plan

### Tasks

1. **`web/locales/en/main.ftl`** — Convert `help-sound-body`, `help-data-body`, `help-info-modes-body` from single-line with `{"\\n\\n"}` to Fluent multiline syntax (indented continuation lines with blank lines for paragraph breaks)
2. **`web/locales/de/main.ftl`** — Same conversions for German translations
3. **Verify** — `cargo test -p web` and `cargo clippy --workspace`

### Acceptance Criteria

- **Given** the start page info help is displayed, **When** viewing `help-info-modes-body`, **Then** paragraph breaks appear between training modes (no literal `\n\n` visible)
- **Given** the settings help modal is open, **When** viewing `help-sound-body`, **Then** paragraph breaks appear between Sound, Duration, Concert Pitch, and Tuning System sections
- **Given** the settings help modal is open, **When** viewing `help-data-body`, **Then** paragraph breaks appear between Export, Import, and Reset sections
- **Given** the same pages in German locale, **When** viewing the same help texts, **Then** identical paragraph break behavior
- **Given** `process_markdown()` receives text with `\n\n`, **When** processing, **Then** newlines are converted to `<br><br>` as before
- **Given** `cargo test -p web`, **When** running tests, **Then** all existing tests pass

## Additional Context

### Dependencies

None — pure locale file and rendering fix.

### Testing Strategy

- Unit tests: existing `test_process_markdown_newlines` in `help_content.rs` remains valid
- Manual: check all 3 help texts in both English and German in browser

### Notes

- All other help body texts are single-line and don't use any line break mechanism — no adaptation needed
- `process_markdown()` is unchanged — bold, italic, and newline processing all remain as-is
