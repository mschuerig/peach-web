use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::profile::PerceptualProfile;
use crate::training::comparison::Comparison;
use crate::training::CompletedComparison;
use crate::types::{Cents, DetunedMIDINote, DirectedInterval, Direction, Frequency, MIDINote};

/// Kazez narrow: reduce difficulty after correct answer.
/// Formula: p * (1.0 - 0.05 * sqrt(p))
pub fn kazez_narrow(p: f64) -> f64 {
    assert!(p >= 0.0, "kazez_narrow: p must be non-negative, got {p}");
    p * (1.0 - 0.05 * p.sqrt())
}

/// Kazez widen: increase difficulty after incorrect answer.
/// Formula: p * (1.0 + 0.09 * sqrt(p))
pub fn kazez_widen(p: f64) -> f64 {
    assert!(p >= 0.0, "kazez_widen: p must be non-negative, got {p}");
    p * (1.0 + 0.09 * p.sqrt())
}

/// Training configuration parameters.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrainingSettings {
    note_range_min: MIDINote,
    note_range_max: MIDINote,
    reference_pitch: Frequency,
    min_cent_difference: Cents,
    max_cent_difference: Cents,
}

impl TrainingSettings {
    pub fn new(
        note_range_min: MIDINote,
        note_range_max: MIDINote,
        reference_pitch: Frequency,
        min_cent_difference: Cents,
        max_cent_difference: Cents,
    ) -> Self {
        assert!(
            note_range_min.raw_value() <= note_range_max.raw_value(),
            "note_range_min ({}) must be <= note_range_max ({})",
            note_range_min.raw_value(),
            note_range_max.raw_value()
        );
        Self {
            note_range_min,
            note_range_max,
            reference_pitch,
            min_cent_difference,
            max_cent_difference,
        }
    }

    pub fn note_range_min(&self) -> MIDINote {
        self.note_range_min
    }

    pub fn note_range_max(&self) -> MIDINote {
        self.note_range_max
    }

    pub fn reference_pitch(&self) -> Frequency {
        self.reference_pitch
    }

    pub fn min_cent_difference(&self) -> Cents {
        self.min_cent_difference
    }

    pub fn max_cent_difference(&self) -> Cents {
        self.max_cent_difference
    }
}

impl Default for TrainingSettings {
    fn default() -> Self {
        Self {
            note_range_min: MIDINote::new(36),
            note_range_max: MIDINote::new(84),
            reference_pitch: Frequency::CONCERT_440,
            min_cent_difference: Cents::new(0.1),
            max_cent_difference: Cents::new(100.0),
        }
    }
}

/// Generate the next comparison challenge using the Kazez adaptive algorithm.
///
/// Algorithm (Blueprint §6.3):
/// 1. Determine magnitude from last_comparison (kazez_narrow/widen), or profile.overall_mean(), or cold-start (100 cents)
/// 2. Clamp to [min_cent_difference, max_cent_difference]
/// 3. Random sign (equally likely positive/negative)
/// 4. Select reference note within range, accounting for interval transposition
/// 5. Return Comparison
pub fn next_comparison(
    profile: &PerceptualProfile,
    settings: &TrainingSettings,
    last_comparison: Option<&CompletedComparison>,
    interval: DirectedInterval,
) -> Comparison {
    let mut rng = rand::rng();

    // Step 1: Determine magnitude
    let raw_magnitude = match last_comparison {
        Some(completed) => {
            let prev_magnitude = completed.comparison().target_note().offset.magnitude();
            if completed.is_correct() {
                kazez_narrow(prev_magnitude)
            } else {
                kazez_widen(prev_magnitude)
            }
        }
        None => {
            // Warm start: use profile overall_mean if available
            // Cold start: use max_cent_difference (100 cents)
            profile
                .overall_mean()
                .unwrap_or(settings.max_cent_difference.raw_value)
        }
    };

    // Step 2: Clamp to difficulty range
    let magnitude = raw_magnitude.clamp(
        settings.min_cent_difference.raw_value,
        settings.max_cent_difference.raw_value,
    );

    // Step 3: Random sign
    let sign: f64 = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
    let cent_offset = magnitude * sign;

    // Step 4: Select reference note within range, accounting for interval transposition
    let semitones = interval.signed_semitones().unsigned_abs() as u8;
    let min = settings.note_range_min.raw_value();
    let max = settings.note_range_max.raw_value();

    // Ensure reference note allows transposition to stay within MIDI 0-127
    let (ref_min, ref_max) = match interval.direction {
        Direction::Up => (min, max.saturating_sub(semitones).max(min)),
        Direction::Down => (min.saturating_add(semitones).min(max), max),
    };

    let reference_note = MIDINote::random(ref_min..=ref_max);

    // Step 5: Transpose and create target
    let transposed = reference_note.transposed(interval);
    let target_note = DetunedMIDINote {
        note: transposed,
        offset: Cents::new(cent_offset),
    };

    Comparison::new(reference_note, target_note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Interval;

    // --- AC7: Kazez narrow ---

    #[test]
    fn test_kazez_narrow_at_50() {
        let result = kazez_narrow(50.0);
        // 50 * (1.0 - 0.05 * sqrt(50)) ≈ 50 * (1.0 - 0.3536) ≈ 50 * 0.6464 ≈ 32.32
        let expected = 50.0 * (1.0 - 0.05 * 50.0_f64.sqrt());
        assert!((result - expected).abs() < 1e-10);
        assert!((result - 32.322).abs() < 0.01);
    }

    #[test]
    fn test_kazez_narrow_at_zero() {
        assert_eq!(kazez_narrow(0.0), 0.0);
    }

    #[test]
    fn test_kazez_narrow_at_one() {
        let result = kazez_narrow(1.0);
        // 1 * (1 - 0.05 * 1) = 0.95
        assert!((result - 0.95).abs() < 1e-10);
    }

    // --- AC8: Kazez widen ---

    #[test]
    fn test_kazez_widen_at_50() {
        let result = kazez_widen(50.0);
        // 50 * (1.0 + 0.09 * sqrt(50)) ≈ 50 * (1.0 + 0.6364) ≈ 50 * 1.6364 ≈ 81.82
        let expected = 50.0 * (1.0 + 0.09 * 50.0_f64.sqrt());
        assert!((result - expected).abs() < 1e-10);
        assert!((result - 81.82).abs() < 0.01);
    }

    #[test]
    fn test_kazez_widen_at_zero() {
        assert_eq!(kazez_widen(0.0), 0.0);
    }

    #[test]
    fn test_kazez_widen_at_one() {
        let result = kazez_widen(1.0);
        // 1 * (1 + 0.09 * 1) = 1.09
        assert!((result - 1.09).abs() < 1e-10);
    }

    // --- AC9: Cold start ---

    #[test]
    fn test_cold_start_uses_max_cent_difference() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        let comparison = next_comparison(&profile, &settings, None, interval);
        // Cold start: magnitude defaults to max_cent_difference (100)
        let magnitude = comparison.target_note().offset.magnitude();
        assert!(
            (magnitude - 100.0).abs() < 1e-10,
            "Cold start magnitude should be 100.0, got {magnitude}"
        );
    }

    // --- AC10: Warm start ---

    #[test]
    fn test_warm_start_uses_overall_mean() {
        let mut profile = PerceptualProfile::new();
        // Train two notes to establish an overall_mean
        profile.update(MIDINote::new(60), 40.0, true);
        profile.update(MIDINote::new(72), 60.0, true);
        // overall_mean = 50.0

        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        let comparison = next_comparison(&profile, &settings, None, interval);
        let magnitude = comparison.target_note().offset.magnitude();
        assert!(
            (magnitude - 50.0).abs() < 1e-10,
            "Warm start magnitude should be 50.0 (overall_mean), got {magnitude}"
        );
    }

    #[test]
    fn test_warm_start_clamped_to_range() {
        let mut profile = PerceptualProfile::new();
        // Train with very low mean that would be below min_cent_difference
        profile.update(MIDINote::new(60), 0.01, true);
        // overall_mean = 0.01

        let settings = TrainingSettings::default(); // min = 0.1
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        let comparison = next_comparison(&profile, &settings, None, interval);
        let magnitude = comparison.target_note().offset.magnitude();
        assert!(
            magnitude >= settings.min_cent_difference.raw_value,
            "Magnitude {magnitude} should be >= min {}", settings.min_cent_difference.raw_value
        );
    }

    // --- AC11: Reference note in range ---

    #[test]
    fn test_reference_note_within_range() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default(); // 36..=84
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        for _ in 0..100 {
            let comparison = next_comparison(&profile, &settings, None, interval);
            let ref_val = comparison.reference_note().raw_value();
            assert!(
                (36..=84).contains(&ref_val),
                "Reference note {ref_val} out of range 36..=84"
            );
        }
    }

    #[test]
    fn test_target_note_within_midi_range_with_interval() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Octave, Direction::Up);

        for _ in 0..100 {
            let comparison = next_comparison(&profile, &settings, None, interval);
            let target_val = comparison.target_note().note.raw_value();
            assert!(
                target_val <= 127,
                "Target note {target_val} out of MIDI range"
            );
        }
    }

    #[test]
    fn test_interval_down_keeps_reference_in_range() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Octave, Direction::Down);

        for _ in 0..100 {
            let comparison = next_comparison(&profile, &settings, None, interval);
            let ref_val = comparison.reference_note().raw_value();
            let target_val = comparison.target_note().note.raw_value();
            assert!(
                (36..=84).contains(&ref_val),
                "Reference note {ref_val} out of range"
            );
            assert!(
                target_val <= 127,
                "Target note {target_val} out of MIDI range"
            );
        }
    }

    // --- Adaptive difficulty (kazez applied) ---

    #[test]
    fn test_correct_answer_narrows_difficulty() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        // First comparison: cold start at 100 cents
        let comp1 = next_comparison(&profile, &settings, None, interval);

        // Simulate correct answer
        let completed = CompletedComparison::new(
            comp1,
            comp1.is_target_higher(),
            crate::TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );

        let comp2 = next_comparison(&profile, &settings, Some(&completed), interval);
        let mag2 = comp2.target_note().offset.magnitude();
        // After correct: kazez_narrow(100) = 100*(1 - 0.05*10) ≈ 50
        let expected = kazez_narrow(100.0);
        assert!(
            (mag2 - expected).abs() < 1e-10,
            "After correct answer, magnitude should be {expected}, got {mag2}"
        );
    }

    #[test]
    fn test_incorrect_answer_widens_difficulty() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        let comp1 = next_comparison(&profile, &settings, None, interval);

        // Simulate incorrect answer
        let completed = CompletedComparison::new(
            comp1,
            !comp1.is_target_higher(),
            crate::TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );

        let comp2 = next_comparison(&profile, &settings, Some(&completed), interval);
        let mag2 = comp2.target_note().offset.magnitude();
        // After incorrect at 100: kazez_widen(100) = 100*(1 + 0.09*10) = 190, clamped to 100
        assert!(
            (mag2 - 100.0).abs() < 1e-10,
            "After incorrect answer at max, magnitude should be clamped to 100, got {mag2}"
        );
    }

    // --- TrainingSettings ---

    #[test]
    fn test_training_settings_defaults() {
        let s = TrainingSettings::default();
        assert_eq!(s.note_range_min().raw_value(), 36);
        assert_eq!(s.note_range_max().raw_value(), 84);
        assert_eq!(s.reference_pitch().raw_value(), 440.0);
        assert!((s.min_cent_difference().raw_value - 0.1).abs() < 1e-10);
        assert!((s.max_cent_difference().raw_value - 100.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "note_range_min")]
    fn test_training_settings_panics_on_invalid_range() {
        TrainingSettings::new(
            MIDINote::new(84),
            MIDINote::new(36),
            Frequency::CONCERT_440,
            Cents::new(0.1),
            Cents::new(100.0),
        );
    }

    #[test]
    fn test_training_settings_serde_roundtrip() {
        let s = TrainingSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let parsed: TrainingSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }

    // --- Sign randomness ---

    #[test]
    fn test_next_comparison_produces_both_signs() {
        let profile = PerceptualProfile::new();
        let settings = TrainingSettings::default();
        let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

        let mut pos_count = 0;
        let mut neg_count = 0;
        for _ in 0..200 {
            let comp = next_comparison(&profile, &settings, None, interval);
            if comp.target_note().offset.raw_value > 0.0 {
                pos_count += 1;
            } else {
                neg_count += 1;
            }
        }
        assert!(pos_count > 0, "Should produce positive offsets");
        assert!(neg_count > 0, "Should produce negative offsets");
    }

    #[test]
    #[should_panic(expected = "must be non-negative")]
    fn test_kazez_narrow_panics_on_negative() {
        kazez_narrow(-1.0);
    }

    #[test]
    #[should_panic(expected = "must be non-negative")]
    fn test_kazez_widen_panics_on_negative() {
        kazez_widen(-1.0);
    }

    #[test]
    #[should_panic(expected = "must be non-negative")]
    fn test_kazez_narrow_panics_on_nan() {
        kazez_narrow(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "must be non-negative")]
    fn test_kazez_widen_panics_on_nan() {
        kazez_widen(f64::NAN);
    }
}
