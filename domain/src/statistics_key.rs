use crate::training_discipline::TrainingDiscipline;
use crate::types::{RhythmDirection, TempoRange};

/// Key for per-discipline statistics in the perceptual profile.
///
/// Pitch disciplines have a 1:1 mapping (one key per discipline).
/// Rhythm disciplines expand to multiple keys: one per (TempoRange × RhythmDirection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatisticsKey {
    Pitch(TrainingDiscipline),
    Rhythm(TrainingDiscipline, TempoRange, RhythmDirection),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_key_derives() {
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        let key2 = key;
        assert_eq!(key, key2);
    }

    #[test]
    fn test_rhythm_key_derives() {
        let key = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        let key2 = key;
        assert_eq!(key, key2);
    }

    #[test]
    fn test_different_keys_not_equal() {
        let pitch = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        let rhythm = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        assert_ne!(pitch, rhythm);
    }

    #[test]
    fn test_hash_usable_in_hashmap() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        map.insert(key, 42);
        assert_eq!(map.get(&key), Some(&42));
    }

    #[test]
    fn test_rhythm_keys_differ_by_tempo_range() {
        let slow = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        let fast = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Fast,
            RhythmDirection::Early,
        );
        assert_ne!(slow, fast);
    }

    #[test]
    fn test_rhythm_keys_differ_by_direction() {
        let early = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        let late = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Late,
        );
        assert_ne!(early, late);
    }
}
