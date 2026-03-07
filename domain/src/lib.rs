pub mod error;
pub mod ports;
pub mod profile;
pub mod progress_timeline;
pub mod records;
pub mod session;
pub mod strategy;
pub mod timeline;
pub mod training;
pub mod training_mode;
pub mod trend;
pub mod tuning;
pub mod types;

pub use error::DomainError;
pub use ports::{
    AudioError, NotePlayer, PitchComparisonObserver, PitchMatchingObserver, PlaybackHandle,
    Resettable, StorageError, TrainingDataStore, UserSettings,
};
pub use profile::{PerceptualNote, PerceptualProfile};
pub use progress_timeline::{BucketSize, ProgressTimeline, TimeBucket};
pub use records::{PitchComparisonRecord, PitchMatchingRecord};
pub use session::{
    FEEDBACK_DURATION_SECS, PITCH_MATCHING_VELOCITY, PitchComparisonPlaybackData,
    PitchComparisonSession, PitchComparisonSessionState, PitchMatchingPlaybackData,
    PitchMatchingSession, PitchMatchingSessionState,
};
pub use strategy::{TrainingSettings, kazez_narrow, kazez_widen, next_pitch_comparison};
pub use timeline::{PeriodAggregate, ThresholdTimeline, TimelineDataPoint};
pub use training::*;
pub use training_mode::{TrainingMode, TrainingModeConfig, TrainingModeState};
pub use trend::{Trend, TrendAnalyzer};
pub use tuning::TuningSystem;
pub use types::*;
