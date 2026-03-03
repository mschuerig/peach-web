# Story 1.8: Persistence & Profile Hydration

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want my training data and settings to persist across page refreshes and browser restarts,
so that my perceptual profile accumulates over time and my preferences are remembered.

## Acceptance Criteria

1. **AC1 — IndexedDB record persistence:** Given the IndexedDB adapter is implemented, when a comparison is completed during training, then a ComparisonRecord is saved to `comparison_records` in the `peach` database (FR36), and the record contains: referenceNote, targetNote, centOffset, isCorrect, interval, tuningSystem, timestamp.

2. **AC2 — Fetch all records:** Given records exist in IndexedDB, when fetchAllComparisons() is called, then all records are returned sorted by timestamp ascending.

3. **AC3 — localStorage settings write:** Given the localStorage adapter is implemented, when a setting changes, then it is immediately saved with `peach.` prefix keys (FR30).

4. **AC4 — localStorage settings read:** Given settings exist in localStorage, when the app loads, then settings are read and applied (FR38).

5. **AC5 — Default settings fallback:** Given no settings exist in localStorage, when the app loads, then sensible defaults are used: noteRangeMin=36, noteRangeMax=84, noteDuration=1.0, referencePitch=440, tuningSystem=equalTemperament, varyLoudness=0.0.

6. **AC6 — Profile hydration:** Given comparison records exist in IndexedDB, when the app launches, then all records are replayed through profile.update() to rebuild the PerceptualProfile (FR21), and TrendAnalyzer and ThresholdTimeline are hydrated from the same records, and hydration completes in under 1 second for up to 10,000 records (NFR4).

7. **AC7 — Hydrated profile used:** Given the profile has been hydrated, when the user starts training, then the adaptive algorithm uses hydrated profile data to select the first comparison.

8. **AC8 — Storage write failure notification:** Given a storage write operation, when the write fails, then the user is informed that data may not have been saved (NFR8), and training continues — non-blocking.

9. **AC9 — Data survives refresh:** Given training records have been saved, when the page is refreshed, browser crashes, or device restarts, then all previously saved records are intact (NFR6).

## Tasks / Subtasks

- [ ] Task 1: Add ComparisonRecord and TrainingDataStore to domain crate (AC: 1,2)
  - [ ] 1.1 Create `domain/src/records.rs` with `ComparisonRecord` struct: `reference_note: u8`, `target_note: u8`, `cent_offset: f64`, `is_correct: bool`, `interval: u8`, `tuning_system: String`, `timestamp: String`. Derive `Clone, Debug, PartialEq, Serialize, Deserialize`.
  - [ ] 1.2 Implement `ComparisonRecord::from_completed(completed: &CompletedComparison) -> Self` — extract flat values from nested domain types. Use `Interval::between()` for interval field, default to 0 on error.
  - [ ] 1.3 Add `TrainingDataStore` trait to `domain/src/ports.rs`: `async fn save_comparison(&self, record: ComparisonRecord) -> Result<(), StorageError>`, `async fn fetch_all_comparisons(&self) -> Result<Vec<ComparisonRecord>, StorageError>`, `async fn delete_all(&self) -> Result<(), StorageError>`
  - [ ] 1.4 Add `StorageError` enum to `domain/src/error.rs`: `WriteFailed(String)`, `ReadFailed(String)`, `DeleteFailed(String)`, `DatabaseOpenFailed(String)`
  - [ ] 1.5 Add `pub mod records;` to `domain/src/lib.rs` and re-export `ComparisonRecord`, `StorageError`, `TrainingDataStore`
  - [ ] 1.6 Add unit tests: `ComparisonRecord::from_completed` roundtrip, field extraction correctness
  - [ ] 1.7 `cargo test -p domain` — all tests pass, `cargo clippy -p domain` — zero warnings

- [ ] Task 2: Create IndexedDB adapter (AC: 1,2,9)
  - [ ] 2.1 Add `web-sys` features to `web/Cargo.toml`: `"IdbDatabase"`, `"IdbFactory"`, `"IdbObjectStore"`, `"IdbObjectStoreParameters"`, `"IdbRequest"`, `"IdbTransaction"`, `"IdbTransactionMode"`, `"IdbOpenDbRequest"`, `"IdbVersionChangeEvent"`, `"IdbCursorWithValue"`, `"IdbCursor"`, `"IdbCursorDirection"`, `"DomException"`, `"DomStringList"`
  - [ ] 2.2 Add `serde-wasm-bindgen` dependency to `web/Cargo.toml` for JsValue ↔ Rust struct conversion
  - [ ] 2.3 Add `serde_json` as a regular dependency to `web/Cargo.toml` (needed for ComparisonRecord construction in web crate)
  - [ ] 2.4 Create `web/src/adapters/indexeddb_store.rs` with `IndexedDbStore` struct
  - [ ] 2.5 Implement `open()` async constructor: open database `"peach"` version 1, create `"comparison_records"` object store with auto-increment key and `"timestamp"` index in `onupgradeneeded`
  - [ ] 2.6 Implement `save_comparison(&self, record: ComparisonRecord) -> Result<(), StorageError>`: serialize record to JsValue via `serde-wasm-bindgen`, put into object store in a readwrite transaction
  - [ ] 2.7 Implement `fetch_all_comparisons(&self) -> Result<Vec<ComparisonRecord>, StorageError>`: open cursor on `"timestamp"` index (ascending order), collect all records, deserialize from JsValue
  - [ ] 2.8 Implement `delete_all(&self) -> Result<(), StorageError>`: clear the `"comparison_records"` object store
  - [ ] 2.9 Create async helper `idb_request_to_future(request: IdbRequest) -> Result<JsValue, JsValue>` to wrap IDB callback API in Promises/Futures
  - [ ] 2.10 Add `pub mod indexeddb_store;` to `web/src/adapters/mod.rs`

- [ ] Task 3: Create localStorage settings adapter (AC: 3,4,5)
  - [ ] 3.1 Create `web/src/adapters/localstorage_settings.rs` with `LocalStorageSettings` struct
  - [ ] 3.2 Implement `UserSettings` trait: each getter reads from localStorage with `peach.` prefix, falls back to default if key missing or parse fails
  - [ ] 3.3 Storage keys: `peach.note_range_min` (u8), `peach.note_range_max` (u8), `peach.note_duration` (f64), `peach.reference_pitch` (f64), `peach.tuning_system` (string: `"equalTemperament"` or `"justIntonation"`), `peach.vary_loudness` (f64)
  - [ ] 3.4 Implement `LocalStorageSettings::set_*()` methods for each setting — write to localStorage immediately (for story 2.1 Settings UI to use)
  - [ ] 3.5 Default values: noteRangeMin=36 (C2), noteRangeMax=84 (C6), noteDuration=1.0, referencePitch=440.0, tuningSystem=EqualTemperament, varyLoudness=0.0
  - [ ] 3.6 Add `pub mod localstorage_settings;` to `web/src/adapters/mod.rs`

- [ ] Task 4: Create DataStoreObserver bridge (AC: 1,8)
  - [ ] 4.1 Add `DataStoreObserver` to `web/src/bridge.rs`: wraps `Rc<IndexedDbStore>` and a `WriteSignal<Option<String>>` for error notification
  - [ ] 4.2 Implement `ComparisonObserver` for `DataStoreObserver`: on `comparison_completed`, construct `ComparisonRecord::from_completed()`, spawn async save via `spawn_local`, on error set the error signal (NFR8)
  - [ ] 4.3 Observer must NOT panic — catch all errors and log via `log::error!()`, set error signal for UI notification

- [ ] Task 5: Update composition root with hydration (AC: 6,7)
  - [ ] 5.1 In `web/src/app.rs`, add `RwSignal<bool>` for `is_profile_loaded` (initially false)
  - [ ] 5.2 In `web/src/app.rs`, create `Rc<IndexedDbStore>` via async `IndexedDbStore::open()` in a `spawn_local` block on mount
  - [ ] 5.3 After DB open, call `fetch_all_comparisons()` to load all records
  - [ ] 5.4 Replay each record through `profile.update(MIDINote::new(record.reference_note), record.cent_offset.abs(), record.is_correct)` — sorted by timestamp ascending
  - [ ] 5.5 Create `TrendAnalyzer` and hydrate: for each record, call `trend_analyzer.push(record.cent_offset.abs())`
  - [ ] 5.6 Create `ThresholdTimeline` and hydrate: for each record, call `timeline.push(&record.timestamp, record.cent_offset.abs(), record.is_correct, record.reference_note)`
  - [ ] 5.7 After hydration, set `is_profile_loaded` to true
  - [ ] 5.8 Provide `IndexedDbStore`, `TrendAnalyzer`, `ThresholdTimeline`, and `is_profile_loaded` via `provide_context()`
  - [ ] 5.9 Replace `DefaultSettings` usage in ComparisonView with `LocalStorageSettings`

- [ ] Task 6: Wire DataStoreObserver into ComparisonView (AC: 1,8)
  - [ ] 6.1 In `web/src/components/comparison_view.rs`, retrieve `IndexedDbStore` from context
  - [ ] 6.2 Create `DataStoreObserver` wrapping the store
  - [ ] 6.3 Add `DataStoreObserver` to the session's observers list alongside existing `ProfileObserver`
  - [ ] 6.4 Add `RwSignal<Option<String>>` for storage error display
  - [ ] 6.5 Add non-blocking error notification element to ComparisonView — shows briefly when storage write fails, does not interrupt training

- [ ] Task 7: Add TrendAnalyzer observer to ComparisonView (AC: 6)
  - [ ] 7.1 Retrieve `TrendAnalyzer` from context in ComparisonView
  - [ ] 7.2 Create `TrendObserver` in `web/src/bridge.rs`: implements `ComparisonObserver`, on completion pushes `abs(cent_offset)` to TrendAnalyzer
  - [ ] 7.3 Create `TimelineObserver` in `web/src/bridge.rs`: implements `ComparisonObserver`, on completion pushes data point to ThresholdTimeline
  - [ ] 7.4 Add both observers to the session's observers list

- [ ] Task 8: Verify and test (AC: all)
  - [ ] 8.1 `cargo test -p domain` — all existing tests pass plus new ComparisonRecord tests (no regressions, expect 223+ tests)
  - [ ] 8.2 `cargo clippy -p domain` — zero warnings
  - [ ] 8.3 `cargo clippy -p web` — zero warnings
  - [ ] 8.4 `trunk serve` — manual browser test: train several comparisons → refresh page → train again → verify profile uses hydrated data (adaptive algorithm doesn't start from scratch)
  - [ ] 8.5 Open browser DevTools → Application → IndexedDB → verify `peach` database has `comparison_records` store with saved records
  - [ ] 8.6 Open browser DevTools → Application → localStorage → verify `peach.*` keys exist (after story 2.1; for now verify read-with-defaults works)
  - [ ] 8.7 Test storage error path: simulate by opening IndexedDB in another tab with higher version → verify error notification appears and training continues
  - [ ] 8.8 Test cold start: clear all browser data → open app → verify defaults applied, empty profile, training works normally

## Dev Notes

### Core Architecture: Split Storage with Observer-Driven Persistence

Story 1.7 established the in-memory training loop. This story adds **persistence** — every completed comparison is automatically saved to IndexedDB via an observer, and on app launch the profile is rebuilt from stored records. The architecture mandates:

- **IndexedDB** for training records (structured, async, handles 10k+ records)
- **localStorage** for user settings (key-value, sync reads, `peach.*` prefix)
- **Profile is NEVER persisted directly** — it is always rebuilt from records on launch (§11.2 invariant)

```
App Launch Sequence:
mount Leptos → open IndexedDB → fetch all records → replay through profile.update()
→ hydrate TrendAnalyzer + ThresholdTimeline → set is_profile_loaded=true → show start page

Training Loop (per comparison):
handle_answer() → ComparisonSession broadcasts to observers:
  → ProfileObserver: updates in-memory profile (existing, story 1.7)
  → DataStoreObserver: saves ComparisonRecord to IndexedDB (NEW)
  → TrendObserver: pushes abs_offset to TrendAnalyzer (NEW)
  → TimelineObserver: pushes data point to ThresholdTimeline (NEW)
```

### ComparisonRecord — Flat Storage DTO (Domain Crate)

The `ComparisonRecord` is a flat persistence record distinct from the nested `CompletedComparison`. It lives in the domain crate because it's part of the `TrainingDataStore` port interface.

```rust
// domain/src/records.rs
use serde::{Deserialize, Serialize};

/// Flat persistence record for a completed comparison.
/// Blueprint §10.1 — field names match storage schema exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRecord {
    pub reference_note: u8,
    pub target_note: u8,
    pub cent_offset: f64,
    pub is_correct: bool,
    pub interval: u8,          // semitone distance 0-12, 0 if error
    pub tuning_system: String,  // "equalTemperament" or "justIntonation"
    pub timestamp: String,      // ISO 8601
}
```

**Conversion from CompletedComparison** (blueprint §10.3):

```rust
use crate::training::CompletedComparison;
use crate::types::Interval;

impl ComparisonRecord {
    pub fn from_completed(completed: &CompletedComparison) -> Self {
        let comparison = completed.comparison();
        let interval = Interval::between(
            comparison.reference_note(),
            comparison.target_note().note,
        )
        .map(|i| i.semitones())
        .unwrap_or(0); // 0 if interval > octave (blueprint: "0 if error")

        // tuning_system serialized as camelCase string per serde convention
        let tuning_system = match completed.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        };

        Self {
            reference_note: comparison.reference_note().raw_value(),
            target_note: comparison.target_note().note.raw_value(),
            cent_offset: comparison.target_note().offset.raw_value,
            is_correct: completed.is_correct(),
            interval,
            tuning_system: tuning_system.to_string(),
            timestamp: completed.timestamp().to_string(),
        }
    }
}
```

**Critical:** The `cent_offset` field stores the SIGNED value (positive = sharp, negative = flat). Profile hydration uses `abs()` when replaying — see §11.2.

### TrainingDataStore Port Trait (Domain Crate)

The trait must be object-safe and async-compatible for WASM. Since Rust async traits in `dyn` context are complex, use a non-async trait that returns results, with the web adapter handling async internally. Alternatively, define the trait with async methods using `async-trait` or just define it as a regular trait where the web crate calls methods synchronously from within async contexts.

**Recommended approach — keep the trait simple and synchronous in the domain, let the web crate handle async:**

The domain crate cannot depend on async runtimes. The `TrainingDataStore` trait should NOT be async in the domain crate. Instead, define it as a data access interface. The web adapter wraps IndexedDB's async API and provides the data. The hydration logic in `app.rs` calls `fetch_all_comparisons()` which is an async function on the adapter directly — NOT through the trait.

```rust
// domain/src/ports.rs — add to existing file

/// Error type for storage operations.
#[derive(Debug, Clone)]
pub enum StorageError {
    WriteFailed(String),
    ReadFailed(String),
    DeleteFailed(String),
    DatabaseOpenFailed(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::WriteFailed(msg) => write!(f, "Storage write failed: {msg}"),
            StorageError::ReadFailed(msg) => write!(f, "Storage read failed: {msg}"),
            StorageError::DeleteFailed(msg) => write!(f, "Storage delete failed: {msg}"),
            StorageError::DatabaseOpenFailed(msg) => write!(f, "Database open failed: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}
```

**Why NOT `thiserror` for StorageError:** The `StorageError` is intentionally simple — 4 variants with String messages. Using `thiserror` is fine too but not required for this simple enum. If you prefer consistency with the existing `DomainError` which uses thiserror, use it.

**Why NOT an async trait in domain crate:** The domain crate is pure Rust with no async runtime. The web adapter (`IndexedDbStore`) exposes async methods directly. The composition root in `app.rs` calls these async methods via `spawn_local`. The domain doesn't need to know about async.

### IndexedDB Adapter Implementation

The IndexedDB API is callback-based. Wrap each `IdbRequest` in a `js_sys::Promise` to convert to `JsFuture`.

```rust
// web/src/adapters/indexeddb_store.rs

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbObjectStore, IdbRequest, IdbTransactionMode};

use domain::ports::StorageError;
use domain::records::ComparisonRecord;

const DB_NAME: &str = "peach";
const DB_VERSION: u32 = 1;
const COMPARISON_STORE: &str = "comparison_records";
const TIMESTAMP_INDEX: &str = "timestamp";

pub struct IndexedDbStore {
    db: IdbDatabase,
}
```

**Opening the database:**

```rust
impl IndexedDbStore {
    pub async fn open() -> Result<Self, StorageError> {
        let factory = web_sys::window()
            .ok_or_else(|| StorageError::DatabaseOpenFailed("No window".into()))?
            .indexed_db()
            .map_err(|e| StorageError::DatabaseOpenFailed(format!("{e:?}")))?
            .ok_or_else(|| StorageError::DatabaseOpenFailed("IndexedDB not available".into()))?;

        let open_request = factory
            .open_with_u32(DB_NAME, DB_VERSION)
            .map_err(|e| StorageError::DatabaseOpenFailed(format!("{e:?}")))?;

        // Handle onupgradeneeded — create object stores
        let on_upgrade = Closure::once(move |event: web_sys::IdbVersionChangeEvent| {
            let db: IdbDatabase = event
                .target()
                .unwrap()
                .unchecked_into::<IdbOpenDbRequest>()
                .result()
                .unwrap()
                .unchecked_into();

            if !db.object_store_names().contains(COMPARISON_STORE) {
                let mut params = web_sys::IdbObjectStoreParameters::new();
                params.auto_increment(true);
                let store = db
                    .create_object_store_with_optional_parameters(COMPARISON_STORE, &params)
                    .unwrap();
                store
                    .create_index_with_str(TIMESTAMP_INDEX, TIMESTAMP_INDEX)
                    .unwrap();
            }
        });
        open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        open_request.forget(); // prevent Closure from being dropped

        let db_jsvalue = idb_request_to_future(open_request.unchecked_into())
            .await
            .map_err(|e| StorageError::DatabaseOpenFailed(format!("{e:?}")))?;

        let db: IdbDatabase = db_jsvalue.unchecked_into();
        Ok(Self { db })
    }
}
```

**IMPORTANT Closure lifetime note:** The `on_upgrade` Closure must remain alive until the `onupgradeneeded` event fires. Since `open()` is async and we await the result, the Closure needs to be either `.forget()`-ed or stored. Use `.forget()` here because `onupgradeneeded` fires at most once per open request.

**Correction on the code above:** The `open_request` is used both for setting the handler and for awaiting. You need to clone/reuse the request carefully. The pattern is:

```rust
// Cast open_request to IdbRequest for the future
let request_for_future: IdbRequest = open_request.clone().unchecked_into();
open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
on_upgrade.forget();

let db_jsvalue = idb_request_to_future(request_for_future).await...
```

**The async helper for converting IDB requests to Futures:**

```rust
async fn idb_request_to_future(request: IdbRequest) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve_clone = resolve.clone();
        let reject_clone = reject.clone();

        let on_success = Closure::once(move |_event: web_sys::Event| {
            // request.result() gives the result value
            let target: IdbRequest = _event.target().unwrap().unchecked_into();
            let result = target.result().unwrap_or(JsValue::UNDEFINED);
            resolve_clone.call1(&JsValue::NULL, &result).unwrap();
        });

        let on_error = Closure::once(move |_event: web_sys::Event| {
            let target: IdbRequest = _event.target().unwrap().unchecked_into();
            let error = target.error().unwrap_or(None);
            let err_val = error
                .map(|e| JsValue::from_str(&e.message()))
                .unwrap_or(JsValue::from_str("Unknown IDB error"));
            reject_clone.call1(&JsValue::NULL, &err_val).unwrap();
        });

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_success.forget();
        on_error.forget();
    });

    JsFuture::from(promise).await
}
```

**Saving a record:**

```rust
pub async fn save_comparison(&self, record: &ComparisonRecord) -> Result<(), StorageError> {
    let transaction = self.db
        .transaction_with_str_and_mode(COMPARISON_STORE, IdbTransactionMode::Readwrite)
        .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

    let store = transaction
        .object_store(COMPARISON_STORE)
        .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

    let js_value = serde_wasm_bindgen::to_value(record)
        .map_err(|e| StorageError::WriteFailed(format!("Serialization: {e}")))?;

    let request = store
        .add(&js_value)
        .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

    idb_request_to_future(request)
        .await
        .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

    Ok(())
}
```

**Fetching all records (sorted by timestamp via index):**

```rust
pub async fn fetch_all_comparisons(&self) -> Result<Vec<ComparisonRecord>, StorageError> {
    let transaction = self.db
        .transaction_with_str_and_mode(COMPARISON_STORE, IdbTransactionMode::Readonly)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let store = transaction
        .object_store(COMPARISON_STORE)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    // Use timestamp index for ascending order
    let index = store
        .index(TIMESTAMP_INDEX)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let request = index
        .open_cursor()
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let mut records = Vec::new();

    // Cursor iteration pattern: each onsuccess gives one record or null (done)
    loop {
        let result = idb_request_to_future(request.clone())
            .await
            .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

        if result.is_null() || result.is_undefined() {
            break; // No more records
        }

        let cursor: web_sys::IdbCursorWithValue = result.unchecked_into();
        let value = cursor.value()
            .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

        let record: ComparisonRecord = serde_wasm_bindgen::from_value(value)
            .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;

        records.push(record);

        cursor.continue_()
            .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;
    }

    Ok(records)
}
```

**Alternative to cursor iteration:** Use `getAll()` on the index for simpler code. `IdbIndex::get_all()` returns all values sorted by the index key. This is simpler and likely faster:

```rust
pub async fn fetch_all_comparisons(&self) -> Result<Vec<ComparisonRecord>, StorageError> {
    let transaction = self.db
        .transaction_with_str_and_mode(COMPARISON_STORE, IdbTransactionMode::Readonly)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    let store = transaction
        .object_store(COMPARISON_STORE)
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
        let record: ComparisonRecord = serde_wasm_bindgen::from_value(value)
            .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;
        records.push(record);
    }

    Ok(records)
}
```

**Prefer `getAll()` approach** — simpler, no cursor management, and supported in all modern browsers. Verify `web-sys` exposes `IdbIndex::get_all()` — if not, use the cursor approach. The web-sys feature needed is `"IdbIndex"`.

**Deleting all records:**

```rust
pub async fn delete_all(&self) -> Result<(), StorageError> {
    let transaction = self.db
        .transaction_with_str_and_mode(COMPARISON_STORE, IdbTransactionMode::Readwrite)
        .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

    let store = transaction
        .object_store(COMPARISON_STORE)
        .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

    let request = store
        .clear()
        .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

    idb_request_to_future(request)
        .await
        .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

    Ok(())
}
```

### localStorage Settings Adapter

Synchronous reads via `web_sys::window().unwrap().local_storage()`. Each getter reads one key, parses, and falls back to default on missing/invalid.

```rust
// web/src/adapters/localstorage_settings.rs
use domain::ports::UserSettings;
use domain::types::{Frequency, MIDINote, NoteDuration};
use domain::TuningSystem;

pub struct LocalStorageSettings;

impl LocalStorageSettings {
    fn get_string(key: &str) -> Option<String> {
        web_sys::window()?
            .local_storage().ok()??
            .get_item(key).ok()?
    }

    fn get_f64(key: &str, default: f64) -> f64 {
        Self::get_string(key)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(default)
    }

    fn get_u8(key: &str, default: u8) -> u8 {
        Self::get_string(key)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(default)
    }

    /// Write a value to localStorage. Used by Settings UI (story 2.1).
    pub fn set(key: &str, value: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item(key, value);
        }
    }
}

impl UserSettings for LocalStorageSettings {
    fn note_range_min(&self) -> MIDINote {
        MIDINote::new(Self::get_u8("peach.note_range_min", 36))
    }
    fn note_range_max(&self) -> MIDINote {
        MIDINote::new(Self::get_u8("peach.note_range_max", 84))
    }
    fn note_duration(&self) -> NoteDuration {
        NoteDuration::new(Self::get_f64("peach.note_duration", 1.0))
    }
    fn reference_pitch(&self) -> Frequency {
        Frequency::new(Self::get_f64("peach.reference_pitch", 440.0))
    }
    fn tuning_system(&self) -> TuningSystem {
        match Self::get_string("peach.tuning_system").as_deref() {
            Some("justIntonation") => TuningSystem::JustIntonation,
            _ => TuningSystem::EqualTemperament, // default
        }
    }
    fn vary_loudness(&self) -> f64 {
        Self::get_f64("peach.vary_loudness", 0.0)
    }
}
```

**Key conventions** (from architecture — Storage Boundaries):

| localStorage Key | Type | Default | Notes |
|---|---|---|---|
| `peach.note_range_min` | u8 | 36 (C2) | MIDI note |
| `peach.note_range_max` | u8 | 84 (C6) | MIDI note |
| `peach.note_duration` | f64 | 1.0 | Seconds |
| `peach.reference_pitch` | f64 | 440.0 | Hz |
| `peach.tuning_system` | string | `"equalTemperament"` | camelCase per serde convention |
| `peach.vary_loudness` | f64 | 0.0 | 0.0-1.0 range |
| `peach.version` | string | — | For future migration |

**Note on DefaultSettings:** After this story, `DefaultSettings` in `web/src/adapters/default_settings.rs` is superseded by `LocalStorageSettings`. The `DefaultSettings` file can be kept for now (it serves as documentation of defaults) or removed — either way, `ComparisonView` should switch to `LocalStorageSettings`.

### DataStoreObserver — Async Persistence Bridge

The observer receives synchronous `comparison_completed` callbacks but must save asynchronously to IndexedDB. Use `spawn_local` inside the observer to fire-and-forget the async save.

```rust
// web/src/bridge.rs — add to existing file

use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::ComparisonObserver;
use domain::records::ComparisonRecord;
use domain::training::CompletedComparison;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::indexeddb_store::IndexedDbStore;

pub struct DataStoreObserver {
    store: Rc<IndexedDbStore>,
    error_signal: RwSignal<Option<String>>,
}

impl DataStoreObserver {
    pub fn new(store: Rc<IndexedDbStore>, error_signal: RwSignal<Option<String>>) -> Self {
        Self { store, error_signal }
    }
}

impl ComparisonObserver for DataStoreObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let record = ComparisonRecord::from_completed(completed);
        let store = Rc::clone(&self.store);
        let error_signal = self.error_signal;

        spawn_local(async move {
            if let Err(e) = store.save_comparison(&record).await {
                log::error!("Storage write failed: {e}");
                error_signal.set(Some(
                    "Training data may not have been saved. Training continues.".to_string()
                ));
            }
        });
    }
}
```

**Critical observer rules (from architecture):**
- Observer must NOT panic — all errors caught and logged
- Observer must NOT return errors — fire-and-forget
- Storage write failures inform user via signal (NFR8), training continues
- Observer takes `&CompletedComparison` by reference — it reads and copies what it needs

### TrendObserver and TimelineObserver

```rust
// web/src/bridge.rs — add alongside DataStoreObserver

pub struct TrendObserver(pub Rc<RefCell<TrendAnalyzer>>);

impl ComparisonObserver for TrendObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let abs_offset = completed.comparison().target_note().offset.raw_value.abs();
        self.0.borrow_mut().push(abs_offset);
    }
}

pub struct TimelineObserver(pub Rc<RefCell<ThresholdTimeline>>);

impl ComparisonObserver for TimelineObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let comparison = completed.comparison();
        let abs_offset = comparison.target_note().offset.raw_value.abs();
        self.0.borrow_mut().push(
            completed.timestamp(),
            abs_offset,
            completed.is_correct(),
            comparison.reference_note().raw_value(),
        );
    }
}
```

### Composition Root — Profile Hydration (app.rs)

The composition root wiring changes significantly. Hydration is async and must complete before training buttons should be used. The `is_profile_loaded` signal controls readiness.

```rust
// web/src/app.rs — updated composition root
pub fn App() -> impl IntoView {
    // Shared state — created immediately (cold start)
    let profile = SendWrapper::new(Rc::new(RefCell::new(PerceptualProfile::new())));
    let audio_ctx_manager = SendWrapper::new(Rc::new(RefCell::new(AudioContextManager::new())));
    let trend_analyzer = SendWrapper::new(Rc::new(RefCell::new(TrendAnalyzer::new())));
    let timeline = SendWrapper::new(Rc::new(RefCell::new(ThresholdTimeline::new())));
    let is_profile_loaded = RwSignal::new(false);

    // Provide context BEFORE hydration so child components can access
    provide_context(profile.clone());
    provide_context(audio_ctx_manager);
    provide_context(trend_analyzer.clone());
    provide_context(timeline.clone());
    provide_context(is_profile_loaded);

    // Async hydration — runs after mount
    let profile_for_hydration = Rc::clone(&*profile);
    let trend_for_hydration = Rc::clone(&*trend_analyzer);
    let timeline_for_hydration = Rc::clone(&*timeline);

    spawn_local(async move {
        match IndexedDbStore::open().await {
            Ok(store) => {
                let store = Rc::new(store);

                // Fetch all records for hydration
                match store.fetch_all_comparisons().await {
                    Ok(records) => {
                        let mut prof = profile_for_hydration.borrow_mut();
                        let mut trend = trend_for_hydration.borrow_mut();
                        let mut tl = timeline_for_hydration.borrow_mut();

                        for record in &records {
                            // Profile hydration (§11.2):
                            // profile.update(note, abs(centOffset), isCorrect)
                            prof.update(
                                MIDINote::new(record.reference_note),
                                record.cent_offset.abs(),
                                record.is_correct,
                            );

                            // TrendAnalyzer hydration
                            trend.push(record.cent_offset.abs());

                            // ThresholdTimeline hydration
                            tl.push(
                                &record.timestamp,
                                record.cent_offset.abs(),
                                record.is_correct,
                                record.reference_note,
                            );
                        }

                        log::info!("Profile hydrated from {} records", records.len());
                    }
                    Err(e) => {
                        log::error!("Failed to fetch records for hydration: {e}");
                    }
                }

                // Provide store for observers to use
                // NOTE: provide_context can only be called in the component's
                // synchronous render. Use a signal or StoredValue instead.
                // Store the Rc<IndexedDbStore> in a signal for child access.
            }
            Err(e) => {
                log::error!("Failed to open IndexedDB: {e}");
            }
        }

        is_profile_loaded.set(true);
    });

    // ... router setup unchanged
}
```

**CRITICAL: Providing IndexedDbStore to child components.** Since `provide_context` must be called synchronously during render, and the store is created asynchronously, you have two options:

1. **Option A — Signal wrapping:** Create `RwSignal<Option<Rc<IndexedDbStore>>>` synchronously, provide it, then set it after async open.
2. **Option B — Open synchronously, hydrate async:** Open the DB request synchronously (the `IdbFactory::open()` call is sync — it returns immediately with an `IdbOpenDbRequest`). Only the result is async. This is cleaner.

**Recommended: Option A** — provides clean async flow:

```rust
let db_store: RwSignal<Option<Rc<IndexedDbStore>>> = RwSignal::new(None);
provide_context(db_store);

spawn_local(async move {
    match IndexedDbStore::open().await {
        Ok(store) => {
            let store = Rc::new(store);
            // ... hydration ...
            db_store.set(Some(store));
        }
        Err(e) => log::error!("Failed to open IndexedDB: {e}"),
    }
    is_profile_loaded.set(true);
});
```

Then in `ComparisonView`, read the store from the signal:
```rust
let db_store: RwSignal<Option<Rc<IndexedDbStore>>> = use_context().expect("db_store not provided");
// When creating session observers, check if store is available
if let Some(store) = db_store.get_untracked() {
    observers.push(Box::new(DataStoreObserver::new(store, error_signal)));
}
```

**Why `get_untracked()`:** The observer list is built once when the component mounts. We don't need reactive tracking — just read the current value. By the time the user clicks "Comparison" on the start page, hydration is almost certainly complete.

**Edge case: User clicks training before hydration completes.** The `is_profile_loaded` signal starts as `false`. The Start Page should disable the Comparison button (or at minimum, allow it — the profile just starts empty, which is safe). The hydration is expected to be fast (<1s for 10k records per NFR4), so this is a brief window. **Do NOT block the user** — per UX spec, training buttons should be enabled after hydration, but oscillator fallback ensures audio is always available.

### Storage Error Notification UI

A non-blocking, auto-dismissing notification in ComparisonView:

```rust
// In ComparisonView, after the training area
{move || {
    if let Some(msg) = storage_error.get() {
        view! {
            <div
                class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-amber-100 border border-amber-400 text-amber-800 px-4 py-2 rounded-lg shadow-md text-sm dark:bg-amber-900 dark:border-amber-700 dark:text-amber-200"
                role="alert"
            >
                {msg}
            </div>
        }.into_any()
    } else {
        view! { <span></span> }.into_any()
    }
}}
```

Auto-dismiss after 5 seconds:
```rust
// When error_signal changes, schedule auto-clear
Effect::new(move || {
    if storage_error.get().is_some() {
        let signal = storage_error;
        gloo_timers::callback::Timeout::new(5000, move || {
            signal.set(None);
        }).forget();
    }
});
```

### Dependencies to Add

**web/Cargo.toml — new dependencies:**
```toml
serde_json = "1"
serde-wasm-bindgen = "0.6"
```

**web/Cargo.toml — new web-sys features (add to existing list):**
```toml
"IdbDatabase"
"IdbFactory"
"IdbIndex"
"IdbObjectStore"
"IdbObjectStoreParameters"
"IdbRequest"
"IdbTransaction"
"IdbTransactionMode"
"IdbOpenDbRequest"
"IdbVersionChangeEvent"
"IdbCursorWithValue"
"IdbCursor"
"IdbCursorDirection"
"DomException"
"DomStringList"
"Storage"
```

**domain/Cargo.toml — promote serde_json to regular dependency:**

Currently `serde_json` is only a `[dev-dependencies]`. The `ComparisonRecord` module does NOT require `serde_json` at runtime — it uses `serde::Serialize`/`Deserialize` derives only. Serialization to `JsValue` happens in the web crate via `serde-wasm-bindgen`. **No change needed to domain/Cargo.toml.**

### What This Story Does NOT Implement

Explicitly scoped out (later stories):

- **Settings UI** (story 2.1) — `LocalStorageSettings` reads from localStorage but there's no UI to change settings yet. The `set()` method is provided for story 2.1 to use.
- **PitchMatchingRecord persistence** (story 4.4) — The `comparison_records` object store is created. `pitch_matching_records` store can be added when needed (DB version upgrade from 1 to 2).
- **Reset all training data** (story 2.3) — `delete_all()` is implemented but not wired to any UI yet.
- **Data export/import** (story 6.3) — Not in scope.
- **Interval mode persistence** (story 2.2) — `peach.intervals` localStorage key not used yet.
- **SoundFont loading state** (story 5.2) — `soundfont_status` signal not created yet.
- **Profile display** (stories 3.x) — TrendAnalyzer and ThresholdTimeline are hydrated but not displayed.

### Existing Code Dependencies (Verify Before Implementing)

**Domain crate types used directly:**

- `CompletedComparison` — already has all getters: `comparison()`, `is_correct()`, `tuning_system()`, `timestamp()`. Verify field access paths for record construction.
- `Comparison` — `reference_note() -> MIDINote`, `target_note() -> DetunedMIDINote`
- `DetunedMIDINote` — `note: MIDINote` (public field), `offset: Cents` (public field). `offset.raw_value` is `f64` (public field on `Cents`).
- `MIDINote::new(u8)` — constructor, `raw_value() -> u8` getter
- `MIDINote::raw_value()` — returns `u8`
- `Interval::between(MIDINote, MIDINote) -> Result<Interval, DomainError>` — returns error if > 12 semitones
- `Interval::semitones() -> u8`
- `TuningSystem` — `EqualTemperament`, `JustIntonation` variants. Serializes as `"equalTemperament"`, `"justIntonation"` via `#[serde(rename_all = "camelCase")]`
- `PerceptualProfile::new()` — cold start constructor
- `PerceptualProfile::update(note: MIDINote, cent_offset: f64, is_correct: bool)` — cent_offset must be abs value, must not be NaN
- `PerceptualProfile::reset()` — clears all 128 notes
- `PerceptualProfile::reset_matching()` — zeros matching accumulators
- `TrendAnalyzer::new()` — empty constructor
- `TrendAnalyzer::push(abs_offset: f64)` — abs_offset must be non-negative
- `TrendAnalyzer::reset()` — clears all data
- `ThresholdTimeline::new()` — empty constructor
- `ThresholdTimeline::push(timestamp: &str, cent_offset: f64, is_correct: bool, reference_note: u8)` — timestamp must be >= 10 chars ISO 8601
- `ThresholdTimeline::reset()` — clears all data
- `ComparisonObserver` trait — `fn comparison_completed(&mut self, completed: &CompletedComparison)`
- `UserSettings` trait — 6 methods: `note_range_min`, `note_range_max`, `note_duration`, `reference_pitch`, `tuning_system`, `vary_loudness`

**Web crate types used:**

- `AudioContextManager::new()` — existing, unchanged
- `OscillatorNotePlayer` — existing, unchanged
- `ProfileObserver` — existing in `bridge.rs`, wraps `Rc<RefCell<PerceptualProfile>>`
- `SendWrapper` — from `send_wrapper` crate, wraps non-Send types for Leptos context

### Project Structure Notes

```
domain/src/
├── records.rs                     (NEW — ComparisonRecord flat DTO)
├── ports.rs                       (MODIFIED — add StorageError)
├── lib.rs                         (MODIFIED — add pub mod records, re-exports)
├── error.rs                       (possibly MODIFIED if StorageError goes here)
└── ... (all other files unchanged)

web/src/
├── app.rs                         (MODIFIED — hydration, new context providers, IndexedDbStore signal)
├── bridge.rs                      (MODIFIED — add DataStoreObserver, TrendObserver, TimelineObserver)
├── main.rs                        (unchanged)
├── adapters/
│   ├── mod.rs                     (MODIFIED — add indexeddb_store, localstorage_settings modules)
│   ├── audio_context.rs           (unchanged)
│   ├── audio_oscillator.rs        (unchanged)
│   ├── default_settings.rs        (kept but superseded by localstorage_settings)
│   ├── indexeddb_store.rs          (NEW — IndexedDB TrainingDataStore implementation)
│   └── localstorage_settings.rs   (NEW — localStorage UserSettings implementation)
├── components/
│   ├── comparison_view.rs         (MODIFIED — wire DataStoreObserver, TrendObserver, TimelineObserver, error notification, use LocalStorageSettings)
│   └── ... (other views unchanged)
web/Cargo.toml                     (MODIFIED — add serde_json, serde-wasm-bindgen, web-sys IDB features)
```

- Alignment with architecture: `indexeddb_store.rs` and `localstorage_settings.rs` follow one-adapter-per-file convention
- `records.rs` in domain crate — flat DTOs used by the port interface, separate from the rich domain types in `training/`
- `bridge.rs` grows with 3 new observers — consider splitting into separate files if it becomes too large, but for now keep together since they're all small observer implementations

### Previous Story Intelligence (Story 1.7)

**Patterns established in 1.7 that MUST be followed:**

- `Rc<RefCell<>>` for shared ownership — used for profile, AudioContextManager, ComparisonSession. Apply same pattern for IndexedDbStore, TrendAnalyzer, ThresholdTimeline.
- `SendWrapper` for Leptos context — all `Rc<RefCell<>>` values provided via `provide_context` must be wrapped in `SendWrapper` for WASM single-threaded compatibility with Leptos 0.8's Send+Sync requirement.
- `spawn_local` for async operations — same pattern applies for IndexedDB operations.
- `ComparisonSession::new(profile, observers, resettables)` — observers is `Vec<Box<dyn ComparisonObserver>>`. Story 1.8 adds DataStoreObserver, TrendObserver, TimelineObserver to this list.
- Observer list currently: `[ProfileObserver]`. After 1.8: `[ProfileObserver, DataStoreObserver, TrendObserver, TimelineObserver]`.
- `ComparisonSession::start()` takes `HashSet<DirectedInterval>`, NOT `Vec` — confirmed in 1.7 debug log.
- `StoredValue::new_local()` for keeping non-Send closures alive — confirmed available in Leptos 0.8.
- Domain test count at end of story 1.7: **223 tests passing**. Do not regress.

**Code review feedback from 1.7 applicable here:**

- Eager `AudioContext::get_or_create()` in ComparisonView — already done, no change needed.
- Start Page Comparison button is a `<button>` with `use_navigate()` — already done.

**Story 1.7 Dev Notes — explicit forward references to 1.8:**

- "No persistence (story 1.8) — No IndexedDB, no localStorage. Profile is in-memory only, resets on page refresh."
- "This will be replaced by `LocalStorageSettings` in story 1.8."
- "vec![] // No resettables yet — story 1.8 adds persistence"
- "`DataStoreObserver` (story 1.8) — persists ComparisonRecord to IndexedDB"

**Known issue from 1.7:**

- Audio clicks at note start/end — no action needed, deferred to story 5.2 (SoundFont).

### Git History Context

Recent commits follow create story → implement → code review → done pattern:
```
58a6ad7 Apply code review fixes for story 1.7 and mark as done
264e817 Implement story 1.7 Comparison Training UI
cb35ec4 Add story 1.7 Comparison Training UI and mark as ready-for-dev
0719c2d Apply code review fixes for story 1.6 and mark as done
0b69ded Implement story 1.6 Comparison Session State Machine
```

**Files modified in recent commits relevant to this story:**
- `web/src/app.rs` — composition root, will be modified again
- `web/src/bridge.rs` — observers, will be extended
- `web/src/components/comparison_view.rs` — training UI, will be modified
- `web/Cargo.toml` — dependencies, will be extended
- `domain/src/lib.rs` — exports, will be extended

### References

- [Source: docs/planning-artifacts/epics.md#Story 1.8: Persistence & Profile Hydration]
- [Source: docs/planning-artifacts/architecture.md#Data Architecture]
- [Source: docs/planning-artifacts/architecture.md#Storage Boundaries]
- [Source: docs/planning-artifacts/architecture.md#Error Handling]
- [Source: docs/planning-artifacts/architecture.md#Loading & Initialization]
- [Source: docs/planning-artifacts/architecture.md#Format Patterns — Serialization]
- [Source: docs/planning-artifacts/architecture.md#Naming Patterns — Storage Keys]
- [Source: docs/planning-artifacts/architecture.md#Implementation Sequence]
- [Source: docs/planning-artifacts/architecture.md#Project Structure]
- [Source: docs/planning-artifacts/prd.md#FR36-40 Data Persistence]
- [Source: docs/planning-artifacts/prd.md#NFR4 Profile Hydration Performance]
- [Source: docs/planning-artifacts/prd.md#NFR6-8 Data Integrity]
- [Source: docs/planning-artifacts/ux-design-specification.md#Loading States]
- [Source: docs/planning-artifacts/ux-design-specification.md#Error States]
- [Source: docs/planning-artifacts/ux-design-specification.md#Form Behavior (Settings)]
- [Source: docs/ios-reference/domain-blueprint.md#§9.6 TrainingDataStore]
- [Source: docs/ios-reference/domain-blueprint.md#§10 Persistence Record Schemas]
- [Source: docs/ios-reference/domain-blueprint.md#§11 Composition Rules]
- [Source: docs/ios-reference/domain-blueprint.md#§11.2 Profile Hydration Invariant]
- [Source: docs/project-context.md#Storage Key Conventions]
- [Source: docs/project-context.md#Serialization]
- [Source: docs/project-context.md#Error Handling]
- [Source: docs/project-context.md#Observer Contract]
- [Source: docs/implementation-artifacts/1-7-comparison-training-ui.md#Dev Notes]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### Change Log

### File List
