use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::ComparisonObserver;
use domain::records::ComparisonRecord;
use domain::training::CompletedComparison;
use domain::{PerceptualProfile, ThresholdTimeline, TrendAnalyzer};
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::indexeddb_store::IndexedDbStore;

pub struct ProfileObserver(Rc<RefCell<PerceptualProfile>>);

impl ProfileObserver {
    pub fn new(profile: Rc<RefCell<PerceptualProfile>>) -> Self {
        Self(profile)
    }
}

impl ComparisonObserver for ProfileObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let mut profile = self.0.borrow_mut();
        let cent_offset = completed.comparison().target_note().offset.raw_value.abs();
        profile.update(
            completed.comparison().reference_note(),
            cent_offset,
            completed.is_correct(),
        );
    }
}

pub struct DataStoreObserver {
    store_signal: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage>,
    error_signal: RwSignal<Option<String>>,
}

impl DataStoreObserver {
    pub fn new(
        store_signal: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage>,
        error_signal: RwSignal<Option<String>>,
    ) -> Self {
        Self {
            store_signal,
            error_signal,
        }
    }
}

impl ComparisonObserver for DataStoreObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let store = match self.store_signal.get_untracked() {
            Some(store) => store,
            None => {
                log::warn!("IndexedDB not yet available, record not persisted");
                return;
            }
        };
        let record = ComparisonRecord::from_completed(completed);
        let error_signal = self.error_signal;

        spawn_local(async move {
            if let Err(e) = store.save_comparison(&record).await {
                log::error!("Storage write failed: {e}");
                error_signal.set(Some(
                    "Training data may not have been saved. Training continues.".to_string(),
                ));
            }
        });
    }
}

pub struct TrendObserver(Rc<RefCell<TrendAnalyzer>>);

impl TrendObserver {
    pub fn new(analyzer: Rc<RefCell<TrendAnalyzer>>) -> Self {
        Self(analyzer)
    }
}

impl ComparisonObserver for TrendObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let abs_offset = completed.comparison().target_note().offset.raw_value.abs();
        self.0.borrow_mut().push(abs_offset);
    }
}

pub struct TimelineObserver(Rc<RefCell<ThresholdTimeline>>);

impl TimelineObserver {
    pub fn new(timeline: Rc<RefCell<ThresholdTimeline>>) -> Self {
        Self(timeline)
    }
}

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
