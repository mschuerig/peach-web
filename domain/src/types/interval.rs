use serde::{Deserialize, Serialize};

use super::midi::MIDINote;
use crate::error::DomainError;

/// Musical interval within one octave (0-12 semitones).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Interval {
    Prime = 0,
    MinorSecond = 1,
    MajorSecond = 2,
    MinorThird = 3,
    MajorThird = 4,
    PerfectFourth = 5,
    Tritone = 6,
    PerfectFifth = 7,
    MinorSixth = 8,
    MajorSixth = 9,
    MinorSeventh = 10,
    MajorSeventh = 11,
    Octave = 12,
}

impl Interval {
    /// All 13 intervals in chromatic order (Prime through Octave).
    pub fn all_chromatic() -> &'static [Interval] {
        &[
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
            Interval::Octave,
        ]
    }

    /// Short display label for this interval (e.g. "P1", "m2", "P5", "P8").
    pub fn short_label(&self) -> &'static str {
        match self {
            Interval::Prime => "P1",
            Interval::MinorSecond => "m2",
            Interval::MajorSecond => "M2",
            Interval::MinorThird => "m3",
            Interval::MajorThird => "M3",
            Interval::PerfectFourth => "P4",
            Interval::Tritone => "d5",
            Interval::PerfectFifth => "P5",
            Interval::MinorSixth => "m6",
            Interval::MajorSixth => "M6",
            Interval::MinorSeventh => "m7",
            Interval::MajorSeventh => "M7",
            Interval::Octave => "P8",
        }
    }

    /// Interval code for CSV export (iOS compatibility).
    ///
    /// Same as `short_label()` except Tritone returns `"A4"` (iOS CSV format).
    pub fn csv_code(&self) -> &'static str {
        match self {
            Interval::Tritone => "A4",
            other => other.short_label(),
        }
    }

    /// Parse an interval code string back to an `Interval`.
    ///
    /// Accepts both `"A4"` and `"d5"` for tritone.
    pub fn from_csv_code(code: &str) -> Option<Interval> {
        match code {
            "P1" => Some(Interval::Prime),
            "m2" => Some(Interval::MinorSecond),
            "M2" => Some(Interval::MajorSecond),
            "m3" => Some(Interval::MinorThird),
            "M3" => Some(Interval::MajorThird),
            "P4" => Some(Interval::PerfectFourth),
            "A4" | "d5" => Some(Interval::Tritone),
            "P5" => Some(Interval::PerfectFifth),
            "m6" => Some(Interval::MinorSixth),
            "M6" => Some(Interval::MajorSixth),
            "m7" => Some(Interval::MinorSeventh),
            "M7" => Some(Interval::MajorSeventh),
            "P8" => Some(Interval::Octave),
            _ => None,
        }
    }

    /// Human-readable display name (e.g. "Minor Second", "Perfect Fifth").
    ///
    /// Returns just the interval name without direction suffix.
    pub fn display_name(&self) -> &'static str {
        match self {
            Interval::Prime => "Prime",
            Interval::MinorSecond => "Minor Second",
            Interval::MajorSecond => "Major Second",
            Interval::MinorThird => "Minor Third",
            Interval::MajorThird => "Major Third",
            Interval::PerfectFourth => "Perfect Fourth",
            Interval::Tritone => "Tritone",
            Interval::PerfectFifth => "Perfect Fifth",
            Interval::MinorSixth => "Minor Sixth",
            Interval::MajorSixth => "Major Sixth",
            Interval::MinorSeventh => "Minor Seventh",
            Interval::MajorSeventh => "Major Seventh",
            Interval::Octave => "Octave",
        }
    }

    /// Number of semitones in this interval.
    pub fn semitones(&self) -> u8 {
        *self as u8
    }

    /// Determine the interval between two notes. Errors if distance > 12 semitones.
    pub fn between(reference: MIDINote, target: MIDINote) -> Result<Interval, DomainError> {
        let diff = (target.raw_value() as i16 - reference.raw_value() as i16).unsigned_abs() as u8;
        Self::from_semitones(diff)
    }

    pub fn from_semitones(semitones: u8) -> Result<Interval, DomainError> {
        match semitones {
            0 => Ok(Interval::Prime),
            1 => Ok(Interval::MinorSecond),
            2 => Ok(Interval::MajorSecond),
            3 => Ok(Interval::MinorThird),
            4 => Ok(Interval::MajorThird),
            5 => Ok(Interval::PerfectFourth),
            6 => Ok(Interval::Tritone),
            7 => Ok(Interval::PerfectFifth),
            8 => Ok(Interval::MinorSixth),
            9 => Ok(Interval::MajorSixth),
            10 => Ok(Interval::MinorSeventh),
            11 => Ok(Interval::MajorSeventh),
            12 => Ok(Interval::Octave),
            n => Err(DomainError::IntervalOutOfRange(n)),
        }
    }
}

/// Direction of a musical interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Up = 0,
    Down = 1,
}

/// An interval with direction (up or down).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DirectedInterval {
    pub interval: Interval,
    pub direction: Direction,
}

impl DirectedInterval {
    pub fn new(interval: Interval, direction: Direction) -> Self {
        Self {
            interval,
            direction,
        }
    }

    /// Determine the directed interval between two notes.
    /// Prime is always Up. Target >= reference → Up, otherwise Down.
    pub fn between(reference: MIDINote, target: MIDINote) -> Result<Self, DomainError> {
        let interval = Interval::between(reference, target)?;
        // Prime is always Up (same note). Otherwise Up if target >= reference.
        let direction = if target.raw_value() >= reference.raw_value() {
            Direction::Up
        } else {
            Direction::Down
        };
        Ok(Self {
            interval,
            direction,
        })
    }

    /// Signed semitone offset: positive for Up, negative for Down.
    pub fn signed_semitones(&self) -> i16 {
        let s = self.interval.semitones() as i16;
        match self.direction {
            Direction::Up => s,
            Direction::Down => -s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_semitones() {
        assert_eq!(Interval::Prime.semitones(), 0);
        assert_eq!(Interval::PerfectFifth.semitones(), 7);
        assert_eq!(Interval::Octave.semitones(), 12);
    }

    #[test]
    fn test_interval_between_perfect_fifth() {
        let c4 = MIDINote::new(60);
        let g4 = MIDINote::new(67);
        assert_eq!(Interval::between(c4, g4).unwrap(), Interval::PerfectFifth);
    }

    #[test]
    fn test_interval_between_reversed() {
        let c4 = MIDINote::new(60);
        let g4 = MIDINote::new(67);
        assert_eq!(Interval::between(g4, c4).unwrap(), Interval::PerfectFifth);
    }

    #[test]
    fn test_interval_between_unison() {
        let c4 = MIDINote::new(60);
        assert_eq!(Interval::between(c4, c4).unwrap(), Interval::Prime);
    }

    #[test]
    fn test_interval_between_octave() {
        let c4 = MIDINote::new(60);
        let c5 = MIDINote::new(72);
        assert_eq!(Interval::between(c4, c5).unwrap(), Interval::Octave);
    }

    #[test]
    fn test_interval_between_exceeds_octave() {
        let c4 = MIDINote::new(60);
        let d5 = MIDINote::new(74);
        assert!(Interval::between(c4, d5).is_err());
    }

    #[test]
    fn test_directed_interval_between_up() {
        let c4 = MIDINote::new(60);
        let g4 = MIDINote::new(67);
        let di = DirectedInterval::between(c4, g4).unwrap();
        assert_eq!(di.interval, Interval::PerfectFifth);
        assert_eq!(di.direction, Direction::Up);
    }

    #[test]
    fn test_directed_interval_between_down() {
        let c4 = MIDINote::new(60);
        let g4 = MIDINote::new(67);
        let di = DirectedInterval::between(g4, c4).unwrap();
        assert_eq!(di.interval, Interval::PerfectFifth);
        assert_eq!(di.direction, Direction::Down);
    }

    #[test]
    fn test_directed_interval_prime_always_up() {
        let c4 = MIDINote::new(60);
        let di = DirectedInterval::between(c4, c4).unwrap();
        assert_eq!(di.interval, Interval::Prime);
        assert_eq!(di.direction, Direction::Up);
    }

    #[test]
    fn test_directed_interval_signed_semitones() {
        let up = DirectedInterval::new(Interval::PerfectFifth, Direction::Up);
        assert_eq!(up.signed_semitones(), 7);

        let down = DirectedInterval::new(Interval::PerfectFifth, Direction::Down);
        assert_eq!(down.signed_semitones(), -7);
    }

    #[test]
    fn test_short_label_all_intervals() {
        let expected = [
            (Interval::Prime, "P1"),
            (Interval::MinorSecond, "m2"),
            (Interval::MajorSecond, "M2"),
            (Interval::MinorThird, "m3"),
            (Interval::MajorThird, "M3"),
            (Interval::PerfectFourth, "P4"),
            (Interval::Tritone, "d5"),
            (Interval::PerfectFifth, "P5"),
            (Interval::MinorSixth, "m6"),
            (Interval::MajorSixth, "M6"),
            (Interval::MinorSeventh, "m7"),
            (Interval::MajorSeventh, "M7"),
            (Interval::Octave, "P8"),
        ];
        for (interval, label) in expected {
            assert_eq!(interval.short_label(), label, "wrong label for {interval:?}");
        }
    }

    #[test]
    fn test_all_chromatic_order_and_count() {
        let all = Interval::all_chromatic();
        assert_eq!(all.len(), 13);
        assert_eq!(all[0], Interval::Prime);
        assert_eq!(all[12], Interval::Octave);
        // Verify chromatic order matches discriminant values
        for (i, interval) in all.iter().enumerate() {
            assert_eq!(interval.semitones() as usize, i);
        }
    }

    #[test]
    fn test_csv_code_all_intervals() {
        let expected = [
            (Interval::Prime, "P1"),
            (Interval::MinorSecond, "m2"),
            (Interval::MajorSecond, "M2"),
            (Interval::MinorThird, "m3"),
            (Interval::MajorThird, "M3"),
            (Interval::PerfectFourth, "P4"),
            (Interval::Tritone, "A4"),
            (Interval::PerfectFifth, "P5"),
            (Interval::MinorSixth, "m6"),
            (Interval::MajorSixth, "M6"),
            (Interval::MinorSeventh, "m7"),
            (Interval::MajorSeventh, "M7"),
            (Interval::Octave, "P8"),
        ];
        for (interval, code) in expected {
            assert_eq!(interval.csv_code(), code, "wrong csv_code for {interval:?}");
        }
    }

    #[test]
    fn test_csv_code_tritone_differs_from_short_label() {
        assert_eq!(Interval::Tritone.csv_code(), "A4");
        assert_eq!(Interval::Tritone.short_label(), "d5");
    }

    #[test]
    fn test_from_csv_code_all_values() {
        let expected = [
            ("P1", Interval::Prime),
            ("m2", Interval::MinorSecond),
            ("M2", Interval::MajorSecond),
            ("m3", Interval::MinorThird),
            ("M3", Interval::MajorThird),
            ("P4", Interval::PerfectFourth),
            ("A4", Interval::Tritone),
            ("d5", Interval::Tritone),
            ("P5", Interval::PerfectFifth),
            ("m6", Interval::MinorSixth),
            ("M6", Interval::MajorSixth),
            ("m7", Interval::MinorSeventh),
            ("M7", Interval::MajorSeventh),
            ("P8", Interval::Octave),
        ];
        for (code, interval) in expected {
            assert_eq!(Interval::from_csv_code(code), Some(interval), "code={code}");
        }
    }

    #[test]
    fn test_from_csv_code_invalid() {
        assert_eq!(Interval::from_csv_code("X1"), None);
        assert_eq!(Interval::from_csv_code(""), None);
        assert_eq!(Interval::from_csv_code("p1"), None);
    }

    #[test]
    fn test_csv_code_roundtrip_all_intervals() {
        for interval in Interval::all_chromatic() {
            let code = interval.csv_code();
            let back = Interval::from_csv_code(code).unwrap();
            assert_eq!(back, *interval, "roundtrip failed for {interval:?} -> {code}");
        }
    }

    #[test]
    fn test_display_name_all_intervals() {
        let expected = [
            (Interval::Prime, "Prime"),
            (Interval::MinorSecond, "Minor Second"),
            (Interval::MajorSecond, "Major Second"),
            (Interval::MinorThird, "Minor Third"),
            (Interval::MajorThird, "Major Third"),
            (Interval::PerfectFourth, "Perfect Fourth"),
            (Interval::Tritone, "Tritone"),
            (Interval::PerfectFifth, "Perfect Fifth"),
            (Interval::MinorSixth, "Minor Sixth"),
            (Interval::MajorSixth, "Major Sixth"),
            (Interval::MinorSeventh, "Minor Seventh"),
            (Interval::MajorSeventh, "Major Seventh"),
            (Interval::Octave, "Octave"),
        ];
        for (interval, name) in expected {
            assert_eq!(interval.display_name(), name, "wrong display_name for {interval:?}");
        }
    }

    #[test]
    fn test_from_semitones_public() {
        assert_eq!(Interval::from_semitones(0).unwrap(), Interval::Prime);
        assert_eq!(Interval::from_semitones(7).unwrap(), Interval::PerfectFifth);
        assert_eq!(Interval::from_semitones(12).unwrap(), Interval::Octave);
        assert!(Interval::from_semitones(13).is_err());
    }

    #[test]
    fn test_interval_serde_camel_case() {
        let json = serde_json::to_string(&Interval::PerfectFifth).unwrap();
        assert_eq!(json, "\"perfectFifth\"");

        let parsed: Interval = serde_json::from_str("\"minorThird\"").unwrap();
        assert_eq!(parsed, Interval::MinorThird);
    }

    #[test]
    fn test_direction_serde_camel_case() {
        let json = serde_json::to_string(&Direction::Up).unwrap();
        assert_eq!(json, "\"up\"");
    }

    #[test]
    fn test_directed_interval_serde_roundtrip() {
        let di = DirectedInterval::new(Interval::MajorThird, Direction::Down);
        let json = serde_json::to_string(&di).unwrap();
        let parsed: DirectedInterval = serde_json::from_str(&json).unwrap();
        assert_eq!(di, parsed);
    }
}
