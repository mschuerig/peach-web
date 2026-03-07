use serde::{Deserialize, Serialize};

use crate::types::{Cents, DetunedMIDINote, Frequency, Interval, MIDINote};

/// Reference MIDI note for tuning calculations (A4).
const REFERENCE_MIDI: i16 = 69;

/// Intervals indexed by semitone count (0-11), used by the octave+remainder
/// decomposition in frequency calculation.
const INTERVALS_BY_SEMITONE: [Interval; 12] = [
    Interval::Prime,
    Interval::MinorSecond,
    Interval::MajorSecond,
    Interval::MinorThird,
    Interval::MajorThird,
    Interval::PerfectFourth,
    Interval::Tritone,
    Interval::PerfectFifth,
    Interval::MinorSixth,
    Interval::MajorSixth,
    Interval::MinorSeventh,
    Interval::MajorSeventh,
];

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
    /// Decomposes the MIDI distance from reference note (A4 = 69) into whole octaves
    /// plus a remainder interval (0-11 semitones), looks up the tuning-system-specific
    /// cent offset for that interval, and converts the total cent offset to Hz.
    ///
    /// Formula: `ref_pitch * 2^(total_cents / 1200)`
    pub fn frequency(&self, note: DetunedMIDINote, reference_pitch: Frequency) -> Frequency {
        let total_cents = self.total_cent_offset(note);
        let hz = reference_pitch.raw_value() * 2.0_f64.powf(total_cents / Cents::PER_OCTAVE);
        Frequency::new(hz)
    }

    /// Convenience: frequency for a plain MIDINote (no detuning).
    pub fn frequency_for_note(&self, note: MIDINote, reference_pitch: Frequency) -> Frequency {
        self.frequency(DetunedMIDINote::from(note), reference_pitch)
    }

    /// Cent offset for a given interval in this tuning system.
    pub fn cent_offset(&self, interval: Interval) -> f64 {
        match self {
            TuningSystem::EqualTemperament => interval.semitones() as f64 * Cents::PER_SEMITONE_ET,
            TuningSystem::JustIntonation => just_intonation_cents(interval),
        }
    }

    /// Total cent offset from reference MIDI note (A4 = 69) for the given note,
    /// including any microtonal detuning offset.
    fn total_cent_offset(&self, note: DetunedMIDINote) -> f64 {
        let diff = note.note.raw_value() as i16 - REFERENCE_MIDI;
        let octaves = diff.div_euclid(12) as f64;
        let remainder = diff.rem_euclid(12) as usize;
        let interval = INTERVALS_BY_SEMITONE[remainder];
        octaves * Cents::PER_OCTAVE + self.cent_offset(interval) + note.offset.raw_value
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
        Interval::Octave => Cents::PER_OCTAVE,
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
        assert!((freq.raw_value() - 440.0).abs() < 1e-10);
    }

    #[test]
    fn test_equal_temperament_c4() {
        let note = DetunedMIDINote::from(MIDINote::new(60));
        let freq = TuningSystem::EqualTemperament.frequency(note, Frequency::CONCERT_440);
        // C4 = 440 * 2^(-9/12) ≈ 261.6256
        assert!((freq.raw_value() - 261.6256).abs() < 0.001);
    }

    #[test]
    fn test_equal_temperament_with_detune() {
        let note = DetunedMIDINote {
            note: MIDINote::new(69),
            offset: Cents::new(100.0), // +100 cents = one semitone up = A#4
        };
        let freq = TuningSystem::EqualTemperament.frequency(note, Frequency::CONCERT_440);
        // A#4 = 440 * 2^(1/12) ≈ 466.164
        assert!((freq.raw_value() - 466.164).abs() < 0.001);
    }

    #[test]
    fn test_frequency_for_note_convenience() {
        let freq = TuningSystem::EqualTemperament
            .frequency_for_note(MIDINote::new(69), Frequency::CONCERT_440);
        assert!((freq.raw_value() - 440.0).abs() < 1e-10);
    }

    #[test]
    fn test_equal_temperament_cent_offsets() {
        assert_eq!(
            TuningSystem::EqualTemperament.cent_offset(Interval::Prime),
            0.0
        );
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

    #[test]
    fn test_ji_perfect_fifth_exact_ratio() {
        // JI perfect fifth = 3/2 ratio → A4 (440) + P5 = E5 at exactly 660.0 Hz
        let e5 = DetunedMIDINote::from(MIDINote::new(76));
        let freq = TuningSystem::JustIntonation.frequency(e5, Frequency::CONCERT_440);
        assert!((freq.raw_value() - 660.0).abs() < 0.001);
    }

    #[test]
    fn test_ji_major_third_exact_ratio() {
        // JI major third = 5/4 ratio → A4 (440) + M3 = C#5 at exactly 550.0 Hz
        let cs5 = DetunedMIDINote::from(MIDINote::new(73));
        let freq = TuningSystem::JustIntonation.frequency(cs5, Frequency::CONCERT_440);
        assert!((freq.raw_value() - 550.0).abs() < 0.001);
    }

    #[test]
    fn test_ji_octave_exact() {
        // Octave is identical in both tuning systems (2/1 ratio)
        let a5 = DetunedMIDINote::from(MIDINote::new(81));
        let freq = TuningSystem::JustIntonation.frequency(a5, Frequency::CONCERT_440);
        assert!((freq.raw_value() - 880.0).abs() < 1e-10);
    }

    #[test]
    fn test_ji_unison_exact() {
        // Unison: same note = same frequency
        let a4 = DetunedMIDINote::from(MIDINote::new(69));
        let freq = TuningSystem::JustIntonation.frequency(a4, Frequency::CONCERT_440);
        assert!((freq.raw_value() - 440.0).abs() < 1e-10);
    }

    #[test]
    fn test_et_and_ji_differ_for_non_octave_intervals() {
        // ET and JI should produce different frequencies for the same MIDI note
        // (except unison and octaves)
        let e5 = DetunedMIDINote::from(MIDINote::new(76)); // perfect fifth above A4
        let et = TuningSystem::EqualTemperament.frequency(e5, Frequency::CONCERT_440);
        let ji = TuningSystem::JustIntonation.frequency(e5, Frequency::CONCERT_440);
        // ET: 440 * 2^(7/12) ≈ 659.255 Hz
        // JI: 440 * 3/2 = 660.0 Hz
        assert!((et.raw_value() - 659.255).abs() < 0.01);
        assert!((ji.raw_value() - 660.0).abs() < 0.001);
        assert!((et.raw_value() - ji.raw_value()).abs() > 0.5);
    }

    #[test]
    fn test_ji_below_reference_note() {
        // E4 (MIDI 64) is 5 semitones below A4
        // Decomposition: down one octave + up a perfect fifth (remainder=7)
        // JI: 440 / 2 * (3/2) = 330.0 Hz
        let e4 = DetunedMIDINote::from(MIDINote::new(64));
        let freq = TuningSystem::JustIntonation.frequency(e4, Frequency::CONCERT_440);
        assert!((freq.raw_value() - 330.0).abs() < 0.001);
    }

    #[test]
    fn test_ji_with_detuning() {
        // A4 + 50 cents in JI should equal ET result (both are just cents from ref)
        let detuned = DetunedMIDINote {
            note: MIDINote::new(69),
            offset: Cents::new(50.0),
        };
        let freq = TuningSystem::JustIntonation.frequency(detuned, Frequency::CONCERT_440);
        let expected = 440.0 * 2.0_f64.powf(50.0 / 1200.0);
        assert!((freq.raw_value() - expected).abs() < 0.001);
    }
}
