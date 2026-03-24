use serde::{Deserialize, Serialize};

use crate::training::{CompletedPitchDiscriminationTrial, CompletedPitchMatchingTrial};
use crate::tuning::TuningSystem;
use crate::types::Interval;

/// IndexedDB object store name for pitch discrimination records.
pub const PITCH_DISCRIMINATION_STORE: &str = "pitch_discrimination_records";
/// IndexedDB object store name for pitch matching records.
pub const PITCH_MATCHING_STORE: &str = "pitch_matching_records";

/// Enum wrapping all training record types for generic persistence.
#[derive(Clone, Debug, PartialEq)]
pub enum TrainingRecord {
    PitchDiscrimination(PitchDiscriminationRecord),
    PitchMatching(PitchMatchingRecord),
}

impl TrainingRecord {
    /// Returns the timestamp of the underlying record.
    pub fn timestamp(&self) -> &str {
        match self {
            TrainingRecord::PitchDiscrimination(r) => &r.timestamp,
            TrainingRecord::PitchMatching(r) => &r.timestamp,
        }
    }

    /// Returns the IndexedDB object store name for this record type.
    pub fn store_name(&self) -> &'static str {
        match self {
            TrainingRecord::PitchDiscrimination(_) => PITCH_DISCRIMINATION_STORE,
            TrainingRecord::PitchMatching(_) => PITCH_MATCHING_STORE,
        }
    }
}

/// Flat persistence record for a completed pitch discrimination trial.
/// Blueprint §10.1 — field names match storage schema exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PitchDiscriminationRecord {
    pub reference_note: u8,
    pub target_note: u8,
    pub cent_offset: f64,
    pub is_correct: bool,
    pub interval: u8,
    pub tuning_system: String,
    pub timestamp: String,
}

impl PitchDiscriminationRecord {
    /// Construct a flat persistence record from a completed pitch discrimination trial.
    ///
    /// The `interval` field stores semitone distance. `Interval::between()` returns
    /// `Err` when the distance exceeds one octave (13+ semitones), which can happen
    /// with large transposition intervals. Defaulting to 0 (Prime) is safe here
    /// because this field is informational for the storage schema, not used for
    /// any logic that depends on accurate interval reconstruction.
    pub fn from_completed(completed: &CompletedPitchDiscriminationTrial) -> Self {
        let pitch_discrimination_trial = completed.pitch_discrimination_trial();
        let interval = Interval::between(
            pitch_discrimination_trial.reference_note(),
            pitch_discrimination_trial.target_note().note,
        )
        .map(|i| i.semitones())
        .unwrap_or(0);

        let tuning_system = match completed.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        };

        Self {
            reference_note: pitch_discrimination_trial.reference_note().raw_value(),
            target_note: pitch_discrimination_trial.target_note().note.raw_value(),
            cent_offset: pitch_discrimination_trial.target_note().offset.raw_value,
            is_correct: completed.is_correct(),
            interval,
            tuning_system: tuning_system.to_string(),
            timestamp: completed.timestamp().to_string(),
        }
    }
}

/// Flat persistence record for a completed pitch matching attempt.
/// Blueprint §10.2 — field names match storage schema exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PitchMatchingRecord {
    pub reference_note: u8,
    pub target_note: u8,
    pub initial_cent_offset: f64,
    pub user_cent_error: f64,
    pub interval: u8,
    pub tuning_system: String,
    pub timestamp: String,
}

impl PitchMatchingRecord {
    /// Construct a flat persistence record from a completed pitch matching attempt.
    ///
    /// The `interval` field stores semitone distance. `Interval::between()` returns
    /// `Err` when the distance exceeds one octave (13+ semitones). Defaulting to 0
    /// (Prime) is safe here because this field is informational for the storage schema.
    pub fn from_completed(completed: &CompletedPitchMatchingTrial) -> Self {
        let interval = Interval::between(completed.reference_note(), completed.target_note())
            .map(|i| i.semitones())
            .unwrap_or(0);

        let tuning_system = match completed.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        };

        Self {
            reference_note: completed.reference_note().raw_value(),
            target_note: completed.target_note().raw_value(),
            initial_cent_offset: completed.initial_cent_offset(),
            user_cent_error: completed.user_cent_error(),
            interval,
            tuning_system: tuning_system.to_string(),
            timestamp: completed.timestamp().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::training::PitchDiscriminationTrial;
    use crate::types::{Cents, DetunedMIDINote, MIDINote};

    fn make_completed(
        ref_note: u8,
        target_note: u8,
        offset: f64,
        higher: bool,
        tuning: TuningSystem,
    ) -> CompletedPitchDiscriminationTrial {
        let comp = PitchDiscriminationTrial::new(
            MIDINote::new(ref_note),
            DetunedMIDINote {
                note: MIDINote::new(target_note),
                offset: Cents::new(offset),
            },
        );
        CompletedPitchDiscriminationTrial::new(
            comp,
            higher,
            tuning,
            "2026-03-03T14:00:00Z".to_string(),
        )
    }

    #[test]
    fn test_from_completed_extracts_fields_correctly() {
        let completed = make_completed(60, 64, 25.0, true, TuningSystem::EqualTemperament);
        let record = PitchDiscriminationRecord::from_completed(&completed);

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
        let record = PitchDiscriminationRecord::from_completed(&completed);

        assert_eq!(record.cent_offset, -30.0);
        assert!(record.is_correct); // target lower (offset < 0), user said lower
        assert_eq!(record.interval, 0); // same note = prime
        assert_eq!(record.tuning_system, "justIntonation");
    }

    #[test]
    fn test_from_completed_interval_exceeds_octave_defaults_to_zero() {
        // 13 semitones apart — exceeds octave, Interval::between returns Err
        let completed = make_completed(60, 73, 10.0, true, TuningSystem::EqualTemperament);
        let record = PitchDiscriminationRecord::from_completed(&completed);

        assert_eq!(record.interval, 0);
    }

    #[test]
    fn test_from_completed_octave_interval() {
        let completed = make_completed(60, 72, 15.0, true, TuningSystem::EqualTemperament);
        let record = PitchDiscriminationRecord::from_completed(&completed);

        assert_eq!(record.interval, 12);
    }

    #[test]
    fn test_serde_roundtrip() {
        let completed = make_completed(69, 76, 42.5, true, TuningSystem::EqualTemperament);
        let record = PitchDiscriminationRecord::from_completed(&completed);

        let json = serde_json::to_string(&record).unwrap();
        let parsed: PitchDiscriminationRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, parsed);
    }

    #[test]
    fn test_serde_field_names_match_storage_schema() {
        let record = PitchDiscriminationRecord {
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
        let record = PitchDiscriminationRecord::from_completed(&completed);

        assert_eq!(record.cent_offset, 0.0);
        assert!(record.is_correct);
    }

    // --- PitchMatchingRecord tests ---

    fn make_pitch_matching(
        ref_note: u8,
        target_note: u8,
        initial_offset: f64,
        user_error: f64,
        tuning: TuningSystem,
    ) -> CompletedPitchMatchingTrial {
        CompletedPitchMatchingTrial::new(
            MIDINote::new(ref_note),
            MIDINote::new(target_note),
            initial_offset,
            user_error,
            tuning,
            "2026-03-04T10:00:00Z".to_string(),
        )
    }

    #[test]
    fn test_pitch_matching_from_completed_extracts_fields() {
        let completed = make_pitch_matching(60, 67, 15.0, 3.2, TuningSystem::EqualTemperament);
        let record = PitchMatchingRecord::from_completed(&completed);

        assert_eq!(record.reference_note, 60);
        assert_eq!(record.target_note, 67);
        assert_eq!(record.initial_cent_offset, 15.0);
        assert_eq!(record.user_cent_error, 3.2);
        assert_eq!(record.interval, 7); // 7 semitones = perfect fifth
        assert_eq!(record.tuning_system, "equalTemperament");
        assert_eq!(record.timestamp, "2026-03-04T10:00:00Z");
    }

    #[test]
    fn test_pitch_matching_interval_exceeds_octave_defaults_to_zero() {
        let completed = make_pitch_matching(60, 73, 10.0, 5.0, TuningSystem::EqualTemperament);
        let record = PitchMatchingRecord::from_completed(&completed);

        assert_eq!(record.interval, 0);
    }

    #[test]
    fn test_pitch_matching_serde_roundtrip() {
        let completed = make_pitch_matching(69, 76, -18.5, 2.1, TuningSystem::JustIntonation);
        let record = PitchMatchingRecord::from_completed(&completed);

        let json = serde_json::to_string(&record).unwrap();
        let parsed: PitchMatchingRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, parsed);
    }

    #[test]
    fn test_pitch_matching_negative_error() {
        let completed = make_pitch_matching(60, 60, 10.0, -5.0, TuningSystem::EqualTemperament);
        let record = PitchMatchingRecord::from_completed(&completed);

        assert_eq!(record.user_cent_error, -5.0);
        assert_eq!(record.interval, 0); // same note = prime
    }

    // --- TrainingRecord method tests ---

    #[test]
    fn test_training_record_timestamp_discrimination() {
        let record = TrainingRecord::PitchDiscrimination(PitchDiscriminationRecord {
            reference_note: 60,
            target_note: 60,
            cent_offset: 10.0,
            is_correct: true,
            interval: 0,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-03T14:00:00Z".to_string(),
        });
        assert_eq!(record.timestamp(), "2026-03-03T14:00:00Z");
    }

    #[test]
    fn test_training_record_timestamp_matching() {
        let record = TrainingRecord::PitchMatching(PitchMatchingRecord {
            reference_note: 60,
            target_note: 67,
            initial_cent_offset: 15.0,
            user_cent_error: 3.0,
            interval: 7,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-04T10:00:00Z".to_string(),
        });
        assert_eq!(record.timestamp(), "2026-03-04T10:00:00Z");
    }

    #[test]
    fn test_training_record_store_name_discrimination() {
        let record = TrainingRecord::PitchDiscrimination(PitchDiscriminationRecord {
            reference_note: 60,
            target_note: 60,
            cent_offset: 10.0,
            is_correct: true,
            interval: 0,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-03T14:00:00Z".to_string(),
        });
        assert_eq!(record.store_name(), PITCH_DISCRIMINATION_STORE);
    }

    #[test]
    fn test_training_record_store_name_matching() {
        let record = TrainingRecord::PitchMatching(PitchMatchingRecord {
            reference_note: 60,
            target_note: 67,
            initial_cent_offset: 15.0,
            user_cent_error: 3.0,
            interval: 7,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-04T10:00:00Z".to_string(),
        });
        assert_eq!(record.store_name(), PITCH_MATCHING_STORE);
    }
}
