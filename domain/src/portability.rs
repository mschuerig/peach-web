//! Helpers for cross-platform data portability (iOS ↔ web CSV format).
//!
//! Converts between the web's internal representations (semitone u8, MIDI u8)
//! and the iOS CSV string formats (interval codes like "P5", note names like "C#4").

/// Convert semitone count (0–12) to iOS interval code string.
///
/// Returns `None` for values outside 0–12.
pub fn to_interval_code(semitones: u8) -> Option<&'static str> {
    match semitones {
        0 => Some("P1"),
        1 => Some("m2"),
        2 => Some("M2"),
        3 => Some("m3"),
        4 => Some("M3"),
        5 => Some("P4"),
        6 => Some("A4"),
        7 => Some("P5"),
        8 => Some("m6"),
        9 => Some("M6"),
        10 => Some("m7"),
        11 => Some("M7"),
        12 => Some("P8"),
        _ => None,
    }
}

/// Parse an iOS interval code string back to semitone count (0–12).
///
/// Accepts both "A4" and "d5" for the tritone (semitone 6).
/// Returns `None` for unrecognized codes.
pub fn from_interval_code(code: &str) -> Option<u8> {
    match code {
        "P1" => Some(0),
        "m2" => Some(1),
        "M2" => Some(2),
        "m3" => Some(3),
        "M3" => Some(4),
        "P4" => Some(5),
        "A4" | "d5" => Some(6),
        "P5" => Some(7),
        "m6" => Some(8),
        "M6" => Some(9),
        "m7" => Some(10),
        "M7" => Some(11),
        "P8" => Some(12),
        _ => None,
    }
}

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Convert a MIDI note number (0–127) to a note name string (e.g. 60 → "C4").
///
/// Uses sharps only (matching iOS convention). MIDI note 0 is C-1, 60 is C4.
pub fn midi_note_name(midi: u8) -> String {
    let note_index = (midi % 12) as usize;
    let octave = (midi as i8 / 12) - 1;
    format!("{}{}", NOTE_NAMES[note_index], octave)
}

/// Truncate an ISO 8601 timestamp to second precision.
///
/// Strips fractional seconds: `"2026-03-04T14:30:00.456Z"` → `"2026-03-04T14:30:00Z"`.
/// If the timestamp already has second precision, returns it unchanged.
/// Returns the input unchanged if it doesn't match the expected format.
pub fn truncate_timestamp_to_second(ts: &str) -> String {
    // Find the dot that starts fractional seconds (before Z or +/- timezone)
    if let Some(dot_pos) = ts.rfind('.') {
        // Find the timezone marker after the dot
        if let Some(tz_pos) = ts[dot_pos..].find(['Z', '+', '-']) {
            let mut result = ts[..dot_pos].to_string();
            result.push_str(&ts[dot_pos + tz_pos..]);
            return result;
        }
    }
    ts.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- to_interval_code tests ---

    #[test]
    fn test_to_interval_code_all_values() {
        let expected = [
            (0, "P1"),
            (1, "m2"),
            (2, "M2"),
            (3, "m3"),
            (4, "M3"),
            (5, "P4"),
            (6, "A4"),
            (7, "P5"),
            (8, "m6"),
            (9, "M6"),
            (10, "m7"),
            (11, "M7"),
            (12, "P8"),
        ];
        for (semitones, code) in expected {
            assert_eq!(
                to_interval_code(semitones),
                Some(code),
                "semitones={semitones}"
            );
        }
    }

    #[test]
    fn test_to_interval_code_out_of_range() {
        assert_eq!(to_interval_code(13), None);
        assert_eq!(to_interval_code(255), None);
    }

    // --- from_interval_code tests ---

    #[test]
    fn test_from_interval_code_all_values() {
        let expected = [
            ("P1", 0),
            ("m2", 1),
            ("M2", 2),
            ("m3", 3),
            ("M3", 4),
            ("P4", 5),
            ("A4", 6),
            ("d5", 6),
            ("P5", 7),
            ("m6", 8),
            ("M6", 9),
            ("m7", 10),
            ("M7", 11),
            ("P8", 12),
        ];
        for (code, semitones) in expected {
            assert_eq!(from_interval_code(code), Some(semitones), "code={code}");
        }
    }

    #[test]
    fn test_from_interval_code_invalid() {
        assert_eq!(from_interval_code("X1"), None);
        assert_eq!(from_interval_code(""), None);
        assert_eq!(from_interval_code("p1"), None); // case-sensitive
    }

    // --- round-trip tests ---

    #[test]
    fn test_interval_code_roundtrip() {
        for semitones in 0..=12u8 {
            let code = to_interval_code(semitones).unwrap();
            let back = from_interval_code(code).unwrap();
            assert_eq!(back, semitones, "roundtrip failed for {semitones} -> {code}");
        }
    }

    // --- midi_note_name tests ---

    #[test]
    fn test_midi_note_name_middle_c() {
        assert_eq!(midi_note_name(60), "C4");
    }

    #[test]
    fn test_midi_note_name_sharps() {
        assert_eq!(midi_note_name(61), "C#4");
        assert_eq!(midi_note_name(66), "F#4");
        assert_eq!(midi_note_name(68), "G#4");
    }

    #[test]
    fn test_midi_note_name_octave_boundaries() {
        assert_eq!(midi_note_name(0), "C-1");
        assert_eq!(midi_note_name(12), "C0");
        assert_eq!(midi_note_name(21), "A0"); // lowest piano key
        assert_eq!(midi_note_name(108), "C8"); // highest piano key
        assert_eq!(midi_note_name(127), "G9");
    }

    #[test]
    fn test_midi_note_name_from_ios_examples() {
        // From the iOS CSV sample data
        assert_eq!(midi_note_name(51), "D#3");
        assert_eq!(midi_note_name(80), "G#5");
        assert_eq!(midi_note_name(48), "C3");
        assert_eq!(midi_note_name(78), "F#5");
        assert_eq!(midi_note_name(47), "B2");
        assert_eq!(midi_note_name(75), "D#5");
    }

    // --- truncate_timestamp_to_second tests ---

    #[test]
    fn test_truncate_timestamp_with_fractional() {
        assert_eq!(
            truncate_timestamp_to_second("2026-03-04T14:30:00.456Z"),
            "2026-03-04T14:30:00Z"
        );
    }

    #[test]
    fn test_truncate_timestamp_already_second_precision() {
        assert_eq!(
            truncate_timestamp_to_second("2026-03-04T14:30:00Z"),
            "2026-03-04T14:30:00Z"
        );
    }

    #[test]
    fn test_truncate_timestamp_many_fractional_digits() {
        assert_eq!(
            truncate_timestamp_to_second("2026-03-04T14:30:00.123456789Z"),
            "2026-03-04T14:30:00Z"
        );
    }

    #[test]
    fn test_truncate_timestamp_with_timezone_offset() {
        assert_eq!(
            truncate_timestamp_to_second("2026-03-04T14:30:00.456+02:00"),
            "2026-03-04T14:30:00+02:00"
        );
    }
}
