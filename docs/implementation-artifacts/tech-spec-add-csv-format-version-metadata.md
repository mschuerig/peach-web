---
title: 'Add CSV Format Version Metadata'
slug: 'add-csv-format-version-metadata'
created: '2026-03-10'
status: 'completed'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['Rust', 'wasm32-unknown-unknown', 'Leptos 0.8 CSR', 'web-sys']
files_to_modify: ['web/src/adapters/csv_export_import.rs']
code_patterns: ['match-based version dispatch', 'comment-style metadata line', 'Result<T, String> error propagation', 'hand-rolled CSV (no external crate)', 'const for format constants']
test_patterns: ['inline #[cfg(test)] mod tests', 'no existing CSV tests — adding first test module for pure parsing logic']
---

# Tech-Spec: Add CSV Format Version Metadata

**Created:** 2026-03-10

## Overview

### Problem Statement

Exported CSV files have no format version identifier, making it impossible to evolve the CSV format in the future while maintaining backward compatibility on import. Without versioning, any format change would break existing exported files with no way to distinguish old from new.

### Solution

Prepend `# peach-export-format:1` as the first line of all CSV exports. On import, extract the version number from line 1 and dispatch to the appropriate parser function via a `match` on the version. Reject files that lack a version line or have an unrecognized version.

### Scope

**In Scope:**

- Export: prepend metadata line before the header row
- Import: version reader logic, version-dispatched parsing via `match`, 3 new error conditions (missing version, unsupported version, invalid metadata)
- Tests for all new and changed behavior

**Out of Scope:**

- Localized/translated error messages (English-only)
- Test data generator script (iOS-only, not present in peach-web)
- UI changes (existing error display path in settings_view handles new error strings)
- Trait-based parser abstraction (using idiomatic `match` dispatch instead)

## Context for Development

### Codebase Patterns

- All CSV export/import logic lives in a single adapter file: `web/src/adapters/csv_export_import.rs` (~458 lines)
- Hand-rolled CSV parsing (no external crate) — safe because all values are numeric/enum strings
- Errors returned as `String` via `Result<T, String>` (no custom error enum in the web crate for CSV)
- Domain types (`PitchComparisonRecord`, `PitchMatchingRecord`) live in `domain/src/records.rs`
- `Interval::from_csv_code()` and `Interval::csv_code()` handle interval string conversion
- Inline `#[cfg(test)] mod tests` for domain crate; web crate CSV parsing is currently untested (browser-dependent parts deferred)
- The iOS app uses the identical 12-column CSV format with `# peach-export-format:1` — cross-platform compatibility is a goal

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `web/src/adapters/csv_export_import.rs` | All CSV export/import logic — the only file being modified |
| `web/src/components/settings_view.rs` | UI caller — uses `parse_import_file()`, displays errors via `ImportExportStatus::Error(msg)`. No changes needed. |
| `domain/src/records.rs` | `PitchComparisonRecord`, `PitchMatchingRecord` structs |
| `domain/src/types/interval.rs` | `Interval::csv_code()`, `Interval::from_csv_code()` |
| `docs/project-context.md` | Coding rules and conventions |
| `../peach/docs/implementation-artifacts/40-1-add-csv-format-version-metadata.md` | iOS reference story |

### Investigation Findings

**Anchor points in `csv_export_import.rs`:**

- `const CSV_HEADER` (line 32): Header string constant — new `FORMAT_VERSION`, `METADATA_PREFIX`, `METADATA_LINE` constants go alongside this
- `export_all_data()` (line 51): Builds CSV string starting with `csv.push_str(CSV_HEADER)` — insert metadata line before header
- `parse_import_file()` (line 177): Entry point for import parsing — must read version from line 1, then dispatch. Currently reads header as `lines.next()` on line 184
- `parse_comparison_row()` (line 238) and `parse_pitch_matching_row()` (line 277): Row-level parsing — these stay unchanged, called by the v1 parse path
- `truncate_timestamp_to_second()` (line 441): Utility — unchanged but testable

**Callers (confirmed single consumer):**

- `settings_view.rs` line 665: `csv_export_import::parse_import_file(&content)` — error strings displayed as-is, no changes needed

**Test gap:** No `#[cfg(test)]` module exists in this file. The pure parsing functions (`parse_import_file`, row parsers, `truncate_timestamp_to_second`) have no browser dependencies and can be unit tested. This spec adds the first test module.

### Technical Decisions

1. **Comment-style metadata line (`# peach-export-format:1`):** Standard CSV has no metadata mechanism; `#`-prefixed comment lines are a widely-used convention. The header row remains on line 2, so the file is still valid CSV in spreadsheet tools.

2. **Reject unversioned files:** The web app has not shipped with export/import yet (story 8.8 still in review), so there are no unversioned files in the wild. Requiring the version line from day one avoids a permanent legacy code path.

3. **`match`-based version dispatch (not trait objects):** Idiomatic Rust for in-app version dispatch. Adding a future v2 requires: (a) a new `parse_v1`-style function, (b) one new `match` arm, (c) bumping the format version constant. No dynamic dispatch overhead, and Rust's exhaustive matching naturally handles the unknown-version case.

4. **All changes in one file:** The entire change is contained within `csv_export_import.rs`. No new files, no domain crate changes, no UI changes.

## Implementation Plan

### Tasks

All tasks modify a single file: `web/src/adapters/csv_export_import.rs`

- [x] Task 1: Add format version constants
  - Action: Add three constants alongside `CSV_HEADER`:
    ```rust
    const FORMAT_VERSION: u32 = 1;
    const METADATA_PREFIX: &str = "# peach-export-format:";
    const METADATA_LINE: &str = "# peach-export-format:1";
    ```
  - Notes: `METADATA_LINE` is derived from prefix + version conceptually, but since Rust `const` doesn't support `format!`, define it as a literal. A test will verify consistency between the three constants.

- [x] Task 2: Add version reader function
  - Action: Create a private function that extracts the version from line 1:
    ```rust
    fn read_format_version(first_line: &str) -> Result<u32, String>
    ```
  - Logic:
    1. Check if `first_line` starts with `METADATA_PREFIX`
    2. If not → return `Err` with missing version message
    3. Extract substring after prefix → parse as `u32`
    4. If parse fails → return `Err` with invalid metadata message
    5. Return `Ok(version)`
  - Notes: Operates on a single line, not the whole file. The caller is responsible for splitting lines and passing line 1.

- [x] Task 3: Add v1 parse function
  - Action: Extract the current body of `parse_import_file` (everything after the empty-file check and header extraction) into a new private function:
    ```rust
    fn parse_v1(lines: std::str::Lines) -> Result<ParsedImportData, String>
    ```
  - Logic: Receives an iterator positioned after the header line. Contains all current row-parsing logic (the `for (line_num, line) in lines.enumerate()` loop and the `has_data` check). Header validation is done by the caller before dispatching.
  - Notes: `parse_comparison_row` and `parse_pitch_matching_row` are unchanged — `parse_v1` calls them as before.

- [x] Task 4: Refactor `parse_import_file` into version-dispatched orchestrator
  - Action: Rewrite `parse_import_file` to:
    1. Trim and check for empty input (existing logic)
    2. Extract line 1 via `lines.next()`
    3. Call `read_format_version(first_line)` → propagate `Err` with `?`
    4. Extract line 2 (header) via `lines.next()` → validate against `CSV_HEADER`
    5. `match version` to dispatch:
       ```rust
       match version {
           1 => parse_v1(lines),
           v => Err(format!(
               "Unsupported export format version {v}. Please update the app to import this file."
           )),
       }
       ```
  - Notes: The public API signature is unchanged — `pub fn parse_import_file(content: &str) -> Result<ParsedImportData, String>`. The `settings_view.rs` caller requires no changes.

- [x] Task 5: Update `export_all_data` to prepend metadata line
  - Action: In `export_all_data()`, insert the metadata line before the header:
    ```rust
    let mut csv = String::new();
    csv.push_str(METADATA_LINE);
    csv.push('\n');
    csv.push_str(CSV_HEADER);
    csv.push('\n');
    ```
  - Notes: Replaces the current two lines (`csv.push_str(CSV_HEADER); csv.push('\n');`).

- [x] Task 6: Add `#[cfg(test)]` module with unit tests
  - Action: Add a test module at the bottom of the file. Tests cover:
    - **Version reader tests:**
      - `test_read_format_version_valid` — `"# peach-export-format:1"` → `Ok(1)`
      - `test_read_format_version_higher` — `"# peach-export-format:42"` → `Ok(42)`
      - `test_read_format_version_missing_prefix` — `"trainingType,timestamp,..."` → `Err` containing "does not contain format version"
      - `test_read_format_version_invalid_number` — `"# peach-export-format:abc"` → `Err` containing "unreadable format metadata"
      - `test_read_format_version_empty_after_prefix` — `"# peach-export-format:"` → `Err` containing "unreadable format metadata"
    - **Metadata constant consistency:**
      - `test_metadata_line_matches_prefix_and_version` — verify `METADATA_LINE == format!("{METADATA_PREFIX}{FORMAT_VERSION}")`
    - **Import orchestrator tests (full pipeline via `parse_import_file`):**
      - `test_import_valid_v1_comparison` — full CSV with metadata line + header + one comparison row → parses correctly
      - `test_import_valid_v1_pitch_matching` — full CSV with metadata line + header + one pitch matching row → parses correctly
      - `test_import_missing_version` — CSV starting with header (no metadata line) → `Err` containing "does not contain format version"
      - `test_import_unsupported_version` — `"# peach-export-format:99\n..."` → `Err` containing "Unsupported export format version 99"
      - `test_import_invalid_metadata` — `"# peach-export-format:xyz\n..."` → `Err` containing "unreadable format metadata"
      - `test_import_empty_file` — `""` → `Err` "File is empty"
      - `test_import_version_only_no_data` — metadata line + header, no data rows → `Err` "No records found"
    - **Export format test:**
      - Note: `export_all_data` requires `IndexedDbStore` (browser API) and cannot be unit tested. Export format is verified indirectly via the round-trip acceptance criteria and manual browser testing.
  - Notes: Tests use `domain::Interval` and `domain::records::*` which are available in the web crate's test build. A helper function `make_csv(rows: &[&str]) -> String` constructs test CSV strings with the metadata line and header prepended.

### Acceptance Criteria

- [x] AC 1: Given a CSV export is triggered, when the file is generated, then the first line is `# peach-export-format:1` and the second line is the 12-column header row.
- [x] AC 2: Given a valid v1 CSV file with the metadata line, when imported, then all records parse correctly (same behavior as before this change).
- [x] AC 3: Given a CSV file without a metadata line (starts with the header row), when imported, then the import fails with an error message containing "does not contain format version metadata".
- [x] AC 4: Given a CSV file with an unrecognized version (e.g., `# peach-export-format:99`), when imported, then the import fails with an error message containing "Unsupported export format version 99" and advising to update the app.
- [x] AC 5: Given a CSV file with a malformed metadata line (e.g., `# peach-export-format:abc`), when imported, then the import fails with an error message containing "unreadable format metadata".
- [x] AC 6: Given this change is deployed, when the existing settings UI displays import errors, then the 3 new error messages render correctly without any UI code changes.
- [x] AC 7: Given a future v2 format is needed, when a developer adds it, then they only need to: (a) create a `parse_v2` function, (b) add a `2 => parse_v2(lines)` arm to the match, (c) bump `FORMAT_VERSION` to 2. No changes to `parse_v1`, `read_format_version`, or the orchestrator structure.

## Additional Context

### Dependencies

- No new crate dependencies required
- Depends on story 8.8 (export/import architecture cleanup) being merged first — that story established the current structure of `csv_export_import.rs`

### Testing Strategy

**Unit tests (in `#[cfg(test)]` module):**
- Version reader: 5 tests (valid version, higher version, missing prefix, invalid number, empty after prefix)
- Constant consistency: 1 test
- Import orchestrator: 7 tests (valid comparison, valid pitch matching, missing version, unsupported version, invalid metadata, empty file, version-only no data)

**Manual browser testing:**
- Export a CSV → open in text editor → verify line 1 is `# peach-export-format:1`
- Import the exported CSV → verify it imports successfully (round-trip)
- Import a CSV without the version line → verify error message appears in the UI
- Open exported CSV in a spreadsheet app → verify it loads correctly (the `#` line may appear as a data row, but header and data are intact)

### Notes

- **iOS cross-compatibility:** The format is identical to the iOS app's `# peach-export-format:1`. Files exported from either platform should import on the other once both ship this feature.
- **Future metadata extensibility:** Additional `#`-prefixed lines (e.g., app version, export date) can be added before the header without breaking the version parser — `read_format_version` only inspects line 1.
- **Error message style:** Messages are user-facing strings displayed in the settings UI error banner. They should be clear and actionable, not developer-oriented.

### Error Messages

| Condition | Error message |
| --------- | ------------- |
| Missing version | `"This file does not contain format version metadata. It may have been created by an older version of the app. Please re-export your data with the current version."` |
| Unsupported version | `"Unsupported export format version {v}. Please update the app to import this file."` |
| Invalid metadata | `"The file contains unreadable format metadata on line 1: '{first_line}'."` |

## Review Notes

- Adversarial review completed
- Findings: 12 total, 3 fixed, 9 skipped (noise/pre-existing/deferred)
- Resolution approach: auto-fix
