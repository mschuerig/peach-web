use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

use domain::ports::StorageError;
use domain::records::{
    CONTINUOUS_RHYTHM_MATCHING_STORE, ContinuousRhythmMatchingRecord, PITCH_DISCRIMINATION_STORE,
    PITCH_MATCHING_STORE, PitchDiscriminationRecord, PitchMatchingRecord,
    RHYTHM_OFFSET_DETECTION_STORE, RhythmOffsetDetectionRecord, TrainingRecord,
};

const DB_NAME: &str = "peach";
const DB_VERSION: u32 = 3;
const TIMESTAMP_INDEX: &str = "timestamp";

/// Central registry of all IndexedDB object store names.
/// New disciplines add their store name here; all store creation, deletion,
/// and iteration derive from this single source of truth.
pub const STORE_NAMES: &[&str] = &[
    PITCH_DISCRIMINATION_STORE,
    PITCH_MATCHING_STORE,
    RHYTHM_OFFSET_DETECTION_STORE,
    CONTINUOUS_RHYTHM_MATCHING_STORE,
];

/// Legacy store names from DB versions 1-2. Deleted on upgrade to v3.
const LEGACY_STORE_NAMES: &[&str] = &["comparison_records"];

pub struct IndexedDbStore {
    db: IdbDatabase,
}

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

        // Handle onupgradeneeded — create object stores from the registry.
        // unwrap()s here are acceptable: this JS closure runs synchronously during
        // IDB upgrade, where event.target()/result() are guaranteed by the browser,
        // and store creation failure means a fundamentally broken database that
        // should surface as a panic rather than be silently ignored.
        let on_upgrade = Closure::once(move |event: web_sys::IdbVersionChangeEvent| {
            let db: IdbDatabase = event
                .target()
                .unwrap()
                .unchecked_into::<IdbOpenDbRequest>()
                .result()
                .unwrap()
                .unchecked_into();

            // Remove legacy stores from previous DB versions
            for &name in LEGACY_STORE_NAMES {
                if db.object_store_names().contains(name) {
                    db.delete_object_store(name).unwrap();
                }
            }

            // Create all registered stores that don't yet exist
            for &name in STORE_NAMES {
                if !db.object_store_names().contains(name) {
                    let params = web_sys::IdbObjectStoreParameters::new();
                    params.set_auto_increment(true);
                    let store = db
                        .create_object_store_with_optional_parameters(name, &params)
                        .unwrap();
                    store
                        .create_index_with_str(TIMESTAMP_INDEX, TIMESTAMP_INDEX)
                        .unwrap();
                }
            }
        });
        open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        // Cast open_request to IdbRequest for the future
        let request_for_future: IdbRequest = open_request.unchecked_into();

        let db_jsvalue = idb_request_to_future(request_for_future)
            .await
            .map_err(|e| StorageError::DatabaseOpenFailed(format!("{e:?}")))?;

        let db: IdbDatabase = db_jsvalue.unchecked_into();
        Ok(Self { db })
    }

    /// Save a training record to the appropriate object store, dispatched by variant.
    pub async fn save_record(&self, record: &TrainingRecord) -> Result<(), StorageError> {
        let store_name = record.store_name();

        let js_value = match record {
            TrainingRecord::PitchDiscrimination(r) => serde_wasm_bindgen::to_value(r),
            TrainingRecord::PitchMatching(r) => serde_wasm_bindgen::to_value(r),
            TrainingRecord::RhythmOffsetDetection(r) => serde_wasm_bindgen::to_value(r),
            TrainingRecord::ContinuousRhythmMatching(r) => serde_wasm_bindgen::to_value(r),
        }
        .map_err(|e| StorageError::WriteFailed(format!("Serialization: {e}")))?;

        let transaction = self
            .db
            .transaction_with_str_and_mode(store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

        let store = transaction
            .object_store(store_name)
            .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

        let request = store
            .add(&js_value)
            .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

        idb_request_to_future(request)
            .await
            .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

        Ok(())
    }

    /// Fetch all training records from all populated stores, wrapped in TrainingRecord.
    ///
    /// When adding a new `TrainingRecord` variant, add a corresponding fetch block here.
    /// The compiler enforces exhaustive matches in `save_record` and `store_name`,
    /// but this method must be updated manually.
    pub async fn fetch_all_records(&self) -> Result<Vec<TrainingRecord>, StorageError> {
        let mut all = Vec::new();

        for value in self
            .fetch_jsvalues_from_store(PITCH_DISCRIMINATION_STORE)
            .await?
        {
            let record: PitchDiscriminationRecord = serde_wasm_bindgen::from_value(value)
                .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;
            all.push(TrainingRecord::PitchDiscrimination(record));
        }

        for value in self.fetch_jsvalues_from_store(PITCH_MATCHING_STORE).await? {
            let record: PitchMatchingRecord = serde_wasm_bindgen::from_value(value)
                .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;
            all.push(TrainingRecord::PitchMatching(record));
        }

        for value in self
            .fetch_jsvalues_from_store(RHYTHM_OFFSET_DETECTION_STORE)
            .await?
        {
            let record: RhythmOffsetDetectionRecord = serde_wasm_bindgen::from_value(value)
                .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;
            all.push(TrainingRecord::RhythmOffsetDetection(record));
        }

        for value in self
            .fetch_jsvalues_from_store(CONTINUOUS_RHYTHM_MATCHING_STORE)
            .await?
        {
            let record: ContinuousRhythmMatchingRecord = serde_wasm_bindgen::from_value(value)
                .map_err(|e| StorageError::ReadFailed(format!("Deserialization: {e}")))?;
            all.push(TrainingRecord::ContinuousRhythmMatching(record));
        }

        Ok(all)
    }

    /// Fetch all raw JsValues from a named object store, ordered by timestamp index.
    async fn fetch_jsvalues_from_store(
        &self,
        store_name: &str,
    ) -> Result<Vec<JsValue>, StorageError> {
        let transaction = self
            .db
            .transaction_with_str_and_mode(store_name, IdbTransactionMode::Readonly)
            .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

        let store = transaction
            .object_store(store_name)
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
        let mut values = Vec::with_capacity(array.length() as usize);

        for i in 0..array.length() {
            values.push(array.get(i));
        }

        Ok(values)
    }

    /// Delete all records from all registered object stores.
    pub async fn delete_all(&self) -> Result<(), StorageError> {
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(STORE_NAMES)
                    .map_err(|e| StorageError::DeleteFailed(format!("{e}")))?,
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

        for &name in STORE_NAMES {
            let store = transaction
                .object_store(name)
                .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

            let request = store
                .clear()
                .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

            idb_request_to_future(request)
                .await
                .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;
        }

        Ok(())
    }
}

/// Convert an IDB request into a Rust future via a JS Promise wrapper.
/// unwrap()s in the closures below are for browser-guaranteed invariants:
/// event.target() is always the IdbRequest, and resolve/reject.call1() only fails
/// on JS engine-level failures. Returning Result is not possible from JS closures.
async fn idb_request_to_future(request: IdbRequest) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let on_success = Closure::once(move |event: web_sys::Event| {
            let target: IdbRequest = event.target().unwrap().unchecked_into();
            let result = target.result().unwrap_or(JsValue::UNDEFINED);
            resolve.call1(&JsValue::NULL, &result).unwrap();
        });

        let on_error = Closure::once(move |event: web_sys::Event| {
            let target: IdbRequest = event.target().unwrap().unchecked_into();
            let error = target.error().unwrap_or(None);
            let err_val = error
                .map(|e| JsValue::from_str(&e.message()))
                .unwrap_or(JsValue::from_str("Unknown IDB error"));
            reject.call1(&JsValue::NULL, &err_val).unwrap();
        });

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_success.forget();
        on_error.forget();
    });

    JsFuture::from(promise).await
}
