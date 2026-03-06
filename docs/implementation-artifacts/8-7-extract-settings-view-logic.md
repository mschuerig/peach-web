# Story 8.7: Extract Business Logic from Settings View

Status: ready-for-dev

## Story

As a developer,
I want business logic extracted from settings_view.rs into proper domain and adapter layers,
so that the codebase follows clean architecture with views as pure presentation.

## Acceptance Criteria

1. `settings_view.rs` contains only presentation logic (Leptos components, signal wiring, DOM rendering) -- no business logic, no constants that duplicate domain knowledge, no data transformation
2. The `INTERVALS` constant is removed; code that needs ordered intervals derives the list from the `Interval` enum's existing natural ordering (discriminant values 0..=12)
3. Short labels (P1, m2, M2, m3, M3, P4, TT, P5, m6, M6, m7, M7, P8) live on the `Interval` type itself as a `short_label(&self) -> &'static str` method in the domain crate
4. `encode_one()` and `decode_one()` in `interval_codes.rs` reuse `Interval::short_label()` instead of duplicating the label mapping
5. Export and import orchestration (file reading, async coordination, dialog state) is extracted from `settings_view.rs` into an adapter or service module
6. `persist_intervals()` is moved out of the view into the existing `LocalStorageSettings` adapter
7. `ResetStatus` and `ImportExportStatus` enums are moved out of the view to appropriate locations
8. `project-context.md` includes a prominent rule that views must not contain business logic
9. All existing functionality works identically after refactoring -- zero behavioral changes
10. `cargo test -p domain` passes, `cargo clippy` clean on both crates

## Tasks / Subtasks

- [ ] Task 1: Add `short_label()` method to `Interval` in domain crate (AC: #3, #10)
  - [ ] 1.1 Add `pub fn short_label(&self) -> &'static str` method to `Interval` in `domain/src/types/interval.rs` returning: Prime->"P1", MinorSecond->"m2", MajorSecond->"M2", MinorThird->"m3", MajorThird->"M3", PerfectFourth->"P4", Tritone->"TT", PerfectFifth->"P5", MinorSixth->"m6", MajorSixth->"M6", MinorSeventh->"m7", MajorSeventh->"M7", Octave->"P8"
  - [ ] 1.2 Add `pub fn all_chromatic() -> &'static [Interval]` associated function returning a static slice of all 13 intervals in chromatic order (replaces the `INTERVALS` const in the view)
  - [ ] 1.3 Add unit tests for both methods in the existing `#[cfg(test)] mod tests` block

- [ ] Task 2: Refactor `interval_codes.rs` to use `Interval::short_label()` (AC: #4, #10)
  - [ ] 2.1 Replace the match in `encode_one()` (~line 67-90) with a call to `directed.interval.short_label()` plus the direction suffix (u/d)
  - [ ] 2.2 Replace the interval-parsing match in `decode_one()` (~line 92-125) with a lookup that iterates `Interval::all_chromatic()` and matches on `short_label()`
  - [ ] 2.3 Run existing tests in `interval_codes.rs` (11 tests) -- all must pass unchanged

- [ ] Task 3: Move `persist_intervals()` to `LocalStorageSettings` (AC: #6)
  - [ ] 3.1 Add `pub fn set_selected_intervals(intervals: &HashSet<DirectedInterval>)` to `LocalStorageSettings` in `web/src/adapters/localstorage_settings.rs` (mirrors existing `get_selected_intervals()`)
  - [ ] 3.2 Update call sites in `settings_view.rs` to use `LocalStorageSettings::set_selected_intervals()`
  - [ ] 3.3 Delete `persist_intervals()` from `settings_view.rs`

- [ ] Task 4: Extract export/import orchestration from settings_view (AC: #5, #7)
  - [ ] 4.1 Create `web/src/adapters/data_portability_service.rs` (or extend existing `data_portability.rs`) to house the orchestration logic: file reading via FileReader, JSON parsing, merge/replace decision flow, status tracking
  - [ ] 4.2 Move `ImportExportStatus` enum to the new/extended module
  - [ ] 4.3 Move `ResetStatus` enum alongside `ImportExportStatus` (both are state machines for adapter-level operations, not view state)
  - [ ] 4.4 The view retains only: signal declarations, button click handlers that call service functions, and reactive rendering of status signals
  - [ ] 4.5 Register the new module in `web/src/adapters/mod.rs` if a new file was created

- [ ] Task 5: Remove `INTERVALS` constant and `interval_short_label()` from view (AC: #1, #2)
  - [ ] 5.1 Replace all uses of the `INTERVALS` array in `settings_view.rs` with `Interval::all_chromatic()`
  - [ ] 5.2 Replace all calls to `interval_short_label(interval)` with `interval.short_label()`
  - [ ] 5.3 Delete the `INTERVALS` constant and `interval_short_label()` function from `settings_view.rs`

- [ ] Task 6: Add "no logic in views" rule to project-context.md (AC: #8)
  - [ ] 6.1 Add to the "Anti-Patterns -- NEVER Do These" section in `docs/project-context.md`:
    `- DO NOT put business logic in view components -- views are pure presentation. Constants, data transformations, persistence calls, encoding/decoding, and orchestration belong in domain types or adapter modules. Views only declare signals, wire event handlers, and render DOM.`

- [ ] Task 7: Final verification (AC: #9, #10)
  - [ ] 7.1 `cargo test -p domain` -- all tests pass
  - [ ] 7.2 `cargo clippy` -- zero warnings on both crates
  - [ ] 7.3 `trunk serve` -- settings page works identically: interval selection, export, import, reset all function as before

## Dev Notes

### Current State of settings_view.rs

929 lines. The following logic does NOT belong in a view and must be extracted:

**Lines 123-139 -- `interval_short_label()`:** A match mapping each `Interval` variant to its short code (P1, m2, etc.). This is intrinsic to the domain type, not a UI concern. These codes are never localized.

**Lines 144-158 -- `INTERVALS` constant:** A `[Interval; 13]` array listing all variants in chromatic order. Superfluous because `Interval` enum discriminants already encode this order (0..=12). The enum derives `Ord`, so the ordering is built-in.

**Lines 160-170 -- `ResetStatus` enum:** State machine for the reset confirmation flow. Not presentation.

**Lines 172-176 -- `ImportExportStatus` enum:** State machine for import/export flow. Not presentation.

**Lines 178-186 -- `persist_intervals()`:** Serializes intervals to JSON and writes to localStorage. This is adapter-layer persistence, not view logic. There is already a symmetric `get_selected_intervals()` in `LocalStorageSettings`.

**Lines 600-779 -- Export/import orchestration:** ~180 lines of async file reading, JSON parsing, merge/replace logic, error handling. The core `data_portability` module already exists in `web/src/adapters/data_portability.rs` with the actual data operations. The orchestration (FileReader API, dialog state, status updates) should also live in the adapter layer.

### interval_codes.rs Relationship

`web/src/interval_codes.rs` (269 lines) has private `encode_one()` and `decode_one()` functions that duplicate the same short-label mapping. After `Interval::short_label()` exists in the domain crate, both `interval_codes.rs` and any remaining view usage can call it directly.

**encode_one current approach:** Match on interval variant -> string code + direction suffix. Replace with: `format!("{}{}", di.interval.short_label(), direction_suffix)`.

**decode_one current approach:** Split code into interval part + direction suffix, match interval part against hardcoded strings. Replace with: iterate `Interval::all_chromatic()`, find the one whose `short_label()` matches the prefix.

### Interval Ordering

The `Interval` enum in `domain/src/types/interval.rs` has explicit discriminant values:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Interval {
    Prime = 0,
    MinorSecond = 1,
    MajorSecond = 2,
    // ... through ...
    Octave = 12,
}
```

It derives `Ord`, so `Interval::all_chromatic()` simply returns a static array in discriminant order. The `INTERVALS` const in the view is a redundant copy of this.

### Existing Adapter Pattern

The `web/src/adapters/` directory already contains:
- `localstorage_settings.rs` -- has `get_selected_intervals()` but missing the symmetric setter
- `data_portability.rs` -- core import/export data operations
- `indexeddb_store.rs`, `audio_context.rs`, `audio_oscillator.rs`, `audio_soundfont.rs`, `note_player.rs`

Follow this established pattern. Do NOT create a new `services/` directory.

### What the View Should Retain

After refactoring, `settings_view.rs` should contain only:
- `SettingsView()` component function
- Signal declarations (`RwSignal<ResetStatus>`, `RwSignal<ImportExportStatus>`, etc.)
- Event handler closures that delegate to adapter functions
- Leptos view! macro blocks for DOM rendering
- No constants, no match-based transformations, no persistence calls, no async orchestration

### Project Structure Notes

- Domain crate change: `domain/src/types/interval.rs` -- add `short_label()` and `all_chromatic()`
- Web crate adapter changes: `localstorage_settings.rs`, possibly `data_portability.rs` or new `data_portability_service.rs`
- Web crate view change: `settings_view.rs` -- remove extracted logic, update call sites
- Web crate utility change: `interval_codes.rs` -- simplify encode/decode
- Documentation change: `docs/project-context.md` -- add anti-pattern rule
- No new crate dependencies needed

### References

- [Source: web/src/components/settings_view.rs#L123-186] Logic to extract (labels, constants, enums, persist)
- [Source: web/src/components/settings_view.rs#L600-779] Export/import orchestration to extract
- [Source: web/src/interval_codes.rs#L67-125] encode_one/decode_one to refactor
- [Source: domain/src/types/interval.rs] Interval enum with Ord derive and discriminants
- [Source: web/src/adapters/localstorage_settings.rs#L44-54] Existing get_selected_intervals()
- [Source: web/src/adapters/data_portability.rs] Existing data portability operations
- [Source: docs/project-context.md#Anti-Patterns] Where to add new rule

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
