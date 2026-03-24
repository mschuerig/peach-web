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

    #[error("invalid note range: min {min} > max {max}")]
    InvalidNoteRange { min: u8, max: u8 },

    #[error("transposition out of MIDI range: note {note} + {semitones} semitones")]
    TranspositionOutOfRange { note: u8, semitones: i16 },

    #[error("invalid tempo: {0} BPM (must be 40-200)")]
    InvalidTempo(u16),
}
