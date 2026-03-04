use std::collections::HashSet;

use domain::{DirectedInterval, Direction, Interval};

/// Encode a set of directed intervals as a sorted, comma-separated string.
///
/// Format: `P1`, `m2u`, `M3d`, etc.
pub fn encode_intervals(intervals: &HashSet<DirectedInterval>) -> String {
    let mut codes: Vec<String> = intervals.iter().map(encode_one).collect();
    codes.sort();
    codes.join(",")
}

/// Decode a comma-separated interval code string into a set of directed intervals.
///
/// Invalid codes are silently skipped.
pub fn decode_intervals(code: &str) -> HashSet<DirectedInterval> {
    code.split(',')
        .filter_map(|s| decode_one(s.trim()))
        .collect()
}

/// Human-readable label for an interval with direction (e.g. "Perfect Fifth Up").
pub fn interval_label(interval: Interval, direction: Direction) -> String {
    let name = match interval {
        Interval::Prime => return "Prime".to_string(),
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
    };
    let dir = match direction {
        Direction::Up => "Up",
        Direction::Down => "Down",
    };
    format!("{name} {dir}")
}

fn encode_one(di: &DirectedInterval) -> String {
    let base = match di.interval {
        Interval::Prime => return "P1".to_string(),
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
    };
    let dir = match di.direction {
        Direction::Up => "u",
        Direction::Down => "d",
    };
    format!("{base}{dir}")
}

fn decode_one(code: &str) -> Option<DirectedInterval> {
    if code == "P1" {
        return Some(DirectedInterval::new(Interval::Prime, Direction::Up));
    }

    if code.len() < 3 {
        return None;
    }

    let (base, dir_char) = code.split_at(code.len() - 1);
    let direction = match dir_char {
        "u" => Direction::Up,
        "d" => Direction::Down,
        _ => return None,
    };

    let interval = match base {
        "m2" => Interval::MinorSecond,
        "M2" => Interval::MajorSecond,
        "m3" => Interval::MinorThird,
        "M3" => Interval::MajorThird,
        "P4" => Interval::PerfectFourth,
        "d5" => Interval::Tritone,
        "P5" => Interval::PerfectFifth,
        "m6" => Interval::MinorSixth,
        "M6" => Interval::MajorSixth,
        "m7" => Interval::MinorSeventh,
        "M7" => Interval::MajorSeventh,
        "P8" => Interval::Octave,
        _ => return None,
    };

    Some(DirectedInterval::new(interval, direction))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_prime() {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
        assert_eq!(encode_intervals(&set), "P1");
    }

    #[test]
    fn test_encode_multiple() {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(
            Interval::MajorThird,
            Direction::Up,
        ));
        set.insert(DirectedInterval::new(
            Interval::MajorThird,
            Direction::Down,
        ));
        set.insert(DirectedInterval::new(
            Interval::MinorSixth,
            Direction::Up,
        ));
        let encoded = encode_intervals(&set);
        // Sorted alphabetically
        assert_eq!(encoded, "M3d,M3u,m6u");
    }

    #[test]
    fn test_decode_prime() {
        let set = decode_intervals("P1");
        assert_eq!(set.len(), 1);
        assert!(set.contains(&DirectedInterval::new(Interval::Prime, Direction::Up)));
    }

    #[test]
    fn test_decode_multiple() {
        let set = decode_intervals("M3u,M3d,m6u");
        assert_eq!(set.len(), 3);
        assert!(set.contains(&DirectedInterval::new(Interval::MajorThird, Direction::Up)));
        assert!(set.contains(&DirectedInterval::new(Interval::MajorThird, Direction::Down)));
        assert!(set.contains(&DirectedInterval::new(Interval::MinorSixth, Direction::Up)));
    }

    #[test]
    fn test_decode_skips_invalid() {
        let set = decode_intervals("M3u,INVALID,P5d");
        assert_eq!(set.len(), 2);
        assert!(set.contains(&DirectedInterval::new(Interval::MajorThird, Direction::Up)));
        assert!(set.contains(&DirectedInterval::new(Interval::PerfectFifth, Direction::Down)));
    }

    #[test]
    fn test_decode_empty_string() {
        let set = decode_intervals("");
        assert!(set.is_empty());
    }

    #[test]
    fn test_roundtrip_all_intervals() {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));

        let intervals = [
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
        ];
        for interval in &intervals {
            set.insert(DirectedInterval::new(*interval, Direction::Up));
            set.insert(DirectedInterval::new(*interval, Direction::Down));
        }

        let encoded = encode_intervals(&set);
        let decoded = decode_intervals(&encoded);
        assert_eq!(set, decoded);
    }

    #[test]
    fn test_roundtrip_single() {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(
            Interval::PerfectFifth,
            Direction::Down,
        ));
        let encoded = encode_intervals(&set);
        assert_eq!(encoded, "P5d");
        let decoded = decode_intervals(&encoded);
        assert_eq!(set, decoded);
    }

    #[test]
    fn test_interval_label_prime() {
        assert_eq!(interval_label(Interval::Prime, Direction::Up), "Prime");
    }

    #[test]
    fn test_interval_label_with_direction() {
        assert_eq!(
            interval_label(Interval::PerfectFifth, Direction::Up),
            "Perfect Fifth Up"
        );
        assert_eq!(
            interval_label(Interval::MinorThird, Direction::Down),
            "Minor Third Down"
        );
    }
}
