use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{PitchComparisonObserver, PitchMatchingObserver};
use domain::records::{PitchComparisonRecord, PitchMatchingRecord};
use domain::training::{CompletedPitchComparison, CompletedPitchMatching};
use domain::{PerceptualProfile, ProgressTimeline, ThresholdTimeline, TrendAnalyzer};
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

impl PitchComparisonObserver for ProfileObserver {
    fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison) {
        let mut profile = self.0.borrow_mut();
        let cent_offset = domain::Cents::new(
            completed
                .pitch_comparison()
                .target_note()
                .offset
                .raw_value
                .abs(),
        );
        profile.update(
            completed.pitch_comparison().reference_note(),
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

impl PitchComparisonObserver for DataStoreObserver {
    fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison) {
        let store = match self.store_signal.get_untracked() {
            Some(store) => store,
            None => {
                log::warn!("IndexedDB not yet available, record not persisted");
                return;
            }
        };
        let record = PitchComparisonRecord::from_completed(completed);
        let error_signal = self.error_signal;

        spawn_local(async move {
            if let Err(e) = store.save_pitch_comparison(&record).await {
                log::error!("Storage write failed: {e}");
                // TODO: localize when called from reactive context
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

impl PitchComparisonObserver for TrendObserver {
    fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison) {
        let abs_offset = completed
            .pitch_comparison()
            .target_note()
            .offset
            .raw_value
            .abs();
        self.0.borrow_mut().push(abs_offset);
    }
}

pub struct TimelineObserver(Rc<RefCell<ThresholdTimeline>>);

impl TimelineObserver {
    pub fn new(timeline: Rc<RefCell<ThresholdTimeline>>) -> Self {
        Self(timeline)
    }
}

impl PitchComparisonObserver for TimelineObserver {
    fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison) {
        let comparison = completed.pitch_comparison();
        let abs_offset = comparison.target_note().offset.raw_value.abs();
        self.0.borrow_mut().push(
            completed.timestamp(),
            abs_offset,
            completed.is_correct(),
            comparison.reference_note().raw_value(),
        );
    }
}

// Note: PitchMatchingProfileObserver is not needed because PitchMatchingSession
// updates the profile directly in commit_pitch() — unlike PitchComparisonSession which
// delegates to the observer.

pub struct PitchMatchingDataStoreObserver {
    store_signal: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage>,
    error_signal: RwSignal<Option<String>>,
}

impl PitchMatchingDataStoreObserver {
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

impl PitchMatchingObserver for PitchMatchingDataStoreObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching) {
        let store = match self.store_signal.get_untracked() {
            Some(store) => store,
            None => {
                log::warn!("IndexedDB not yet available, pitch matching record not persisted");
                return;
            }
        };
        let record = PitchMatchingRecord::from_completed(completed);
        let error_signal = self.error_signal;

        spawn_local(async move {
            if let Err(e) = store.save_pitch_matching(&record).await {
                log::error!("Storage write failed: {e}");
                // TODO: localize when called from reactive context
                error_signal.set(Some(
                    "Training data may not have been saved. Training continues.".to_string(),
                ));
            }
        });
    }
}

pub struct ProgressTimelineObserver(Rc<RefCell<ProgressTimeline>>);

impl ProgressTimelineObserver {
    pub fn new(timeline: Rc<RefCell<ProgressTimeline>>) -> Self {
        Self(timeline)
    }
}

fn compute_start_of_today() -> f64 {
    let date = js_sys::Date::new_0();
    date.set_hours(0);
    date.set_minutes(0);
    date.set_seconds(0);
    date.set_milliseconds(0);
    date.get_time() / 1000.0
}

impl PitchComparisonObserver for ProgressTimelineObserver {
    fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison) {
        let record = PitchComparisonRecord::from_completed(completed);
        let now = js_sys::Date::now() / 1000.0;
        let start_of_today = compute_start_of_today();
        self.0
            .borrow_mut()
            .add_comparison(&record, now, start_of_today);
    }
}

impl PitchMatchingObserver for ProgressTimelineObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching) {
        let record = PitchMatchingRecord::from_completed(completed);
        let now = js_sys::Date::now() / 1000.0;
        let start_of_today = compute_start_of_today();
        self.0
            .borrow_mut()
            .add_matching(&record, now, start_of_today);
    }
}
