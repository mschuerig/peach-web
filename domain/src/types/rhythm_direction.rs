/// Direction of a rhythm timing offset relative to the beat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhythmDirection {
    /// Played before the beat (negative offset).
    Early,
    /// Played on or after the beat (zero or positive offset).
    Late,
}

impl RhythmDirection {
    /// All rhythm direction variants.
    pub const ALL: [RhythmDirection; 2] = [RhythmDirection::Early, RhythmDirection::Late];

    /// Classify a timing offset (in milliseconds) into a direction.
    /// Negative → Early, zero or positive → Late.
    ///
    /// # Panics
    /// Debug-asserts that `offset_ms` is not NaN.
    pub fn from_offset_ms(offset_ms: f64) -> Self {
        debug_assert!(!offset_ms.is_nan(), "offset_ms must not be NaN");
        if offset_ms < 0.0 {
            RhythmDirection::Early
        } else {
            RhythmDirection::Late
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_offset_is_early() {
        assert_eq!(
            RhythmDirection::from_offset_ms(-5.0),
            RhythmDirection::Early
        );
        assert_eq!(
            RhythmDirection::from_offset_ms(-0.001),
            RhythmDirection::Early
        );
    }

    #[test]
    fn test_zero_offset_is_late() {
        assert_eq!(RhythmDirection::from_offset_ms(0.0), RhythmDirection::Late);
    }

    #[test]
    fn test_positive_offset_is_late() {
        assert_eq!(RhythmDirection::from_offset_ms(5.0), RhythmDirection::Late);
        assert_eq!(
            RhythmDirection::from_offset_ms(0.001),
            RhythmDirection::Late
        );
    }

    #[test]
    fn test_all_contains_two_variants() {
        assert_eq!(RhythmDirection::ALL.len(), 2);
        assert!(RhythmDirection::ALL.contains(&RhythmDirection::Early));
        assert!(RhythmDirection::ALL.contains(&RhythmDirection::Late));
    }
}
