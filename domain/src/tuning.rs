use serde::{Deserialize, Serialize};

use crate::types::{DetunedMIDINote, Frequency, Interval, MIDINote};

/// Tuning system for frequency calculation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TuningSystem {
    EqualTemperament,
    JustIntonation,
}

impl TuningSystem {
    /// Calculate the frequency of a detuned MIDI note using the given reference pitch.
    ///
    /// Formula: `ref_pitch * 2^((midi - 69 + cents/100) / 12)`
    pub fn frequency(&self, note: DetunedMIDINote, reference_pitch: Frequency) -> Frequency {
        let midi = note.note.raw_value as f64;
        let cents = note.offset.raw_value;
        let hz = reference_pitch.raw_value
            * 2.0_f64.powf((midi - 69.0 + cents / 100.0) / 12.0);
        Frequency::new(hz)
    }

    /// Convenience: frequency for a plain MIDINote (no detuning).
    pub fn frequency_for_note(&self, note: MIDINote, reference_pitch: Frequency) -> Frequency {
        self.frequency(DetunedMIDINote::from(note), reference_pitch)
    }

    /// Cent offset for a given interval in this tuning system.
    pub fn cent_offset(&self, interval: Interval) -> f64 {
        match self {
            TuningSystem::EqualTemperament => interval.semitones() as f64 * 100.0,
            TuningSystem::JustIntonation => just_intonation_cents(interval),
        }
    }
}

/// Just intonation cent offsets derived from pure ratio tuning.
fn just_intonation_cents(interval: Interval) -> f64 {
    match interval {
        Interval::Prime => 0.0,
        Interval::MinorSecond => 111.731,
        Interval::MajorSecond => 203.910,
        Interval::MinorThird => 315.641,
        Interval::MajorThird => 386.314,
        Interval::PerfectFourth => 498.045,
        Interval::Tritone => 590.224,
        Interval::PerfectFifth => 701.955,
        Interval::MinorSixth => 813.686,
        Interval::MajorSixth => 884.359,
        Interval::MinorSeventh => 1017.596,
        Interval::MajorSeventh => 1088.269,
        Interval::Octave => 1200.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Cents;

    #[test]
    fn test_equal_temperament_a4_440() {
        let note = DetunedMIDINote::from(MIDINote::new(69));
        let freq = TuningSystem::EqualTemperament.frequency(note, Frequency::CONCERT_440);
        assert!((freq.raw_value - 440.0).abs() < 1e-10);
    }

    #[test]
    fn test_equal_temperament_c4() {
        let note = DetunedMIDINote::from(MIDINote::new(60));
        let freq = TuningSystem::EqualTemperament.frequency(note, Frequency::CONCERT_440);
        // C4 = 440 * 2^(-9/12) ≈ 261.6256
        assert!((freq.raw_value - 261.6256).abs() < 0.001);
    }

    #[test]
    fn test_equal_temperament_with_detune() {
        let note = DetunedMIDINote {
            note: MIDINote::new(69),
            offset: Cents::new(100.0), // +100 cents = one semitone up = A#4
        };
        let freq = TuningSystem::EqualTemperament.frequency(note, Frequency::CONCERT_440);
        // A#4 = 440 * 2^(1/12) ≈ 466.164
        assert!((freq.raw_value - 466.164).abs() < 0.001);
    }

    #[test]
    fn test_frequency_for_note_convenience() {
        let freq =
            TuningSystem::EqualTemperament.frequency_for_note(MIDINote::new(69), Frequency::CONCERT_440);
        assert!((freq.raw_value - 440.0).abs() < 1e-10);
    }

    #[test]
    fn test_equal_temperament_cent_offsets() {
        assert_eq!(TuningSystem::EqualTemperament.cent_offset(Interval::Prime), 0.0);
        assert_eq!(
            TuningSystem::EqualTemperament.cent_offset(Interval::PerfectFifth),
            700.0
        );
        assert_eq!(
            TuningSystem::EqualTemperament.cent_offset(Interval::Octave),
            1200.0
        );
    }

    #[test]
    fn test_just_intonation_perfect_fifth() {
        let cents = TuningSystem::JustIntonation.cent_offset(Interval::PerfectFifth);
        assert!((cents - 701.955).abs() < 0.001);
    }

    #[test]
    fn test_just_intonation_all_offsets() {
        let ji = TuningSystem::JustIntonation;
        assert_eq!(ji.cent_offset(Interval::Prime), 0.0);
        assert!((ji.cent_offset(Interval::MinorSecond) - 111.731).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MajorSecond) - 203.910).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MinorThird) - 315.641).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MajorThird) - 386.314).abs() < 0.001);
        assert!((ji.cent_offset(Interval::PerfectFourth) - 498.045).abs() < 0.001);
        assert!((ji.cent_offset(Interval::Tritone) - 590.224).abs() < 0.001);
        assert!((ji.cent_offset(Interval::PerfectFifth) - 701.955).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MinorSixth) - 813.686).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MajorSixth) - 884.359).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MinorSeventh) - 1017.596).abs() < 0.001);
        assert!((ji.cent_offset(Interval::MajorSeventh) - 1088.269).abs() < 0.001);
        assert_eq!(ji.cent_offset(Interval::Octave), 1200.0);
    }

    #[test]
    fn test_tuning_system_serde() {
        let json = serde_json::to_string(&TuningSystem::EqualTemperament).unwrap();
        assert_eq!(json, "\"equalTemperament\"");

        let parsed: TuningSystem = serde_json::from_str("\"justIntonation\"").unwrap();
        assert_eq!(parsed, TuningSystem::JustIntonation);
    }
}
