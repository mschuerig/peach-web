use serde::{Deserialize, Serialize};

use crate::tuning::TuningSystem;
use crate::types::{DetunedMIDINote, MIDINote};

/// A comparison challenge: a reference note and a detuned target note.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    reference_note: MIDINote,
    target_note: DetunedMIDINote,
}

impl Comparison {
    pub fn new(reference_note: MIDINote, target_note: DetunedMIDINote) -> Self {
        Self {
            reference_note,
            target_note,
        }
    }

    pub fn reference_note(&self) -> MIDINote {
        self.reference_note
    }

    pub fn target_note(&self) -> DetunedMIDINote {
        self.target_note
    }

    /// Whether the target is tuned higher than the reference.
    pub fn is_target_higher(&self) -> bool {
        self.target_note.offset.raw_value > 0.0
    }

    /// Whether the user's answer (higher/lower) is correct.
    pub fn is_correct(&self, user_answered_higher: bool) -> bool {
        user_answered_higher == self.is_target_higher()
    }
}

/// A completed comparison with the user's answer and metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompletedComparison {
    comparison: Comparison,
    user_answered_higher: bool,
    tuning_system: TuningSystem,
    timestamp: String,
}

impl CompletedComparison {
    pub fn new(
        comparison: Comparison,
        user_answered_higher: bool,
        tuning_system: TuningSystem,
        timestamp: String,
    ) -> Self {
        Self {
            comparison,
            user_answered_higher,
            tuning_system,
            timestamp,
        }
    }

    pub fn comparison(&self) -> &Comparison {
        &self.comparison
    }

    pub fn user_answered_higher(&self) -> bool {
        self.user_answered_higher
    }

    pub fn tuning_system(&self) -> TuningSystem {
        self.tuning_system
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// Whether the user's answer was correct.
    pub fn is_correct(&self) -> bool {
        self.comparison.is_correct(self.user_answered_higher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Cents;

    #[test]
    fn test_comparison_is_target_higher_positive_offset() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(25.0),
            },
        );
        assert!(comp.is_target_higher());
    }

    #[test]
    fn test_comparison_is_target_higher_negative_offset() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(-25.0),
            },
        );
        assert!(!comp.is_target_higher());
    }

    #[test]
    fn test_comparison_is_target_higher_zero_offset() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(0.0),
            },
        );
        assert!(!comp.is_target_higher());
    }

    #[test]
    fn test_comparison_is_correct_user_right() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(25.0),
            },
        );
        assert!(comp.is_correct(true));
        assert!(!comp.is_correct(false));
    }

    #[test]
    fn test_comparison_is_correct_user_wrong() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(-25.0),
            },
        );
        assert!(comp.is_correct(false));
        assert!(!comp.is_correct(true));
    }

    #[test]
    fn test_comparison_getters() {
        let ref_note = MIDINote::new(60);
        let target = DetunedMIDINote {
            note: MIDINote::new(67),
            offset: Cents::new(10.0),
        };
        let comp = Comparison::new(ref_note, target);
        assert_eq!(comp.reference_note(), ref_note);
        assert_eq!(comp.target_note(), target);
    }

    #[test]
    fn test_completed_comparison_correct() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(30.0),
            },
        );
        let completed = CompletedComparison::new(
            comp,
            true,
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );
        assert!(completed.is_correct());
    }

    #[test]
    fn test_completed_comparison_incorrect() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(30.0),
            },
        );
        let completed = CompletedComparison::new(
            comp,
            false,
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );
        assert!(!completed.is_correct());
    }

    #[test]
    fn test_completed_comparison_getters() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(30.0),
            },
        );
        let completed = CompletedComparison::new(
            comp,
            true,
            TuningSystem::JustIntonation,
            "2026-03-03T14:00:00Z".to_string(),
        );
        assert_eq!(*completed.comparison(), comp);
        assert!(completed.user_answered_higher());
        assert_eq!(completed.tuning_system(), TuningSystem::JustIntonation);
        assert_eq!(completed.timestamp(), "2026-03-03T14:00:00Z");
    }

    #[test]
    fn test_comparison_serde_roundtrip() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(67),
                offset: Cents::new(25.5),
            },
        );
        let json = serde_json::to_string(&comp).unwrap();
        let parsed: Comparison = serde_json::from_str(&json).unwrap();
        assert_eq!(comp, parsed);
    }

    #[test]
    fn test_completed_comparison_serde_roundtrip() {
        let comp = Comparison::new(
            MIDINote::new(60),
            DetunedMIDINote {
                note: MIDINote::new(60),
                offset: Cents::new(30.0),
            },
        );
        let completed = CompletedComparison::new(
            comp,
            true,
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&completed).unwrap();
        let parsed: CompletedComparison = serde_json::from_str(&json).unwrap();
        assert_eq!(completed, parsed);
    }
}
