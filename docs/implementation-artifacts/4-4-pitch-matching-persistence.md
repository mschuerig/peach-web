# Story 4.4: Pitch Matching Persistence

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want my pitch matching results to persist and feed into my perceptual profile on app launch,
so that my matching accuracy is tracked over time and survives page refreshes.

## Acceptance Criteria

1. **Given** the IndexedDB adapter **When** `fetch_all_pitch_matchings()` is called **Then** all `PitchMatchingRecord`s are returned from the `pitch_matching_records` store sorted by timestamp ascending (FR37).

2. **Given** pitch matching records exist in IndexedDB **When** the app launches **Then** all records are replayed through `profile.update_matching(note, user_cent_error.abs())` during hydration, so the Profile view shows accurate pitch matching statistics (FR18, FR21).

3. **Given** a pitch matching record has an invalid MIDI note value **When** hydration processes that record **Then** it is skipped (not crash) and a warning is logged, matching the comparison hydration pattern.

4. **Given** a storage write fails during pitch matching **When** the error occurs **Then** the user is informed via the existing error notification (NFR8) **And** training continues.

5. **Given** the app launches with both comparison and pitch matching records **When** hydration completes **Then** the profile contains accurate data from both record types and `is_profile_loaded` is set to `true` only after both are processed.

## Tasks / Subtasks

- [x] Task 1: Implement `fetch_all_pitch_matchings()` in IndexedDB adapter (AC: #1)
  - [x] 1.1 Add `pub async fn fetch_all_pitch_matchings(&self) -> Result<Vec<PitchMatchingRecord>, StorageError>` to `IndexedDbStore`
  - [x] 1.2 Mirror `fetch_all_comparisons()` exactly: open readonly transaction on `PITCH_MATCHING_STORE`, query via `TIMESTAMP_INDEX`, deserialize each record
- [x] Task 2: Add pitch matching hydration to `app.rs` (AC: #2, #3, #5)
  - [x] 2.1 After comparison hydration block, add `store.fetch_all_pitch_matchings().await`
  - [x] 2.2 For each record: `MIDINote::try_new(record.reference_note)` with skip-on-invalid (same pattern as comparisons)
  - [x] 2.3 Call `prof.update_matching(note, record.user_cent_error.abs())` for each valid record
  - [x] 2.4 Log skipped count and total hydrated count
- [x] Task 3: Verify end-to-end (AC: #4, #5)
  - [x] 3.1 `cargo test -p domain` — confirm no regressions (293 tests pass)
  - [x] 3.2 `trunk build` — confirm WASM compilation
  - [x] 3.3 `cargo clippy` — zero warnings
  - [x] 3.4 Manual browser test: do pitch matching, refresh page, verify profile stats persist

## Dev Notes

### Scope — This Is a Very Small Story

Story 4.3 already implemented the bulk of pitch matching persistence infrastructure. This story completes the **read path** — fetching records and replaying them during profile hydration. Only two files need modification, each with ~20 lines of new code.

### What Already Exists (DO NOT Rebuild)

| Component | File | Status |
|-----------|------|--------|
| `pitch_matching_records` IndexedDB store | `web/src/adapters/indexeddb_store.rs` | DB v2, exists |
| `save_pitch_matching()` | `web/src/adapters/indexeddb_store.rs:105-130` | Complete |
| `PitchMatchingDataStoreObserver` | `web/src/bridge.rs:114-152` | Wired and working |
| `PitchMatchingRecord` + `from_completed()` | `domain/src/records.rs:57-95` | Complete with tests |
| `profile.update_matching()` | `domain/src/profile.rs:192-200` | Complete with tests |
| Profile view matching stats display | `web/src/components/profile_view.rs:123-153` | Complete |
| Storage error notification | `web/src/components/pitch_matching_view.rs:541-554` | Complete |
| `delete_all()` clears both stores | `web/src/adapters/indexeddb_store.rs:167-193` | Complete |
| `TrainingDataStore` trait with both signatures | `domain/src/ports.rs:75-81` | Complete |

### Task 1: fetch_all_pitch_matchings() — Mirror Existing Pattern

Copy `fetch_all_comparisons()` (lines 132-165 in `indexeddb_store.rs`) and change:
- Store constant: `COMPARISON_STORE` → `PITCH_MATCHING_STORE`
- Record type: `ComparisonRecord` → `PitchMatchingRecord`
- Method name: `fetch_all_comparisons` → `fetch_all_pitch_matchings`

Everything else is identical — same transaction mode, same index query, same deserialization loop.

```rust
pub async fn fetch_all_pitch_matchings(&self) -> Result<Vec<PitchMatchingRecord>, StorageError> {
    let transaction = self
        .db
        .transaction_with_str_and_mode(PITCH_MATCHING_STORE, IdbTransactionMode::Readonly)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let store = transaction
        .object_store(PITCH_MATCHING_STORE)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let index = store
        .index(TIMESTAMP_INDEX)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let request = index
        .get_all()
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let result = idb_request_to_future(request)
        .await
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let array: js_sys::Array = result.unchecked_into();
    let mut records = Vec::with_capacity(array.length() as usize);

    for i in 0..array.length() {
        let value = array.get(i);
        let record: PitchMatchingRecord = serde_wasm_bindgen::from_value(value)
            .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;
        records.push(record);
    }

    Ok(records)
}
```

### Task 2: Pitch Matching Hydration — Add After Comparison Hydration

In `web/src/app.rs`, inside the `spawn_local` block, after the comparison hydration `match` block (around line 89) but **before** `db_store.set(Some(store))` (line 91):

```rust
// Pitch matching hydration
match store.fetch_all_pitch_matchings().await {
    Ok(records) => {
        let mut prof = profile_for_hydration.borrow_mut();
        let mut skipped = 0u32;

        for record in &records {
            let note = match MIDINote::try_new(record.reference_note) {
                Ok(n) => n,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            prof.update_matching(note, record.user_cent_error.abs());
        }

        if skipped > 0 {
            log::warn!(
                "Skipped {skipped} pitch matching records with invalid MIDI note values during hydration"
            );
        }
        log::info!(
            "Profile pitch matching hydrated from {} records",
            records.len() - skipped as usize
        );
    }
    Err(e) => {
        log::error!("Failed to fetch pitch matching records for hydration: {e}");
    }
}
```

**Critical notes:**
- The `profile_for_hydration` borrow from comparison hydration is **already dropped** by this point (it's inside a separate `match` arm scope), so re-borrowing is safe.
- Pitch matching hydration only updates the profile — it does NOT feed into `TrendAnalyzer` or `ThresholdTimeline` (those track comparison training only).
- `user_cent_error` field on `PitchMatchingRecord` is a signed `f64`; pass `.abs()` to `update_matching()`.

### Project Structure Notes

Files to modify:
- `web/src/adapters/indexeddb_store.rs` — add `fetch_all_pitch_matchings()` method
- `web/src/app.rs` — add pitch matching hydration block

No new files. No Cargo.toml changes. No new dependencies.

### References

- [Source: docs/planning-artifacts/epics.md#Story 4.4]
- [Source: docs/planning-artifacts/architecture.md#Data Architecture — split storage]
- [Source: docs/project-context.md#Storage Edge Cases — hydration replays ALL records]
- [Source: web/src/adapters/indexeddb_store.rs:132-165 — fetch_all_comparisons() pattern]
- [Source: web/src/app.rs:44-99 — comparison hydration pattern]
- [Source: domain/src/profile.rs:192-200 — update_matching() API]
- [Source: domain/src/records.rs:57-95 — PitchMatchingRecord schema]
- [Source: docs/implementation-artifacts/4-3-pitch-matching-training-ui.md — previous story intelligence]

## Previous Story Intelligence

### From Story 4.3 (Pitch Matching Training UI)
- PitchMatchingDataStoreObserver saves records to IndexedDB on every completed attempt — write path works
- PitchMatchingSession calls `profile.update_matching()` directly in `commit_pitch()` — no separate ProfileObserver needed (avoids double-counting)
- IndexedDB upgraded to v2 with `pitch_matching_records` object store and timestamp index
- `delete_all()` already clears both comparison_records and pitch_matching_records
- Storage error notification auto-dismisses after 5 seconds
- All 293 domain tests pass, trunk build and clippy clean

### From Story 4.3 Debug Log
- Document-level Enter/Space handler was removed (unreachable, passed wrong value)
- `let _ = adjust_frequency()` replaced with proper error logging per project rules

## Git Intelligence

Recent commits (newest first):
```
dd922d7 Apply code review fixes for story 4.3 and mark as done
f4f8901 Implement story 4.3 Pitch Matching Training UI
7c87ee9 Add story 4.3 Pitch Matching Training UI and mark as ready-for-dev
6de4315 Apply code review fixes for story 4.2 and mark as done
7c532fd Implement story 4.2 Vertical Pitch Slider
```

Convention: story creation commit → implementation commit → code review fixes commit.

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Bug fix (from 4.3): Tunable note started immediately after reference note instead of waiting for slider touch. Root cause: training loop in `pitch_matching_view.rs` called `note_player.play()` right after `on_reference_finished()`. Fix: moved tunable note playback to `slider_on_change` handler, triggered on first touch (AwaitingSliderTouch → PlayingTunable transition).
- Bug fix (from 4.3): Touching slider without moving always showed "dead center". Root cause: `calculate_adjusted_frequency()` and `commit_pitch()` ignored `initial_cent_offset` — slider center (0.0) mapped directly to the target frequency (correct answer). Fix: both functions now incorporate `initial_cent_offset`: `cent_offset = initial_offset + value * 20.0`.

### Completion Notes List

- Task 1: Added `fetch_all_pitch_matchings()` to `IndexedDbStore`, mirroring `fetch_all_comparisons()` with `PITCH_MATCHING_STORE` and `PitchMatchingRecord` types
- Task 2: Added pitch matching hydration block in `app.rs` after comparison hydration, before `db_store.set()` and `is_profile_loaded.set(true)` — uses same skip-on-invalid pattern with `MIDINote::try_new()`, calls `prof.update_matching(note, record.user_cent_error.abs())`
- Task 3: All automated checks pass (293 domain tests, trunk build, clippy zero warnings). Manual browser test revealed two bugs from story 4.3 — both fixed.
- AC#4 (storage write error notification) was already implemented in story 4.3 — no changes needed
- AC#5 satisfied: `is_profile_loaded.set(true)` executes after both comparison and pitch matching hydration complete
- Bug fixes: Fixed tunable note timing (waits for slider touch) and cent error calculation (accounts for initial_cent_offset)

### File List

- `web/src/adapters/indexeddb_store.rs` — added `fetch_all_pitch_matchings()` method
- `web/src/app.rs` — added pitch matching hydration block after comparison hydration
- `domain/src/session/pitch_matching_session.rs` — fixed `calculate_adjusted_frequency()` and `commit_pitch()` to incorporate `initial_cent_offset`; updated 5 related tests
- `web/src/components/pitch_matching_view.rs` — moved tunable note playback from training loop to `slider_on_change` (starts on first touch)

### Change Log

- 2026-03-04: Implemented pitch matching persistence read path and hydration (Tasks 1-3)
- 2026-03-04: Fixed two pitch matching bugs from story 4.3 — tunable note timing and cent error calculation
