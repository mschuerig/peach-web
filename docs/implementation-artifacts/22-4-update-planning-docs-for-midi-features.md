# Story 22.4: Update Planning Docs for MIDI Features

Status: review

## Story

As a developer,
I want living documentation updated to reflect the MIDI input features added in Epic 22,
so that planning and architecture docs remain accurate for AI agents and contributors.

## Context

Stories 22.1–22.3 added Web MIDI API integration: note-on detection for rhythm tapping, MIDI wiring into continuous rhythm matching, and pitch bend control for pitch matching. None of these features are reflected in the planning docs yet. This story follows the same pattern as Story 14.3 (Update Living Docs) — update living docs only, leave historical records untouched.

Key features implemented that docs must reflect:
- **Web MIDI API adapter** (`web/src/adapters/midi_input.rs`) — feature detection, note-on parsing, pitch bend parsing, listener setup/cleanup
- **MIDI note-on as rhythm tap input** — progressive enhancement alongside pointer/keyboard
- **MIDI pitch bend as pitch matching input** — maps to slider's `[-1.0, +1.0]` range, auto-starts note, commits on return-to-center
- **Progressive enhancement pattern** — MIDI unavailable or denied → silent fallback, no error UI

## Acceptance Criteria

1. **AC1 — PRD updated:** `docs/planning-artifacts/prd.md` adds MIDI input as a supported input method. Add functional requirements for MIDI note-on (rhythm tap) and MIDI pitch bend (pitch matching) input. Update FR41–FR46 (Input & Accessibility) section to mention MIDI controller input alongside pointer/keyboard/touch.

2. **AC2 — Architecture doc updated:** `docs/planning-artifacts/architecture.md` documents:
   - The `midi_input.rs` adapter module in the project structure and file tree
   - Web MIDI API as a browser API dependency (feature-detected, progressive enhancement)
   - MIDI event flow: `midimessage` → adapter parsing → existing tap/slider pipeline
   - `web-sys` MIDI feature flags (`MidiAccess`, `MidiInput`, `MidiInputMap`, `MidiMessageEvent`, `MidiOptions`, `MidiPort`, `MidiConnectionEvent`)
   - `MidiCleanupHandle` cleanup pattern

3. **AC3 — UX design spec updated:** `docs/planning-artifacts/ux-design-specification.md` mentions MIDI as an additional input method for rhythm training (tap) and pitch matching (pitch bend wheel). Update multi-input interaction design challenge to include MIDI. No new screens — this is progressive enhancement.

4. **AC4 — arc42 doc updated:** `docs/arc42-architecture.md` adds MIDI adapter to the building block view and updates the runtime view to show MIDI event flow where applicable.

5. **AC5 — Project context updated:** `docs/project-context.md` adds:
   - Web MIDI API to the technology stack (progressive enhancement, not required)
   - `midi_input.rs` to any file/module listings
   - MIDI-specific patterns: `MidiCleanupHandle`, `is_midi_available()` guard, non-blocking setup with `spawn_local`

6. **AC6 — Epics doc updated:** `docs/planning-artifacts/epics.md` adds Story 22.3 (MIDI Pitch Bend for Pitch Matching) to Epic 22. Currently only Stories 22.1 and 22.2 are documented in the epics file.

7. **AC7 — Sprint status updated:** `docs/implementation-artifacts/sprint-status.yaml` adds `22-4-update-planning-docs-for-midi-features` entry under Epic 22.

8. **AC8 — NOT updated (historical records):**
   - `docs/ios-reference/` — frozen reference docs
   - Completed story files (22-1, 22-2, 22-3) — historical records
   - Research docs (`docs/planning-artifacts/research/`) — point-in-time research

## Tasks / Subtasks

- [x] Task 1: Update `docs/planning-artifacts/prd.md` (AC: 1)
  - [x] 1.1 Add MIDI controller input to the Input & Accessibility FRs (FR41–FR46 area)
  - [x] 1.2 Add FR for MIDI note-on as rhythm tap input (progressive enhancement)
  - [x] 1.3 Add FR for MIDI pitch bend as pitch matching input
  - [x] 1.4 Mention MIDI in product scope or any relevant phase descriptions

- [x] Task 2: Update `docs/planning-artifacts/architecture.md` (AC: 2)
  - [x] 2.1 Add `midi_input.rs` to the project directory structure listing under `web/src/adapters/`
  - [x] 2.2 Document Web MIDI API as a browser dependency (feature-detected)
  - [x] 2.3 Add `web-sys` MIDI feature flags to any dependency listing
  - [x] 2.4 Document MIDI event flow (adapter → existing tap/slider pipeline)
  - [x] 2.5 Add `MidiCleanupHandle` to cleanup/lifecycle patterns if documented
  - [x] 2.6 Update FR-to-file mapping if new MIDI FRs are added in Task 1

- [x] Task 3: Update `docs/planning-artifacts/ux-design-specification.md` (AC: 3)
  - [x] 3.1 Add MIDI to multi-input interaction design challenge
  - [x] 3.2 Mention MIDI controller input in rhythm training and pitch matching sections
  - [x] 3.3 Note progressive enhancement — no new UI needed, no MIDI settings

- [x] Task 4: Update `docs/arc42-architecture.md` (AC: 4)
  - [x] 4.1 Add MIDI adapter to building block view / adapter listing
  - [x] 4.2 Update runtime view to mention MIDI event flow where relevant

- [x] Task 5: Update `docs/project-context.md` (AC: 5)
  - [x] 5.1 Add Web MIDI API to technology stack (with progressive enhancement note)
  - [x] 5.2 Add `midi_input.rs` to any module/file listings
  - [x] 5.3 Add MIDI patterns: `MidiCleanupHandle`, `is_midi_available()` guard, non-blocking setup

- [x] Task 6: Add Story 22.3 to `docs/planning-artifacts/epics.md` (AC: 6)
  - [x] 6.1 Add Story 22.3 (MIDI Pitch Bend for Pitch Matching) to Epic 22 section, following the BDD format of existing stories

- [x] Task 7: Update sprint status (AC: 7)
  - [x] 7.1 Add `22-4-update-planning-docs-for-midi-features` entry to sprint-status.yaml
  - [x] 7.2 Re-open `epic-22` status to `in-progress`

## Dev Notes

### Scope — Living Docs Only

This is a documentation-only story. No code changes. Follow the same approach as Story 14.3: update docs that describe the CURRENT system, leave historical records untouched.

### What Was Implemented in Epic 22

**Story 22.1 — MIDI Adapter Module:**
- `web/src/adapters/midi_input.rs` — new module
- `is_midi_available()` — browser feature detection via `Navigator.request_midi_access`
- `is_note_on(data: &[u8]) -> bool` — status `0x90`–`0x9F`, velocity > 0
- `setup_midi_listeners(on_note_on)` → `MidiCleanupHandle`
- `web-sys` features: `MidiAccess`, `MidiInput`, `MidiInputMap`, `MidiMessageEvent`, `MidiOptions`, `MidiPort`, `MidiConnectionEvent`, `Navigator`

**Story 22.2 — MIDI Wiring for Rhythm Training:**
- MIDI note-on events feed into the same tap pipeline as pointer/keyboard: `bridge_event_to_audio_time` → `evaluate_tap` → `RhythmOffset`
- Setup after AudioContext resume (user gesture)
- Failure → `log::warn!`, training continues without MIDI
- Cleanup handle stored in `StoredValue::new_local(SendWrapper::new(handle))`

**Story 22.3 — MIDI Pitch Bend for Pitch Matching:**
- `is_pitch_bend(data: &[u8]) -> bool` — status `0xE0`–`0xEF`
- `parse_pitch_bend(data: &[u8]) -> f64` — 14-bit value normalized to `[-1.0, +1.0]`
- `setup_midi_pitch_bend_listeners(on_pitch_bend)` → `MidiCleanupHandle`
- Pitch bend drives the same slider pipeline: `adjust_pitch()`, auto-start on first deflection, commit on return-to-center (±3.125% dead-zone)
- `VerticalPitchSlider` gained `external_value: Option<Signal<f64>>` prop
- All web-layer only, no domain crate changes

### Key MIDI Architecture Pattern

```
MIDI Controller → Web MIDI API → midimessage event
  → midi_input.rs (parse note-on / pitch bend)
    → existing on_tap / slider_on_change closure
      → domain session (evaluate_tap / adjust_pitch)
        → UIObserver → signal updates → re-render
```

Progressive enhancement: `is_midi_available()` → setup or skip. Failure → warn + continue.

### Files to Update

| File | What to Add |
|---|---|
| `prd.md` | MIDI FRs for note-on tap and pitch bend input |
| `architecture.md` | `midi_input.rs` in file tree, Web MIDI API dependency, MIDI event flow, feature flags |
| `ux-design-specification.md` | MIDI as input method for rhythm + pitch matching |
| `arc42-architecture.md` | MIDI adapter in building blocks, event flow in runtime view |
| `project-context.md` | Web MIDI in tech stack, MIDI patterns, `midi_input.rs` in module listing |
| `epics.md` | Story 22.3 BDD acceptance criteria |
| `sprint-status.yaml` | Story 22.4 entry, epic-22 reopened |

### Anti-Patterns to Avoid

- Do NOT update completed story files (22-1, 22-2, 22-3) — they are historical records
- Do NOT update `docs/ios-reference/` — frozen reference
- Do NOT add MIDI as a required dependency — it is strictly progressive enhancement
- Do NOT invent new UI for MIDI — there are no MIDI-specific screens or settings
- Do NOT add MIDI to the domain crate description — all MIDI code is web-layer only

### References

- [Source: docs/implementation-artifacts/22-1-midi-adapter-module-with-note-on-detection.md] — MIDI adapter implementation details
- [Source: docs/implementation-artifacts/22-2-wire-midi-input-into-continuous-rhythm-matching-view.md] — MIDI wiring for rhythm training
- [Source: docs/implementation-artifacts/22-3-midi-pitch-bend-for-pitch-matching.md] — Pitch bend implementation
- [Source: docs/implementation-artifacts/14-3-update-living-docs.md] — Prior living docs update pattern
- [Source: docs/planning-artifacts/epics.md#Epic 22] — Current epic definition (missing Story 22.3)
- [Source: docs/project-context.md] — Current project context (no MIDI input references)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

None — documentation-only story, no code changes.

### Completion Notes List

- Updated PRD with FR50-52 (MIDI note-on tap, pitch bend slider, feature detection fallback) and MIDI mention in executive summary and journey requirements
- Updated architecture doc: added `midi_input.rs` to file tree, new "Web MIDI API" section documenting feature detection, adapter, event flow, cleanup pattern, and feature flags. Added MIDI FR-to-file mapping row.
- Updated UX design spec: expanded multi-input design challenge and platform strategy to include MIDI controller, added MIDI keyboard shortcuts table entries and progressive enhancement note
- Updated arc42: added MIDI adapter to building block view, new runtime sequence diagram (6.4) for MIDI event flow, updated business context
- Updated project-context.md: Web MIDI API in tech stack, `midi_input.rs` in adapter listing, full MIDI patterns section with cleanup handle, guard, feature flags
- Added Story 22.3 (MIDI Pitch Bend for Pitch Matching) to epics.md with full BDD acceptance criteria
- Sprint status entry already existed; epic-22 already in-progress
- AC8 verified: no changes to ios-reference, completed stories, or research docs

### Change Log

- 2026-03-27: All 7 tasks completed — living docs updated for Epic 22 MIDI features

### File List

- docs/planning-artifacts/prd.md (modified)
- docs/planning-artifacts/architecture.md (modified)
- docs/planning-artifacts/ux-design-specification.md (modified)
- docs/arc42-architecture.md (modified)
- docs/project-context.md (modified)
- docs/planning-artifacts/epics.md (modified)
- docs/implementation-artifacts/sprint-status.yaml (modified)
- docs/implementation-artifacts/22-4-update-planning-docs-for-midi-features.md (modified)
