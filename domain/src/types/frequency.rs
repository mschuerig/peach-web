use serde::{Deserialize, Serialize};

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

    /// Create a new Frequency. Panics if value <= 0.0 (programming error invariant).
    pub fn new(raw_value: f64) -> Self {
        assert!(raw_value > 0.0, "Frequency must be > 0.0, got {raw_value}");
        Self { raw_value }
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
    #[should_panic(expected = "Frequency must be > 0.0")]
    fn test_frequency_panics_on_zero() {
        Frequency::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Frequency must be > 0.0")]
    fn test_frequency_panics_on_negative() {
        Frequency::new(-1.0);
    }

    #[test]
    fn test_frequency_serde_roundtrip() {
        let f = Frequency::new(440.0);
        let json = serde_json::to_string(&f).unwrap();
        let parsed: Frequency = serde_json::from_str(&json).unwrap();
        assert_eq!(f, parsed);
    }
}
