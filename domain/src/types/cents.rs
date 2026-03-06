use serde::{Deserialize, Serialize};

/// Cent deviation from a pitch — unrestricted f64.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cents {
    pub raw_value: f64,
}

impl Cents {
    pub const PER_OCTAVE: f64 = 1200.0;
    pub const PER_SEMITONE_ET: f64 = 100.0;

    pub fn new(raw_value: f64) -> Self {
        Self { raw_value }
    }

    pub fn magnitude(&self) -> f64 {
        self.raw_value.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cents_positive() {
        let c = Cents::new(50.0);
        assert_eq!(c.raw_value, 50.0);
        assert_eq!(c.magnitude(), 50.0);
    }

    #[test]
    fn test_cents_negative() {
        let c = Cents::new(-30.5);
        assert_eq!(c.raw_value, -30.5);
        assert_eq!(c.magnitude(), 30.5);
    }

    #[test]
    fn test_cents_zero() {
        let c = Cents::new(0.0);
        assert_eq!(c.magnitude(), 0.0);
    }

    #[test]
    fn test_cents_serde_roundtrip() {
        let c = Cents::new(-25.5);
        let json = serde_json::to_string(&c).unwrap();
        let parsed: Cents = serde_json::from_str(&json).unwrap();
        assert_eq!(c, parsed);
    }
}
