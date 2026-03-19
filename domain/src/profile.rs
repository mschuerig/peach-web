use std::collections::HashMap;

use crate::metric_point::MetricPoint;
use crate::training_mode::TrainingMode;
use crate::training_mode_statistics::TrainingModeStatistics;
use crate::trend::Trend;
use crate::types::Cents;

/// Cold-start difficulty for untrained modes (cents).
pub const COLD_START_DIFFICULTY: f64 = 100.0;

/// Mode-aware perceptual profile — single source of truth for all per-mode statistics.
///
/// Each `TrainingMode` gets its own `TrainingModeStatistics` (Welford accumulator,
/// EWMA, trend, time-ordered metrics). This aligns with the iOS `PerceptualProfile`
/// post-Epic-44 architecture.
#[derive(Clone, Debug)]
pub struct PerceptualProfile {
    modes: HashMap<TrainingMode, TrainingModeStatistics>,
}

impl PerceptualProfile {
    pub fn new() -> Self {
        let mut modes = HashMap::new();
        for mode in TrainingMode::ALL {
            modes.insert(mode, TrainingModeStatistics::new());
        }
        Self { modes }
    }

    // --- Per-mode query API ---

    /// Direct access to a mode's statistics.
    pub fn statistics(&self, mode: TrainingMode) -> &TrainingModeStatistics {
        self.modes.get(&mode).expect("all modes initialized")
    }

    /// Whether a mode has any recorded data.
    pub fn has_data(&self, mode: TrainingMode) -> bool {
        self.statistics(mode).record_count() > 0
    }

    /// Training mode state (NoData or Active).
    pub fn state(&self, mode: TrainingMode) -> crate::training_mode::TrainingModeState {
        if self.has_data(mode) {
            crate::training_mode::TrainingModeState::Active
        } else {
            crate::training_mode::TrainingModeState::NoData
        }
    }

    /// Trend for a mode (None if < 2 records).
    pub fn trend(&self, mode: TrainingMode) -> Option<Trend> {
        self.statistics(mode).trend
    }

    /// Current EWMA for a mode.
    pub fn current_ewma(&self, mode: TrainingMode) -> Option<f64> {
        self.statistics(mode).ewma
    }

    /// Record count for a mode.
    pub fn record_count(&self, mode: TrainingMode) -> usize {
        self.statistics(mode).record_count()
    }

    // --- Strategy-facing API (replaces old per-note aggregate) ---

    /// Comparison mean for the given interval.
    /// Maps interval to the correct mode (unison if interval == 0, otherwise interval mode).
    /// Used by `KazezNoteStrategy` as warm-start difficulty fallback.
    pub fn comparison_mean(&self, interval: u8) -> Option<Cents> {
        let mode = if interval == 0 {
            TrainingMode::UnisonPitchComparison
        } else {
            TrainingMode::IntervalPitchComparison
        };
        let stats = self.statistics(mode);
        if stats.record_count() > 0 {
            Some(Cents::new(stats.welford.mean()))
        } else {
            None
        }
    }

    /// Weighted matching mean across both matching modes (backward compat for UI).
    pub fn matching_mean(&self) -> Option<Cents> {
        let unison = self.statistics(TrainingMode::UnisonMatching);
        let interval = self.statistics(TrainingMode::IntervalMatching);
        let u_count = unison.record_count();
        let i_count = interval.record_count();
        let total = u_count + i_count;
        if total == 0 {
            return None;
        }
        let sum = unison.welford.mean() * u_count as f64 + interval.welford.mean() * i_count as f64;
        Some(Cents::new(sum / total as f64))
    }

    /// Weighted matching std dev across both matching modes.
    pub fn matching_std_dev(&self) -> Option<Cents> {
        let unison = self.statistics(TrainingMode::UnisonMatching);
        let interval = self.statistics(TrainingMode::IntervalMatching);
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
        // Both modes have data — use combined population std dev
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

    /// Total matching sample count across both modes.
    pub fn matching_sample_count(&self) -> usize {
        self.record_count(TrainingMode::UnisonMatching)
            + self.record_count(TrainingMode::IntervalMatching)
    }

    // --- Incremental update ---

    /// Add a single metric point for the given mode.
    /// For comparison modes, `is_correct` filters: only correct answers contribute.
    /// For matching modes, all answers contribute (pass `true`).
    pub fn add_point(&mut self, mode: TrainingMode, point: MetricPoint<Cents>, is_correct: bool) {
        if !is_correct {
            return;
        }
        let config = mode.config();
        let stats = self.modes.get_mut(&mode).expect("all modes initialized");
        stats.add_point(point, config);
    }

    // --- Batch operations ---

    /// Rebuild all modes from pre-sorted metric points.
    pub fn rebuild_all(&mut self, points: HashMap<TrainingMode, Vec<MetricPoint<Cents>>>) {
        for mode in TrainingMode::ALL {
            let stats = self.modes.get_mut(&mode).expect("all modes initialized");
            if let Some(mode_points) = points.get(&mode) {
                stats.rebuild(mode_points.clone(), mode.config());
            } else {
                stats.reset();
            }
        }
    }

    /// Reset all modes to empty state.
    pub fn reset_all(&mut self) {
        for stats in self.modes.values_mut() {
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
        for mode in TrainingMode::ALL {
            assert_eq!(
                profile.state(mode),
                crate::training_mode::TrainingModeState::NoData
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
            TrainingMode::UnisonPitchComparison,
            MetricPoint::new(1000.0, Cents::new(20.0)),
            true,
        );
        assert!(profile.has_data(TrainingMode::UnisonPitchComparison));
        assert!(!profile.has_data(TrainingMode::IntervalPitchComparison));
        assert!(!profile.has_data(TrainingMode::UnisonMatching));
    }

    #[test]
    fn test_add_point_filters_incorrect() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingMode::UnisonPitchComparison,
            MetricPoint::new(1000.0, Cents::new(20.0)),
            false, // incorrect — should be filtered
        );
        assert!(!profile.has_data(TrainingMode::UnisonPitchComparison));
    }

    #[test]
    fn test_comparison_mean_unison() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingMode::UnisonPitchComparison,
            MetricPoint::new(1000.0, Cents::new(40.0)),
            true,
        );
        profile.add_point(
            TrainingMode::UnisonPitchComparison,
            MetricPoint::new(2000.0, Cents::new(60.0)),
            true,
        );
        let mean = profile.comparison_mean(0).unwrap();
        assert!((mean.raw_value - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_comparison_mean_interval() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingMode::IntervalPitchComparison,
            MetricPoint::new(1000.0, Cents::new(30.0)),
            true,
        );
        let mean = profile.comparison_mean(7).unwrap();
        assert!((mean.raw_value - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_comparison_mean_no_data() {
        let profile = PerceptualProfile::new();
        assert_eq!(profile.comparison_mean(0), None);
    }

    #[test]
    fn test_matching_mean_single_mode() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingMode::UnisonMatching,
            MetricPoint::new(1000.0, Cents::new(5.0)),
            true,
        );
        profile.add_point(
            TrainingMode::UnisonMatching,
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
            TrainingMode::UnisonMatching,
            MetricPoint::new(1000.0, Cents::new(10.0)),
            true,
        );
        // Interval: 1 sample of 20
        profile.add_point(
            TrainingMode::IntervalMatching,
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
            TrainingMode::UnisonMatching,
            MetricPoint::new(1000.0, Cents::new(5.0)),
            true,
        );
        profile.add_point(
            TrainingMode::IntervalMatching,
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
            TrainingMode::UnisonPitchComparison,
            vec![
                MetricPoint::new(1000.0, Cents::new(20.0)),
                MetricPoint::new(2000.0, Cents::new(30.0)),
            ],
        );
        profile.rebuild_all(points);
        assert_eq!(profile.record_count(TrainingMode::UnisonPitchComparison), 2);
        assert_eq!(
            profile.record_count(TrainingMode::IntervalPitchComparison),
            0
        );
    }

    #[test]
    fn test_reset_all() {
        let mut profile = PerceptualProfile::new();
        profile.add_point(
            TrainingMode::UnisonPitchComparison,
            MetricPoint::new(1000.0, Cents::new(20.0)),
            true,
        );
        profile.add_point(
            TrainingMode::UnisonMatching,
            MetricPoint::new(2000.0, Cents::new(10.0)),
            true,
        );
        profile.reset_all();
        for mode in TrainingMode::ALL {
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
            TrainingMode::IntervalMatching,
            MetricPoint::new(1000.0, Cents::new(5.0)),
            true,
        );
        assert_eq!(
            profile.state(TrainingMode::IntervalMatching),
            crate::training_mode::TrainingModeState::Active
        );
    }
}
