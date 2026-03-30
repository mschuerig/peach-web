---
title: 'Align web CSV format with iOS v3'
type: 'bugfix'
created: '2026-03-30'
status: 'done'
baseline_commit: '9a1d6a0'
context: []
---

# Align web CSV format with iOS v3

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The web CSV v3 column order diverges from iOS v3: web places `tempoBPM,offsetMs` at cols 10-11 and `initialCentOffset,userCentError` at cols 12-13, while iOS has them reversed. The web parser uses hardcoded column indices, so importing an iOS-exported file silently misparses data (e.g. tempo read as cent offset). Additionally, the web exports tritone as `A4` instead of the canonical `d5`.

**Approach:** Make the v3 import parser header-driven (build column-name-to-index map from the header row), align the export column order to match iOS, and switch tritone export from `A4` to `d5`. V1 import stays index-based (fixed 12-column format, no ambiguity).

## Boundaries & Constraints

**Always:** Existing web-exported v3 files (with the old column order) must still import correctly via header-driven parsing. V1 import compatibility preserved. Tritone import must continue accepting both `A4` and `d5`.

**Ask First:** Adding support for v2 import (currently iOS-only intermediary format).

**Never:** Changing the iOS app. Breaking existing stored data (IndexedDB records are unaffected).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Import iOS v3 file | Header: `...initialCentOffset,userCentError,tempoBPM,offsetMs...` | All 4 record types parsed correctly by column name | N/A |
| Import old web v3 file | Header: `...tempoBPM,offsetMs,initialCentOffset,userCentError...` | All 4 record types parsed correctly by column name | N/A |
| Export v3 file | N/A | Header matches iOS order; tritone exported as `d5` | N/A |
| Import v3 with unknown extra column | Header has 20+ columns | Parser ignores unknown columns, parses known ones | N/A |
| Import v3 missing required column | e.g. `tempoBPM` missing from header | Error: descriptive message naming the missing column | Return Err |
| Import v1 file | 12-column v1 header | Parsed with existing index-based logic (unchanged) | N/A |

</frozen-after-approval>

## Code Map

- `web/src/adapters/csv_export_import.rs` -- All CSV export/import logic, parser functions, and tests
- `domain/src/types/interval.rs` -- `csv_code()` returns `"A4"` for tritone, needs to return `"d5"`

## Tasks & Acceptance

**Execution:**
- [ ] `domain/src/types/interval.rs` -- Change `csv_code()` for Tritone from `"A4"` to `"d5"` -- Align with iOS canonical name. Update the test that asserts `A4`.
- [ ] `web/src/adapters/csv_export_import.rs` -- Reorder `CSV_HEADER` constant to match iOS: `...isCorrect,initialCentOffset,userCentError,tempoBPM,offsetMs,meanOffsetMs...`
- [ ] `web/src/adapters/csv_export_import.rs` -- Add `build_column_index(header)` function that parses the header row into a `HashMap<&str, usize>` and validates required columns are present
- [ ] `web/src/adapters/csv_export_import.rs` -- Refactor `parse_v3()` to use column index map: pass it into each row parser instead of hardcoded indices. Refactor `parse_pitch_discrimination_row` (shared v1/v3), `parse_pitch_matching_row_v3`, `parse_rhythm_offset_row`, `parse_continuous_rhythm_row`.
- [ ] `web/src/adapters/csv_export_import.rs` -- Update export format strings for each record type to match new column order
- [ ] `web/src/adapters/csv_export_import.rs` -- Update all test CSV row literals and `make_csv` to use new header. Add cross-platform test with iOS column order. Add test with old web column order to prove backward compat.

**Acceptance Criteria:**
- Given a v3 file exported by iOS (iOS column order), when imported on web, then all 4 record types are parsed with correct field values
- Given a v3 file exported by the old web version (old column order), when imported on web, then all 4 record types are parsed with correct field values
- Given a web export, when the file is inspected, then the header matches iOS column order and tritone intervals appear as `d5`
- Given `cargo test -p domain`, when run, then all interval tests pass with `d5` for tritone csv_code
- Given `cargo test -p web`, when run, then all CSV import/export tests pass

## Verification

**Commands:**
- `cargo test -p domain` -- expected: all tests pass, including updated tritone csv_code test
- `cargo test -p web` -- expected: all CSV tests pass with header-driven parsing
- `cargo clippy --workspace` -- expected: no warnings
