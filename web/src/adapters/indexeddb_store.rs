use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

use domain::ports::StorageError;
use domain::records::{ComparisonRecord, PitchMatchingRecord};

const DB_NAME: &str = "peach";
const DB_VERSION: u32 = 2;
const COMPARISON_STORE: &str = "comparison_records";
const PITCH_MATCHING_STORE: &str = "pitch_matching_records";
const TIMESTAMP_INDEX: &str = "timestamp";

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

        // Handle onupgradeneeded — create object stores.
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

            if !db.object_store_names().contains(COMPARISON_STORE) {
                let params = web_sys::IdbObjectStoreParameters::new();
                params.set_auto_increment(true);
                let store = db
                    .create_object_store_with_optional_parameters(COMPARISON_STORE, &params)
                    .unwrap();
                store
                    .create_index_with_str(TIMESTAMP_INDEX, TIMESTAMP_INDEX)
                    .unwrap();
            }

            if !db.object_store_names().contains(PITCH_MATCHING_STORE) {
                let params = web_sys::IdbObjectStoreParameters::new();
                params.set_auto_increment(true);
                let store = db
                    .create_object_store_with_optional_parameters(PITCH_MATCHING_STORE, &params)
                    .unwrap();
                store
                    .create_index_with_str(TIMESTAMP_INDEX, TIMESTAMP_INDEX)
                    .unwrap();
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

    pub async fn save_comparison(&self, record: &ComparisonRecord) -> Result<(), StorageError> {
        let transaction = self
            .db
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

    pub async fn save_pitch_matching(
        &self,
        record: &PitchMatchingRecord,
    ) -> Result<(), StorageError> {
        let transaction = self
            .db
            .transaction_with_str_and_mode(PITCH_MATCHING_STORE, IdbTransactionMode::Readwrite)
            .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

        let store = transaction
            .object_store(PITCH_MATCHING_STORE)
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

    pub async fn fetch_all_comparisons(&self) -> Result<Vec<ComparisonRecord>, StorageError> {
        let transaction = self
            .db
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

    pub async fn delete_all(&self) -> Result<(), StorageError> {
        let store_names = [COMPARISON_STORE, PITCH_MATCHING_STORE];
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(
                &serde_wasm_bindgen::to_value(&store_names)
                    .map_err(|e| StorageError::DeleteFailed(format!("{e}")))?,
                IdbTransactionMode::Readwrite,
            )
            .map_err(|e| StorageError::DeleteFailed(format!("{e:?}")))?;

        for name in store_names {
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
