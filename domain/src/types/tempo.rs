use crate::error::DomainError;

/// Tempo in beats per minute. Range 40–200 BPM. Default 80.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TempoBPM {
    bpm: u16,
}

const MIN_BPM: u16 = 40;
const MAX_BPM: u16 = 200;
const DEFAULT_BPM: u16 = 80;

impl TempoBPM {
    /// Create a new TempoBPM. Returns `Err` if value is outside 40–200.
    pub fn try_new(bpm: u16) -> Result<Self, DomainError> {
        if !(MIN_BPM..=MAX_BPM).contains(&bpm) {
            Err(DomainError::InvalidTempo(bpm))
        } else {
            Ok(Self { bpm })
        }
    }

    /// Create a new TempoBPM. Panics if value is outside 40–200.
    pub fn new(bpm: u16) -> Self {
        Self::try_new(bpm).expect("tempo must be 40-200 BPM")
    }

    /// Access the raw BPM value.
    pub fn bpm(&self) -> u16 {
        self.bpm
    }

    /// Duration of a sixteenth note at this tempo, in seconds.
    /// = 60.0 / (bpm * 4)
    pub fn sixteenth_note_duration_secs(&self) -> f64 {
        60.0 / (self.bpm as f64 * 4.0)
    }
}

impl Default for TempoBPM {
    fn default() -> Self {
        Self { bpm: DEFAULT_BPM }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_80() {
        assert_eq!(TempoBPM::default().bpm(), 80);
    }

    #[test]
    fn test_valid_boundaries() {
        assert_eq!(TempoBPM::new(40).bpm(), 40);
        assert_eq!(TempoBPM::new(200).bpm(), 200);
        assert_eq!(TempoBPM::new(120).bpm(), 120);
    }

    #[test]
    fn test_try_new_rejects_below_range() {
        assert!(TempoBPM::try_new(39).is_err());
        assert!(TempoBPM::try_new(0).is_err());
    }

    #[test]
    fn test_try_new_rejects_above_range() {
        assert!(TempoBPM::try_new(201).is_err());
        assert!(TempoBPM::try_new(u16::MAX).is_err());
    }

    #[test]
    #[should_panic(expected = "tempo must be 40-200 BPM")]
    fn test_new_panics_below_range() {
        TempoBPM::new(39);
    }

    #[test]
    #[should_panic(expected = "tempo must be 40-200 BPM")]
    fn test_new_panics_above_range() {
        TempoBPM::new(201);
    }

    #[test]
    fn test_sixteenth_note_duration_at_120bpm() {
        let tempo = TempoBPM::new(120);
        // 60 / (120 * 4) = 60 / 480 = 0.125
        assert!((tempo.sixteenth_note_duration_secs() - 0.125).abs() < 1e-10);
    }

    #[test]
    fn test_sixteenth_note_duration_at_80bpm() {
        let tempo = TempoBPM::default();
        // 60 / (80 * 4) = 60 / 320 = 0.1875
        assert!((tempo.sixteenth_note_duration_secs() - 0.1875).abs() < 1e-10);
    }

    #[test]
    fn test_sixteenth_note_duration_at_60bpm() {
        let tempo = TempoBPM::new(60);
        // 60 / (60 * 4) = 60 / 240 = 0.25
        assert!((tempo.sixteenth_note_duration_secs() - 0.25).abs() < 1e-10);
    }
}
