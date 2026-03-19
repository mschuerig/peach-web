pub mod error;
pub mod metric_point;
pub mod ports;
pub mod profile;
pub mod progress_timeline;
pub mod records;
pub mod session;
pub mod strategy;
pub mod training;
pub mod training_mode;
pub mod training_mode_statistics;
pub mod trend;
pub mod tuning;
pub mod types;
pub mod welford;

pub use error::DomainError;
pub use metric_point::MetricPoint;
pub use ports::{
    AudioError, NotePlayer, PitchComparisonObserver, PitchMatchingObserver, PlaybackHandle,
    Resettable, StorageError, TrainingDataStore, UserSettings,
};
pub use profile::{COLD_START_DIFFICULTY, PerceptualProfile};
pub use progress_timeline::{BucketSize, ProgressTimeline, TimeBucket, parse_iso8601_to_epoch};
pub use records::{PitchComparisonRecord, PitchMatchingRecord};
pub use session::{
    FEEDBACK_DURATION_SECS, PITCH_MATCHING_VELOCITY, PitchComparisonPlaybackData,
    PitchComparisonSession, PitchComparisonSessionState, PitchMatchingPlaybackData,
    PitchMatchingSession, PitchMatchingSessionState,
};
pub use strategy::{TrainingSettings, kazez_narrow, kazez_widen, next_pitch_comparison};
pub use training::*;
pub use training_mode::{TrainingMode, TrainingModeConfig, TrainingModeState};
pub use training_mode_statistics::TrainingModeStatistics;
pub use trend::Trend;
pub use tuning::TuningSystem;
pub use types::*;
pub use welford::{WelfordAccumulator, WelfordMeasurement};
