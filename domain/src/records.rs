use serde::{Deserialize, Serialize};

use crate::training::CompletedComparison;
use crate::tuning::TuningSystem;
use crate::types::Interval;

/// Flat persistence record for a completed comparison.
/// Blueprint §10.1 — field names match storage schema exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRecord {
    pub reference_note: u8,
    pub target_note: u8,
    pub cent_offset: f64,
    pub is_correct: bool,
    pub interval: u8,
    pub tuning_system: String,
    pub timestamp: String,
}

impl ComparisonRecord {
    /// Construct a flat persistence record from a completed comparison.
    ///
    /// The `interval` field stores semitone distance. `Interval::between()` returns
    /// `Err` when the distance exceeds one octave (13+ semitones), which can happen
    /// with large transposition intervals. Defaulting to 0 (Prime) is safe here
    /// because this field is informational for the storage schema, not used for
    /// any logic that depends on accurate interval reconstruction.
    pub fn from_completed(completed: &CompletedComparison) -> Self {
        let comparison = completed.comparison();
        let interval = Interval::between(
            comparison.reference_note(),
            comparison.target_note().note,
        )
        .map(|i| i.semitones())
        .unwrap_or(0);

        let tuning_system = match completed.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        };

        Self {
            reference_note: comparison.reference_note().raw_value(),
            target_note: comparison.target_note().note.raw_value(),
            cent_offset: comparison.target_note().offset.raw_value,
            is_correct: completed.is_correct(),
            interval,
            tuning_system: tuning_system.to_string(),
            timestamp: completed.timestamp().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::training::Comparison;
    use crate::types::{Cents, DetunedMIDINote, MIDINote};

    fn make_completed(
        ref_note: u8,
        target_note: u8,
        offset: f64,
        higher: bool,
        tuning: TuningSystem,
    ) -> CompletedComparison {
        let comp = Comparison::new(
            MIDINote::new(ref_note),
            DetunedMIDINote {
                note: MIDINote::new(target_note),
                offset: Cents::new(offset),
            },
        );
        CompletedComparison::new(comp, higher, tuning, "2026-03-03T14:00:00Z".to_string())
    }

    #[test]
    fn test_from_completed_extracts_fields_correctly() {
        let completed = make_completed(60, 64, 25.0, true, TuningSystem::EqualTemperament);
        let record = ComparisonRecord::from_completed(&completed);

        assert_eq!(record.reference_note, 60);
        assert_eq!(record.target_note, 64);
        assert_eq!(record.cent_offset, 25.0);
        assert!(record.is_correct); // target higher (offset > 0), user said higher
        assert_eq!(record.interval, 4); // 4 semitones = major third
        assert_eq!(record.tuning_system, "equalTemperament");
        assert_eq!(record.timestamp, "2026-03-03T14:00:00Z");
    }

    #[test]
    fn test_from_completed_negative_offset() {
        let completed = make_completed(60, 60, -30.0, false, TuningSystem::JustIntonation);
        let record = ComparisonRecord::from_completed(&completed);

        assert_eq!(record.cent_offset, -30.0);
        assert!(record.is_correct); // target lower (offset < 0), user said lower
        assert_eq!(record.interval, 0); // same note = prime
        assert_eq!(record.tuning_system, "justIntonation");
    }

    #[test]
    fn test_from_completed_interval_exceeds_octave_defaults_to_zero() {
        // 13 semitones apart — exceeds octave, Interval::between returns Err
        let completed = make_completed(60, 73, 10.0, true, TuningSystem::EqualTemperament);
        let record = ComparisonRecord::from_completed(&completed);

        assert_eq!(record.interval, 0);
    }

    #[test]
    fn test_from_completed_octave_interval() {
        let completed = make_completed(60, 72, 15.0, true, TuningSystem::EqualTemperament);
        let record = ComparisonRecord::from_completed(&completed);

        assert_eq!(record.interval, 12);
    }

    #[test]
    fn test_serde_roundtrip() {
        let completed = make_completed(69, 76, 42.5, true, TuningSystem::EqualTemperament);
        let record = ComparisonRecord::from_completed(&completed);

        let json = serde_json::to_string(&record).unwrap();
        let parsed: ComparisonRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, parsed);
    }

    #[test]
    fn test_serde_field_names_match_storage_schema() {
        let record = ComparisonRecord {
            reference_note: 60,
            target_note: 67,
            cent_offset: 25.0,
            is_correct: true,
            interval: 7,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-03T14:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"reference_note\""));
        assert!(json.contains("\"target_note\""));
        assert!(json.contains("\"cent_offset\""));
        assert!(json.contains("\"is_correct\""));
        assert!(json.contains("\"interval\""));
        assert!(json.contains("\"tuning_system\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_from_completed_zero_offset() {
        // offset=0.0: is_target_higher=false, user_answered_higher=false → correct
        let completed = make_completed(60, 60, 0.0, false, TuningSystem::EqualTemperament);
        let record = ComparisonRecord::from_completed(&completed);

        assert_eq!(record.cent_offset, 0.0);
        assert!(record.is_correct);
    }
}
