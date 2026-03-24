# Story 14.3: Update Living Documentation

Status: draft

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

6. **AC6 — Epics doc updated:** `docs/planning-artifacts/epics.md` — add Epic 14 with stories 14.1–14.4.

7. **AC7 — Sprint status updated:** `docs/implementation-artifacts/sprint-status.yaml` — add Epic 14 entries (stories 14.1–14.3).

8. **AC8 — NOT updated (historical records):**
   - `docs/ios-reference/domain-blueprint.md` — frozen reference
   - All completed story files (1-x through 13-x)
   - `docs/ios-reference/ios-changes-since-f70e3f.md` — reference doc, not living doc
   - `docs/ios-reference/domain-blueprint.md` — frozen reference

## Tasks / Subtasks

- [ ] Task 1: Update `docs/planning-artifacts/architecture.md`
- [ ] Task 2: Update `docs/planning-artifacts/prd.md`
- [ ] Task 3: Update `docs/planning-artifacts/ux-design-specification.md`
- [ ] Task 4: Update `docs/arc42-architecture.md`
- [ ] Task 5: Update `docs/project-context.md`
- [ ] Task 6: Add Epic 14 to `docs/planning-artifacts/epics.md`
- [ ] Task 7: Add Epic 14 to `docs/implementation-artifacts/sprint-status.yaml`

## Dev Notes

- Use find-and-replace carefully — some terms appear in historical context that should NOT change
- Focus on sections that describe the CURRENT architecture, not sections describing past decisions
- The living docs may also need updates for rhythm training later (Phases 4-7), but this story only covers terminology changes
