# Story 16.4: Generalize IndexedDB Store and Hydration

Status: draft

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

- [ ] Task 1: Define store name registry (constant array or derived from discipline)
- [ ] Task 2: Implement `save_record(TrainingRecord)` with dispatch
- [ ] Task 3: Implement generic fetch methods
- [ ] Task 4: Update `delete_all` to use registry
- [ ] Task 5: Update `onupgradeneeded` for rhythm stores
- [ ] Task 6: Refactor hydration in `app.rs` to discipline-driven loop
- [ ] Task 7: Update `TrainingDataStore` trait in domain
- [ ] Task 8: Update tests
- [ ] Task 9: `cargo test --workspace`, `cargo clippy --workspace`, `trunk build` pass

## Dev Notes

- The per-discipline IndexedDB store names can stay as they are (`comparison_records`, `pitch_matching_records`) for backward compatibility, or be renamed to match new terminology (`pitch_discrimination_records`, `pitch_matching_records`). Since we're resetting IndexedDB anyway (single user), renaming is fine.
- For rhythm: add `rhythm_offset_detection_records` and `continuous_rhythm_matching_records` stores.
- The `TrainingRecord` enum (from story 16.3) serves as the universal record container for both save and fetch operations.
- DB version must be bumped to trigger `onupgradeneeded` for new stores.
