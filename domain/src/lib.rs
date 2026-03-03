pub mod error;
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
    AudioError, ComparisonObserver, NotePlayer, PlaybackHandle, Resettable, StorageError,
    TrainingDataStore, UserSettings,
};
pub use records::ComparisonRecord;
pub use profile::{PerceptualNote, PerceptualProfile};
pub use session::{
    ComparisonPlaybackData, ComparisonSession, ComparisonSessionState, FEEDBACK_DURATION_SECS,
};
pub use strategy::{kazez_narrow, kazez_widen, next_comparison, TrainingSettings};
pub use timeline::{PeriodAggregate, ThresholdTimeline, TimelineDataPoint};
pub use training::*;
pub use trend::{Trend, TrendAnalyzer};
pub use tuning::TuningSystem;
pub use types::*;
