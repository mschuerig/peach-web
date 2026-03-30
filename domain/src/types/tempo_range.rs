use crate::types::TempoBPM;

/// Tempo range classification for rhythm discipline statistics.
/// Each range covers a portion of the 40–200 BPM spectrum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TempoRange {
    /// 40–59 BPM
    VerySlow,
    /// 60–79 BPM
    Slow,
    /// 80–99 BPM
    Moderate,
    /// 100–119 BPM
    Brisk,
    /// 120–159 BPM
    Fast,
    /// 160–200 BPM
    VeryFast,
}

impl TempoRange {
    /// All tempo range variants in ascending order.
    pub const ALL: [TempoRange; 6] = [
        TempoRange::VerySlow,
        TempoRange::Slow,
        TempoRange::Moderate,
        TempoRange::Brisk,
        TempoRange::Fast,
        TempoRange::VeryFast,
    ];

    /// Midpoint BPM for threshold calculations.
    pub fn midpoint_bpm(self) -> u16 {
        match self {
            TempoRange::VerySlow => 50,
            TempoRange::Slow => 70,
            TempoRange::Moderate => 90,
            TempoRange::Brisk => 110,
            TempoRange::Fast => 140,
            TempoRange::VeryFast => 180,
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
        if bpm < 60 {
            TempoRange::VerySlow
        } else if bpm < 80 {
            TempoRange::Slow
        } else if bpm < 100 {
            TempoRange::Moderate
        } else if bpm < 120 {
            TempoRange::Brisk
        } else if bpm < 160 {
            TempoRange::Fast
        } else {
            TempoRange::VeryFast
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_very_slow_range_boundaries() {
        assert_eq!(
            TempoRange::from_bpm(TempoBPM::new(40)),
            TempoRange::VerySlow
        );
        assert_eq!(
            TempoRange::from_bpm(TempoBPM::new(59)),
            TempoRange::VerySlow
        );
    }

    #[test]
    fn test_slow_range_boundaries() {
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(60)), TempoRange::Slow);
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(79)), TempoRange::Slow);
    }

    #[test]
    fn test_moderate_range_boundaries() {
        assert_eq!(
            TempoRange::from_bpm(TempoBPM::new(80)),
            TempoRange::Moderate
        );
        assert_eq!(
            TempoRange::from_bpm(TempoBPM::new(99)),
            TempoRange::Moderate
        );
    }

    #[test]
    fn test_brisk_range_boundaries() {
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(100)), TempoRange::Brisk);
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(119)), TempoRange::Brisk);
    }

    #[test]
    fn test_fast_range_boundaries() {
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(120)), TempoRange::Fast);
        assert_eq!(TempoRange::from_bpm(TempoBPM::new(159)), TempoRange::Fast);
    }

    #[test]
    fn test_very_fast_range_boundaries() {
        assert_eq!(
            TempoRange::from_bpm(TempoBPM::new(160)),
            TempoRange::VeryFast
        );
        assert_eq!(
            TempoRange::from_bpm(TempoBPM::new(200)),
            TempoRange::VeryFast
        );
    }

    #[test]
    fn test_all_contains_six_variants() {
        assert_eq!(TempoRange::ALL.len(), 6);
        assert!(TempoRange::ALL.contains(&TempoRange::VerySlow));
        assert!(TempoRange::ALL.contains(&TempoRange::Slow));
        assert!(TempoRange::ALL.contains(&TempoRange::Moderate));
        assert!(TempoRange::ALL.contains(&TempoRange::Brisk));
        assert!(TempoRange::ALL.contains(&TempoRange::Fast));
        assert!(TempoRange::ALL.contains(&TempoRange::VeryFast));
    }
}
