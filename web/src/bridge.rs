use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{PitchDiscriminationObserver, PitchMatchingObserver};
use domain::records::{PitchDiscriminationRecord, PitchMatchingRecord};
use domain::training::{CompletedPitchDiscriminationTrial, CompletedPitchMatchingTrial};
use domain::{
    MetricPoint, PerceptualProfile, ProgressTimeline, TrainingDiscipline, parse_iso8601_to_epoch,
};
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::indexeddb_store::IndexedDbStore;

/// Observer that feeds completed comparisons into the profile via `add_point()`.
/// Determines the training mode (unison vs interval) from the comparison interval.
pub struct ProfileObserver(Rc<RefCell<PerceptualProfile>>);

impl ProfileObserver {
    pub fn new(profile: Rc<RefCell<PerceptualProfile>>) -> Self {
        Self(profile)
    }
}

impl PitchDiscriminationObserver for ProfileObserver {
    fn pitch_discrimination_completed(&mut self, completed: &CompletedPitchDiscriminationTrial) {
        let comparison = completed.pitch_discrimination_trial();
        let ref_note = comparison.reference_note().raw_value();
        let target_note = comparison.target_note().note.raw_value();
        let interval = target_note.abs_diff(ref_note);

        let mode = if interval == 0 {
            TrainingDiscipline::UnisonPitchDiscrimination
        } else {
            TrainingDiscipline::IntervalPitchDiscrimination
        };

        let metric = comparison.target_note().offset.raw_value.abs();
        let timestamp_secs = parse_iso8601_to_epoch(completed.timestamp());
        let point = MetricPoint::new(timestamp_secs, domain::Cents::new(metric));

        self.0
            .borrow_mut()
            .add_point(mode, point, completed.is_correct());
    }
}

/// Observer that feeds completed pitch matching results into the profile.
pub struct PitchMatchingProfileObserver(Rc<RefCell<PerceptualProfile>>);

impl PitchMatchingProfileObserver {
    pub fn new(profile: Rc<RefCell<PerceptualProfile>>) -> Self {
        Self(profile)
    }
}

impl PitchMatchingObserver for PitchMatchingProfileObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatchingTrial) {
        let ref_note = completed.reference_note().raw_value();
        let target_note = completed.target_note().raw_value();
        let interval = target_note.abs_diff(ref_note);

        let mode = if interval == 0 {
            TrainingDiscipline::UnisonPitchMatching
        } else {
            TrainingDiscipline::IntervalPitchMatching
        };

        let metric = completed.user_cent_error().abs();
        let timestamp_secs = parse_iso8601_to_epoch(completed.timestamp());
        let point = MetricPoint::new(timestamp_secs, domain::Cents::new(metric));

        self.0.borrow_mut().add_point(mode, point, true);
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

impl PitchDiscriminationObserver for DataStoreObserver {
    fn pitch_discrimination_completed(&mut self, completed: &CompletedPitchDiscriminationTrial) {
        let store = match self.store_signal.get_untracked() {
            Some(store) => store,
            None => {
                log::warn!("IndexedDB not yet available, record not persisted");
                return;
            }
        };
        let record = PitchDiscriminationRecord::from_completed(completed);
        let error_signal = self.error_signal;

        spawn_local(async move {
            if let Err(e) = store.save_pitch_comparison(&record).await {
                log::error!("Storage write failed: {e}");
                error_signal.set(Some(
                    "Training data may not have been saved. Training continues.".to_string(),
                ));
            }
        });
    }
}

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
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatchingTrial) {
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

pub(crate) fn compute_start_of_today() -> f64 {
    let date = js_sys::Date::new_0();
    date.set_hours(0);
    date.set_minutes(0);
    date.set_seconds(0);
    date.set_milliseconds(0);
    date.get_time() / 1000.0
}

impl PitchDiscriminationObserver for ProgressTimelineObserver {
    fn pitch_discrimination_completed(&mut self, completed: &CompletedPitchDiscriminationTrial) {
        let record = PitchDiscriminationRecord::from_completed(completed);
        let start_of_today = compute_start_of_today();
        self.0.borrow_mut().add_comparison(&record, start_of_today);
    }
}

impl PitchMatchingObserver for ProgressTimelineObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatchingTrial) {
        let record = PitchMatchingRecord::from_completed(completed);
        let start_of_today = compute_start_of_today();
        self.0.borrow_mut().add_matching(&record, start_of_today);
    }
}
