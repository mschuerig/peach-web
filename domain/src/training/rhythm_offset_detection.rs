use crate::types::{RhythmOffset, TempoBPM};

/// A completed rhythm offset detection trial with all context for persistence.
#[derive(Clone, Debug, PartialEq)]
pub struct CompletedRhythmOffsetDetectionTrial {
    tempo: TempoBPM,
    offset: RhythmOffset,
    is_correct: bool,
    timestamp: String,
}

impl CompletedRhythmOffsetDetectionTrial {
    pub fn new(tempo: TempoBPM, offset: RhythmOffset, is_correct: bool, timestamp: String) -> Self {
        assert!(!timestamp.is_empty(), "timestamp must not be empty");
        Self {
            tempo,
            offset,
            is_correct,
            timestamp,
        }
    }

    pub fn tempo(&self) -> TempoBPM {
        self.tempo
    }

    pub fn offset(&self) -> RhythmOffset {
        self.offset
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// Extract the metric value: absolute offset as percentage of one sixteenth note.
    pub fn metric_value(&self) -> f64 {
        self.offset.percentage_of_sixteenth(self.tempo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trial(
        bpm: u16,
        offset_ms: f64,
        is_correct: bool,
    ) -> CompletedRhythmOffsetDetectionTrial {
        CompletedRhythmOffsetDetectionTrial::new(
            TempoBPM::new(bpm),
            RhythmOffset::new(offset_ms),
            is_correct,
            "2026-03-24T12:00:00Z".to_string(),
        )
    }

    #[test]
    fn test_fields_accessible() {
        let trial = make_trial(80, -9.375, true);
        assert_eq!(trial.tempo().bpm(), 80);
        assert_eq!(trial.offset().ms(), -9.375);
        assert!(trial.is_correct());
        assert_eq!(trial.timestamp(), "2026-03-24T12:00:00Z");
    }

    #[test]
    fn test_metric_value_at_80bpm() {
        // 9.375ms at 80 BPM = 5.0% of a sixteenth note
        let trial = make_trial(80, 9.375, true);
        assert!((trial.metric_value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_negative_offset() {
        // Negative offset should give same metric (uses abs internally)
        let trial = make_trial(80, -9.375, false);
        assert!((trial.metric_value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_zero_offset() {
        let trial = make_trial(120, 0.0, true);
        assert!((trial.metric_value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_at_120bpm() {
        // 12.5ms at 120 BPM: sixteenth = 125ms → 10.0%
        let trial = make_trial(120, 12.5, true);
        assert!((trial.metric_value() - 10.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "timestamp must not be empty")]
    fn test_empty_timestamp_panics() {
        CompletedRhythmOffsetDetectionTrial::new(
            TempoBPM::new(80),
            RhythmOffset::new(0.0),
            true,
            String::new(),
        );
    }
}
