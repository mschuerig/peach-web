# Story 14.3: Update Living Documentation

Status: done

## Story

As a developer,
I want living documentation updated to reflect the new terminology,
so that planning and architecture docs remain accurate and useful for AI agents and contributors.

## Context

Stories 14.1–14.3 rename domain types and update CSV format. Living docs must reflect these changes. Historical docs (story files, domain blueprint) are NOT updated — they serve as records of what was true at the time.

## Acceptance Criteria

1. **AC1 — Architecture doc updated:** `docs/planning-artifacts/architecture.md` reflects new type names (`TrainingDiscipline`, `PitchDiscriminationTrial`, etc.) wherever current types are referenced.

2. **AC2 — PRD updated:** `docs/planning-artifacts/prd.md` uses new terminology in functional requirements that reference training modes/disciplines.

3. **AC3 — UX design spec updated:** `docs/planning-artifacts/ux-design-specification.md` uses new terminology for screen names and UI labels.

4. **AC4 — arc42 doc updated:** `docs/arc42-architecture.md` uses new type names in building block and runtime views.

5. **AC5 — Project context updated:** `docs/project-context.md` type references updated.

6. **AC6 — Epics doc updated:** `docs/planning-artifacts/epics.md` — add Epic 14 with stories 14.1–14.3.

7. **AC7 — Sprint status updated:** `docs/implementation-artifacts/sprint-status.yaml` — add Epic 14 entries (stories 14.1–14.3).

8. **AC8 — NOT updated (historical records):**
   - `docs/ios-reference/domain-blueprint.md` — frozen reference
   - All completed story files (1-x through 13-x)
   - `docs/ios-reference/ios-changes-since-f70e3f.md` — reference doc, not living doc
   - `docs/ios-reference/domain-blueprint.md` — frozen reference

## Tasks / Subtasks

- [x] Task 1: Update `docs/planning-artifacts/architecture.md`
- [x] Task 2: Update `docs/planning-artifacts/prd.md`
- [x] Task 3: Update `docs/planning-artifacts/ux-design-specification.md`
- [x] Task 4: Update `docs/arc42-architecture.md`
- [x] Task 5: Update `docs/project-context.md`
- [x] Task 6: Add Epic 14 to `docs/planning-artifacts/epics.md`
- [x] Task 7: Add Epic 14 to `docs/implementation-artifacts/sprint-status.yaml`

## Dev Notes

- Use find-and-replace carefully — some terms appear in historical context that should NOT change
- Focus on sections that describe the CURRENT architecture, not sections describing past decisions
- The living docs may also need updates for rhythm training later (Phases 4-7), but this story only covers terminology changes

## Dev Agent Record

### Implementation Plan

Documentation-only story — systematic find-and-replace of old terminology across all living docs, with careful exclusion of historical records.

### Completion Notes

All 7 tasks completed. Key changes across files:

**architecture.md:** ~20 edits — updated all type names (`ComparisonSession` → `PitchDiscriminationSession`, `CompletedComparison` → `CompletedPitchDiscriminationTrial`, etc.), file paths (`comparison_session.rs` → `pitch_discrimination_session.rs`), route paths, observer/trait names, naming examples.

**prd.md:** ~12 edits — "training modes" → "training disciplines", "comparison training" → "pitch discrimination training", button label "Comparison" → "Compare", FR descriptions updated.

**ux-design-specification.md:** ~30 edits — button labels ("Comparison" → "Compare", "Pitch Matching" → "Match"), route paths, view names (`ComparisonView` → `PitchDiscriminationView`), loop mechanics section renamed, all "comparison" → "trial" in appropriate contexts.

**arc42-architecture.md:** ~10 edits — building block tables, sequence diagrams, port/adapter mapping, glossary entries.

**project-context.md:** Already up-to-date from prior stories. IndexedDB store name `comparison_records` kept as-is (matches actual code — store name was intentionally not renamed in 14.2).

**epics.md:** Added Epic 14 section with stories 14.1–14.3. Old epic/story text preserved as historical record per AC8.

**sprint-status.yaml:** Added `14-3-update-living-docs: in-progress` entry under Epic 14.

### Debug Log

No issues encountered.

## File List

- `docs/planning-artifacts/architecture.md` (modified)
- `docs/planning-artifacts/prd.md` (modified)
- `docs/planning-artifacts/ux-design-specification.md` (modified)
- `docs/arc42-architecture.md` (modified)
- `docs/planning-artifacts/epics.md` (modified)
- `docs/implementation-artifacts/sprint-status.yaml` (modified)
- `docs/implementation-artifacts/14-3-update-living-docs.md` (modified)

## Change Log

- 2026-03-24: Updated all living documentation to reflect iOS terminology alignment (Story 14.3)
