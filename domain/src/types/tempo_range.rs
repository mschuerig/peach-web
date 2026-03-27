use crate::types::TempoBPM;

/// Tempo range classification for rhythm discipline statistics.
/// Each range covers a portion of the 40–200 BPM spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TempoRange {
    /// 40–79 BPM
    Slow,
    /// 80–119 BPM
    Medium,
    /// 120–200 BPM
    Fast,
}

impl TempoRange {
    /// All tempo range variants.
    pub const ALL: [TempoRange; 3] = [TempoRange::Slow, TempoRange::Medium, TempoRange::Fast];

    /// Midpoint BPM for threshold calculations.
    /// Slow: 60, Medium: 100, Fast: 160.
    pub fn midpoint_bpm(self) -> u16 {
        match self {
            TempoRange::Slow => 60,
            TempoRange::Medium => 100,
            TempoRange::Fast => 160,
        }
    }

    /// Duration of a 16th note in milliseconds at the midpoint tempo.
    pub fn sixteenth_note_ms(self) -> f64 {
        // One beat = 60_000 / bpm ms. A 16th note = beat / 4.
        60_000.0 / self.midpoint_bpm() as f64 / 4.0
    }

    /// Classify a tempo into its range.
    pub fn from_bpm(tempo: TempoBPM) -> Self {
        let bpm = tempo.bpm();
        if bpm < 80 {
            TempoRange::Slow
        } else if bpm < 120 {
            TempoRange::Medium
        } else {
            TempoRange::Fast
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_range_boundaries() {
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(40)), TempoRange::Slow);
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(79)), TempoRange::Slow);
    }

    #[test]
    fn test_medium_range_boundaries() {
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(80)), TempoRange::Medium);
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(119)), TempoRange::Medium);
    }

    #[test]
    fn test_fast_range_boundaries() {
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(120)), TempoRange::Fast);
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(200)), TempoRange::Fast);
    }

    #[test]
    fn test_all_contains_three_variants() {
        assert_eq!(TempoRange::ALL.len(), 3);
        assert!(TempoRange::ALL.contains(&TempoRange::Slow));
        assert!(TempoRange::ALL.contains(&TempoRange::Medium));
        assert!(TempoRange::ALL.contains(&TempoRange::Fast));
    }
}
