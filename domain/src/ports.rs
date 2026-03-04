use crate::records::ComparisonRecord;
use crate::records::PitchMatchingRecord;
use crate::training::{CompletedComparison, CompletedPitchMatching};
use crate::tuning::TuningSystem;
use crate::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteRange, NoteDuration};

/// Error type for audio engine operations.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("audio engine failed to start: {0}")]
    EngineStartFailed(String),
    #[error("invalid frequency: {0}")]
    InvalidFrequency(String),
    #[error("invalid duration: {0}")]
    InvalidDuration(String),
    #[error("audio context unavailable")]
    ContextUnavailable,
    #[error("playback failed: {0}")]
    PlaybackFailed(String),
}

/// Handle to a currently playing note. Allows stopping or adjusting the note.
pub trait PlaybackHandle {
    /// Stop playback. Idempotent — subsequent calls are no-ops.
    fn stop(&mut self);

    /// Adjust the frequency of the playing note in real time (for pitch matching).
    fn adjust_frequency(&mut self, frequency: Frequency) -> Result<(), AudioError>;
}

/// Observer for comparison training events.
pub trait ComparisonObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison);
}

/// Observer for pitch matching training events.
pub trait PitchMatchingObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching);
}

/// Trait for components that can be reset when training data is cleared.
pub trait Resettable {
    fn reset(&mut self);
}

/// Trait for reading user settings. Implementations live in the web crate.
pub trait UserSettings {
    fn note_range(&self) -> NoteRange;
    fn note_duration(&self) -> NoteDuration;
    fn reference_pitch(&self) -> Frequency;
    fn tuning_system(&self) -> TuningSystem;
    fn vary_loudness(&self) -> f64; // 0.0-1.0 (UnitInterval range)
}

/// Error type for storage operations (IndexedDB, localStorage).
#[derive(Debug, Clone, thiserror::Error)]
pub enum StorageError {
    #[error("storage write failed: {0}")]
    WriteFailed(String),
    #[error("storage read failed: {0}")]
    ReadFailed(String),
    #[error("storage delete failed: {0}")]
    DeleteFailed(String),
    #[error("database open failed: {0}")]
    DatabaseOpenFailed(String),
}

/// Port trait for persisting training data (Blueprint §9.6).
///
/// The domain crate has no async runtime, so this trait defines synchronous
/// signatures. The web adapter (`IndexedDbStore`) provides matching async
/// methods directly rather than implementing this trait, because IndexedDB
/// is inherently asynchronous. The trait remains as the canonical domain
/// contract — future adapters (e.g. in-memory for testing) can implement it.
pub trait TrainingDataStore {
    fn save_comparison(&self, record: ComparisonRecord) -> Result<(), StorageError>;
    fn fetch_all_comparisons(&self) -> Result<Vec<ComparisonRecord>, StorageError>;
    fn save_pitch_matching(&self, record: PitchMatchingRecord) -> Result<(), StorageError>;
    fn fetch_all_pitch_matchings(&self) -> Result<Vec<PitchMatchingRecord>, StorageError>;
    fn delete_all(&self) -> Result<(), StorageError>;
}

/// Port trait for playing audio notes. Implementations live in the web crate.
pub trait NotePlayer {
    type Handle: PlaybackHandle;

    /// Play a note indefinitely until explicitly stopped via the returned handle.
    fn play(
        &self,
        frequency: Frequency,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<Self::Handle, AudioError>;

    /// Play a note for a specific duration. Returns immediately; audio stops automatically.
    fn play_for_duration(
        &self,
        frequency: Frequency,
        duration: NoteDuration,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<(), AudioError>;

    /// Stop all currently playing notes immediately.
    fn stop_all(&self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_write_failed_display() {
        let err = StorageError::WriteFailed("disk full".to_string());
        assert_eq!(err.to_string(), "storage write failed: disk full");
    }

    #[test]
    fn test_storage_error_read_failed_display() {
        let err = StorageError::ReadFailed("corruption".to_string());
        assert_eq!(err.to_string(), "storage read failed: corruption");
    }

    #[test]
    fn test_storage_error_delete_failed_display() {
        let err = StorageError::DeleteFailed("locked".to_string());
        assert_eq!(err.to_string(), "storage delete failed: locked");
    }

    #[test]
    fn test_storage_error_database_open_failed_display() {
        let err = StorageError::DatabaseOpenFailed("version mismatch".to_string());
        assert_eq!(err.to_string(), "database open failed: version mismatch");
    }

    #[test]
    fn test_storage_error_is_clone() {
        let err = StorageError::WriteFailed("test".to_string());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_audio_error_engine_start_failed_display() {
        let err = AudioError::EngineStartFailed("no device".to_string());
        assert_eq!(err.to_string(), "audio engine failed to start: no device");
    }

    #[test]
    fn test_audio_error_invalid_frequency_display() {
        let err = AudioError::InvalidFrequency("negative value".to_string());
        assert_eq!(err.to_string(), "invalid frequency: negative value");
    }

    #[test]
    fn test_audio_error_invalid_duration_display() {
        let err = AudioError::InvalidDuration("too short".to_string());
        assert_eq!(err.to_string(), "invalid duration: too short");
    }

    #[test]
    fn test_audio_error_context_unavailable_display() {
        let err = AudioError::ContextUnavailable;
        assert_eq!(err.to_string(), "audio context unavailable");
    }

    #[test]
    fn test_audio_error_playback_failed_display() {
        let err = AudioError::PlaybackFailed("scheduling error".to_string());
        assert_eq!(err.to_string(), "playback failed: scheduling error");
    }

    #[test]
    fn test_audio_error_is_debug() {
        let err = AudioError::ContextUnavailable;
        let debug = format!("{:?}", err);
        assert!(debug.contains("ContextUnavailable"));
    }

    /// Marker test: verifies that NotePlayer and PlaybackHandle traits compile
    /// with a mock implementation.
    #[test]
    fn test_traits_compile_with_mock() {
        struct MockHandle;

        impl PlaybackHandle for MockHandle {
            fn stop(&mut self) {}
            fn adjust_frequency(&mut self, _frequency: Frequency) -> Result<(), AudioError> {
                Ok(())
            }
        }

        struct MockPlayer;

        impl NotePlayer for MockPlayer {
            type Handle = MockHandle;

            fn play(
                &self,
                _frequency: Frequency,
                _velocity: MIDIVelocity,
                _amplitude_db: AmplitudeDB,
            ) -> Result<Self::Handle, AudioError> {
                Ok(MockHandle)
            }

            fn play_for_duration(
                &self,
                _frequency: Frequency,
                _duration: NoteDuration,
                _velocity: MIDIVelocity,
                _amplitude_db: AmplitudeDB,
            ) -> Result<(), AudioError> {
                Ok(())
            }

            fn stop_all(&self) {}
        }

        let player = MockPlayer;
        let mut handle = player
            .play(
                Frequency::new(440.0),
                MIDIVelocity::new(80),
                AmplitudeDB::new(0.0),
            )
            .unwrap();
        handle
            .adjust_frequency(Frequency::new(880.0))
            .unwrap();
        handle.stop();
        player
            .play_for_duration(
                Frequency::new(440.0),
                NoteDuration::new(1.0),
                MIDIVelocity::new(80),
                AmplitudeDB::new(0.0),
            )
            .unwrap();
        player.stop_all();
    }
}
