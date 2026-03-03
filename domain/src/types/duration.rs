use serde::{Deserialize, Serialize};

/// Note duration in seconds — clamped to 0.3..=3.0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteDuration {
    pub raw_value: f64,
}

impl NoteDuration {
    const MIN: f64 = 0.3;
    const MAX: f64 = 3.0;

    pub fn new(raw_value: f64) -> Self {
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
}
