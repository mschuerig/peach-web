use serde::{Deserialize, Serialize};

use crate::tuning::TuningSystem;
use crate::types::MIDINote;

/// A pitch matching challenge: user adjusts pitch to match reference.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PitchMatchingChallenge {
    reference_note: MIDINote,
    target_note: MIDINote,
    initial_cent_offset: f64,
}

impl PitchMatchingChallenge {
    pub fn new(reference_note: MIDINote, target_note: MIDINote, initial_cent_offset: f64) -> Self {
        Self {
            reference_note,
            target_note,
            initial_cent_offset,
        }
    }

    pub fn reference_note(&self) -> MIDINote {
        self.reference_note
    }

    pub fn target_note(&self) -> MIDINote {
        self.target_note
    }

    pub fn initial_cent_offset(&self) -> f64 {
        self.initial_cent_offset
    }
}

/// A completed pitch matching attempt with the user's error and metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompletedPitchMatching {
    reference_note: MIDINote,
    target_note: MIDINote,
    initial_cent_offset: f64,
    user_cent_error: f64,
    tuning_system: TuningSystem,
    timestamp: String,
}

impl CompletedPitchMatching {
    pub fn new(
        reference_note: MIDINote,
        target_note: MIDINote,
        initial_cent_offset: f64,
        user_cent_error: f64,
        tuning_system: TuningSystem,
        timestamp: String,
    ) -> Self {
        Self {
            reference_note,
            target_note,
            initial_cent_offset,
            user_cent_error,
            tuning_system,
            timestamp,
        }
    }

    pub fn reference_note(&self) -> MIDINote {
        self.reference_note
    }

    pub fn target_note(&self) -> MIDINote {
        self.target_note
    }

    pub fn initial_cent_offset(&self) -> f64 {
        self.initial_cent_offset
    }

    pub fn user_cent_error(&self) -> f64 {
        self.user_cent_error
    }

    pub fn tuning_system(&self) -> TuningSystem {
        self.tuning_system
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_matching_challenge_new() {
        let challenge = PitchMatchingChallenge::new(
            MIDINote::new(60),
            MIDINote::new(67),
            15.0,
        );
        assert_eq!(challenge.reference_note().raw_value(), 60);
        assert_eq!(challenge.target_note().raw_value(), 67);
        assert_eq!(challenge.initial_cent_offset(), 15.0);
    }

    #[test]
    fn test_pitch_matching_challenge_negative_offset() {
        let challenge = PitchMatchingChallenge::new(
            MIDINote::new(69),
            MIDINote::new(69),
            -18.5,
        );
        assert_eq!(challenge.initial_cent_offset(), -18.5);
    }

    #[test]
    fn test_completed_pitch_matching_new() {
        let completed = CompletedPitchMatching::new(
            MIDINote::new(60),
            MIDINote::new(67),
            15.0,
            3.2,
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );
        assert_eq!(completed.reference_note().raw_value(), 60);
        assert_eq!(completed.target_note().raw_value(), 67);
        assert_eq!(completed.initial_cent_offset(), 15.0);
        assert_eq!(completed.user_cent_error(), 3.2);
        assert_eq!(completed.tuning_system(), TuningSystem::EqualTemperament);
        assert_eq!(completed.timestamp(), "2026-03-03T14:00:00Z");
    }

    #[test]
    fn test_completed_pitch_matching_negative_error() {
        let completed = CompletedPitchMatching::new(
            MIDINote::new(60),
            MIDINote::new(60),
            10.0,
            -5.0,
            TuningSystem::JustIntonation,
            "2026-03-03T14:00:00Z".to_string(),
        );
        assert_eq!(completed.user_cent_error(), -5.0);
    }

    #[test]
    fn test_pitch_matching_challenge_serde_roundtrip() {
        let challenge = PitchMatchingChallenge::new(
            MIDINote::new(60),
            MIDINote::new(67),
            15.0,
        );
        let json = serde_json::to_string(&challenge).unwrap();
        let parsed: PitchMatchingChallenge = serde_json::from_str(&json).unwrap();
        assert_eq!(challenge, parsed);
    }

    #[test]
    fn test_completed_pitch_matching_serde_roundtrip() {
        let completed = CompletedPitchMatching::new(
            MIDINote::new(60),
            MIDINote::new(67),
            15.0,
            3.2,
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&completed).unwrap();
        let parsed: CompletedPitchMatching = serde_json::from_str(&json).unwrap();
        assert_eq!(completed, parsed);
    }
}
