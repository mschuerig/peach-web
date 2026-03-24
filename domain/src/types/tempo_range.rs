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
