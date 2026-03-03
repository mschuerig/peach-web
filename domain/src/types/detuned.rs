use serde::{Deserialize, Serialize};

use super::cents::Cents;
use super::midi::MIDINote;

/// A MIDI note with an optional cent offset (detuning).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetunedMIDINote {
    pub note: MIDINote,
    pub offset: Cents,
}

impl From<MIDINote> for DetunedMIDINote {
    fn from(note: MIDINote) -> Self {
        Self {
            note,
            offset: Cents::new(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detuned_from_midi_note() {
        let note = MIDINote::new(69);
        let detuned = DetunedMIDINote::from(note);
        assert_eq!(detuned.note, note);
        assert_eq!(detuned.offset.raw_value, 0.0);
    }

    #[test]
    fn test_detuned_with_offset() {
        let detuned = DetunedMIDINote {
            note: MIDINote::new(60),
            offset: Cents::new(15.0),
        };
        assert_eq!(detuned.note.raw_value(), 60);
        assert_eq!(detuned.offset.raw_value, 15.0);
    }

    #[test]
    fn test_detuned_serde_roundtrip() {
        let d = DetunedMIDINote {
            note: MIDINote::new(60),
            offset: Cents::new(15.0),
        };
        let json = serde_json::to_string(&d).unwrap();
        let parsed: DetunedMIDINote = serde_json::from_str(&json).unwrap();
        assert_eq!(d, parsed);
    }
}
