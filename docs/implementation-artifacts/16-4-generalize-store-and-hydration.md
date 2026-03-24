# Story 16.4: Generalize IndexedDB Store and Hydration

Status: done

## Story

As a developer,
I want the IndexedDB store to use a generic record interface and the hydration pipeline to be discipline-driven,
so that adding a rhythm discipline doesn't require new store methods or hydration loops.

## Context

Currently the IndexedDB store has per-discipline methods (`save_pitch_comparison`, `fetch_all_pitch_comparisons`, etc.) and `app.rs` has per-discipline hydration loops. This is the second-biggest coupling multiplier after observers.

### Current State (10 store methods)
- `save_pitch_comparison()`, `fetch_all_pitch_comparisons()`
- `save_pitch_matching()`, `fetch_all_pitch_matchings()`
- `delete_all()` (lists stores in array literal)

### Target State
- `save_record(TrainingRecord)` — dispatches to correct store
- `fetch_all(discipline) -> Vec<TrainingRecord>` — or per-record-type fetch
- `delete_all()` — iterates registered stores
- Hydration: single loop over disciplines, each discipline extracts its own metrics

## Acceptance Criteria

1. **AC1 — Generic save:** `IndexedDbStore::save_record(record: &TrainingRecord)` pattern-matches to determine the correct object store and serializes.

2. **AC2 — Generic fetch:** Either:
   - `fetch_all_records() -> Vec<TrainingRecord>` fetching from all stores, or
   - `fetch_records(store_name: &str) -> Vec<JsValue>` with per-discipline deserialization
   Choose the approach that gives the best ergonomics for hydration.

3. **AC3 — Store registration:** Object store names are derived from `TrainingDiscipline` or a central registry constant, not scattered across individual methods.

4. **AC4 — delete_all uses registry:** `delete_all()` iterates registered store names instead of a hardcoded array.

5. **AC5 — Schema upgrade:** `onupgradeneeded` creates stores for all registered disciplines. New rhythm stores are created when the DB version is bumped.

6. **AC6 — Hydration pipeline generalized:** `app.rs` hydration code fetches records per discipline and feeds them into the profile using each discipline's metric extraction logic, instead of separate per-discipline loops.

7. **AC7 — TrainingDataStore trait updated:** The domain `TrainingDataStore` trait uses `TrainingRecord` instead of per-discipline method signatures.

8. **AC8 — Backward compatibility:** Existing pitch data in IndexedDB is preserved. If store names change, handle migration (or document that IndexedDB reset is needed — single user).

9. **AC9 — All tests pass:** `cargo test --workspace` and `trunk build` pass.

## Tasks / Subtasks

- [x] Task 1: Define store name registry (constant array or derived from discipline)
- [x] Task 2: Implement `save_record(TrainingRecord)` with dispatch
- [x] Task 3: Implement generic fetch methods
- [x] Task 4: Update `delete_all` to use registry
- [x] Task 5: Update `onupgradeneeded` for rhythm stores
- [x] Task 6: Refactor hydration in `app.rs` to discipline-driven loop
- [x] Task 7: Update `TrainingDataStore` trait in domain
- [x] Task 8: Update tests
- [x] Task 9: `cargo test --workspace`, `cargo clippy --workspace`, `trunk build` pass

## Dev Notes

- The per-discipline IndexedDB store names can stay as they are (`comparison_records`, `pitch_matching_records`) for backward compatibility, or be renamed to match new terminology (`pitch_discrimination_records`, `pitch_matching_records`). Since we're resetting IndexedDB anyway (single user), renaming is fine.
- For rhythm: add `rhythm_offset_detection_records` and `continuous_rhythm_matching_records` stores.
- The `TrainingRecord` enum (from story 16.3) serves as the universal record container for both save and fetch operations.
- DB version must be bumped to trigger `onupgradeneeded` for new stores.

## Dev Agent Record

### Implementation Plan

- Added `STORE_NAMES` constant registry and `LEGACY_STORE_NAMES` for migration
- Replaced 4 per-discipline methods with `save_record()` and `fetch_all_records()`
- `onupgradeneeded` iterates `STORE_NAMES` to create stores and deletes legacy `comparison_records`
- `delete_all()` iterates `STORE_NAMES` instead of hardcoded array
- Bumped DB_VERSION from 2 to 3
- Added `timestamp()` and `store_name()` methods to `TrainingRecord`
- Unified `ProgressTimeline::rebuild()` to accept `&[TrainingRecord]` instead of separate slices
- Unified `add_discrimination()`/`add_matching()` into single `add_record()`
- Refactored `app.rs` hydration to single discipline-driven loop using `extract_discrimination_metric`/`extract_matching_metric`
- Updated `bridge.rs` to call `store.save_record()` directly
- Updated `csv_export_import.rs` to use `fetch_all_records()` and `save_record()`
- AC7 (`TrainingDataStore` trait) was already satisfied from story 16.3

### Completion Notes

All 9 ACs satisfied. 379 domain tests pass. Clippy clean. Trunk build succeeds.
Backward compatibility (AC8): `onupgradeneeded` deletes legacy `comparison_records` store and creates new `pitch_discrimination_records`. Single-user app, IndexedDB reset is acceptable.

## File List

- `domain/src/records.rs` — added `timestamp()`, `store_name()` methods + tests on `TrainingRecord` (modified)
- `domain/src/progress_timeline.rs` — unified `rebuild()` to `&[TrainingRecord]`, merged `add_discrimination`/`add_matching` into `add_record()` (modified)
- `web/src/adapters/indexeddb_store.rs` — full rewrite: `STORE_NAMES` registry, `save_record()`, `fetch_all_records()`, `delete_all()` uses registry, DB v3 upgrade (modified)
- `web/src/app.rs` — refactored hydration to single discipline-driven loop (modified)
- `web/src/bridge.rs` — simplified `save_record` dispatch (modified)
- `web/src/adapters/csv_export_import.rs` — updated export/import to use generic store methods (modified)
- `docs/project-context.md` — updated IndexedDB store names documentation (modified)

## Change Log

- 2026-03-24: Implemented story 16.4 — generalized IndexedDB store and hydration pipeline
