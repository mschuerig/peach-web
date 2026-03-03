use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("interval distance {0} exceeds one octave (12 semitones)")]
    IntervalOutOfRange(u8),
}
