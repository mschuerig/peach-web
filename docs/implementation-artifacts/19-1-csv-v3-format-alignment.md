# Story 19.1: CSV V3 Format Alignment

Status: draft

## Story

As a user,
I want CSV export/import to use the V3 format compatible with the iOS app,
so that I can transfer all training data (pitch and rhythm) between platforms.

## Context

Prerequisite: All rhythm record types exist (Epics 17-18).

The iOS app uses CSV V3 with 19 columns covering all 4 training types. The web app currently uses V1 with 12 columns covering only pitch.

## Acceptance Criteria

1. **AC1 — Export V3 header (19 columns):**
   ```
   trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,tempoBPM,offsetMs,initialCentOffset,userCentError,meanOffsetMs,meanOffsetMsPosition0,meanOffsetMsPosition1,meanOffsetMsPosition2,meanOffsetMsPosition3
   ```

2. **AC2 — Format version:** `# peach-export-format:3`

3. **AC3 — Training type discriminators:**
   - `pitchDiscrimination` (both unison and interval)
   - `pitchMatching` (both unison and interval)
   - `rhythmOffsetDetection`
   - `continuousRhythmMatching`

4. **AC4 — Pitch rows:** Fill pitch-relevant columns, leave rhythm columns empty.

5. **AC5 — Rhythm offset detection rows:** Fill `trainingType`, `timestamp`, `isCorrect`, `tempoBPM`, `offsetMs`. Leave pitch and continuous-rhythm columns empty.

6. **AC6 — Continuous rhythm matching rows:** Fill `trainingType`, `timestamp`, `tempoBPM`, `meanOffsetMs`, `meanOffsetMsPosition0-3`. Leave pitch and offset-detection columns empty.

7. **AC7 — Import V3:** Parse all 4 training types from iOS-exported V3 files. Populate corresponding records.

8. **AC8 — Import V1 backward compatibility:** Parse old web V1 files. Map `pitchComparison` → `pitchDiscrimination`.

9. **AC9 — Round-trip fidelity:** Export → import round-trip produces identical records for all training types.

10. **AC10 — Discipline-driven export:** Export iterates over registered disciplines (or all record types), not hardcoded per-discipline code. Uses `TrainingRecord` enum.

11. **AC11 — All tests pass.** Test fixtures for V1, V3, and cross-platform import.

## Tasks / Subtasks

- [ ] Task 1: Update export schema to V3 (19 columns)
- [ ] Task 2: Implement per-training-type row formatting
- [ ] Task 3: Implement V3 import parser
- [ ] Task 4: Add V1 backward-compat mapping
- [ ] Task 5: Add round-trip tests
- [ ] Task 6: Add cross-platform import test (iOS V3 fixture)
- [ ] Task 7: `cargo test --workspace` passes

## Dev Notes

- Column order must match iOS exactly for cross-platform compatibility
- Empty columns are just empty strings between commas: `,,`
- Consider discipline-driven CSV architecture: each discipline knows its columns and how to serialize/parse. This aligns with iOS Epic 55.3.
