use crate::metric_point::MetricPoint;
use crate::training_discipline::TrainingDisciplineConfig;
use crate::trend::Trend;
use crate::types::Cents;
use crate::welford::WelfordAccumulator;

/// Per-discipline statistical engine: Welford accumulator, EWMA, trend, and time-ordered metrics.
/// Mirrors iOS `TrainingDisciplineStatistics` — each `TrainingDiscipline` gets one instance.
#[derive(Clone, Debug, PartialEq)]
pub struct TrainingDisciplineStatistics {
    pub welford: WelfordAccumulator<Cents>,
    pub ewma: Option<f64>,
    pub trend: Option<Trend>,
    pub metrics: Vec<MetricPoint<Cents>>,
}

impl TrainingDisciplineStatistics {
    pub fn new() -> Self {
        Self {
            welford: WelfordAccumulator::new(),
            ewma: None,
            trend: None,
            metrics: Vec::new(),
        }
    }

    /// Record count (delegated to Welford accumulator).
    pub fn record_count(&self) -> usize {
        self.welford.count()
    }

    /// Incremental update: add a single metric point.
    /// Updates Welford, appends to metrics, recomputes EWMA and trend.
    pub fn add_point(&mut self, point: MetricPoint<Cents>, config: &TrainingDisciplineConfig) {
        self.welford.update(point.value);
        self.metrics.push(point);
        self.recompute_ewma(config);
        self.recompute_trend();
    }

    /// Batch rebuild from sorted metric points. Resets state first.
    pub fn rebuild(&mut self, points: Vec<MetricPoint<Cents>>, config: &TrainingDisciplineConfig) {
        self.welford.reset();
        self.metrics.clear();
        self.ewma = None;
        self.trend = None;

        for point in points {
            self.welford.update(point.value);
            self.metrics.push(point);
        }

        self.recompute_ewma(config);
        self.recompute_trend();
    }

    /// Reset to empty state.
    pub fn reset(&mut self) {
        self.welford.reset();
        self.ewma = None;
        self.trend = None;
        self.metrics.clear();
    }

    /// Recompute EWMA from session-bucketed metrics.
    /// Groups metrics by session gap, computes per-session means,
    /// then applies exponential decay between sessions.
    fn recompute_ewma(&mut self, config: &TrainingDisciplineConfig) {
        if self.metrics.is_empty() {
            self.ewma = None;
            return;
        }

        // Group metrics into session buckets (gap-based)
        let session_gap = config.session_gap_secs;
        let mut session_means: Vec<(f64, f64)> = Vec::new(); // (timestamp, mean)

        let mut session_sum = 0.0;
        let mut session_count = 0usize;
        let mut session_start = self.metrics[0].timestamp;

        for (i, point) in self.metrics.iter().enumerate() {
            if i > 0 && point.timestamp - self.metrics[i - 1].timestamp >= session_gap {
                // Close previous session
                if session_count > 0 {
                    session_means.push((session_start, session_sum / session_count as f64));
                }
                session_sum = 0.0;
                session_count = 0;
                session_start = point.timestamp;
            }
            session_sum += point.statistical_value();
            session_count += 1;
        }
        // Close final session
        if session_count > 0 {
            session_means.push((session_start, session_sum / session_count as f64));
        }

        if session_means.is_empty() {
            self.ewma = None;
            return;
        }

        let halflife = config.ewma_halflife_secs;
        let mut ewma = session_means[0].1;

        for i in 1..session_means.len() {
            let dt = session_means[i].0 - session_means[i - 1].0;
            let alpha = 1.0 - (-f64::ln(2.0) * dt / halflife).exp();
            ewma = alpha * session_means[i].1 + (1.0 - alpha) * ewma;
        }

        self.ewma = Some(ewma);
    }

    /// Recompute trend from latest metric vs population statistics and EWMA.
    /// Lower metric = better (closer to perfect pitch).
    fn recompute_trend(&mut self) {
        if self.welford.count() < 2 {
            self.trend = None;
            return;
        }

        let ewma = match self.ewma {
            Some(e) => e,
            None => {
                self.trend = None;
                return;
            }
        };

        let latest = match self.metrics.last() {
            Some(p) => p.statistical_value(),
            None => {
                self.trend = None;
                return;
            }
        };

        let running_stddev = self.welford.population_std_dev().unwrap_or(0.0);

        // Declining if latest > mean + stddev
        // Improving if latest < ewma
        // Stable otherwise
        self.trend = Some(if latest > self.welford.mean() + running_stddev {
            Trend::Declining
        } else if latest < ewma {
            Trend::Improving
        } else {
            Trend::Stable
        });
    }
}

impl Default for TrainingDisciplineStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> &'static TrainingDisciplineConfig {
        use crate::training_discipline::TrainingDiscipline;
        TrainingDiscipline::UnisonPitchDiscrimination.config()
    }

    #[test]
    fn test_new_is_empty() {
        let stats = TrainingDisciplineStatistics::new();
        assert_eq!(stats.record_count(), 0);
        assert_eq!(stats.ewma, None);
        assert_eq!(stats.trend, None);
        assert!(stats.metrics.is_empty());
    }

    #[test]
    fn test_add_point_updates_welford() {
        let mut stats = TrainingDisciplineStatistics::new();
        stats.add_point(MetricPoint::new(1000.0, Cents::new(20.0)), default_config());
        assert_eq!(stats.record_count(), 1);
        assert!((stats.welford.mean() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_add_point_computes_ewma() {
        let mut stats = TrainingDisciplineStatistics::new();
        stats.add_point(MetricPoint::new(1000.0, Cents::new(20.0)), default_config());
        assert_eq!(stats.ewma, Some(20.0));
    }

    #[test]
    fn test_two_sessions_ewma_between() {
        let mut stats = TrainingDisciplineStatistics::new();
        let config = default_config();
        // Two sessions separated by > session_gap (1800s)
        stats.add_point(MetricPoint::new(1000.0, Cents::new(30.0)), config);
        stats.add_point(MetricPoint::new(5000.0, Cents::new(10.0)), config);
        let ewma = stats.ewma.unwrap();
        assert!(
            ewma > 10.0 && ewma < 30.0,
            "EWMA {} should be between 10 and 30",
            ewma
        );
    }

    #[test]
    fn test_trend_none_with_one_record() {
        let mut stats = TrainingDisciplineStatistics::new();
        stats.add_point(MetricPoint::new(1000.0, Cents::new(20.0)), default_config());
        assert_eq!(stats.trend, None);
    }

    #[test]
    fn test_trend_improving() {
        let mut stats = TrainingDisciplineStatistics::new();
        let config = default_config();
        // Many high values followed by low values
        for i in 0..10 {
            stats.add_point(
                MetricPoint::new(i as f64 * 4000.0, Cents::new(50.0)),
                config,
            );
        }
        for i in 10..20 {
            stats.add_point(
                MetricPoint::new(i as f64 * 4000.0, Cents::new(10.0)),
                config,
            );
        }
        assert_eq!(stats.trend, Some(Trend::Improving));
    }

    #[test]
    fn test_rebuild_replaces_state() {
        let mut stats = TrainingDisciplineStatistics::new();
        let config = default_config();
        stats.add_point(MetricPoint::new(1000.0, Cents::new(99.0)), config);

        let points = vec![
            MetricPoint::new(2000.0, Cents::new(10.0)),
            MetricPoint::new(3000.0, Cents::new(20.0)),
        ];
        stats.rebuild(points, config);
        assert_eq!(stats.record_count(), 2);
        assert!((stats.welford.mean() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut stats = TrainingDisciplineStatistics::new();
        stats.add_point(MetricPoint::new(1000.0, Cents::new(20.0)), default_config());
        stats.reset();
        assert_eq!(stats.record_count(), 0);
        assert_eq!(stats.ewma, None);
        assert_eq!(stats.trend, None);
        assert!(stats.metrics.is_empty());
    }
}
