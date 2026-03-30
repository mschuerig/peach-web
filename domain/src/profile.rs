use std::collections::HashMap;

use crate::metric_point::MetricPoint;
use crate::statistics_key::StatisticsKey;
use crate::training_discipline::TrainingDiscipline;
use crate::training_discipline_statistics::TrainingDisciplineStatistics;
use crate::trend::Trend;
use crate::types::Cents;

/// Cold-start difficulty for untrained disciplines (cents).
pub const COLD_START_DIFFICULTY: f64 = 100.0;

/// Discipline-aware perceptual profile — single source of truth for all per-discipline statistics.
///
/// Uses `StatisticsKey` as map key: pitch disciplines have 1 key each,
/// rhythm disciplines expand to 18 keys (6 tempo ranges × 3 directions).
/// Aligns with iOS `PerceptualProfile` post-Epic-44 architecture.
#[derive(Clone, Debug)]
pub struct PerceptualProfile {
    entries: HashMap<StatisticsKey, TrainingDisciplineStatistics>,
}

impl PerceptualProfile {
    /// Creates a profile with all keys initialized to empty statistics.
    /// 4 pitch keys + 36 rhythm keys = 40 total for 6 disciplines.
    pub fn new() -> Self {
        let mut entries = HashMap::new();
        for discipline in TrainingDiscipline::ALL {
            for key in discipline.statistics_keys() {
                entries.insert(key, TrainingDisciplineStatistics::new());
            }
        }
        Self { entries }
    }

    // --- Per-key query API ---

    /// Direct access to statistics for a specific key.
    ///
    /// # Panics
    /// Panics in debug builds if the key does not match its discipline's key expansion
    /// (e.g., `Pitch` wrapping a rhythm discipline). In release builds, returns the
    /// entry or panics if the key was not initialized.
    pub fn statistics_for_key(&self, key: &StatisticsKey) -> &TrainingDisciplineStatistics {
        debug_assert!(
            self.entries.contains_key(key),
            "StatisticsKey {key:?} not found — possible discipline/variant mismatch"
        );
        self.entries.get(key).expect("all keys initialized")
    }

    // --- Per-discipline query API (merged across keys) ---

    /// Merged statistics for a discipline (aggregates across all its keys).
    /// Returns `None` if no data across any key for this discipline.
    pub fn discipline_statistics(
        &self,
        discipline: TrainingDiscipline,
    ) -> Option<TrainingDisciplineStatistics> {
        self.merged_statistics(&discipline.statistics_keys())
    }

    /// Merge statistics from multiple keys by collecting all metrics chronologically
    /// and rebuilding Welford/EWMA/trend from scratch.
    ///
    /// # Panics
    /// Debug-panics if `keys` span more than one discipline — mixed-discipline
    /// merges produce wrong EWMA/trend parameters.
    pub fn merged_statistics(
        &self,
        keys: &[StatisticsKey],
    ) -> Option<TrainingDisciplineStatistics> {
        debug_assert!(
            {
                let disciplines: std::collections::HashSet<_> = keys
                    .iter()
                    .map(|k| match k {
                        StatisticsKey::Pitch(d) | StatisticsKey::Rhythm(d, _, _) => *d,
                    })
                    .collect();
                disciplines.len() <= 1
            },
            "merged_statistics called with keys spanning multiple disciplines"
        );

        // Collect all metrics from the requested keys
        let mut all_metrics: Vec<MetricPoint> = Vec::new();
        let mut config = None;

        for key in keys {
            if let Some(stats) = self.entries.get(key) {
                all_metrics.extend(stats.metrics.iter().cloned());
                // All keys for a discipline share the same config
                if config.is_none() {
                    config = Some(match key {
                        StatisticsKey::Pitch(d) | StatisticsKey::Rhythm(d, _, _) => d.config(),
                    });
                }
            }
        }

        if all_metrics.is_empty() {
            return None;
        }

        // Sort chronologically (total_cmp handles NaN deterministically)
        all_metrics.sort_by(|a, b| a.timestamp.total_cmp(&b.timestamp));

        // Rebuild statistics from the merged, sorted metrics
        let config = config.expect("at least one key had metrics");
        let mut merged = TrainingDisciplineStatistics::new();
        merged.rebuild(all_metrics, config);
        Some(merged)
    }

    /// Whether a discipline has any recorded data (across any of its keys).
    pub fn has_data(&self, discipline: TrainingDiscipline) -> bool {
        discipline
            .statistics_keys()
            .iter()
            .any(|key| self.statistics_for_key(key).record_count() > 0)
    }

    /// Training discipline state (NoData or Active).
    pub fn state(
        &self,
        discipline: TrainingDiscipline,
    ) -> crate::training_discipline::TrainingDisciplineState {
        if self.has_data(discipline) {
            crate::training_discipline::TrainingDisciplineState::Active
        } else {
            crate::training_discipline::TrainingDisciplineState::NoData
        }
    }

    /// Trend for a discipline (None if < 2 records).
    /// For single-key disciplines, returns the key's trend directly.
    /// For multi-key disciplines, returns the merged trend.
    pub fn trend(&self, discipline: TrainingDiscipline) -> Option<Trend> {
        let keys = discipline.statistics_keys();
        if keys.len() == 1 {
            self.statistics_for_key(&keys[0]).trend
        } else {
            self.discipline_statistics(discipline).and_then(|s| s.trend)
        }
    }

    /// Current EWMA for a discipline.
    /// For single-key disciplines, returns the key's EWMA directly.
    /// For multi-key disciplines, returns the merged EWMA.
    pub fn current_ewma(&self, discipline: TrainingDiscipline) -> Option<f64> {
        let keys = discipline.statistics_keys();
        if keys.len() == 1 {
            self.statistics_for_key(&keys[0]).ewma
        } else {
            self.discipline_statistics(discipline).and_then(|s| s.ewma)
        }
    }

    /// Record count for a discipline (sum across all its keys).
    pub fn record_count(&self, discipline: TrainingDiscipline) -> usize {
        discipline
            .statistics_keys()
            .iter()
            .map(|key| self.statistics_for_key(key).record_count())
            .sum()
    }

    // --- Strategy-facing API ---

    /// Discrimination mean for the given interval.
    /// Maps interval to the correct discipline (unison if interval == 0, otherwise interval mode).
    /// Used by `KazezNoteStrategy` as warm-start difficulty fallback.
    pub fn discrimination_mean(&self, interval: u8) -> Option<Cents> {
        let discipline = if interval == 0 {
            TrainingDiscipline::UnisonPitchDiscrimination
        } else {
            TrainingDiscipline::IntervalPitchDiscrimination
        };
        self.discipline_statistics(discipline)
            .map(|s| Cents::new(s.welford.mean()))
    }

    // --- Incremental update ---

    /// Add a single metric point for the given key.
    /// For discrimination disciplines, `is_correct` filters: only correct answers contribute.
    /// For matching disciplines, all answers contribute (pass `true`).
    pub fn add_point(&mut self, key: StatisticsKey, point: MetricPoint, is_correct: bool) {
        if !is_correct {
            return;
        }
        debug_assert!(
            self.entries.contains_key(&key),
            "StatisticsKey {key:?} not found — possible discipline/variant mismatch"
        );
        let discipline = match key {
            StatisticsKey::Pitch(d) | StatisticsKey::Rhythm(d, _, _) => d,
        };
        let config = discipline.config();
        let stats = self.entries.get_mut(&key).expect("all keys initialized");
        stats.add_point(point, config);
    }

    // --- Batch operations ---

    /// Rebuild from pre-sorted metric points keyed by StatisticsKey.
    ///
    /// # Panics
    /// Debug-panics if `points` contains keys not matching any valid
    /// discipline/variant combination (indicates a caller bug).
    pub fn rebuild_all(&mut self, points: HashMap<StatisticsKey, Vec<MetricPoint>>) {
        debug_assert!(
            points.keys().all(|k| self.entries.contains_key(k)),
            "rebuild_all called with unrecognized StatisticsKey(s)"
        );

        for discipline in TrainingDiscipline::ALL {
            for key in discipline.statistics_keys() {
                let stats = self.entries.get_mut(&key).expect("all keys initialized");
                if let Some(key_points) = points.get(&key) {
                    stats.rebuild(key_points.clone(), discipline.config());
                } else {
                    stats.reset();
                }
            }
        }
    }

    /// Reset all entries to empty state.
    pub fn reset_all(&mut self) {
        for stats in self.entries.values_mut() {
            stats.reset();
        }
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

    #[test]
    fn test_new_profile_all_modes_empty() {
        let profile = PerceptualProfile::new();
        for mode in TrainingDiscipline::ALL {
            assert_eq!(
                profile.state(mode),
                crate::training_discipline::TrainingDisciplineState::NoData
            );
            assert_eq!(profile.trend(mode), None);
            assert_eq!(profile.current_ewma(mode), None);
            assert_eq!(profile.record_count(mode), 0);
        }
    }

    #[test]
    fn test_new_profile_has_40_entries() {
        let profile = PerceptualProfile::new();
        // 4 pitch + 36 rhythm (2 × 6 × 3)
        assert_eq!(profile.entries.len(), 40);
    }

    #[test]
    fn test_add_point_updates_correct_key() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        profile.add_point(key, MetricPoint::new(1000.0, 20.0), true);
        assert!(profile.has_data(TrainingDiscipline::UnisonPitchDiscrimination));
        assert!(!profile.has_data(TrainingDiscipline::IntervalPitchDiscrimination));
        assert!(!profile.has_data(TrainingDiscipline::UnisonPitchMatching));
    }

    #[test]
    fn test_add_point_filters_incorrect() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        profile.add_point(key, MetricPoint::new(1000.0, 20.0), false);
        assert!(!profile.has_data(TrainingDiscipline::UnisonPitchDiscrimination));
    }

    #[test]
    fn test_discrimination_mean_unison() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        profile.add_point(key, MetricPoint::new(1000.0, 40.0), true);
        profile.add_point(key, MetricPoint::new(2000.0, 60.0), true);
        let mean = profile.discrimination_mean(0).unwrap();
        assert!((mean.raw_value - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_discrimination_mean_interval() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::IntervalPitchDiscrimination);
        profile.add_point(key, MetricPoint::new(1000.0, 30.0), true);
        let mean = profile.discrimination_mean(7).unwrap();
        assert!((mean.raw_value - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_discrimination_mean_no_data() {
        let profile = PerceptualProfile::new();
        assert_eq!(profile.discrimination_mean(0), None);
    }

    #[test]
    fn test_discipline_statistics_single_key() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        profile.add_point(key, MetricPoint::new(1000.0, 20.0), true);
        profile.add_point(key, MetricPoint::new(2000.0, 40.0), true);

        let stats = profile
            .discipline_statistics(TrainingDiscipline::UnisonPitchDiscrimination)
            .unwrap();
        assert_eq!(stats.record_count(), 2);
        assert!((stats.welford.mean() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_discipline_statistics_no_data() {
        let profile = PerceptualProfile::new();
        assert!(
            profile
                .discipline_statistics(TrainingDiscipline::UnisonPitchDiscrimination)
                .is_none()
        );
    }

    #[test]
    fn test_merged_statistics_rhythm_multiple_keys() {
        use crate::types::{RhythmDirection, TempoRange};

        let mut profile = PerceptualProfile::new();
        let key_slow_early = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        let key_fast_late = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Fast,
            RhythmDirection::Late,
        );

        profile.add_point(key_slow_early, MetricPoint::new(1000.0, 10.0), true);
        profile.add_point(key_fast_late, MetricPoint::new(2000.0, 20.0), true);

        let merged = profile
            .discipline_statistics(TrainingDiscipline::RhythmOffsetDetection)
            .unwrap();
        assert_eq!(merged.record_count(), 2);
        assert!((merged.welford.mean() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_merged_statistics_chronological_order() {
        use crate::types::{RhythmDirection, TempoRange};

        let mut profile = PerceptualProfile::new();
        let key1 = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        let key2 = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Fast,
            RhythmDirection::Late,
        );

        // Add later timestamp first to key1, earlier to key2
        profile.add_point(key1, MetricPoint::new(2000.0, 30.0), true);
        profile.add_point(key2, MetricPoint::new(1000.0, 10.0), true);

        let merged = profile
            .discipline_statistics(TrainingDiscipline::RhythmOffsetDetection)
            .unwrap();
        // Metrics should be sorted: [1000.0→10.0, 2000.0→30.0]
        assert_eq!(merged.metrics.len(), 2);
        assert!((merged.metrics[0].timestamp - 1000.0).abs() < 1e-10);
        assert!((merged.metrics[1].timestamp - 2000.0).abs() < 1e-10);
    }

    #[test]
    fn test_rebuild_all() {
        let mut profile = PerceptualProfile::new();
        let mut points = HashMap::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        points.insert(
            key,
            vec![
                MetricPoint::new(1000.0, 20.0),
                MetricPoint::new(2000.0, 30.0),
            ],
        );
        profile.rebuild_all(points);
        assert_eq!(
            profile.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
            2
        );
        assert_eq!(
            profile.record_count(TrainingDiscipline::IntervalPitchDiscrimination),
            0
        );
    }

    #[test]
    fn test_reset_all() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination);
        profile.add_point(key, MetricPoint::new(1000.0, 20.0), true);
        let key2 = StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchMatching);
        profile.add_point(key2, MetricPoint::new(2000.0, 10.0), true);
        profile.reset_all();
        for mode in TrainingDiscipline::ALL {
            assert!(!profile.has_data(mode));
        }
    }

    #[test]
    fn test_cold_start_difficulty_constant() {
        assert_eq!(COLD_START_DIFFICULTY, 100.0);
    }

    #[test]
    fn test_state_active_after_data() {
        let mut profile = PerceptualProfile::new();
        let key = StatisticsKey::Pitch(TrainingDiscipline::IntervalPitchMatching);
        profile.add_point(key, MetricPoint::new(1000.0, 5.0), true);
        assert_eq!(
            profile.state(TrainingDiscipline::IntervalPitchMatching),
            crate::training_discipline::TrainingDisciplineState::Active
        );
    }

    #[test]
    fn test_record_count_sums_across_keys() {
        use crate::types::{RhythmDirection, TempoRange};

        let mut profile = PerceptualProfile::new();
        let key1 = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Slow,
            RhythmDirection::Early,
        );
        let key2 = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            TempoRange::Fast,
            RhythmDirection::Late,
        );

        profile.add_point(key1, MetricPoint::new(1000.0, 10.0), true);
        profile.add_point(key1, MetricPoint::new(2000.0, 20.0), true);
        profile.add_point(key2, MetricPoint::new(3000.0, 30.0), true);

        assert_eq!(
            profile.record_count(TrainingDiscipline::RhythmOffsetDetection),
            3
        );
    }
}
