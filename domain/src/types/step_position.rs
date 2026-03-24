use std::fmt;

/// Position within a beat, using standard rhythm syllables.
/// First = downbeat ("Beat"), Second = "E", Third = "And", Fourth = "A".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepPosition {
    First,
    Second,
    Third,
    Fourth,
}

impl StepPosition {
    /// All four step positions in order.
    pub const ALL: [StepPosition; 4] = [
        StepPosition::First,
        StepPosition::Second,
        StepPosition::Third,
        StepPosition::Fourth,
    ];
}

impl fmt::Display for StepPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            StepPosition::First => "Beat",
            StepPosition::Second => "E",
            StepPosition::Third => "And",
            StepPosition::Fourth => "A",
        };
        write!(f, "{}", label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_labels() {
        assert_eq!(StepPosition::First.to_string(), "Beat");
        assert_eq!(StepPosition::Second.to_string(), "E");
        assert_eq!(StepPosition::Third.to_string(), "And");
        assert_eq!(StepPosition::Fourth.to_string(), "A");
    }

    #[test]
    fn test_all_contains_four_variants() {
        assert_eq!(StepPosition::ALL.len(), 4);
        assert_eq!(StepPosition::ALL[0], StepPosition::First);
        assert_eq!(StepPosition::ALL[1], StepPosition::Second);
        assert_eq!(StepPosition::ALL[2], StepPosition::Third);
        assert_eq!(StepPosition::ALL[3], StepPosition::Fourth);
    }

    #[test]
    fn test_clone_copy_eq_hash() {
        let pos = StepPosition::First;
        let pos2 = pos;
        assert_eq!(pos, pos2);

        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(StepPosition::First);
        set.insert(StepPosition::Second);
        set.insert(StepPosition::First); // duplicate
        assert_eq!(set.len(), 2);
    }
}
