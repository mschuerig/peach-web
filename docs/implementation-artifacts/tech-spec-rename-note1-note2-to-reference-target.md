---
title: 'Rename note1/note2 to referenceNote/targetNote'
slug: 'rename-note1-note2-to-reference-target'
created: '2026-03-04'
status: 'completed'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['Rust', 'Leptos']
files_to_modify:
  - 'domain/src/session/comparison_session.rs'
  - 'web/src/components/comparison_view.rs'
  - 'web/src/adapters/audio_oscillator.rs'
  - 'docs/planning-artifacts/epics.md'
  - 'docs/planning-artifacts/architecture.md'
  - 'docs/planning-artifacts/ux-design-specification.md'
  - 'docs/project-context.md'
code_patterns: ['enum variant rename', 'method rename', 'assert message update']
test_patterns: ['inline #[cfg(test)] mod tests']
---

# Tech-Spec: Rename note1/note2 to referenceNote/targetNote

**Created:** 2026-03-04

## Overview

### Problem Statement

The codebase has terminology drift: the original iOS app used `note1`/`note2` for sequential playback phases, but the canonical names are now `referenceNote`/`targetNote`. The data model layer already uses the correct names (`reference_note`, `target_note`), but the session state machine and UI code still use the old `Note1`/`Note2` naming in enum variants, methods, comments, and log messages. Planning docs also carry the old terminology forward.

### Solution

Rename all `note1`/`note2` occurrences in code and upcoming planning docs to use `reference_note`/`target_note` equivalents, aligning the session state machine layer with the rest of the codebase. Historical implementation stories (already completed) are left untouched.

### Scope

**In Scope:**
- Rename `ComparisonSessionState` enum variants: `PlayingNote1` → `PlayingReferenceNote`, `PlayingNote2` → `PlayingTargetNote`
- Rename methods: `on_note1_finished()` → `on_reference_note_finished()`, `on_note2_finished()` → `on_target_note_finished()`
- Update assert messages, comments, and log messages in domain and web crates
- Update planning docs: `epics.md`, `architecture.md`, `ux-design-specification.md`, `project-context.md`

**Out of Scope:**
- Historical implementation stories (1-6, 1-7, and earlier) — already completed, left as-is
- `claude-audit/` files — historical review artifacts
- `ios-reference/` files — original reference material
- Data model types (`Comparison`, `ComparisonRecord`, etc.) — already use correct naming
- `ComparisonPlaybackData` fields — already use `reference_frequency`/`target_frequency`

## Context for Development

### Codebase Patterns

- Enum variants use `PascalCase`: `PlayingReferenceNote`, `PlayingTargetNote`
- Methods use `snake_case`: `on_reference_note_finished()`, `on_target_note_finished()`
- Assert messages reference the method and required state (e.g. `"on_reference_note_finished() requires PlayingReferenceNote state"`)
- Domain crate has inline `#[cfg(test)] mod tests` — ~30 tests reference `PlayingNote1`/`PlayingNote2`/`on_note1_finished`/`on_note2_finished`
- Planning docs use `camelCase` for state names in prose (e.g. `playingReferenceNote`)

### Files to Reference

| File | Purpose | Change Count |
| ---- | ------- | ------------ |
| `domain/src/session/comparison_session.rs` | Enum variants (L23-24), methods (L156-173), asserts, comments, ~30 test refs | ~50 |
| `web/src/components/comparison_view.rs` | State checks (L98, L139, L402), method calls (L367, L403), comments, log msgs | ~13 |
| `docs/planning-artifacts/epics.md` | AC state references in epic 4 stories (L518-584) | 7 |
| `docs/planning-artifacts/architecture.md` | Naming convention example (L304) | 1 |
| `docs/planning-artifacts/ux-design-specification.md` | State table (L454-455), button state (L828) | 3 |
| `docs/project-context.md` | Naming convention example (L156) | 1 |

### Technical Decisions

- Pure rename/refactor — no behavioral changes
- Compiler will catch all Rust code references via enum variant and method renames — zero risk of missed code sites
- Planning docs require manual search-and-replace — grep-verified exhaustive list above
- Test function names updated where they embed old terminology (e.g. `test_on_note1_finished_*` → `test_on_reference_note_finished_*`)
- `claude-audit/` files, `ios-reference/` files, stories 1-6 and 1-7 are explicitly excluded as historical

## Implementation Plan

### Tasks

Task order: domain code first (compiler catches downstream breakage), then web code, then docs.

- [x] Task 1: Rename enum variants in `ComparisonSessionState`
  - File: `domain/src/session/comparison_session.rs`
  - Action: Replace `PlayingNote1` → `PlayingReferenceNote`, `PlayingNote2` → `PlayingTargetNote` in the enum definition (L22-27)
  - Notes: This is the anchor change — the compiler will flag every usage site

- [x] Task 2: Rename methods and update assert messages
  - File: `domain/src/session/comparison_session.rs`
  - Action:
    - Rename `on_note1_finished()` → `on_reference_note_finished()` (L156)
    - Update its assert message: `"on_reference_note_finished() requires PlayingReferenceNote state"`
    - Update its doc comment: `"Called when reference note playback finishes. Transitions to PlayingTargetNote."`
    - Rename `on_note2_finished()` → `on_target_note_finished()` (L165)
    - Update its assert message: `"on_target_note_finished() requires PlayingTargetNote state"`
    - Update its doc comment: `"Called when target note playback finishes. Transitions to AwaitingAnswer."`
  - Notes: Also update any comments in `start()`, `handle_answer()`, `on_feedback_finished()` that reference old names

- [x] Task 3: Update all remaining references in domain code and comments
  - File: `domain/src/session/comparison_session.rs`
  - Action:
    - L126 comment: update `"transitions to PlayingNote1"` → `"transitions to PlayingReferenceNote"`
    - L152: `self.state = ComparisonSessionState::PlayingNote1` → `PlayingReferenceNote`
    - L162: `self.state = ComparisonSessionState::PlayingNote2` → `PlayingTargetNote`
    - L177 comment: `"Valid from PlayingNote2"` → `"Valid from PlayingTargetNote"`
    - L181: state check `PlayingNote2` → `PlayingTargetNote`
    - L183: assert message update to reference `PlayingTargetNote`
    - L229 comment: `"transitions to PlayingNote1"` → `"transitions to PlayingReferenceNote"`
    - L238: `self.state = ComparisonSessionState::PlayingNote1` → `PlayingReferenceNote`

- [x] Task 4: Update test code in domain crate
  - File: `domain/src/session/comparison_session.rs`
  - Action: In `#[cfg(test)] mod tests`, update all occurrences:
    - `ComparisonSessionState::PlayingNote1` → `ComparisonSessionState::PlayingReferenceNote` (all assert_eq! calls)
    - `ComparisonSessionState::PlayingNote2` → `ComparisonSessionState::PlayingTargetNote` (all assert_eq! calls)
    - `session.on_note1_finished()` → `session.on_reference_note_finished()` (all call sites)
    - `session.on_note2_finished()` → `session.on_target_note_finished()` (all call sites)
    - Rename test functions:
      - `test_start_transitions_to_playing_note1` → `test_start_transitions_to_playing_reference_note`
      - `test_on_note1_finished_transitions_to_playing_note2` → `test_on_reference_note_finished_transitions_to_playing_target_note`
      - `test_on_note2_finished_transitions_to_awaiting_answer` → `test_on_target_note_finished_transitions_to_awaiting_answer`
      - `test_on_note1_finished_from_idle_panics` → `test_on_reference_note_finished_from_idle_panics`
      - `test_on_note2_finished_from_idle_panics` → `test_on_target_note_finished_from_idle_panics`
      - `test_handle_answer_from_playing_note1_panics` → `test_handle_answer_from_playing_reference_note_panics`
      - `test_early_answer_from_playing_note2` → `test_early_answer_from_playing_target_note`
    - Update `#[should_panic(expected = ...)]` strings to match new method/state names
    - Update test comments referencing old names (e.g. `"Already in PlayingNote1"`, `"Answer while still playing note 2"`)

- [x] Task 5: Update web crate references
  - File: `web/src/components/comparison_view.rs`
  - Action:
    - L98: `ComparisonSessionState::PlayingNote2` → `PlayingTargetNote`
    - L139: `ComparisonSessionState::PlayingNote2` → `PlayingTargetNote`
    - L343 comment: `"PlayingNote1 phase"` → `"PlayingReferenceNote phase"`
    - L351 log message: `"Note1 playback failed"` → `"Reference note playback failed"`
    - L353 comment: `"Wait for note1 duration"` → `"Wait for reference note duration"`
    - L366 comment: `"Transition: PlayingNote1 → PlayingNote2"` → `"Transition: PlayingReferenceNote → PlayingTargetNote"`
    - L367: `session.borrow_mut().on_note1_finished()` → `on_reference_note_finished()`
    - L370 comment: `"PlayingNote2 phase"` → `"PlayingTargetNote phase"`
    - L377 log message: `"Note2 playback failed"` → `"Target note playback failed"`
    - L379 comment: `"Wait for note2 duration"` → `"Wait for target note duration"`
    - L396 comment: `"stop note2 audio"` → `"stop target note audio"`
    - L402: `ComparisonSessionState::PlayingNote2` → `PlayingTargetNote`
    - L403: `session.borrow_mut().on_note2_finished()` → `on_target_note_finished()`

- [x] Task 6: Verify code compiles and tests pass
  - Action: Run `cargo test -p domain` and `cargo clippy`
  - Notes: If any compilation errors remain, they indicate missed rename sites — fix them

- [x] Task 7: Update `epics.md`
  - File: `docs/planning-artifacts/epics.md`
  - Action: Replace all state references in upcoming stories:
    - L518: `playingNote1` → `playingReferenceNote`
    - L527: `playingNote1` → `playingReferenceNote`
    - L529: `playingNote2` → `playingTargetNote`
    - L531: `playingNote2` → `playingTargetNote`
    - L535: `playingNote2` → `playingTargetNote`
    - L580: `playingNote1` → `playingReferenceNote`
    - L584: `playingNote2` → `playingTargetNote`

- [x] Task 8: Update `architecture.md`
  - File: `docs/planning-artifacts/architecture.md`
  - Action: L304: Replace `PlayingNote1` → `PlayingReferenceNote` in naming convention example

- [x] Task 9: Update `ux-design-specification.md`
  - File: `docs/planning-artifacts/ux-design-specification.md`
  - Action:
    - L454: `playingNote1` → `playingReferenceNote`
    - L455: `playingNote2` → `playingTargetNote`
    - L828: `playingNote1` → `playingReferenceNote`

- [x] Task 10: Update `project-context.md`
  - File: `docs/project-context.md`
  - Action: L156: Replace `PlayingNote1` → `PlayingReferenceNote` in naming convention example

### Acceptance Criteria

- [x] AC1: Given the domain crate, when `cargo test -p domain` is run, then all tests pass with the renamed enum variants and methods.
- [x] AC2: Given the full workspace, when `cargo clippy` is run, then no warnings or errors are produced.
- [x] AC3: Given the domain source, when grepping for `Note1` or `Note2` (case-sensitive) in `domain/src/`, then zero matches are found.
- [x] AC4: Given the web source, when grepping for `Note1` or `Note2` (case-sensitive) in `web/src/`, then zero matches are found. Also zero matches for `note1` or `note2` in comments/log messages.
- [x] AC5: Given the four planning docs (`epics.md`, `architecture.md`, `ux-design-specification.md`, `project-context.md`), when grepping for `Note1`, `Note2`, `playingNote1`, `playingNote2`, `on_note1`, or `on_note2`, then zero matches are found.
- [x] AC6: Given historical docs (`docs/implementation-artifacts/1-*.md`, `docs/claude-audit/`, `docs/ios-reference/`), when comparing to their git-committed state, then zero changes have been made (no historical docs modified).

## Additional Context

### Dependencies

None — pure rename, no new crates or features.

### Testing Strategy

- `cargo test -p domain` — all ~30+ tests must pass with renamed variants/methods
- `cargo clippy` — must pass clean with no warnings
- Grep verification for AC3-AC6 to confirm completeness
- No manual browser testing needed — this is a pure rename with no behavioral change

### Notes

- The compiler enforces completeness for code: renaming the enum variants will cause compile errors at every usage site, ensuring nothing is missed.
- Planning docs require manual search-and-replace — the grep-verified line numbers above are the exhaustive list.
- Future stories/specs should use `PlayingReferenceNote`/`PlayingTargetNote` and `on_reference_note_finished()`/`on_target_note_finished()` exclusively.

## Review Notes

- Adversarial review completed
- Findings: 12 total, 3 fixed, 9 skipped (noise/undecided)
- Resolution approach: auto-fix
- Fixed: added `audio_oscillator.rs` to `files_to_modify`, added missing test rename to Task 4 enumeration
