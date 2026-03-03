use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("interval distance {0} exceeds one octave (12 semitones)")]
    IntervalOutOfRange(u8),

    #[error("invalid frequency: {0} (must be positive, finite)")]
    InvalidFrequency(f64),

    #[error("invalid MIDI note: {0} (must be 0-127)")]
    InvalidMIDINote(u8),

    #[error("invalid MIDI velocity: {0} (must be 1-127)")]
    InvalidMIDIVelocity(u8),

    #[error("invalid training settings: {0}")]
    InvalidSettings(String),

    #[error("transposition out of MIDI range: note {note} + {semitones} semitones")]
    TranspositionOutOfRange { note: u8, semitones: i16 },
}
