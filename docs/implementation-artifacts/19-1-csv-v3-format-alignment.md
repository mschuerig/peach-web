# Story 19.1: CSV V3 Format Alignment

Status: review

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

- [x] Task 1: Update export schema to V3 (19 columns)
- [x] Task 2: Implement per-training-type row formatting
- [x] Task 3: Implement V3 import parser
- [x] Task 4: Add V1 backward-compat mapping
- [x] Task 5: Add round-trip tests
- [x] Task 6: Add cross-platform import test (iOS V3 fixture)
- [x] Task 7: `cargo test --workspace` passes

## Dev Notes

- Column order must match iOS exactly for cross-platform compatibility
- Empty columns are just empty strings between commas: `,,`
- Consider discipline-driven CSV architecture: each discipline knows its columns and how to serialize/parse. This aligns with iOS Epic 55.3.

## Dev Agent Record

### Implementation Plan

- Updated CSV constants: `CSV_HEADER` → 19 columns, `FORMAT_VERSION` → 3, added `CSV_HEADER_V1` for backward compat
- Export: all 4 training types formatted with correct V3 column positions; `pitchComparison` renamed to `pitchDiscrimination`
- Import: version-dispatched parsing — V1 files use 12-column header + `pitchComparison`; V3 files use 19-column header + all 4 training types
- V3 import accepts `pitchComparison` as alias for `pitchDiscrimination` for robustness
- `ParsedImportData` extended with `rhythm_offset_detections` and `continuous_rhythm_matchings` vecs
- `MergeResult` extended with rhythm counts; `import_replace`/`import_merge` handle all 4 record types
- `ContinuousRhythmMatchingRecord` import sets `hit_rate=0.0` and `cycle_count=0` since these fields aren't in the CSV (not needed for profile hydration)
- `format_optional_f64` helper for `Option<f64>` serialization (Some→number, None→empty)

### Debug Log

No issues encountered.

### Completion Notes

All 7 tasks completed. 64 web crate tests pass (was 46, added 18 new). 495 domain tests pass. Clippy clean. Cargo fmt applied.

Key changes:
- Export now writes V3 format with 19 columns, format version 3, and `pitchDiscrimination` type name
- Import parses both V1 (12 columns, `pitchComparison`) and V3 (19 columns, all 4 training types)
- Round-trip fidelity verified for all 4 training types
- Cross-platform iOS V3 fixture test added
- `settings_view.rs` merge counts updated to include rhythm types

## File List

- `web/src/adapters/csv_export_import.rs` — Major: V3 export/import, all new parsers and tests
- `web/src/components/settings_view.rs` — Minor: MergeResult field additions in merge count sum

## Change Log

- 2026-03-25: Implemented CSV V3 format alignment — export/import all 4 training types with 19-column format, V1 backward compatibility, round-trip and cross-platform tests
