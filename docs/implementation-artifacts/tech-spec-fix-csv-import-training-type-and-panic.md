---
title: 'Fix CSV Import/Export Training Type and Import Panic'
slug: 'fix-csv-import-training-type-and-panic'
created: '2026-03-11'
status: 'ready-for-dev'
stepsCompleted: [1, 2, 3, 4]
tech_stack: [Rust, Leptos 0.8, wasm-bindgen, leptos-fluent, wasm-bindgen-futures]
files_to_modify: [web/src/adapters/csv_export_import.rs, web/src/components/settings_view.rs]
code_patterns: [csv-string-dispatch, spawn_local-async, tr!-macro-context]
test_patterns: [inline-cfg-test, make_csv-helper, parse_import_file-assertions]
---

# Tech-Spec: Fix CSV Import/Export Training Type and Import Panic

**Created:** 2026-03-11

## Overview

### Problem Statement

Two bugs in CSV data import/export:

1. **Wrong training type string**: CSV export writes `"comparison"` but should write `"pitchComparison"` (the rename was done elsewhere in the codebase but missed in CSV code). The import parser only accepts `"comparison"`, so files exported by a corrected version would fail to import.
2. **Panic on malformed data**: When import encounters unknown training types or other errors, the `tr!()` macro in the FileReader callback at `settings_view.rs:737` calls `expect_context::<leptos_fluent::I18n>()` which panics because the callback runs outside the Leptos reactive owner. Malformed import data must never crash the app.

### Solution

- Update export to write `"pitchComparison"` instead of `"comparison"`
- Update import parser to accept `"pitchComparison"` (replacing `"comparison"`)
- Fix the I18n context panic in import/export handlers by pre-capturing translated strings before async boundaries
- Add comprehensive unit tests covering the new training type string, backward compatibility, and malformed data handling

### Scope

**In Scope:**

- Rename export training type string from `"comparison"` to `"pitchComparison"` (line 112 in csv_export_import.rs)
- Update import parser match arm to accept `"pitchComparison"` (replacing `"comparison"`)
- Fix all `tr!()` calls inside `spawn_local` blocks that cross `.await` boundaries in settings_view.rs
- Add unit tests for: `"pitchComparison"` parsing, malformed data handling, unknown training types
- Update existing tests that use `"comparison"` to use `"pitchComparison"`

**Out of Scope:**

- Changes to `"pitchMatching"` training type string (already correct)
- UI redesign of import/export flow
- Integration/browser tests
- CSV format version bump (the format itself hasn't changed, just a string value)

## Context for Development

### Codebase Patterns

- CSV export/import is in `web/src/adapters/csv_export_import.rs` (647 lines, 14 existing tests)
- Training type dispatch uses simple string matching in `parse_v1()` at lines 243-258
- Export writes training type as first CSV field (line 112 for comparison, line 133 for pitchMatching)
- The `tr!()` macro internally calls `expect_context::<leptos_fluent::I18n>()` — this requires being inside a Leptos reactive owner
- `settings_view.rs` uses `wasm_bindgen_futures::spawn_local` (line 9) which does NOT preserve the Leptos reactive owner across `.await` points
- After `.await` (e.g. `read_file_as_text().await`, IndexedDB operations), the reactive owner is no longer set, so `tr!()` panics
- This is a **latent bug** affecting ALL `tr!()` calls inside `spawn_local` after `.await` — not just the import error path. Export `tr!()` calls at lines 639, 644, 649 have the same vulnerability.
- Existing test pattern: inline `#[cfg(test)] mod tests` with `make_csv()` helper that builds CSV strings, tests assert via `parse_import_file()`
- `read_file_as_text()` uses `Closure::once` + `futures_channel::oneshot` to bridge FileReader callback to async

### Files to Reference

| File | Purpose | Key Lines |
| ---- | ------- | --------- |
| `web/src/adapters/csv_export_import.rs` | CSV export/import logic, tests | L112 (export comparison type), L243-258 (import dispatch), L493-646 (tests) |
| `web/src/components/settings_view.rs` | Import UI handlers, I18n usage | L9 (spawn_local import), L689-718 (file read + parse), L722-753 (replace handler), L755-792 (merge handler), L625-655 (export handler) |
| `web/src/app.rs` | I18n provider setup | L~260 |

### Technical Decisions

- No backward compatibility needed for `"comparison"` — no installed base exists
- **I18n fix approach**: Pre-capture all translated strings needed inside `spawn_local` blocks BEFORE entering the async block. Store them in local variables, then use those variables after `.await` points. This avoids the need to access the reactive owner after async boundaries.

## Implementation Plan

### Tasks

- [ ] Task 1: Rename export training type string
  - File: `web/src/adapters/csv_export_import.rs`
  - Action: Change line 112 from `"comparison,{},..."` to `"pitchComparison,{},..."` in the `Record::Comparison` export arm
  - Notes: Only the first field of the CSV row changes. `"pitchMatching"` remains unchanged.

- [ ] Task 2: Update import parser to accept both training type strings
  - File: `web/src/adapters/csv_export_import.rs`
  - Action: In `parse_v1()` at lines 243-258, change the match arm from `"comparison"` to `"pitchComparison"`
  - Notes: No backward compatibility needed — no installed base with `"comparison"` exports.

- [ ] Task 3: Fix I18n panic in export `spawn_local` block
  - File: `web/src/components/settings_view.rs`
  - Action: In the export handler (around lines 625-655), pre-capture translated strings before the `spawn_local` block:
    ```rust
    let msg_db_unavailable = tr!("database-not-available");
    let msg_exported = tr!("data-exported");
    // For "export-failed" with dynamic error, pre-capture the template or use format! with a static prefix
    spawn_local(async move {
        // Use msg_db_unavailable, msg_exported instead of tr!() after .await
    });
    ```
  - Notes: The `tr!("export-failed", {"error" => e})` call has a dynamic parameter. Options: (a) use `format!()` with the pre-captured base string and the error, or (b) construct the message without i18n in the error path. Preferred: pre-capture a closure or use simple `format!("Export failed: {e}")` as fallback — but check if `leptos_fluent` supports capturing the i18n signal for later use.

- [ ] Task 4: Fix I18n panic in file-read/parse `spawn_local` block
  - File: `web/src/components/settings_view.rs`
  - Action: In the file input handler (around lines 689-718), pre-capture translated strings before the `spawn_local` block:
    ```rust
    let msg_import_failed_fn = |e: String| format!("{}: {e}", tr!("import-failed-prefix"));
    // Or simply pre-capture:
    let msg_import_failed_label = tr!("import-failed-label");
    spawn_local(async move {
        // Use pre-captured strings after .await
    });
    ```
  - Notes: The `tr!("import-failed", {"error" => e})` at line 710 has a dynamic error parameter. Same approach as Task 3.

- [ ] Task 5: Fix I18n panic in `handle_import_replace` `spawn_local` block
  - File: `web/src/components/settings_view.rs`
  - Action: In `handle_import_replace` (lines 722-753), pre-capture all translated strings before the `spawn_local`:
    ```rust
    let msg_db_unavailable = tr!("database-not-available");
    // For "records-imported" with count param — pre-capture template
    // For "import-failed" with error param — pre-capture template
    spawn_local(async move {
        // Use pre-captured strings, format with runtime values
    });
    ```
  - Notes: This is where the user's panic occurred (line 737). Three `tr!()` calls to fix: lines 733, 737, 744.

- [ ] Task 6: Fix I18n panic in `handle_import_merge` `spawn_local` block
  - File: `web/src/components/settings_view.rs`
  - Action: In `handle_import_merge` (lines 755-792), same pattern as Task 5. Pre-capture all translated strings before the `spawn_local`:
    - `tr!("database-not-available")` at line 766
    - `tr!("records-merged", ...)` at line 772
    - `tr!("import-failed", ...)` at line 779
  - Notes: Three `tr!()` calls to fix, same approach.

- [ ] Task 7: Update existing tests to use `"pitchComparison"`
  - File: `web/src/adapters/csv_export_import.rs`
  - Action: Update `test_import_valid_v1_comparison` (line 558) to use `"pitchComparison"` instead of `"comparison"` in the CSV row. Update `test_import_crlf_line_endings` (line 630) similarly. Update any other tests using `"comparison"` as training type.
  - Notes: These tests should now use the canonical new string.

- [ ] Task 8: Add new tests for backward compatibility and error handling
  - File: `web/src/adapters/csv_export_import.rs`
  - Action: Add the following tests:
    1. `test_import_unknown_training_type_produces_warning` — import with `"unknownType"` produces a warning in `parsed.warnings`, does not error
    2. `test_import_mixed_valid_and_invalid_rows` — CSV with valid `"pitchComparison"` rows, valid `"pitchMatching"` rows, and unknown type rows: valid rows parse correctly, unknown rows produce warnings
    3. `test_import_malformed_comparison_fields` — `"pitchComparison"` row with non-numeric referenceNote produces a warning, not a panic
    5. `test_import_too_few_columns_produces_warning` — row with fewer than 12 columns produces a warning
    6. `test_export_writes_pitch_comparison_type` — cannot test full export (needs IndexedDB), but verify via a roundtrip: export format string contains `"pitchComparison,"`, not `"comparison,"`
  - Notes: Follow existing test patterns using `make_csv()` helper and `parse_import_file()`.

- [ ] Task 9: Run `cargo test -p web` and `cargo clippy --workspace`
  - Action: Verify all tests pass and no clippy warnings
  - Notes: The web crate tests should run natively for the CSV parsing logic (no browser needed). Run `cargo fmt` before committing.

### Acceptance Criteria

- [ ] AC 1: Given a CSV file with `"pitchComparison"` training type rows, when imported, then the rows are parsed as `PitchComparisonRecord` entries with no warnings.
- [ ] AC 2: Given a CSV file with unknown training type rows (e.g. `"foo"`), when imported, then a warning is produced for each unknown row and the import completes without panic.
- [ ] AC 3: Given a CSV file with malformed field values (e.g. non-numeric referenceNote), when imported, then a warning is produced for the malformed row and the import completes without panic.
- [ ] AC 4: Given training data is exported, when the CSV output is inspected, then pitch comparison rows use `"pitchComparison"` as the training type field (not `"comparison"`).
- [ ] AC 5: Given any import or export operation completes (success or failure), when the result is displayed, then no panic occurs from missing I18n context — all user-facing messages display correctly.
- [ ] AC 6: Given the full test suite is run (`cargo test -p web`), when all tests execute, then all tests pass including the new error-handling tests.

## Additional Context

### Dependencies

None — pure bug fix within existing code. No new crate dependencies.

### Testing Strategy

**Unit tests** (in `web/src/adapters/csv_export_import.rs`):
- Update existing `test_import_valid_v1_comparison` to use `"pitchComparison"`
- Update `test_import_crlf_line_endings` to use `"pitchComparison"`
- Add `test_import_unknown_training_type_produces_warning`
- Add `test_import_mixed_valid_and_invalid_rows`
- Add `test_import_malformed_comparison_fields`
- Add `test_import_too_few_columns_produces_warning`
- Add export string verification test

**Manual browser testing** (deferred to user):
- Import a CSV file with `"pitchComparison"` rows → should succeed
- Import a CSV file with unknown types → should show warnings, no crash
- Export data → re-import → roundtrip should work
- Verify I18n messages display correctly for import success, import error, export success, export error

### Notes

- The error log from the user shows `Row 3: unknown trainingType 'pitchComparison', skipped` — confirming the data was exported with the correct new name but the parser doesn't accept it yet
- The I18n panic is a latent bug: `wasm_bindgen_futures::spawn_local` loses the Leptos reactive owner across `.await` points, making `tr!()` panic. It affects ALL async `tr!()` calls in the import/export section (lines 639, 644, 649, 710, 733, 737, 744, 766, 772, 779)
- The existing test `test_import_valid_v1_comparison` uses `"comparison"` — it must be updated to use `"pitchComparison"`
- **Risk**: The I18n pre-capture approach depends on `tr!()` returning a `String` (not a reactive signal). If `tr!()` returns a signal, a different approach is needed. `move_tr!()` returns a signal; `tr!()` returns a `String` — so pre-capture should work.
- **Risk**: If `leptos_fluent::tr!()` with parameters (e.g. `tr!("key", {"param" => value})`) cannot be split into template + value, the pre-captured approach needs adjustment. Fallback: use `format!()` with hardcoded English strings for error paths only, with a TODO to improve later.
