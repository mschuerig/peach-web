# Story 7.0a: Rename Comparison to PitchComparison

Status: review

## Story

As a developer,
I want all `Comparison*` types renamed to `PitchComparison*` throughout the codebase,
so that naming is symmetric with the `PitchMatching*` family and matches the iOS sibling app.

## Context

The iOS sibling app (commit `733d99ab74e5`) performed a systematic rename of all `Comparison*` types to `PitchComparison*`. This achieves naming symmetry: `PitchComparison` / `PitchMatching` instead of `Comparison` / `PitchMatching`.

This rename must happen first in Epic 7 because all subsequent stories reference these types.

**iOS reference:** `docs/implementation-artifacts/tech-spec-rename-comparison-to-pitch-comparison.md` in `mschuerig/peach`

## Acceptance Criteria

1. **AC1 — Type renames:** All public types renamed:
   - `Comparison` → `PitchComparison`
   - `CompletedComparison` → `CompletedPitchComparison`
   - `ComparisonSession` → `PitchComparisonSession`
   - `ComparisonSessionState` → `PitchComparisonSessionState`
   - `ComparisonPlaybackData` → `PitchComparisonPlaybackData`
   - `ComparisonRecord` → `PitchComparisonRecord`
   - `ComparisonObserver` → `PitchComparisonObserver`

2. **AC2 — Method renames:**
   - `ComparisonObserver::comparison_completed()` → `PitchComparisonObserver::pitch_comparison_completed()`
   - `CompletedComparison::comparison()` → `CompletedPitchComparison::pitch_comparison()`
   - `next_comparison()` → `next_pitch_comparison()`
   - `TrainingDataStore::save_comparison()` → `save_pitch_comparison()`
   - `TrainingDataStore::fetch_all_comparisons()` → `fetch_all_pitch_comparisons()`

3. **AC3 — File renames:**
   - `domain/src/training/comparison.rs` → `pitch_comparison.rs`
   - `domain/src/session/comparison_session.rs` → `pitch_comparison_session.rs`
   - `web/src/components/comparison_view.rs` → `pitch_comparison_view.rs`

4. **AC4 — Module declarations updated:** All `mod` and `use` statements in `lib.rs`, `mod.rs`, and consuming files reference the new names

5. **AC5 — Component rename:** `fn ComparisonView()` → `fn PitchComparisonView()`

6. **AC6 — Route path unchanged:** The URL `/training/comparison` does NOT change. Changing routes would break bookmarks and is unnecessary — the route is a user-facing URL, not an internal type name. The iOS app similarly kept its navigation destination names simple.

7. **AC7 — IndexedDB store name unchanged:** The object store `"comparison_records"` does NOT change. Renaming it would break existing user databases and require migration. The store name is a persistence identifier, not a type name.

8. **AC8 — CSV format unchanged:** The CSV export record type prefix `"comparison"` does NOT change. This is a file format identifier for import compatibility.

9. **AC9 — All tests pass:** `cargo test -p domain` and `cargo clippy` pass. `trunk build` succeeds.

10. **AC10 — Documentation updated:** `docs/project-context.md` type name references updated. Story files and planning artifacts are NOT bulk-updated (they're historical records).

## Tasks / Subtasks

### Phase 1: Domain crate file renames

- [x] Task 1: Rename files via `git mv`
  - [x] `git mv domain/src/training/comparison.rs domain/src/training/pitch_comparison.rs`
  - [x] `git mv domain/src/session/comparison_session.rs domain/src/session/pitch_comparison_session.rs`

### Phase 2: Domain crate content renames

- [x] Task 2: Rename types in `domain/src/training/pitch_comparison.rs`
  - [x] `Comparison` → `PitchComparison` (struct + all impls + tests)
  - [x] `CompletedComparison` → `CompletedPitchComparison` (struct + all impls + tests)
  - [x] `.comparison()` accessor → `.pitch_comparison()`

- [x] Task 3: Rename types in `domain/src/session/pitch_comparison_session.rs`
  - [x] `ComparisonSession` → `PitchComparisonSession`
  - [x] `ComparisonSessionState` → `PitchComparisonSessionState`
  - [x] `ComparisonPlaybackData` → `PitchComparisonPlaybackData`
  - [x] Internal method `generate_next_comparison` → `generate_next_pitch_comparison`
  - [x] All test references

- [x] Task 4: Rename in `domain/src/records.rs`
  - [x] `ComparisonRecord` → `PitchComparisonRecord`
  - [x] `from_completed(completed: &CompletedComparison)` → `from_completed(completed: &CompletedPitchComparison)`
  - [x] All test references

- [x] Task 5: Rename in `domain/src/ports.rs`
  - [x] `ComparisonObserver` trait → `PitchComparisonObserver`
  - [x] `comparison_completed()` → `pitch_comparison_completed()`
  - [x] `save_comparison()` → `save_pitch_comparison()`
  - [x] `fetch_all_comparisons()` → `fetch_all_pitch_comparisons()`
  - [x] Parameter types: `CompletedComparison` → `CompletedPitchComparison`, `ComparisonRecord` → `PitchComparisonRecord`

- [x] Task 6: Rename in `domain/src/strategy.rs`
  - [x] Import: `comparison::Comparison` → `pitch_comparison::PitchComparison`
  - [x] `next_comparison()` → `next_pitch_comparison()`
  - [x] Return type and constructor: `Comparison::new()` → `PitchComparison::new()`
  - [x] All test references

- [x] Task 7: Update module declarations
  - [x] `domain/src/training/mod.rs`: `mod comparison` → `mod pitch_comparison`, update re-exports
  - [x] `domain/src/session/mod.rs`: `mod comparison_session` → `mod pitch_comparison_session`, update re-exports
  - [x] `domain/src/lib.rs`: update all re-exports to new names

- [x] Task 8: Verify domain crate
  - [x] `cargo test -p domain` — all 306 tests pass
  - [x] `cargo clippy -p domain` — no warnings

### Phase 3: Web crate renames

- [x] Task 9: Rename web component file
  - [x] `git mv web/src/components/comparison_view.rs web/src/components/pitch_comparison_view.rs`

- [x] Task 10: Rename in `web/src/components/pitch_comparison_view.rs`
  - [x] `fn ComparisonView()` → `fn PitchComparisonView()`
  - [x] All type references: `ComparisonSession`, `ComparisonSessionState`, `ComparisonObserver`, `ComparisonPlaybackData` → `PitchComparison*`
  - [x] Variable names: `comparison` → `pitch_comparison` where it refers to the struct (not generic uses)

- [x] Task 11: Rename in `web/src/components/mod.rs`
  - [x] `mod comparison_view` → `mod pitch_comparison_view`
  - [x] `pub use comparison_view::ComparisonView` → `pub use pitch_comparison_view::PitchComparisonView`

- [x] Task 12: Rename in `web/src/bridge.rs`
  - [x] All `ComparisonObserver` impls → `PitchComparisonObserver`
  - [x] Method: `comparison_completed()` → `pitch_comparison_completed()`
  - [x] Types: `CompletedComparison` → `CompletedPitchComparison`, `ComparisonRecord` → `PitchComparisonRecord`
  - [x] Store calls: `save_comparison()` → `save_pitch_comparison()`

- [x] Task 13: Rename in `web/src/adapters/indexeddb_store.rs`
  - [x] Type: `ComparisonRecord` → `PitchComparisonRecord`
  - [x] Methods: `save_comparison()` → `save_pitch_comparison()`, `fetch_all_comparisons()` → `fetch_all_pitch_comparisons()`
  - [x] **Keep** the object store string constant as `"comparison_records"` (AC7)

- [x] Task 14: Rename in `web/src/adapters/data_portability.rs`
  - [x] Type: `ComparisonRecord` → `PitchComparisonRecord`
  - [x] Variable names: `comparisons` → `pitch_comparisons` etc.
  - [x] Method calls: `save_comparison()` → `save_pitch_comparison()`, `fetch_all_comparisons()` → `fetch_all_pitch_comparisons()`
  - [x] **Keep** CSV record type prefix as `"comparison"` (AC8)

- [x] Task 15: Rename in `web/src/app.rs`
  - [x] Import: `ComparisonView` → `PitchComparisonView`
  - [x] Route: `view=ComparisonView` → `view=PitchComparisonView`
  - [x] **Keep** route path as `"/training/comparison"` (AC6)
  - [x] Hydration: `fetch_all_comparisons()` → `fetch_all_pitch_comparisons()`

- [x] Task 16: Rename in `web/src/components/start_page.rs`
  - [x] Handler names: `on_comparison` → `on_pitch_comparison`, `on_interval_comparison` → `on_interval_pitch_comparison`
  - [x] **Keep** navigation URLs unchanged (AC6)

### Phase 4: Documentation

- [x] Task 17: Update `docs/project-context.md`
  - [x] Replace type references: `ComparisonSession` → `PitchComparisonSession`, `ComparisonView` → `PitchComparisonView`, etc.
  - [x] Update observer signature example: `comparison_completed` → `pitch_comparison_completed`
  - [x] Update file path examples
  - [x] **Keep** route paths, storage keys, and CSV identifiers unchanged

- [x] Task 18: Final verification
  - [x] `cargo test -p domain` — 306 tests pass
  - [x] `cargo clippy` — clean
  - [x] `trunk build` — succeeds
  - [ ] Manual smoke test: comparison training still works via start page

## Dev Notes

### What NOT to Rename

These identifiers stay as-is to preserve backward compatibility:

| Identifier | Reason |
|---|---|
| Route `/training/comparison` | User-facing URL, bookmarks |
| IndexedDB store `"comparison_records"` | Existing user databases |
| CSV prefix `"comparison"` | Import file compatibility |
| localStorage keys | No comparison-related keys exist |

### Mechanical Rename Strategy

This is a mechanical find-and-replace operation. Recommended approach:
1. Use IDE/editor rename-symbol where possible
2. For bulk renames in a file, use search-and-replace with word boundaries
3. After each phase, run `cargo check` to catch any missed references
4. Run full `cargo test -p domain` after Phase 2, full build after Phase 3

### Architecture Compliance

- **No behavioral changes:** This is purely a rename. No logic changes, no new features.
- **Type name fidelity:** After this rename, the domain blueprint language aligns: `PitchComparison` / `PitchMatching` symmetry.

## File List

- `domain/src/training/comparison.rs` → `domain/src/training/pitch_comparison.rs` (renamed + content updated)
- `domain/src/session/comparison_session.rs` → `domain/src/session/pitch_comparison_session.rs` (renamed + content updated)
- `domain/src/training/mod.rs` (modified)
- `domain/src/session/mod.rs` (modified)
- `domain/src/lib.rs` (modified)
- `domain/src/records.rs` (modified)
- `domain/src/ports.rs` (modified)
- `domain/src/strategy.rs` (modified)
- `domain/tests/strategy_convergence.rs` (modified)
- `web/src/components/comparison_view.rs` → `web/src/components/pitch_comparison_view.rs` (renamed + content updated)
- `web/src/components/mod.rs` (modified)
- `web/src/bridge.rs` (modified)
- `web/src/adapters/indexeddb_store.rs` (modified)
- `web/src/adapters/data_portability.rs` (modified)
- `web/src/app.rs` (modified)
- `web/src/components/start_page.rs` (modified)
- `web/src/components/settings_view.rs` (modified)
- `docs/project-context.md` (modified)
- `docs/implementation-artifacts/sprint-status.yaml` (modified)

## Dev Agent Record

### Implementation Notes

Systematic rename of all `Comparison*` types to `PitchComparison*` across domain and web crates.
Purely mechanical rename — no logic or behavioral changes. All 306 domain tests pass, clippy clean, trunk build succeeds.

Preserved backward-compatible identifiers per ACs: route `/training/comparison`, IndexedDB store `"comparison_records"`, CSV prefix `"comparison"`.

## Change Log

- 2026-03-06: Implemented story 7.0a — renamed all Comparison* types to PitchComparison* for iOS naming parity
