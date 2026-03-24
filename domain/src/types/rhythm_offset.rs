use crate::types::rhythm_direction::RhythmDirection;
use crate::types::tempo::TempoBPM;

/// A signed timing offset in milliseconds relative to the beat.
/// Negative values mean the hit was early; positive values mean late.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmOffset {
    ms: f64,
}

impl RhythmOffset {
    /// Create a new RhythmOffset from a signed millisecond value.
    ///
    /// # Panics
    /// Debug-asserts that `ms` is not NaN.
    pub fn new(ms: f64) -> Self {
        debug_assert!(!ms.is_nan(), "offset_ms must not be NaN");
        Self { ms }
    }

    /// The raw signed offset in milliseconds.
    pub fn ms(&self) -> f64 {
        self.ms
    }

    /// Absolute offset in milliseconds.
    pub fn abs_ms(&self) -> f64 {
        self.ms.abs()
    }

    /// Whether this offset is early (negative) or late (non-negative).
    pub fn direction(&self) -> RhythmDirection {
        RhythmDirection::from_offset_ms(self.ms)
    }

    /// Absolute offset as a percentage of one sixteenth note at the given tempo.
    ///
    /// Formula: `abs(offset_ms) / sixteenth_note_duration_ms * 100.0`
    pub fn percentage_of_sixteenth(&self, tempo: TempoBPM) -> f64 {
        let sixteenth_ms = tempo.sixteenth_note_duration_secs() * 1000.0;
        self.ms.abs() / sixteenth_ms * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stores_value() {
        let offset = RhythmOffset::new(-9.375);
        assert_eq!(offset.ms(), -9.375);
    }

    #[test]
    fn test_abs_ms_positive() {
        assert_eq!(RhythmOffset::new(5.0).abs_ms(), 5.0);
    }

    #[test]
    fn test_abs_ms_negative() {
        assert_eq!(RhythmOffset::new(-5.0).abs_ms(), 5.0);
    }

    #[test]
    fn test_abs_ms_zero() {
        assert_eq!(RhythmOffset::new(0.0).abs_ms(), 0.0);
    }

    #[test]
    fn test_direction_negative_is_early() {
        assert_eq!(RhythmOffset::new(-5.0).direction(), RhythmDirection::Early);
    }

    #[test]
    fn test_direction_positive_is_late() {
        assert_eq!(RhythmOffset::new(5.0).direction(), RhythmDirection::Late);
    }

    #[test]
    fn test_direction_zero_is_late() {
        assert_eq!(RhythmOffset::new(0.0).direction(), RhythmDirection::Late);
    }

    #[test]
    fn test_percentage_of_sixteenth_at_80bpm() {
        // At 80 BPM: sixteenth = 0.1875s = 187.5ms
        // offset 9.375ms → 9.375 / 187.5 * 100 = 5.0%
        let offset = RhythmOffset::new(9.375);
        let pct = offset.percentage_of_sixteenth(TempoBPM::new(80));
        assert!((pct - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentage_of_sixteenth_negative_offset() {
        // Negative offset should give same percentage as positive (uses abs)
        let offset = RhythmOffset::new(-9.375);
        let pct = offset.percentage_of_sixteenth(TempoBPM::new(80));
        assert!((pct - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentage_of_sixteenth_at_120bpm() {
        // At 120 BPM: sixteenth = 0.125s = 125ms
        // offset 12.5ms → 12.5 / 125 * 100 = 10.0%
        let offset = RhythmOffset::new(12.5);
        let pct = offset.percentage_of_sixteenth(TempoBPM::new(120));
        assert!((pct - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentage_of_sixteenth_zero_offset() {
        let offset = RhythmOffset::new(0.0);
        let pct = offset.percentage_of_sixteenth(TempoBPM::new(80));
        assert!((pct - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_clone_and_copy() {
        let offset = RhythmOffset::new(3.0);
        let copy = offset;
        assert_eq!(offset, copy);
    }
}
