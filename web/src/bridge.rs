use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{ProfileUpdating, ProgressTimelineUpdating, TrainingRecordPersisting};
use domain::records::TrainingRecord;
use domain::{
    MetricPoint, PerceptualProfile, ProgressTimeline, StatisticsKey, TrainingDiscipline,
    parse_iso8601_to_epoch,
};
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::indexeddb_store::IndexedDbStore;

/// Generic profile port: updates the perceptual profile with any training result.
pub struct ProfilePort(Rc<RefCell<PerceptualProfile>>);

impl ProfilePort {
    pub fn new(profile: Rc<RefCell<PerceptualProfile>>) -> Self {
        Self(profile)
    }
}

impl ProfileUpdating for ProfilePort {
    fn update_profile(
        &mut self,
        key: StatisticsKey,
        timestamp: &str,
        value: f64,
        is_correct: bool,
    ) {
        let timestamp_secs = parse_iso8601_to_epoch(timestamp);
        let point = MetricPoint::new(timestamp_secs, value);
        self.0.borrow_mut().add_point(key, point, is_correct);
    }
}

/// Generic record persistence port: saves any training record to IndexedDB.
pub struct RecordPort {
    store_signal: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage>,
    error_signal: RwSignal<Option<String>>,
}

impl RecordPort {
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

impl TrainingRecordPersisting for RecordPort {
    fn save_record(&self, record: TrainingRecord) {
        let store = match self.store_signal.get_untracked() {
            Some(store) => store,
            None => {
                log::warn!("IndexedDB not yet available, record not persisted");
                return;
            }
        };
        let error_signal = self.error_signal;

        spawn_local(async move {
            let result = match &record {
                TrainingRecord::PitchDiscrimination(r) => store.save_pitch_discrimination(r).await,
                TrainingRecord::PitchMatching(r) => store.save_pitch_matching(r).await,
            };
            if let Err(e) = result {
                log::error!("Storage write failed: {e}");
                error_signal.set(Some(
                    "Training data may not have been saved. Training continues.".to_string(),
                ));
            }
        });
    }
}

/// Generic progress timeline port: updates timeline with any discipline metric.
pub struct TimelinePort(Rc<RefCell<ProgressTimeline>>);

impl TimelinePort {
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

impl ProgressTimelineUpdating for TimelinePort {
    fn add_metric(&mut self, discipline: TrainingDiscipline, timestamp: &str, value: f64) {
        let timestamp_secs = parse_iso8601_to_epoch(timestamp);
        let start_of_today = compute_start_of_today();
        self.0.borrow_mut().add_metric_for_discipline(
            discipline,
            timestamp_secs,
            value,
            start_of_today,
        );
    }
}
