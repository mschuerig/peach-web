/// Direction of a rhythm timing offset relative to the beat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhythmDirection {
    /// Played before the beat (negative offset).
    Early,
    /// Played exactly on the beat (zero offset).
    OnBeat,
    /// Played after the beat (positive offset).
    Late,
}

impl RhythmDirection {
    /// All rhythm direction variants.
    pub const ALL: [RhythmDirection; 3] = [
        RhythmDirection::Early,
        RhythmDirection::OnBeat,
        RhythmDirection::Late,
    ];

    /// Classify a timing offset (in milliseconds) into a direction.
    /// Negative → Early, zero → OnBeat, positive → Late.
    ///
    /// # Panics
    /// Panics if `offset_ms` is NaN or infinite.
    pub fn from_offset_ms(offset_ms: f64) -> Self {
        assert!(
            offset_ms.is_finite(),
            "offset_ms must be finite (got {offset_ms})"
        );
        if offset_ms < 0.0 {
            RhythmDirection::Early
        } else if offset_ms == 0.0 {
            RhythmDirection::OnBeat
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
    fn test_zero_offset_is_on_beat() {
        assert_eq!(
            RhythmDirection::from_offset_ms(0.0),
            RhythmDirection::OnBeat
        );
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
    fn test_all_contains_three_variants() {
        assert_eq!(RhythmDirection::ALL.len(), 3);
        assert!(RhythmDirection::ALL.contains(&RhythmDirection::Early));
        assert!(RhythmDirection::ALL.contains(&RhythmDirection::OnBeat));
        assert!(RhythmDirection::ALL.contains(&RhythmDirection::Late));
    }

    #[test]
    #[should_panic(expected = "offset_ms must be finite")]
    fn test_nan_panics() {
        RhythmDirection::from_offset_ms(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "offset_ms must be finite")]
    fn test_infinity_panics() {
        RhythmDirection::from_offset_ms(f64::INFINITY);
    }
}
