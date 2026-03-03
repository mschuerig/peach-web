use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Frequency in Hz — must be > 0.0.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Frequency {
    raw_value: f64,
}

impl Frequency {
    pub const CONCERT_440: Frequency = Frequency { raw_value: 440.0 };

    /// Access the raw frequency value in Hz.
    pub fn raw_value(&self) -> f64 {
        self.raw_value
    }

    /// Create a new Frequency. Panics if value is not positive and finite.
    /// Use `try_new()` when input is not guaranteed valid.
    pub fn new(raw_value: f64) -> Self {
        Self::try_new(raw_value).expect("Frequency must be positive and finite")
    }

    /// Fallible constructor: returns `Err` if value is <= 0.0, NaN, or infinite.
    pub fn try_new(raw_value: f64) -> Result<Self, DomainError> {
        if raw_value <= 0.0 || raw_value.is_nan() || raw_value.is_infinite() {
            Err(DomainError::InvalidFrequency(raw_value))
        } else {
            Ok(Self { raw_value })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concert_440() {
        assert_eq!(Frequency::CONCERT_440.raw_value, 440.0);
    }

    #[test]
    fn test_frequency_positive() {
        let f = Frequency::new(261.63);
        assert_eq!(f.raw_value, 261.63);
    }

    #[test]
    #[should_panic(expected = "Frequency must be positive and finite")]
    fn test_frequency_panics_on_zero() {
        Frequency::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Frequency must be positive and finite")]
    fn test_frequency_panics_on_negative() {
        Frequency::new(-1.0);
    }

    #[test]
    fn test_try_new_valid() {
        assert!(Frequency::try_new(440.0).is_ok());
        assert_eq!(Frequency::try_new(440.0).unwrap().raw_value(), 440.0);
    }

    #[test]
    fn test_try_new_zero() {
        assert!(Frequency::try_new(0.0).is_err());
    }

    #[test]
    fn test_try_new_negative() {
        assert!(Frequency::try_new(-1.0).is_err());
    }

    #[test]
    fn test_try_new_nan() {
        assert!(Frequency::try_new(f64::NAN).is_err());
    }

    #[test]
    fn test_try_new_infinity() {
        assert!(Frequency::try_new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_try_new_boundary() {
        assert!(Frequency::try_new(f64::MIN_POSITIVE).is_ok());
    }

    #[test]
    fn test_frequency_serde_roundtrip() {
        let f = Frequency::new(440.0);
        let json = serde_json::to_string(&f).unwrap();
        let parsed: Frequency = serde_json::from_str(&json).unwrap();
        assert_eq!(f, parsed);
    }
}
