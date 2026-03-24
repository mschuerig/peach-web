pub mod error;
pub mod metric_point;
pub mod ports;
pub mod profile;
pub mod progress_timeline;
pub mod records;
pub mod session;
pub mod statistics_key;
pub mod strategy;
pub mod training;
pub mod training_discipline;
pub mod training_discipline_statistics;
pub mod trend;
pub mod tuning;
pub mod types;
pub mod welford;

pub use error::DomainError;
pub use metric_point::MetricPoint;
pub use ports::{
    AudioError, NotePlayer, PlaybackHandle, ProfileUpdating, ProgressTimelineUpdating, Resettable,
    StorageError, TrainingDataStore, TrainingRecordPersisting, UserSettings,
};
pub use profile::{COLD_START_DIFFICULTY, PerceptualProfile};
pub use progress_timeline::{BucketSize, ProgressTimeline, TimeBucket, parse_iso8601_to_epoch};
pub use records::{
    PITCH_DISCRIMINATION_STORE, PITCH_MATCHING_STORE, PitchDiscriminationRecord,
    PitchMatchingRecord, RHYTHM_OFFSET_DETECTION_STORE, RhythmOffsetDetectionRecord,
    TrainingRecord,
};
pub use session::{
    FEEDBACK_DURATION_SECS, PITCH_MATCHING_VELOCITY, PitchDiscriminationPlaybackData,
    PitchDiscriminationSession, PitchDiscriminationSessionState, PitchMatchingPlaybackData,
    PitchMatchingSession, PitchMatchingSessionState,
};
pub use statistics_key::StatisticsKey;
pub use strategy::{TrainingSettings, kazez_narrow, kazez_widen, next_pitch_discrimination_trial};
pub use training::*;
pub use training_discipline::{
    TrainingDiscipline, TrainingDisciplineConfig, TrainingDisciplineState,
};
pub use training_discipline_statistics::TrainingDisciplineStatistics;
pub use trend::Trend;
pub use tuning::TuningSystem;
pub use types::*;
pub use welford::WelfordAccumulator;
