# Story 6.3: Data Export & Import

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to export my training data to a CSV file and import it back,
so that I can back up my data, transfer it between browsers, or exchange it with my iOS app.

## Acceptance Criteria

1. **Given** the Settings view **When** I click "Export Data" (FR39) **Then** a CSV file is generated containing all comparison records and pitch matching records in the iOS-compatible flat CSV format **And** the file is downloaded with filename `peach-training-data-YYYY-MM-DD.csv`.

2. **Given** the export file **When** I inspect its contents **Then** it has a single header row with columns: `trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,initialCentOffset,userCentError` **And** comparison rows have `trainingType=comparison` with `centOffset` and `isCorrect` filled, `initialCentOffset` and `userCentError` empty **And** pitch matching rows have `trainingType=pitchMatching` with `initialCentOffset` and `userCentError` filled, `centOffset` and `isCorrect` empty **And** timestamps are ISO 8601 with second precision only (e.g. `2026-03-04T16:17:55Z`).

3. **Given** the Settings view **When** I click "Import Data" **Then** a file picker opens for selecting a CSV file.

4. **Given** I select a valid CSV file **When** the import executes **Then** the user is presented with a choice: "Replace all data" or "Merge with existing data".

5. **Given** the user chooses "Replace all data" **When** the import executes **Then** all existing comparison records and pitch matching records are deleted **And** all records from the file are saved to IndexedDB **And** the page reloads to rebuild the PerceptualProfile from the imported record set.

6. **Given** the user chooses "Merge with existing data" **When** the import executes **Then** imported records are merged with existing records **And** duplicates are ignored based on timestamp+trainingType comparison to the second (records with the same type and timestamps matching to the second are considered identical) **And** the page reloads to rebuild the PerceptualProfile from the combined record set.

7. **Given** I select an invalid or corrupted file **When** the import is attempted **Then** an error message is shown **And** no existing data is modified.

8. **Given** existing data and a completed import (replace or merge) **When** the import finishes **Then** a success message shows how many records were imported (and how many duplicates were skipped if merge mode).

## Tasks / Subtasks

- [x] Task 1: Add interval code conversion helpers to domain crate (AC: #2)
  - [x] 1.1 Add `to_interval_code()` function that converts the web's `interval: u8` (semitones 0-12) to iOS interval codes: `P1`, `m2`, `M2`, `m3`, `M3`, `P4`, `A4`/`d5`, `P5`, `m6`, `M6`, `m7`, `M7`, `P8`
  - [x] 1.2 Add `from_interval_code()` function that parses iOS interval code string back to `u8` semitones
  - [x] 1.3 Add `midi_note_name()` function that converts MIDI note number to name string (e.g. 60 → "C4", 61 → "C#4"). Use sharps only (matching iOS convention seen in export)
  - [x] 1.4 Place these in a new `domain/src/portability.rs` module (or add to existing `types/` if better fit)
  - [x] 1.5 Write unit tests for all conversions (round-trip: semitones → code → semitones)

- [x] Task 2: Implement CSV export functionality in web crate (AC: #1, #2)
  - [x] 2.1 Create `web/src/adapters/data_portability.rs` module with `export_all_data()` async function
  - [x] 2.2 Fetch all comparison records from IndexedDB via `IndexedDbStore::fetch_all_comparisons()`
  - [x] 2.3 Fetch all pitch matching records via `IndexedDbStore::fetch_all_pitch_matchings()`
  - [x] 2.4 Write CSV header: `trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,initialCentOffset,userCentError`
  - [x] 2.5 For each comparison record: write row with `trainingType=comparison`, convert interval u8 to code, add note names, truncate timestamp to second, leave `initialCentOffset` and `userCentError` empty
  - [x] 2.6 For each pitch matching record: write row with `trainingType=pitchMatching`, convert interval, add note names, leave `centOffset` and `isCorrect` empty
  - [x] 2.7 Trigger browser download using `web_sys::Blob` + `web_sys::Url::create_object_url_with_blob()` + dynamically created `<a>` element with `download` attribute
  - [x] 2.8 Filename format: `peach-training-data-YYYY-MM-DD.csv`

- [x] Task 3: Implement CSV parser for import (AC: #3, #7)
  - [x] 3.1 Add `parse_import_file()` function to `data_portability.rs` that reads the CSV text content
  - [x] 3.2 Validate header row matches expected columns (case-sensitive)
  - [x] 3.3 For each data row: check `trainingType` column to determine record type
  - [x] 3.4 For `comparison` rows: parse `referenceNote`, `targetNote`, `centOffset`, `isCorrect`, convert interval code to u8, read `tuningSystem`, `timestamp` → construct `ComparisonRecord`
  - [x] 3.5 For `pitchMatching` rows: parse `referenceNote`, `targetNote`, `initialCentOffset`, `userCentError`, convert interval code, read `tuningSystem`, `timestamp` → construct `PitchMatchingRecord`
  - [x] 3.6 `referenceNoteName` and `targetNoteName` columns are informational only on import — derive from MIDI number, do not rely on them
  - [x] 3.7 Return a structured result (`ParsedImportData` with `Vec<ComparisonRecord>`, `Vec<PitchMatchingRecord>`) or descriptive error

- [x] Task 4: Implement import with replace mode (AC: #4, #5, #8)
  - [x] 4.1 Add `import_replace()` async function to `data_portability.rs`
  - [x] 4.2 Call `IndexedDbStore::delete_all()` to clear existing records
  - [x] 4.3 Save all parsed comparison records to IndexedDB (iterate and call `save_comparison()` for each)
  - [x] 4.4 Save all parsed pitch matching records to IndexedDB
  - [x] 4.5 Return import count for success message

- [x] Task 5: Implement import with merge mode (AC: #4, #6, #8)
  - [x] 5.1 Add `import_merge()` async function to `data_portability.rs`
  - [x] 5.2 Fetch existing comparison records and build a `HashSet` of timestamps truncated to second
  - [x] 5.3 For each imported comparison record: truncate timestamp to second, check against existing comparison timestamp set, skip if duplicate, save if new
  - [x] 5.4 Fetch existing pitch matching records, build separate `HashSet` of truncated timestamps
  - [x] 5.5 For each imported pitch matching record: same dedup logic against pitch matching timestamps
  - [x] 5.6 Return counts: imported, skipped duplicates (separate counts per type)

- [x] Task 6: Rebuild PerceptualProfile after import (AC: #5, #6)
  - [x] 6.1 After import (replace or merge), trigger full profile rehydration by reloading the page
  - [x] 6.2 Call `window().location().reload()` after import completes — this re-runs the full startup hydration sequence which rebuilds profile from all stored records

- [x] Task 7: Add export/import UI to settings view (AC: #1, #3, #4, #8)
  - [x] 7.1 Add a "Data Management" `<fieldset>` section between the settings section and the Danger Zone
  - [x] 7.2 Add "Export Data" button styled consistently with other settings buttons
  - [x] 7.3 Add "Import Data" button that triggers a hidden `<input type="file" accept=".csv">` element
  - [x] 7.4 On file selected: parse the file, then show a `<dialog>` with "Replace all data" / "Merge with existing data" / "Cancel" options (reuse dialog pattern from reset confirmation)
  - [x] 7.5 Show success message with record counts after import completes (before page reload)
  - [x] 7.6 Show error message if file is invalid or import fails
  - [x] 7.7 Add screen reader announcement for export/import completion via `sr_announcement` signal pattern

- [x] Task 8: Handle edge cases (AC: #7)
  - [x] 8.1 Empty file → show "File is empty" error
  - [x] 8.2 File with only header row and no data rows → show "No records found in file"
  - [x] 8.3 File with wrong/missing header → show "Invalid file format" error
  - [x] 8.4 Row with invalid `trainingType` value → skip row, count as warning
  - [x] 8.5 Row with unparseable numeric fields → skip row, count as warning

## Dev Notes

### Critical: Timestamp Second Precision

The iOS app exports timestamps with **second precision only** (no sub-second digits). Example: `"2026-03-04T14:30:00Z"`, never `"2026-03-04T14:30:00.123Z"`.

When comparing timestamps for deduplication during merge import, **truncate timestamps to the second** before comparing. This means:
- `"2026-03-04T14:30:00Z"` and `"2026-03-04T14:30:00.456Z"` are the **same record**
- Truncation strategy: strip everything after the seconds digits before the `Z` (or timezone offset)
- A helper function like `truncate_timestamp_to_second(ts: &str) -> String` should normalize both the imported timestamp and existing stored timestamps before comparison

This is essential for cross-platform data exchange between iOS and web.

### CSV Format Specification (iOS-Compatible)

The iOS app uses a **flat CSV format** (not JSON as originally planned in the PRD, not section-delimited). Both record types share a single header row with a `trainingType` discriminator column. Fields not applicable to a record type are left empty.

**Exact format** (from iOS export `peach-training-data-2026-03-04.csv`):

```csv
trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,initialCentOffset,userCentError
comparison,2026-03-04T16:17:55Z,51,D#3,51,D#3,P1,equalTemperament,100.0,true,,
comparison,2026-03-04T16:18:00Z,80,G#5,80,G#5,P1,equalTemperament,32.32233047033631,false,,
pitchMatching,2026-03-04T19:27:47Z,48,C3,48,C3,P1,equalTemperament,,,-7.649124063256437,-2.7552986512521405
comparison,2026-03-04T19:28:01Z,78,F#5,75,D#5,m3,equalTemperament,49.86901747565537,true,,
pitchMatching,2026-03-04T19:28:10Z,51,D#3,47,B2,M3,equalTemperament,,,-18.496979352059814,-5.323590814196258
```

**Column mapping:**

| CSV Column | Comparison → Web Field | PitchMatching → Web Field |
|---|---|---|
| `trainingType` | `"comparison"` | `"pitchMatching"` |
| `timestamp` | `timestamp` (ISO 8601, second precision) | `timestamp` |
| `referenceNote` | `reference_note: u8` | `reference_note: u8` |
| `referenceNoteName` | Informational only (e.g. "D#3") | Informational only |
| `targetNote` | `target_note: u8` | `target_note: u8` |
| `targetNoteName` | Informational only (e.g. "G#5") | Informational only |
| `interval` | Code → `interval: u8` semitones (P1→0, m2→1, M2→2, m3→3, M3→4, P4→5, A4/d5→6, P5→7, m6→8, M6→9, m7→10, M7→11, P8→12) | Same conversion |
| `tuningSystem` | `tuning_system` (camelCase: `"equalTemperament"`) | Same |
| `centOffset` | `cent_offset: f64` | Empty |
| `isCorrect` | `is_correct: bool` (`"true"`/`"false"`) | Empty |
| `initialCentOffset` | Empty | `initial_cent_offset: f64` |
| `userCentError` | Empty | `user_cent_error: f64` |

**Key observations from the iOS file:**
- Records are **interleaved chronologically** (not grouped by type)
- `referenceNoteName`/`targetNoteName` use **sharps only** (no flats): D#3, F#5, G#5, C#5
- `interval` uses standard music theory codes: P1 (unison), m3 (minor third), M3 (major third), etc.
- `tuningSystem` is camelCase (matching web's serde convention)
- Timestamps have **second precision only** — no fractional seconds
- Empty fields are simply empty (no quotes, no null) — two consecutive commas: `,,`
- `isCorrect` is lowercase `true`/`false`
- No settings are included in the CSV — only training records

### Architecture & Pattern Compliance

- **Crate boundary**: CSV serialization/deserialization lives in the `web` crate. Interval code conversion and note name helpers go in the `domain` crate (they are pure domain knowledge — mapping semitones to music theory names).
- **Domain crate changes**: New `domain/src/portability.rs` module with `to_interval_code()`, `from_interval_code()`, `midi_note_name()`. These are testable with native `cargo test`.
- **Web crate changes**: New `web/src/adapters/data_portability.rs` for CSV export/import logic — follows the one-file-per-adapter convention.
- **Register modules**: Add `pub mod portability;` to `domain/src/lib.rs`, add `pub mod data_portability;` to `web/src/adapters/mod.rs`.

### Existing Code to Reuse (DO NOT Reinvent)

| What | Where | How to Reuse |
|---|---|---|
| `ComparisonRecord` struct + serde | `domain/src/records.rs` | Construct from parsed CSV rows |
| `PitchMatchingRecord` struct + serde | `domain/src/records.rs` | Construct from parsed CSV rows |
| `IndexedDbStore` with all CRUD methods | `web/src/adapters/indexeddb_store.rs` | Use `fetch_all_*`, `save_*`, `delete_all()` |
| Profile hydration sequence | `web/src/app.rs` lines 52-138 | Page reload triggers full rehydration |
| `StorageError` enum | `domain/src/ports.rs` | Reuse for import/export error cases |
| Confirmation dialog pattern | `web/src/components/settings_view.rs` (reset dialog) | Copy pattern for import mode dialog |
| `sr_announcement` signal pattern | `web/src/components/settings_view.rs` | Announce export/import completion |

### Browser File Download Pattern

To trigger a file download from WASM:

```rust
use web_sys::{Blob, BlobPropertyBag, Url, HtmlAnchorElement};

let blob = Blob::new_with_str_sequence_and_options(
    &js_sys::Array::of1(&csv_string.into()),
    BlobPropertyBag::new().type_("text/csv"),
)?;
let url = Url::create_object_url_with_blob(&blob)?;
let document = web_sys::window().unwrap().document().unwrap();
let a: HtmlAnchorElement = document.create_element("a")?.unchecked_into();
a.set_href(&url);
a.set_download(&filename);
a.click();
Url::revoke_object_url(&url)?;
```

### Browser File Upload Pattern

To read a file selected by the user:

```rust
// Hidden <input type="file"> element triggered by button click
// On change event: read file using FileReader API
use web_sys::{HtmlInputElement, FileReader};
use wasm_bindgen::closure::Closure;

// file_input.files().and_then(|fl| fl.get(0)) -> File
// FileReader::read_as_text(&file) -> onload callback -> result string
```

### Import Dialog UX

After the user selects a file and it parses successfully, show a dialog (reuse the `<dialog>` pattern from the reset confirmation in settings_view.rs):

- Title: "Import Training Data"
- Body: "Found X comparison records and Y pitch matching records. How would you like to import?"
- Buttons: "Replace All Data" / "Merge with Existing" / "Cancel"
- The "Replace All Data" button should have the same destructive styling as the "Delete All" button in the Danger Zone

### Profile Rebuild Strategy

After import, the PerceptualProfile must be rebuilt from the combined record set. The simplest and most correct approach is to reload the page:

```rust
web_sys::window().unwrap().location().reload().unwrap();
```

This re-runs the full startup hydration in `app.rs` which fetches all records from IndexedDB and rebuilds the profile. This avoids duplicating hydration logic and guarantees consistency.

### web-sys Features Required

Ensure these `web-sys` features are enabled in `web/Cargo.toml`:

- `Blob`, `BlobPropertyBag` — for creating download blob
- `Url` — for `create_object_url_with_blob` / `revoke_object_url`
- `HtmlAnchorElement` — for triggering download
- `HtmlInputElement` — for file input
- `FileReader` — for reading uploaded file
- `FileList`, `File` — for accessing selected files

Check which are already enabled; only add missing ones.

### No Settings in CSV

The iOS CSV format does **not** include settings — only training records. This is intentional: settings are platform-specific (e.g. sound source availability differs between iOS and web). The export/import feature is for **training data portability only**.

### What NOT to Do

- Do NOT persist the PerceptualProfile directly — it's always rebuilt from records
- Do NOT use JSON format — iOS app uses flat CSV, and cross-platform compatibility is required (PRD G4)
- Do NOT use section-delimited CSV — the iOS format is a single flat table with a `trainingType` discriminator
- Do NOT add a new IndexedDB object store — use existing stores and methods
- Do NOT implement batch IndexedDB writes — saving records one-by-one reuses existing `save_*` methods and is sufficient for typical data sizes
- Do NOT skip timestamp truncation in merge mode — this is the key deduplication mechanism for cross-platform data exchange
- Do NOT rely on `referenceNoteName`/`targetNoteName` during import — always derive from MIDI number; these columns are informational for human readability
- Do NOT include settings in the CSV — the iOS format doesn't include them, and settings are platform-specific
- Do NOT add import/export to the navigation — it belongs in Settings view only

### Accessibility

- Export/import buttons must have `min-h-[44px]` touch targets and visible focus rings (consistent with all other buttons)
- Screen reader announcements for: "Data exported", "X records imported", "Import failed: [reason]"
- Import dialog must trap focus (use native `<dialog>` with `show_modal()` like the reset dialog)
- File input must be accessible — hidden visually but available to screen readers, or use a labeled button that triggers the hidden input

### Project Structure Notes

- New file: `domain/src/portability.rs` (interval code + note name helpers with unit tests)
- Modified: `domain/src/lib.rs` (add `pub mod portability;`)
- New file: `web/src/adapters/data_portability.rs` (CSV export/import logic)
- Modified: `web/src/adapters/mod.rs` (add `pub mod data_portability;`)
- Modified: `web/src/components/settings_view.rs` (add Data Management section with export/import UI)
- Possibly modified: `web/Cargo.toml` (add `web-sys` features if not already enabled)
- No routing changes

### References

- [Source: docs/planning-artifacts/epics.md#Epic 6, Story 6.3]
- [Source: docs/planning-artifacts/prd.md#FR39, FR40]
- [Source: docs/ios-reference/peach-web-prd.md#Data Portability, G4, D5]
- [Source: docs/planning-artifacts/architecture.md#Data Architecture]
- [Source: docs/planning-artifacts/architecture.md#Storage Boundaries]
- [Source: docs/project-context.md#Storage Edge Cases]
- [Source: domain/src/records.rs — ComparisonRecord, PitchMatchingRecord structs]
- [Source: domain/src/ports.rs — TrainingDataStore trait, StorageError]
- [Source: web/src/adapters/indexeddb_store.rs — existing CRUD methods]
- [Source: web/src/adapters/localstorage_settings.rs — all peach.* keys]
- [Source: web/src/app.rs — profile hydration sequence]
- [Source: web/src/components/settings_view.rs — danger zone dialog pattern]

### Previous Story Intelligence (Story 6.2)

- Story 6.2 added `sr_announcement` pattern with `aria-live="polite"` + `aria-atomic="true"` — reuse this for import/export announcements
- Story 6.2 confirmed all accessibility infrastructure is in place (focus rings, ARIA, tab order)
- Story 6.2 only touched `web/src/components/` — clean implementation, no regressions
- Pattern: focused changes, `cargo clippy` clean, manual browser testing

### Git Intelligence

Recent commits show:
- `b5278e8` Code review fixes for story 6.2 (focus rings, sr_announcement bugs)
- `184b81a` Implement story 6.2 Screen Reader Accessibility Polish
- Clean, focused commits with descriptive messages
- All recent stories: implement → clippy clean → mark done

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

No debug issues encountered.

### Completion Notes List

- Task 1: Created `domain/src/portability.rs` with `to_interval_code()`, `from_interval_code()`, `midi_note_name()`, and `truncate_timestamp_to_second()`. All 13 unit tests pass including round-trip and iOS sample data validation.
- Task 2: Implemented `export_all_data()` in `web/src/adapters/data_portability.rs`. Records are sorted chronologically (interleaved by type) matching iOS format. Download triggered via Blob + anchor element pattern.
- Task 3: Implemented `parse_import_file()` with full CSV parsing, header validation, row-by-row type discrimination, and structured error/warning reporting.
- Task 4: Implemented `import_replace()` — deletes all existing data then saves imported records one-by-one.
- Task 5: Implemented `import_merge()` — builds HashSets of existing timestamps (truncated to second) per record type, deduplicates imported records, returns detailed counts.
- Task 6: Implemented `reload_page()` helper that calls `window().location().reload()` to trigger full profile rehydration after import.
- Task 7: Added "Data Management" fieldset to settings view with Export/Import buttons, hidden file input, import mode dialog (Replace All / Merge / Cancel), status messages, error display, and screen reader announcements.
- Task 8: Edge cases handled in `parse_import_file()`: empty file, header-only, wrong header, invalid trainingType (skipped with warning), unparseable fields (skipped with warning).

### Change Log

- 2026-03-04: Implemented story 6.3 Data Export & Import — all 8 tasks complete
- 2026-03-04: Code review fixes — removed premature blob URL revoke (H1), fixed merge dedup within imported file (H2), added file size limit 10MB (M2), log import warnings to console (M1), added CSV escaping limitation comment (L1)

### File List

- domain/src/portability.rs (new)
- domain/src/lib.rs (modified — added `pub mod portability;`)
- web/src/adapters/data_portability.rs (new)
- web/src/adapters/mod.rs (modified — added `pub mod data_portability;`)
- web/src/components/settings_view.rs (modified — added Data Management section)
- web/Cargo.toml (modified — added web-sys features: Blob, BlobPropertyBag, File, FileList, FileReader, HtmlAnchorElement, HtmlInputElement, Url)
