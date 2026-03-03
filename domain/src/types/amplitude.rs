use serde::{Deserialize, Serialize};

/// Amplitude in decibels — f32, clamped to -90.0..=12.0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AmplitudeDB {
    pub raw_value: f32,
}

impl AmplitudeDB {
    const MIN: f32 = -90.0;
    const MAX: f32 = 12.0;

    pub fn new(raw_value: f32) -> Self {
        Self {
            raw_value: raw_value.clamp(Self::MIN, Self::MAX),
        }
    }
}

/// A value clamped to 0.0..=1.0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitInterval {
    pub raw_value: f64,
}

impl UnitInterval {
    pub fn new(raw_value: f64) -> Self {
        Self {
            raw_value: raw_value.clamp(0.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amplitude_normal() {
        let a = AmplitudeDB::new(-6.0);
        assert_eq!(a.raw_value, -6.0);
    }

    #[test]
    fn test_amplitude_clamps_low() {
        let a = AmplitudeDB::new(-100.0);
        assert_eq!(a.raw_value, -90.0);
    }

    #[test]
    fn test_amplitude_clamps_high() {
        let a = AmplitudeDB::new(20.0);
        assert_eq!(a.raw_value, 12.0);
    }

    #[test]
    fn test_amplitude_at_boundaries() {
        assert_eq!(AmplitudeDB::new(-90.0).raw_value, -90.0);
        assert_eq!(AmplitudeDB::new(12.0).raw_value, 12.0);
    }

    #[test]
    fn test_unit_interval_normal() {
        let u = UnitInterval::new(0.5);
        assert_eq!(u.raw_value, 0.5);
    }

    #[test]
    fn test_unit_interval_clamps_low() {
        let u = UnitInterval::new(-0.5);
        assert_eq!(u.raw_value, 0.0);
    }

    #[test]
    fn test_unit_interval_clamps_high() {
        let u = UnitInterval::new(1.5);
        assert_eq!(u.raw_value, 1.0);
    }

    #[test]
    fn test_unit_interval_at_boundaries() {
        assert_eq!(UnitInterval::new(0.0).raw_value, 0.0);
        assert_eq!(UnitInterval::new(1.0).raw_value, 1.0);
    }
}
