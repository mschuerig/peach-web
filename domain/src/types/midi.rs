use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use super::interval::DirectedInterval;

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// A MIDI note number (0-127). Panics on out-of-range (programming error invariant).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct MIDINote {
    raw_value: u8,
}

impl MIDINote {
    /// Access the raw MIDI note value (0-127).
    pub fn raw_value(&self) -> u8 {
        self.raw_value
    }

    /// Create a new MIDINote. Panics if value > 127.
    pub fn new(raw_value: u8) -> Self {
        assert!(raw_value <= 127, "MIDI note must be 0-127, got {raw_value}");
        Self { raw_value }
    }

    /// Standard note name (e.g. "C4", "A4").
    /// Uses octave = (raw_value / 12) - 1, which gives C4 for MIDI 60.
    pub fn name(&self) -> String {
        let note = NOTE_NAMES[(self.raw_value % 12) as usize];
        let octave = (self.raw_value as i8 / 12) - 1;
        format!("{note}{octave}")
    }

    /// Generate a random MIDINote within the given range (inclusive).
    pub fn random(range: RangeInclusive<u8>) -> Self {
        let mut rng = rand::rng();
        let raw_value = rng.random_range(range);
        Self::new(raw_value)
    }

    /// Transpose by a directed interval. Panics if result outside 0-127.
    pub fn transposed(&self, by: DirectedInterval) -> Self {
        let new_value = self.raw_value as i16 + by.signed_semitones();
        assert!(
            (0..=127).contains(&new_value),
            "Transposed MIDI note out of range: {new_value}"
        );
        Self::new(new_value as u8)
    }
}

/// MIDI velocity (1-127). Panics on out-of-range (programming error invariant).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct MIDIVelocity {
    raw_value: u8,
}

impl MIDIVelocity {
    /// Access the raw MIDI velocity value (1-127).
    pub fn raw_value(&self) -> u8 {
        self.raw_value
    }

    /// Create a new MIDIVelocity. Panics if value is 0 or > 127.
    pub fn new(raw_value: u8) -> Self {
        assert!(
            (1..=127).contains(&raw_value),
            "MIDI velocity must be 1-127, got {raw_value}"
        );
        Self { raw_value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::interval::{Direction, Interval};

    #[test]
    fn test_midi_note_c4_name() {
        assert_eq!(MIDINote::new(60).name(), "C4");
    }

    #[test]
    fn test_midi_note_a4_name() {
        assert_eq!(MIDINote::new(69).name(), "A4");
    }

    #[test]
    fn test_midi_note_boundary_names() {
        assert_eq!(MIDINote::new(0).name(), "C-1");
        assert_eq!(MIDINote::new(127).name(), "G9");
        assert_eq!(MIDINote::new(21).name(), "A0");
    }

    #[test]
    #[should_panic(expected = "MIDI note must be 0-127")]
    fn test_midi_note_panics_on_128() {
        MIDINote::new(128);
    }

    #[test]
    fn test_midi_note_random_in_range() {
        for _ in 0..100 {
            let note = MIDINote::random(36..=84);
            assert!((36..=84).contains(&note.raw_value));
        }
    }

    #[test]
    fn test_midi_note_transposed_up_perfect_fifth() {
        let c4 = MIDINote::new(60);
        let result = c4.transposed(DirectedInterval::new(
            Interval::PerfectFifth,
            Direction::Up,
        ));
        assert_eq!(result.raw_value, 67);
    }

    #[test]
    fn test_midi_note_transposed_down() {
        let g4 = MIDINote::new(67);
        let result = g4.transposed(DirectedInterval::new(
            Interval::PerfectFifth,
            Direction::Down,
        ));
        assert_eq!(result.raw_value, 60);
    }

    #[test]
    #[should_panic(expected = "Transposed MIDI note out of range")]
    fn test_midi_note_transposed_panics_out_of_range() {
        let high = MIDINote::new(125);
        high.transposed(DirectedInterval::new(Interval::Octave, Direction::Up));
    }

    #[test]
    fn test_midi_velocity_valid() {
        let v = MIDIVelocity::new(64);
        assert_eq!(v.raw_value, 64);
    }

    #[test]
    fn test_midi_velocity_boundaries() {
        assert_eq!(MIDIVelocity::new(1).raw_value, 1);
        assert_eq!(MIDIVelocity::new(127).raw_value, 127);
    }

    #[test]
    #[should_panic(expected = "MIDI velocity must be 1-127")]
    fn test_midi_velocity_panics_on_zero() {
        MIDIVelocity::new(0);
    }

    #[test]
    #[should_panic(expected = "MIDI velocity must be 1-127")]
    fn test_midi_velocity_panics_on_128() {
        MIDIVelocity::new(128);
    }

    #[test]
    fn test_midi_note_transposed_by_prime() {
        let c4 = MIDINote::new(60);
        let result = c4.transposed(DirectedInterval::new(Interval::Prime, Direction::Up));
        assert_eq!(result.raw_value, 60);
    }

    #[test]
    fn test_midi_note_random_single_element_range() {
        let note = MIDINote::random(42..=42);
        assert_eq!(note.raw_value, 42);
    }

    #[test]
    fn test_midi_note_random_full_range() {
        for _ in 0..100 {
            let note = MIDINote::random(0..=127);
            assert!(note.raw_value <= 127);
        }
    }

    #[test]
    fn test_midi_note_serde_roundtrip() {
        let note = MIDINote::new(60);
        let json = serde_json::to_string(&note).unwrap();
        let parsed: MIDINote = serde_json::from_str(&json).unwrap();
        assert_eq!(note, parsed);
    }

    #[test]
    fn test_midi_velocity_serde_roundtrip() {
        let vel = MIDIVelocity::new(100);
        let json = serde_json::to_string(&vel).unwrap();
        let parsed: MIDIVelocity = serde_json::from_str(&json).unwrap();
        assert_eq!(vel, parsed);
    }
}
