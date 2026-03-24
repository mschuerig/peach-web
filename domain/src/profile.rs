use std::collections::HashMap;

use crate::metric_point::MetricPoint;
use crate::training_discipline::TrainingDiscipline;
use crate::training_discipline_statistics::TrainingDisciplineStatistics;
use crate::trend::Trend;
use crate::types::Cents;

/// Cold-start difficulty for untrained disciplines (cents).
pub const COLD_START_DIFFICULTY: f64 = 100.0;

/// Discipline-aware perceptual profile — single source of truth for all per-discipline statistics.
///
/// Each `TrainingDiscipline` gets its own `TrainingDisciplineStatistics` (Welford accumulator,
/// EWMA, trend, time-ordered metrics). This aligns with the iOS `PerceptualProfile`
/// post-Epic-44 architecture.
#[derive(Clone, Debug)]
pub struct PerceptualProfile {
    disciplines: HashMap<TrainingDiscipline, TrainingDisciplineStatistics>,
}

impl PerceptualProfile {
    pub fn new() -> Self {
        let mut disciplines = HashMap::new();
        for discipline in TrainingDiscipline::ALL {
            disciplines.insert(discipline, TrainingDisciplineStatistics::new());
        }
        Self { disciplines }
    }

    // --- Per-discipline query API ---

    /// Direct access to a discipline's statistics.
    pub fn statistics(&self, discipline: TrainingDiscipline) -> &TrainingDisciplineStatistics {
        self.disciplines
            .get(&discipline)
            .expect("all disciplines initialized")
    }

    /// Whether a discipline has any recorded data.
    pub fn has_data(&self, discipline: TrainingDiscipline) -> bool {
        self.statistics(discipline).record_count() > 0
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
    pub fn trend(&self, discipline: TrainingDiscipline) -> Option<Trend> {
        self.statistics(discipline).trend
    }

    /// Current EWMA for a discipline.
    pub fn current_ewma(&self, discipline: TrainingDiscipline) -> Option<f64> {
        self.statistics(discipline).ewma
    }

    /// Record count for a discipline.
    pub fn record_count(&self, discipline: TrainingDiscipline) -> usize {
        self.statistics(discipline).record_count()
    }

    // --- Strategy-facing API (replaces old per-note aggregate) ---

    /// Discrimination mean for the given interval.
    /// Maps interval to the correct mode (unison if interval == 0, otherwise interval mode).
    /// Used by `KazezNoteStrategy` as warm-start difficulty fallback.
    pub fn discrimination_mean(&self, interval: u8) -> Option<Cents> {
        let mode = if interval == 0 {
            TrainingDiscipline::UnisonPitchDiscrimination
        } else {
            TrainingDiscipline::IntervalPitchDiscrimination
        };
        let stats = self.statistics(mode);
        if stats.record_count() > 0 {
            Some(Cents::new(stats.welford.mean()))
        } else {
            None
        }
    }

    /// Weighted matching mean across both matching disciplines (backward compat for UI).
    pub fn matching_mean(&self) -> Option<Cents> {
        let unison = self.statistics(TrainingDiscipline::UnisonPitchMatching);
        let interval = self.statistics(TrainingDiscipline::IntervalPitchMatching);
        let u_count = unison.record_count();
        let i_count = interval.record_count();
        let total = u_count + i_count;
        if total == 0 {
            return None;
        }
        let sum = unison.welford.mean() * u_count as f64 + interval.welford.mean() * i_count as f64;
        Some(Cents::new(sum / total as f64))
    }

    /// Weighted matching std dev across both matching disciplines.
    pub fn matching_std_dev(&self) -> Option<Cents> {
        let unison = self.statistics(TrainingDiscipline::UnisonPitchMatching);
        let interval = self.statistics(TrainingDiscipline::IntervalPitchMatching);
        let u_count = unison.record_count();
        let i_count = interval.record_count();
        let total = u_count + i_count;
        if total < 2 {
            return None;
        }
        // For a single mode with data, return its std dev directly
        if u_count == 0 {
            return interval.welford.typed_std_dev();
        }
        if i_count == 0 {
            return unison.welford.typed_std_dev();
        }
        // Both disciplines have data — use combined population std dev
        let combined_mean = (unison.welford.mean() * u_count as f64
            + interval.welford.mean() * i_count as f64)
            / total as f64;
        let u_var = unison.welford.population_std_dev().unwrap_or(0.0).powi(2);
        let i_var = interval.welford.population_std_dev().unwrap_or(0.0).powi(2);
        let u_shift = (unison.welford.mean() - combined_mean).powi(2);
        let i_shift = (interval.welford.mean() - combined_mean).powi(2);
        let combined_var = (u_count as f64 * (u_var + u_shift)
            + i_count as f64 * (i_var + i_shift))
            / total as f64;
        Some(Cents::new(combined_var.sqrt()))
    }

    /// Total matching sample count across both disciplines.
    pub fn matching_sample_count(&self) -> usize {
        self.record_count(TrainingDiscipline::UnisonPitchMatching)
            + self.record_count(TrainingDiscipline::IntervalPitchMatching)
    }

    // --- Incremental update ---

    /// Add a single metric point for the given discipline.
    /// For discrimination disciplines, `is_correct` filters: only correct answers contribute.
    /// For matching disciplines, all answers contribute (pass `true`).
    pub fn add_point(
        &mut self,
        discipline: TrainingDiscipline,
        point: MetricPoint<Cents>,
        is_correct: bool,
    ) {
        if !is_correct {
            return;
        }
        let config = discipline.config();
        let stats = self
            .disciplines
            .get_mut(&discipline)
            .expect("all disciplines initialized");
        stats.add_point(point, config);
    }

    // --- Batch operations ---

    /// Rebuild all disciplines from pre-sorted metric points.
    pub fn rebuild_all(&mut self, points: HashMap<TrainingDiscipline, Vec<MetricPoint<Cents>>>) {
        for discipline in TrainingDiscipline::ALL {
            let stats = self
                .disciplines
                .get_mut(&discipline)
                .expect("all disciplines initialized");
            if let Some(discipline_points) = points.get(&discipline) {
                stats.rebuild(discipline_points.clone(), discipline.config());
            } else {
                stats.reset();
            }
        }
    }

    /// Reset all disciplines to empty state.
    pub fn reset_all(&mut self) {
        for stats in self.disciplines.values_mut() {
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
    fn test_add_point_updates_correct_mode() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(1000.0, Cents::new(20.0)),
            true,
        );
        assert!(profile.has_data(TrainingDiscipline::UnisonPitchDiscrimination));
        assert!(!profile.has_data(TrainingDiscipline::IntervalPitchDiscrimination));
        assert!(!profile.has_data(TrainingDiscipline::UnisonPitchMatching));
    }

    #[test]
    fn test_add_point_filters_incorrect() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(1000.0, Cents::new(20.0)),
            false, // incorrect — should be filtered
        );
        assert!(!profile.has_data(TrainingDiscipline::UnisonPitchDiscrimination));
    }

    #[test]
    fn test_discrimination_mean_unison() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(1000.0, Cents::new(40.0)),
            true,
        );
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(2000.0, Cents::new(60.0)),
            true,
        );
        let mean = profile.discrimination_mean(0).unwrap();
        assert!((mean.raw_value - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_discrimination_mean_interval() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingDiscipline::IntervalPitchDiscrimination,
            MetricPoint::new(1000.0, Cents::new(30.0)),
            true,
        );
        let mean = profile.discrimination_mean(7).unwrap();
        assert!((mean.raw_value - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_discrimination_mean_no_data() {
        let profile = PerceptualProfile::new();
        assert_eq!(profile.discrimination_mean(0), None);
    }

    #[test]
    fn test_matching_mean_single_mode() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingDiscipline::UnisonPitchMatching,
            MetricPoint::new(1000.0, Cents::new(5.0)),
            true,
        );
        profile.add_point(
            TrainingDiscipline::UnisonPitchMatching,
            MetricPoint::new(2000.0, Cents::new(15.0)),
            true,
        );
        let mean = profile.matching_mean().unwrap();
        assert!((mean.raw_value - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_matching_mean_both_modes() {
        let mut profile = PerceptualProfile::new();
        // Unison: 1 sample of 10
        profile.add_point(
            TrainingDiscipline::UnisonPitchMatching,
            MetricPoint::new(1000.0, Cents::new(10.0)),
            true,
        );
        // Interval: 1 sample of 20
        profile.add_point(
            TrainingDiscipline::IntervalPitchMatching,
            MetricPoint::new(2000.0, Cents::new(20.0)),
            true,
        );
        let mean = profile.matching_mean().unwrap();
        // Weighted: (10 * 1 + 20 * 1) / 2 = 15
        assert!((mean.raw_value - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_matching_sample_count() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingDiscipline::UnisonPitchMatching,
            MetricPoint::new(1000.0, Cents::new(5.0)),
            true,
        );
        profile.add_point(
            TrainingDiscipline::IntervalPitchMatching,
            MetricPoint::new(2000.0, Cents::new(10.0)),
            true,
        );
        assert_eq!(profile.matching_sample_count(), 2);
    }

    #[test]
    fn test_rebuild_all() {
        let mut profile = PerceptualProfile::new();
        let mut points = HashMap::new();
        points.insert(
            TrainingDiscipline::UnisonPitchDiscrimination,
            vec![
                MetricPoint::new(1000.0, Cents::new(20.0)),
                MetricPoint::new(2000.0, Cents::new(30.0)),
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
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(1000.0, Cents::new(20.0)),
            true,
        );
        profile.add_point(
            TrainingDiscipline::UnisonPitchMatching,
            MetricPoint::new(2000.0, Cents::new(10.0)),
            true,
        );
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
        profile.add_point(
            TrainingDiscipline::IntervalPitchMatching,
            MetricPoint::new(1000.0, Cents::new(5.0)),
            true,
        );
        assert_eq!(
            profile.state(TrainingDiscipline::IntervalPitchMatching),
            crate::training_discipline::TrainingDisciplineState::Active
        );
    }
}
