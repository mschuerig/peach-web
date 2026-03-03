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
    /// Number of semitones in this interval.
    pub fn semitones(&self) -> u8 {
        *self as u8
    }

    /// Determine the interval between two notes. Errors if distance > 12 semitones.
    pub fn between(reference: MIDINote, target: MIDINote) -> Result<Interval, DomainError> {
        let diff = (target.raw_value() as i16 - reference.raw_value() as i16).unsigned_abs() as u8;
        Self::from_semitones(diff)
    }

    fn from_semitones(semitones: u8) -> Result<Interval, DomainError> {
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
