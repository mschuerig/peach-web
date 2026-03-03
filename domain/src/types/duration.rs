use serde::{Deserialize, Serialize};

/// Note duration in seconds — clamped to 0.3..=3.0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteDuration {
    raw_value: f64,
}

impl NoteDuration {
    const MIN: f64 = 0.3;
    const MAX: f64 = 3.0;

    /// Access the raw duration value in seconds (0.3-3.0).
    pub fn raw_value(&self) -> f64 {
        self.raw_value
    }

    pub fn new(raw_value: f64) -> Self {
        assert!(!raw_value.is_nan(), "NoteDuration value must not be NaN");
        Self {
            raw_value: raw_value.clamp(Self::MIN, Self::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_normal() {
        let d = NoteDuration::new(1.5);
        assert_eq!(d.raw_value, 1.5);
    }

    #[test]
    fn test_duration_clamps_low() {
        let d = NoteDuration::new(0.1);
        assert_eq!(d.raw_value, 0.3);
    }

    #[test]
    fn test_duration_clamps_high() {
        let d = NoteDuration::new(5.0);
        assert_eq!(d.raw_value, 3.0);
    }

    #[test]
    fn test_duration_at_boundaries() {
        assert_eq!(NoteDuration::new(0.3).raw_value, 0.3);
        assert_eq!(NoteDuration::new(3.0).raw_value, 3.0);
    }

    #[test]
    #[should_panic(expected = "must not be NaN")]
    fn test_duration_panics_on_nan() {
        NoteDuration::new(f64::NAN);
    }

    #[test]
    fn test_duration_serde_roundtrip() {
        let d = NoteDuration::new(1.5);
        let json = serde_json::to_string(&d).unwrap();
        let parsed: NoteDuration = serde_json::from_str(&json).unwrap();
        assert_eq!(d, parsed);
    }
}
