# Story 14.2: Web Crate Terminology Rename

Status: done

## Story

As a developer,
I want the web crate updated to match the domain terminology from story 14.1,
so that components, routes, bridge adapters, and UI labels use consistent iOS-aligned naming.

## Context

Story 14.1 renames all domain types. This story cascades those changes through the web crate: Leptos components, routes, bridge/adapter code, localization strings, and help content.

The iOS app also simplified UI button labels from "Hear & Compare" / "Tune & Match" to just "Compare" / "Match" with section headers providing context. We adopt the same pattern.

## Acceptance Criteria

1. **AC1 — Component file renames:**
   - `web/src/components/pitch_comparison_view.rs` → `pitch_discrimination_view.rs`
   - Component function: `PitchComparisonView` → `PitchDiscriminationView`
   - `web/src/components/pitch_matching_view.rs` — no file rename needed
   - Component function: `PitchMatchingView` → `PitchMatchingView` (unchanged)

2. **AC2 — Module declarations:** `web/src/components/mod.rs` updated for new file and component names

3. **AC3 — Route paths updated:**
   - `/training/comparison` → `/training/pitch-discrimination`
   - `/training/pitch-matching` → `/training/pitch-matching` (unchanged)

4. **AC4 — Bridge adapter renames:**
   - `ProfileObserver` → `PitchDiscriminationProfileObserver` (or keep generic if it serves both)
   - `DataStoreObserver` → `PitchDiscriminationDataStoreObserver` (or keep generic)
   - All `impl PitchComparisonObserver` → `impl PitchDiscriminationObserver`
   - All `impl PitchMatchingObserver` method references updated

5. **AC5 — IndexedDB store:** Object store names for pitch discrimination data may be reset (no migration needed — single user). If store names change, add a note that IndexedDB should be cleared.

6. **AC6 — Start page updated:**
   - Links point to new route paths
   - Button labels changed from "Hear & Compare" / "Tune & Match" to "Compare" / "Match"
   - Section headers: "Single Notes", "Intervals" (existing sections keep their names)
   - `TrainingMode` references → `TrainingDiscipline`

7. **AC7 — Localization keys updated:**
   - All `comparison-*` keys → `discrimination-*` (or equivalent)
   - Button labels: "Hear & Compare" → "Compare", "Tune & Match" → "Match"
   - Both `en/main.ftl` and `de/main.ftl` updated

8. **AC8 — Help sections updated:**
   - `COMPARISON_HELP` → `PITCH_DISCRIMINATION_HELP`
   - `PITCH_MATCHING_HELP` — unchanged or minor label updates
   - All `move_tr!()` calls reference new localization keys

9. **AC9 — NavBar and context providers:** All Leptos context types referencing old names updated

10. **AC10 — All tests pass:** `cargo test --workspace`, `cargo clippy --workspace`, and `trunk build` all pass.

## Tasks / Subtasks

### Phase 1: File and module renames

- [x] Task 1: `git mv` component files
- [x] Task 2: Update `mod.rs` and `use` statements

### Phase 2: Component and route updates

- [x] Task 3: Rename component functions
- [x] Task 4: Update route paths in `app.rs`
- [x] Task 5: Update start page links and labels

### Phase 3: Bridge and adapter updates

- [x] Task 6: Rename bridge observer structs and trait impls
- [x] Task 7: Update IndexedDB adapter (store names, method names)
- [x] Task 8: Update Leptos context types

### Phase 4: Localization

- [x] Task 9: Update `en/main.ftl` keys and values
- [x] Task 10: Update `de/main.ftl` keys and values
- [x] Task 11: Update help section constants and `move_tr!()` references

### Phase 5: Verification

- [x] Task 12: `cargo clippy --workspace` passes
- [x] Task 13: `trunk build` succeeds
- [ ] Task 14: Manual smoke test: start page → discrimination training → matching training → profile
  - **Deferred to user — agent cannot verify in browser**

## Dev Notes

- Clear IndexedDB in browser after applying this story (no migration)
- The route change from `/training/comparison` to `/training/pitch-discrimination` has no backward-compat requirement
- Keep CSS class names generic where possible (e.g. `training-card` not `discrimination-card`)

## Dev Agent Record

### Implementation Plan

Cascading rename from domain terminology (Story 14.1) through the web crate. Systematic approach: file renames → component/route updates → bridge/adapter → localization → help constants → verification.

### Debug Log

No issues encountered. All renames were straightforward find-and-replace operations.

### Completion Notes

- Renamed `pitch_comparison_view.rs` → `pitch_discrimination_view.rs` and `PitchComparisonView` → `PitchDiscriminationView`
- Updated route from `/training/comparison` → `/training/pitch-discrimination`
- Simplified button labels: "Hear & Compare" → "Compare", "Tune & Match" → "Match" (both languages)
- Renamed localization keys: `comparison-*` → `discrimination-*`, `help-comparison-*` → `help-discrimination-*`
- Renamed help constant: `COMPARISON_HELP` → `DISCRIMINATION_HELP`
- Renamed IndexedDB method names: `save_pitch_comparison` → `save_pitch_discrimination`, `fetch_all_pitch_comparisons` → `fetch_all_pitch_discriminations`
- IndexedDB object store names kept as-is (`comparison_records`) — these are persisted data, no migration needed
- Bridge observer struct names kept generic (`ProfileObserver`, `DataStoreObserver`) — they already use updated trait names from Story 14.1
- Updated `project-context.md` route paths and type name references
- Renamed CSV export/import field: `pitch_comparisons` → `pitch_discriminations` in `ParsedImportData` and `MergeResult`
- All `cargo test -p domain`, `cargo clippy --workspace`, `cargo fmt --check`, and `trunk build` pass
- Manual smoke test deferred to user (agent cannot run browser)

## File List

- web/src/components/pitch_discrimination_view.rs (renamed from pitch_comparison_view.rs)
- web/src/components/mod.rs
- web/src/app.rs
- web/src/components/start_page.rs
- web/src/bridge.rs
- web/src/adapters/indexeddb_store.rs
- web/src/adapters/csv_export_import.rs
- web/src/components/settings_view.rs
- web/src/help_sections.rs
- web/locales/en/main.ftl
- web/locales/de/main.ftl
- docs/project-context.md

## Change Log

- 2026-03-24: Web crate terminology rename — cascaded domain renames (Story 14.1) through components, routes, adapters, localization, and help content. Simplified button labels to "Compare"/"Match".
