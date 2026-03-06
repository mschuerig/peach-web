pub mod error;
pub mod portability;
pub mod ports;
pub mod profile;
pub mod records;
pub mod session;
pub mod strategy;
pub mod timeline;
pub mod training;
pub mod trend;
pub mod tuning;
pub mod types;

pub use error::DomainError;
pub use ports::{
    AudioError, PitchComparisonObserver, NotePlayer, PitchMatchingObserver, PlaybackHandle, Resettable,
    StorageError, TrainingDataStore, UserSettings,
};
pub use records::{PitchComparisonRecord, PitchMatchingRecord};
pub use profile::{PerceptualNote, PerceptualProfile};
pub use session::{
    PitchComparisonPlaybackData, PitchComparisonSession, PitchComparisonSessionState, FEEDBACK_DURATION_SECS,
    PitchMatchingPlaybackData, PitchMatchingSession, PitchMatchingSessionState,
    PITCH_MATCHING_VELOCITY,
};
pub use strategy::{kazez_narrow, kazez_widen, next_pitch_comparison, TrainingSettings};
pub use timeline::{PeriodAggregate, ThresholdTimeline, TimelineDataPoint};
pub use training::*;
pub use trend::{Trend, TrendAnalyzer};
pub use tuning::TuningSystem;
pub use types::*;
