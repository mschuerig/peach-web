use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use super::midi::MIDINote;

/// A validated MIDI note range where min <= max.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NoteRange {
    min: MIDINote,
    max: MIDINote,
}

impl NoteRange {
    /// Create a NoteRange. Panics if min > max.
    /// Use `try_new()` when input is not guaranteed valid.
    pub fn new(min: MIDINote, max: MIDINote) -> Self {
        Self::try_new(min, max).expect("NoteRange min must be <= max")
    }

    /// Fallible constructor: returns `Err` if min > max.
    pub fn try_new(min: MIDINote, max: MIDINote) -> Result<Self, DomainError> {
        if min.raw_value() > max.raw_value() {
            Err(DomainError::InvalidNoteRange {
                min: min.raw_value(),
                max: max.raw_value(),
            })
        } else {
            Ok(Self { min, max })
        }
    }

    pub fn min(&self) -> MIDINote {
        self.min
    }

    pub fn max(&self) -> MIDINote {
        self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_range() {
        let range = NoteRange::new(MIDINote::new(36), MIDINote::new(84));
        assert_eq!(range.min().raw_value(), 36);
        assert_eq!(range.max().raw_value(), 84);
    }

    #[test]
    fn test_equal_range() {
        let range = NoteRange::new(MIDINote::new(60), MIDINote::new(60));
        assert_eq!(range.min(), range.max());
    }

    #[test]
    fn test_full_midi_range() {
        let range = NoteRange::new(MIDINote::new(0), MIDINote::new(127));
        assert_eq!(range.min().raw_value(), 0);
        assert_eq!(range.max().raw_value(), 127);
    }

    #[test]
    #[should_panic(expected = "NoteRange min must be <= max")]
    fn test_panics_on_inverted_range() {
        NoteRange::new(MIDINote::new(84), MIDINote::new(36));
    }

    #[test]
    fn test_try_new_valid() {
        assert!(NoteRange::try_new(MIDINote::new(36), MIDINote::new(84)).is_ok());
    }

    #[test]
    fn test_try_new_equal() {
        assert!(NoteRange::try_new(MIDINote::new(60), MIDINote::new(60)).is_ok());
    }

    #[test]
    fn test_try_new_invalid() {
        let result = NoteRange::try_new(MIDINote::new(84), MIDINote::new(36));
        assert!(result.is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let range = NoteRange::new(MIDINote::new(36), MIDINote::new(84));
        let json = serde_json::to_string(&range).unwrap();
        let parsed: NoteRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, parsed);
    }
}
