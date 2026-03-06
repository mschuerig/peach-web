# Story 8.8: Clean Up Export/Import Architecture

Status: ready-for-dev

## Story

As a developer,
I want the export/import code properly structured with domain logic on domain types, adapter code named honestly, and view orchestration extracted,
so that naming matches intent, there are no duplicate mappings, and the view is pure presentation.

## Acceptance Criteria

1. `domain/src/portability.rs` is deleted -- all its logic lives on domain types or in the web adapter
2. `Interval` gains `csv_code()` (returns "A4" for tritone, iOS CSV compat) and `from_csv_code()` methods; `from_semitones()` is made public
3. `midi_note_name()` free function removed -- callers use `MIDINote::new(n).name()` (identical logic, already exists)
4. Duplicate `NOTE_NAMES` constant in portability.rs removed (already exists in midi.rs)
5. `truncate_timestamp_to_second()` moved to the web adapter (only consumer)
6. Web adapter renamed from `data_portability.rs` to `csv_export_import.rs`
7. `data_portability_service.rs` deleted -- `ResetStatus` and `ImportExportStatus` enums absorbed into the renamed adapter module
8. FileReader-to-future conversion extracted from `settings_view.rs` into the adapter
9. View retains only signal declarations, thin event handlers, and DOM rendering
10. `interval_label()` long-name mapping in `interval_codes.rs` moved to `Interval::display_name()` in the domain crate
11. All existing functionality works identically -- zero behavioral changes
12. `cargo test -p domain` passes, `cargo clippy` clean on both crates

## Tasks / Subtasks

- [ ] Task 1: Enrich `Interval` with CSV and display methods (AC: #2, #10, #12)
  - [ ] 1.1 Make `Interval::from_semitones()` public in `domain/src/types/interval.rs`
  - [ ] 1.2 Add `pub fn csv_code(&self) -> &'static str` -- same as `short_label()` except Tritone returns `"A4"` (iOS CSV compatibility)
  - [ ] 1.3 Add `pub fn from_csv_code(code: &str) -> Option<Interval>` -- parses interval code strings, accepts both `"A4"` and `"d5"` for tritone
  - [ ] 1.4 Add `pub fn display_name(&self) -> &'static str` -- returns long name ("Minor Second", "Perfect Fifth", etc.); Prime returns "Prime", Octave returns "Octave" (no direction suffix -- that's the caller's job)
  - [ ] 1.5 Add unit tests for `csv_code()`, `from_csv_code()`, and `display_name()` (including roundtrip: all 13 intervals through csv_code -> from_csv_code)

- [ ] Task 2: Delete `domain/src/portability.rs` (AC: #1, #3, #4, #5)
  - [ ] 2.1 Remove `pub mod portability;` from `domain/src/lib.rs`
  - [ ] 2.2 Delete `domain/src/portability.rs`
  - [ ] 2.3 Move `truncate_timestamp_to_second()` into the web adapter (Task 3's target file) -- it's only used there
  - [ ] 2.4 Update `web/src/adapters/data_portability.rs` imports: replace `domain::portability::{to_interval_code, from_interval_code, midi_note_name, truncate_timestamp_to_second}` with domain type methods
  - [ ] 2.5 Replace `to_interval_code(r.interval)` calls with `Interval::from_semitones(r.interval).map(|i| i.csv_code()).unwrap_or("P1")` (or equivalent)
  - [ ] 2.6 Replace `from_interval_code(code)` calls with `Interval::from_csv_code(code).map(|i| i.semitones())`
  - [ ] 2.7 Replace `midi_note_name(r.reference_note)` calls with `MIDINote::new(r.reference_note).name()`
  - [ ] 2.8 Verify all existing portability tests are covered by the new Interval method tests (Task 1.5)

- [ ] Task 3: Rename web adapter and absorb enums (AC: #6, #7)
  - [ ] 3.1 Rename `web/src/adapters/data_portability.rs` to `web/src/adapters/csv_export_import.rs`
  - [ ] 3.2 Move `ResetStatus` and `ImportExportStatus` from `data_portability_service.rs` into `csv_export_import.rs`
  - [ ] 3.3 Delete `web/src/adapters/data_portability_service.rs`
  - [ ] 3.4 Update `web/src/adapters/mod.rs`: replace `data_portability` and `data_portability_service` with `csv_export_import`
  - [ ] 3.5 Update all import paths in `settings_view.rs` and anywhere else that references the old module names

- [ ] Task 4: Move `interval_label()` to domain (AC: #10)
  - [ ] 4.1 Replace the `interval_label(interval, direction)` function in `web/src/interval_codes.rs` with calls to `interval.display_name()` from the domain crate
  - [ ] 4.2 The direction suffix ("Up"/"Down") logic stays in `interval_codes.rs` -- `display_name()` returns just the interval name, `interval_label()` appends direction
  - [ ] 4.3 Delete the manual match block in `interval_label()`, replace with: `let name = interval.display_name();` then existing direction formatting

- [ ] Task 5: Extract FileReader orchestration from view (AC: #8, #9)
  - [ ] 5.1 Add `pub async fn read_file_as_text(file: web_sys::File) -> Result<String, String>` to `csv_export_import.rs` -- wraps the FileReader callback-to-future pattern (~40 lines extracted from view)
  - [ ] 5.2 Simplify `handle_file_selected` in `settings_view.rs` to call the new function instead of inline FileReader/Closure/onload setup
  - [ ] 5.3 Verify the view no longer imports `wasm_bindgen::closure::Closure` (should be unused after extraction)

- [ ] Task 6: Final verification (AC: #11, #12)
  - [ ] 6.1 `cargo test -p domain` -- all tests pass (portability tests replaced by Interval method tests)
  - [ ] 6.2 `cargo clippy` -- zero warnings on both crates
  - [ ] 6.3 `trunk serve` -- settings page works identically: export, import (replace + merge), reset all function as before

## Dev Notes

### Background

Story 8.7 extracted some business logic from `settings_view.rs` but left significant architectural issues:
- `domain/src/portability.rs` contains logic that belongs on `Interval` and `MIDINote` types
- The module name "portability" is misleading -- it's CSV export/import serialization
- `data_portability_service.rs` contains only two enum definitions, no actual service logic
- ~180 lines of export/import orchestration remain in the view

This story addresses all of these issues in one coherent cleanup.

### Two Different Tritone Codes

There are two interval code systems that must be preserved:

| Context | Tritone code | Method |
|---|---|---|
| URL query params, UI display | `"d5"` | `Interval::short_label()` |
| CSV export (iOS compatibility) | `"A4"` | `Interval::csv_code()` (new) |

The `from_csv_code()` parser must accept BOTH `"A4"` and `"d5"` for tritone (the existing `from_interval_code` already does this).

### Current File Map

| Current file | Action | Becomes |
|---|---|---|
| `domain/src/portability.rs` | DELETE | Logic moves to `Interval` and `MIDINote` methods |
| `domain/src/types/interval.rs` | EDIT | Add `csv_code()`, `from_csv_code()`, `display_name()`, make `from_semitones()` pub |
| `web/src/adapters/data_portability.rs` | RENAME | `web/src/adapters/csv_export_import.rs` |
| `web/src/adapters/data_portability_service.rs` | DELETE | Enums move into `csv_export_import.rs` |
| `web/src/adapters/mod.rs` | EDIT | Update module declarations |
| `web/src/interval_codes.rs` | EDIT | `interval_label()` uses `Interval::display_name()` |
| `web/src/components/settings_view.rs` | EDIT | FileReader extraction, import path updates |

### What the View Should Retain After This Story

After both 8.7 and 8.8, `settings_view.rs` should contain only:
- `SettingsView()` component and helper sub-components (`SettingsSection`, `SettingsRow`, `Stepper`)
- Signal declarations (`RwSignal<ResetStatus>`, `RwSignal<ImportExportStatus>`, etc.)
- Event handler closures that delegate to adapter functions
- Leptos `view!` macro blocks for DOM rendering
- No constants, no match-based transformations, no persistence calls, no async FileReader plumbing

### References

- [Source: domain/src/portability.rs] Module to delete (80 lines, 4 functions + duplicate constant)
- [Source: domain/src/types/interval.rs] Interval type -- add csv_code, from_csv_code, display_name
- [Source: domain/src/types/midi.rs:43-47] MIDINote::name() -- already does what midi_note_name() does
- [Source: web/src/adapters/data_portability.rs] Adapter to rename and enrich
- [Source: web/src/adapters/data_portability_service.rs] 19-line file to delete (enums only)
- [Source: web/src/interval_codes.rs:44-65] interval_label() match to simplify
- [Source: web/src/components/settings_view.rs:572-641] FileReader orchestration to extract
- [Predecessor: docs/implementation-artifacts/8-7-extract-settings-view-logic.md] Story 8.7 review findings
