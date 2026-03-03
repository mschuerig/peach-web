use crate::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteDuration};

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
}

/// Handle to a currently playing note. Allows stopping or adjusting the note.
pub trait PlaybackHandle {
    /// Stop playback. Idempotent — subsequent calls are no-ops.
    fn stop(&mut self);

    /// Adjust the frequency of the playing note in real time (for pitch matching).
    fn adjust_frequency(&mut self, frequency: Frequency) -> Result<(), AudioError>;
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
    fn stop_all(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

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

            fn stop_all(&mut self) {}
        }

        let mut player = MockPlayer;
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
