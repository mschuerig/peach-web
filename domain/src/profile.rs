use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use crate::types::MIDINote;

/// Custom serde for [PerceptualNote; 128] — serialize as Vec since serde
/// doesn't support arrays > 32 elements natively.
mod note_array_serde {
    use super::PerceptualNote;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(notes: &[PerceptualNote; 128], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        notes.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[PerceptualNote; 128], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<PerceptualNote>::deserialize(deserializer)?;
        let arr: [PerceptualNote; 128] = vec
            .try_into()
            .map_err(|v: Vec<_>| serde::de::Error::custom(format!("expected 128 notes, got {}", v.len())))?;
        Ok(arr)
    }
}

/// Per-note perceptual statistics tracked via Welford's online algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerceptualNote {
    mean: f64,
    std_dev: f64,
    m2: f64,
    sample_count: u32,
    current_difficulty: f64,
}

impl PerceptualNote {
    fn default() -> Self {
        Self {
            mean: 0.0,
            std_dev: 0.0,
            m2: 0.0,
            sample_count: 0,
            current_difficulty: 100.0,
        }
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn current_difficulty(&self) -> f64 {
        self.current_difficulty
    }

    pub fn is_trained(&self) -> bool {
        self.sample_count > 0
    }
}

/// Aggregate perceptual profile across all 128 MIDI notes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerceptualProfile {
    #[serde(with = "note_array_serde")]
    notes: [PerceptualNote; 128],
    matching_count: u32,
    matching_mean_abs: f64,
    matching_m2: f64,
}

impl PerceptualProfile {
    pub fn new() -> Self {
        Self {
            notes: [PerceptualNote::default(); 128],
            matching_count: 0,
            matching_mean_abs: 0.0,
            matching_m2: 0.0,
        }
    }

    /// Update a note's statistics using Welford's online algorithm.
    /// `cent_offset` is the absolute magnitude of the cent difference presented.
    /// `is_correct` is accepted for future use but not used in computation.
    pub fn update(&mut self, note: MIDINote, cent_offset: f64, _is_correct: bool) {
        let stats = &mut self.notes[note.raw_value() as usize];
        stats.sample_count += 1;
        let delta = cent_offset - stats.mean;
        stats.mean += delta / stats.sample_count as f64;
        let delta2 = cent_offset - stats.mean;
        stats.m2 += delta * delta2;
        let variance = if stats.sample_count < 2 {
            0.0
        } else {
            stats.m2 / (stats.sample_count - 1) as f64
        };
        stats.std_dev = variance.sqrt();
    }

    /// Return the top `count` weak spots: untrained notes first (infinite score),
    /// then trained notes sorted by highest mean (weaker = higher threshold).
    pub fn weak_spots(&self, count: usize) -> Vec<MIDINote> {
        let mut scored: Vec<(u8, f64)> = self
            .notes
            .iter()
            .enumerate()
            .map(|(i, note)| {
                let score = if note.is_trained() {
                    note.mean
                } else {
                    f64::INFINITY
                };
                (i as u8, score)
            })
            .collect();

        // Sort descending by score (highest = weakest)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(count)
            .map(|(i, _)| MIDINote::new(i))
            .collect()
    }

    /// Average of per-note means across all trained notes.
    pub fn overall_mean(&self) -> Option<f64> {
        let trained: Vec<f64> = self
            .notes
            .iter()
            .filter(|n| n.is_trained())
            .map(|n| n.mean)
            .collect();

        if trained.is_empty() {
            None
        } else {
            Some(trained.iter().sum::<f64>() / trained.len() as f64)
        }
    }

    /// Sample standard deviation of per-note means across trained notes.
    pub fn overall_std_dev(&self) -> Option<f64> {
        let trained: Vec<f64> = self
            .notes
            .iter()
            .filter(|n| n.is_trained())
            .map(|n| n.mean)
            .collect();

        if trained.len() < 2 {
            return None;
        }

        let mean = trained.iter().sum::<f64>() / trained.len() as f64;
        let variance =
            trained.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (trained.len() - 1) as f64;
        Some(variance.sqrt())
    }

    /// Average mean across trained notes within a MIDI note range.
    pub fn average_threshold(&self, range: RangeInclusive<u8>) -> Option<f64> {
        let trained: Vec<f64> = self
            .notes
            .iter()
            .enumerate()
            .filter(|(i, n)| range.contains(&(*i as u8)) && n.is_trained())
            .map(|(_, n)| n.mean)
            .collect();

        if trained.is_empty() {
            None
        } else {
            Some(trained.iter().sum::<f64>() / trained.len() as f64)
        }
    }

    /// Update aggregate pitch matching statistics using Welford's on abs(cent_error).
    pub fn update_matching(&mut self, _note: MIDINote, cent_error: f64) {
        let abs_error = cent_error.abs();
        self.matching_count += 1;
        let delta = abs_error - self.matching_mean_abs;
        self.matching_mean_abs += delta / self.matching_count as f64;
        let delta2 = abs_error - self.matching_mean_abs;
        self.matching_m2 += delta * delta2;
    }

    /// Mean absolute pitch matching error, or None if no data.
    pub fn matching_mean(&self) -> Option<f64> {
        if self.matching_count > 0 {
            Some(self.matching_mean_abs)
        } else {
            None
        }
    }

    /// Standard deviation of absolute pitch matching errors, or None if fewer than 2 samples.
    pub fn matching_std_dev(&self) -> Option<f64> {
        if self.matching_count >= 2 {
            Some((self.matching_m2 / (self.matching_count - 1) as f64).sqrt())
        } else {
            None
        }
    }

    /// Reset all 128 notes to defaults.
    pub fn reset(&mut self) {
        self.notes = [PerceptualNote::default(); 128];
    }

    /// Zero matching accumulators.
    pub fn reset_matching(&mut self) {
        self.matching_count = 0;
        self.matching_mean_abs = 0.0;
        self.matching_m2 = 0.0;
    }

    /// Read-only access to a specific note's statistics.
    pub fn note_stats(&self, note: MIDINote) -> &PerceptualNote {
        &self.notes[note.raw_value() as usize]
    }
}

impl Default for PerceptualProfile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- AC1: Single update ---

    #[test]
    fn test_single_update_mean_count_stddev() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 50.0, true);

        let stats = profile.note_stats(MIDINote::new(60));
        assert_eq!(stats.mean(), 50.0);
        assert_eq!(stats.sample_count(), 1);
        assert_eq!(stats.std_dev(), 0.0);
        assert!(stats.is_trained());
    }

    // --- AC2: Welford's correctness ---

    #[test]
    fn test_welford_two_samples() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 40.0, true);
        profile.update(MIDINote::new(60), 60.0, false);

        let stats = profile.note_stats(MIDINote::new(60));
        assert_eq!(stats.sample_count(), 2);
        assert!((stats.mean() - 50.0).abs() < 1e-10);
        // Sample std dev: sqrt(((40-50)^2 + (60-50)^2) / (2-1)) = sqrt(200) ≈ 14.142
        assert!((stats.std_dev() - (200.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_welford_three_samples() {
        let mut profile = PerceptualProfile::new();
        let values = [10.0, 20.0, 30.0];
        for &v in &values {
            profile.update(MIDINote::new(42), v, true);
        }

        let stats = profile.note_stats(MIDINote::new(42));
        assert_eq!(stats.sample_count(), 3);
        // Mean: (10 + 20 + 30) / 3 = 20
        assert!((stats.mean() - 20.0).abs() < 1e-10);
        // Sample std dev: sqrt(((10-20)^2 + (20-20)^2 + (30-20)^2) / 2) = sqrt(100) = 10
        assert!((stats.std_dev() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_welford_five_samples() {
        let mut profile = PerceptualProfile::new();
        let values = [2.0, 4.0, 4.0, 4.0, 5.0];
        for &v in &values {
            profile.update(MIDINote::new(69), v, true);
        }

        let stats = profile.note_stats(MIDINote::new(69));
        assert_eq!(stats.sample_count(), 5);
        // Mean: (2+4+4+4+5)/5 = 19/5 = 3.8
        assert!((stats.mean() - 3.8).abs() < 1e-10);
        // Variance (sample): sum of (x - mean)^2 / (n-1)
        // = ((2-3.8)^2 + (4-3.8)^2 + (4-3.8)^2 + (4-3.8)^2 + (5-3.8)^2) / 4
        // = (3.24 + 0.04 + 0.04 + 0.04 + 1.44) / 4
        // = 4.8 / 4 = 1.2
        // Std dev: sqrt(1.2) ≈ 1.0954
        assert!((stats.std_dev() - (1.2_f64).sqrt()).abs() < 1e-10);
    }

    // --- AC3: Weak spots ---

    #[test]
    fn test_weak_spots_untrained_first() {
        let mut profile = PerceptualProfile::new();
        // Train notes 0, 1, 2 with varying means
        profile.update(MIDINote::new(0), 10.0, true);
        profile.update(MIDINote::new(1), 50.0, true);
        profile.update(MIDINote::new(2), 30.0, true);

        let weak = profile.weak_spots(5);
        assert_eq!(weak.len(), 5);
        // First 2 should be untrained (any of notes 3-127)
        assert!(!profile.note_stats(weak[0]).is_trained());
        assert!(!profile.note_stats(weak[1]).is_trained());
    }

    #[test]
    fn test_weak_spots_trained_sorted_by_highest_mean() {
        let mut profile = PerceptualProfile::new();
        // Train all 128 notes with different means
        for i in 0..128u8 {
            profile.update(MIDINote::new(i), i as f64, true);
        }

        let weak = profile.weak_spots(3);
        assert_eq!(weak.len(), 3);
        // Highest mean first: 127, 126, 125
        assert_eq!(weak[0].raw_value(), 127);
        assert_eq!(weak[1].raw_value(), 126);
        assert_eq!(weak[2].raw_value(), 125);
    }

    #[test]
    fn test_weak_spots_count_limits() {
        let profile = PerceptualProfile::new();
        let weak = profile.weak_spots(10);
        assert_eq!(weak.len(), 10);
    }

    // --- AC4: Summary statistics ---

    #[test]
    fn test_overall_mean_no_trained_notes() {
        let profile = PerceptualProfile::new();
        assert_eq!(profile.overall_mean(), None);
    }

    #[test]
    fn test_overall_mean_with_trained_notes() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 40.0, true);
        profile.update(MIDINote::new(72), 60.0, true);

        // overall_mean = average of per-note means = (40 + 60) / 2 = 50
        assert!((profile.overall_mean().unwrap() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_overall_std_dev_none_with_fewer_than_two() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 40.0, true);
        assert_eq!(profile.overall_std_dev(), None);
    }

    #[test]
    fn test_overall_std_dev_with_trained_notes() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 40.0, true);
        profile.update(MIDINote::new(72), 60.0, true);

        // Means: [40, 60], overall mean = 50
        // Sample std dev = sqrt(((40-50)^2 + (60-50)^2) / 1) = sqrt(200) ≈ 14.142
        let std = profile.overall_std_dev().unwrap();
        assert!((std - (200.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_average_threshold_range() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 40.0, true);
        profile.update(MIDINote::new(65), 60.0, true);
        profile.update(MIDINote::new(72), 80.0, true);

        // Range 60..=65 includes notes 60 (mean=40) and 65 (mean=60)
        let avg = profile.average_threshold(60..=65).unwrap();
        assert!((avg - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_average_threshold_no_trained_in_range() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 40.0, true);
        assert_eq!(profile.average_threshold(70..=80), None);
    }

    // --- AC5: Pitch matching accumulators ---

    #[test]
    fn test_matching_no_data() {
        let profile = PerceptualProfile::new();
        assert_eq!(profile.matching_mean(), None);
        assert_eq!(profile.matching_std_dev(), None);
    }

    #[test]
    fn test_matching_single_sample() {
        let mut profile = PerceptualProfile::new();
        profile.update_matching(MIDINote::new(60), -5.0);

        assert!((profile.matching_mean().unwrap() - 5.0).abs() < 1e-10);
        assert_eq!(profile.matching_std_dev(), None); // Need 2+ samples
    }

    #[test]
    fn test_matching_multiple_samples() {
        let mut profile = PerceptualProfile::new();
        profile.update_matching(MIDINote::new(60), 3.0);
        profile.update_matching(MIDINote::new(60), -5.0);
        profile.update_matching(MIDINote::new(60), 7.0);

        // Abs values: 3, 5, 7. Mean = 5.0
        assert!((profile.matching_mean().unwrap() - 5.0).abs() < 1e-10);

        // Sample std dev: sqrt(((3-5)^2 + (5-5)^2 + (7-5)^2) / 2) = sqrt(4) = 2.0
        assert!((profile.matching_std_dev().unwrap() - 2.0).abs() < 1e-10);
    }

    // --- AC6: Reset ---

    #[test]
    fn test_reset_clears_all_notes() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 50.0, true);
        profile.update(MIDINote::new(72), 30.0, false);

        profile.reset();

        assert!(!profile.note_stats(MIDINote::new(60)).is_trained());
        assert!(!profile.note_stats(MIDINote::new(72)).is_trained());
        assert_eq!(profile.overall_mean(), None);
    }

    #[test]
    fn test_reset_matching_clears_accumulators() {
        let mut profile = PerceptualProfile::new();
        profile.update_matching(MIDINote::new(60), 5.0);
        profile.update_matching(MIDINote::new(60), 10.0);

        profile.reset_matching();

        assert_eq!(profile.matching_mean(), None);
        assert_eq!(profile.matching_std_dev(), None);
    }

    // --- Untrained note defaults ---

    #[test]
    fn test_untrained_note_defaults() {
        let profile = PerceptualProfile::new();
        let stats = profile.note_stats(MIDINote::new(60));
        assert_eq!(stats.mean(), 0.0);
        assert_eq!(stats.std_dev(), 0.0);
        assert_eq!(stats.sample_count(), 0);
        assert_eq!(stats.current_difficulty(), 100.0);
        assert!(!stats.is_trained());
    }

    // --- Serde ---

    #[test]
    fn test_perceptual_note_serde_roundtrip() {
        let note = PerceptualNote {
            mean: 42.5,
            std_dev: 3.2,
            m2: 20.48,
            sample_count: 3,
            current_difficulty: 50.0,
        };
        let json = serde_json::to_string(&note).unwrap();
        let parsed: PerceptualNote = serde_json::from_str(&json).unwrap();
        assert_eq!(note, parsed);
    }

    #[test]
    fn test_perceptual_profile_serde_roundtrip() {
        let mut profile = PerceptualProfile::new();
        profile.update(MIDINote::new(60), 50.0, true);
        profile.update_matching(MIDINote::new(60), 3.0);

        let json = serde_json::to_string(&profile).unwrap();
        let parsed: PerceptualProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, parsed);
    }

    // --- Default trait ---

    #[test]
    fn test_profile_default() {
        let profile = PerceptualProfile::default();
        assert_eq!(profile.overall_mean(), None);
        assert_eq!(profile.matching_mean(), None);
    }
}
