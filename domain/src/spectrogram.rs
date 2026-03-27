use crate::progress_timeline::TimeBucket;
use crate::types::TempoRange;

/// Accuracy classification for a spectrogram cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectrogramAccuracyLevel {
    /// Within precise threshold.
    Precise,
    /// Between precise and moderate thresholds.
    Moderate,
    /// Beyond moderate threshold.
    Erratic,
}

/// Hybrid floor/ceiling threshold model matching iOS `SpectrogramThresholds`.
///
/// Thresholds are computed as a percentage of 16th-note duration at the tempo range
/// midpoint, then clamped to absolute ms bounds. This ensures thresholds are
/// musically meaningful at all tempos while staying within perceptible bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrogramThresholds {
    /// Base percentage for "precise" level (e.g. 0.08 = 8%).
    pub precise_base_percent: f64,
    /// Absolute floor in ms for precise threshold.
    pub precise_floor_ms: f64,
    /// Absolute ceiling in ms for precise threshold.
    pub precise_ceiling_ms: f64,
    /// Base percentage for "moderate" level (e.g. 0.20 = 20%).
    pub moderate_base_percent: f64,
    /// Absolute floor in ms for moderate threshold.
    pub moderate_floor_ms: f64,
    /// Absolute ceiling in ms for moderate threshold.
    pub moderate_ceiling_ms: f64,
}

impl Default for SpectrogramThresholds {
    /// iOS defaults: precise = 8% (12-30ms), moderate = 20% (25-50ms).
    fn default() -> Self {
        Self {
            precise_base_percent: 0.08,
            precise_floor_ms: 12.0,
            precise_ceiling_ms: 30.0,
            moderate_base_percent: 0.20,
            moderate_floor_ms: 25.0,
            moderate_ceiling_ms: 50.0,
        }
    }
}

impl SpectrogramThresholds {
    /// Compute the precise threshold in ms for a given tempo range.
    pub fn precise_threshold_ms(&self, range: TempoRange) -> f64 {
        let base_ms = range.sixteenth_note_ms() * self.precise_base_percent;
        base_ms.clamp(self.precise_floor_ms, self.precise_ceiling_ms)
    }

    /// Compute the moderate threshold in ms for a given tempo range.
    pub fn moderate_threshold_ms(&self, range: TempoRange) -> f64 {
        let base_ms = range.sixteenth_note_ms() * self.moderate_base_percent;
        base_ms.clamp(self.moderate_floor_ms, self.moderate_ceiling_ms)
    }

    /// Classify an accuracy value (in ms) for a given tempo range.
    pub fn accuracy_level(&self, accuracy_ms: f64, range: TempoRange) -> SpectrogramAccuracyLevel {
        if accuracy_ms <= self.precise_threshold_ms(range) {
            SpectrogramAccuracyLevel::Precise
        } else if accuracy_ms <= self.moderate_threshold_ms(range) {
            SpectrogramAccuracyLevel::Moderate
        } else {
            SpectrogramAccuracyLevel::Erratic
        }
    }

    /// Classify a percentage-of-16th-note value for a given tempo range.
    /// Converts the percentage back to ms first, then classifies.
    pub fn accuracy_level_from_percent(
        &self,
        percent_of_16th: f64,
        range: TempoRange,
    ) -> SpectrogramAccuracyLevel {
        let ms = percent_of_16th / 100.0 * range.sixteenth_note_ms();
        self.accuracy_level(ms, range)
    }
}

/// Statistics for one direction (early or late) within a spectrogram cell.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrogramCellStats {
    /// Mean accuracy as percentage of 16th note.
    pub mean_percent: f64,
    /// Number of data points.
    pub count: usize,
}

/// A single cell in the spectrogram grid: one TempoRange x one TimeBucket.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrogramCell {
    /// Combined mean accuracy as percentage of 16th note (all directions).
    pub mean_accuracy_percent: Option<f64>,
    /// Early-direction statistics.
    pub early_stats: Option<SpectrogramCellStats>,
    /// Late-direction statistics.
    pub late_stats: Option<SpectrogramCellStats>,
    /// Total record count across all directions.
    pub record_count: usize,
}

impl SpectrogramCell {
    /// An empty cell with no data.
    pub fn empty() -> Self {
        Self {
            mean_accuracy_percent: None,
            early_stats: None,
            late_stats: None,
            record_count: 0,
        }
    }

    /// Whether this cell has any data.
    pub fn has_data(&self) -> bool {
        self.mean_accuracy_percent.is_some()
    }
}

/// A column in the spectrogram: one TimeBucket with cells for each trained TempoRange.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrogramColumn {
    /// The time bucket this column represents.
    pub bucket: TimeBucket,
    /// Cells indexed by position in the trained_ranges vec (0 = bottom row).
    pub cells: Vec<SpectrogramCell>,
}

/// The complete spectrogram grid for a rhythm discipline.
#[derive(Clone, Debug)]
pub struct SpectrogramData {
    /// Only tempo ranges that have training data, in order (Slow, Medium, Fast).
    pub trained_ranges: Vec<TempoRange>,
    /// Columns (one per time bucket), each containing cells for trained ranges.
    pub columns: Vec<SpectrogramColumn>,
    /// Thresholds used for accuracy classification.
    pub thresholds: SpectrogramThresholds,
}

impl SpectrogramData {
    /// Compute the spectrogram grid from time buckets and per-key metric data.
    ///
    /// # Arguments
    /// * `buckets` - Time buckets from `ProgressTimeline::display_buckets()`
    /// * `key_metrics` - Per-StatisticsKey metric points: `(TempoRange, RhythmDirection, Vec<MetricPoint>)`
    ///   where MetricPoint values are percentage-of-16th-note accuracy.
    /// * `thresholds` - Threshold configuration for accuracy classification.
    pub fn compute(
        buckets: &[TimeBucket],
        key_metrics: &[(
            TempoRange,
            crate::types::RhythmDirection,
            Vec<crate::MetricPoint>,
        )],
        thresholds: SpectrogramThresholds,
    ) -> Self {
        // Determine which tempo ranges have data.
        let mut has_data = [false; 3]; // indexed by TempoRange::ALL position
        for (range, _, metrics) in key_metrics {
            if !metrics.is_empty() {
                let idx = TempoRange::ALL.iter().position(|r| r == range).unwrap();
                has_data[idx] = true;
            }
        }
        let trained_ranges: Vec<TempoRange> = TempoRange::ALL
            .iter()
            .enumerate()
            .filter(|(i, _)| has_data[*i])
            .map(|(_, &r)| r)
            .collect();

        if trained_ranges.is_empty() || buckets.is_empty() {
            return Self {
                trained_ranges,
                columns: Vec::new(),
                thresholds,
            };
        }

        // Build columns: one per bucket.
        let columns = buckets
            .iter()
            .map(|bucket| {
                let cells = trained_ranges
                    .iter()
                    .map(|&range| Self::compute_cell(range, bucket, key_metrics))
                    .collect();
                SpectrogramColumn {
                    bucket: bucket.clone(),
                    cells,
                }
            })
            .collect();

        Self {
            trained_ranges,
            columns,
            thresholds,
        }
    }

    /// Compute a single cell for one (TempoRange, TimeBucket) pair.
    fn compute_cell(
        range: TempoRange,
        bucket: &TimeBucket,
        key_metrics: &[(
            TempoRange,
            crate::types::RhythmDirection,
            Vec<crate::MetricPoint>,
        )],
    ) -> SpectrogramCell {
        use crate::types::RhythmDirection;

        let mut all_values: Vec<f64> = Vec::new();
        let mut early_values: Vec<f64> = Vec::new();
        let mut late_values: Vec<f64> = Vec::new();

        for (r, dir, metrics) in key_metrics {
            if *r != range {
                continue;
            }
            // Filter metrics that fall within this bucket's time range.
            for m in metrics {
                if m.timestamp >= bucket.period_start && m.timestamp < bucket.period_end {
                    all_values.push(m.value);
                    match dir {
                        RhythmDirection::Early => early_values.push(m.value),
                        RhythmDirection::Late => late_values.push(m.value),
                        RhythmDirection::OnBeat => {
                            // OnBeat contributes to overall mean but not early/late breakdown.
                        }
                    }
                }
            }
        }

        if all_values.is_empty() {
            return SpectrogramCell::empty();
        }

        let mean = all_values.iter().sum::<f64>() / all_values.len() as f64;

        let early_stats = if early_values.is_empty() {
            None
        } else {
            Some(SpectrogramCellStats {
                mean_percent: early_values.iter().sum::<f64>() / early_values.len() as f64,
                count: early_values.len(),
            })
        };

        let late_stats = if late_values.is_empty() {
            None
        } else {
            Some(SpectrogramCellStats {
                mean_percent: late_values.iter().sum::<f64>() / late_values.len() as f64,
                count: late_values.len(),
            })
        };

        SpectrogramCell {
            mean_accuracy_percent: Some(mean),
            early_stats,
            late_stats,
            record_count: all_values.len(),
        }
    }

    /// Classify a cell's accuracy level for its tempo range.
    pub fn accuracy_level(
        &self,
        cell: &SpectrogramCell,
        range: TempoRange,
    ) -> Option<SpectrogramAccuracyLevel> {
        cell.mean_accuracy_percent
            .map(|pct| self.thresholds.accuracy_level_from_percent(pct, range))
    }

    /// Whether the spectrogram has any data to display.
    pub fn is_empty(&self) -> bool {
        self.trained_ranges.is_empty() || self.columns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MetricPoint;
    use crate::types::RhythmDirection;

    fn make_metric(ts: f64, value: f64) -> MetricPoint {
        MetricPoint::new(ts, value)
    }

    fn make_bucket(start: f64, end: f64, size: crate::BucketSize) -> TimeBucket {
        TimeBucket {
            period_start: start,
            period_end: end,
            bucket_size: size,
            mean: 5.0,
            stddev: 1.0,
            record_count: 1,
        }
    }

    // --- SpectrogramThresholds tests ---

    #[test]
    fn test_default_thresholds() {
        let t = SpectrogramThresholds::default();
        assert_eq!(t.precise_base_percent, 0.08);
        assert_eq!(t.precise_floor_ms, 12.0);
        assert_eq!(t.precise_ceiling_ms, 30.0);
        assert_eq!(t.moderate_base_percent, 0.20);
        assert_eq!(t.moderate_floor_ms, 25.0);
        assert_eq!(t.moderate_ceiling_ms, 50.0);
    }

    #[test]
    fn test_precise_threshold_slow_tempo() {
        // Slow midpoint = 60 BPM. 16th note = 60000/60/4 = 250ms.
        // 8% of 250 = 20ms, within 12-30ms clamp → 20ms.
        let t = SpectrogramThresholds::default();
        let ms = t.precise_threshold_ms(TempoRange::Slow);
        assert!((ms - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_precise_threshold_medium_tempo() {
        // Medium midpoint = 100 BPM. 16th note = 60000/100/4 = 150ms.
        // 8% of 150 = 12ms, at floor → 12ms.
        let t = SpectrogramThresholds::default();
        let ms = t.precise_threshold_ms(TempoRange::Medium);
        assert!((ms - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_precise_threshold_fast_tempo() {
        // Fast midpoint = 160 BPM. 16th note = 60000/160/4 = 93.75ms.
        // 8% of 93.75 = 7.5ms, below floor 12ms → clamped to 12ms.
        let t = SpectrogramThresholds::default();
        let ms = t.precise_threshold_ms(TempoRange::Fast);
        assert!((ms - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_moderate_threshold_slow_tempo() {
        // 20% of 250ms = 50ms, at ceiling → 50ms.
        let t = SpectrogramThresholds::default();
        let ms = t.moderate_threshold_ms(TempoRange::Slow);
        assert!((ms - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_moderate_threshold_medium_tempo() {
        // 20% of 150ms = 30ms, within 25-50ms → 30ms.
        let t = SpectrogramThresholds::default();
        let ms = t.moderate_threshold_ms(TempoRange::Medium);
        assert!((ms - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_moderate_threshold_fast_tempo() {
        // 20% of 93.75ms = 18.75ms, below floor 25ms → clamped to 25ms.
        let t = SpectrogramThresholds::default();
        let ms = t.moderate_threshold_ms(TempoRange::Fast);
        assert!((ms - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_level_precise() {
        let t = SpectrogramThresholds::default();
        // 10ms at Slow (threshold=20ms) → precise
        assert_eq!(
            t.accuracy_level(10.0, TempoRange::Slow),
            SpectrogramAccuracyLevel::Precise
        );
    }

    #[test]
    fn test_accuracy_level_moderate() {
        let t = SpectrogramThresholds::default();
        // 35ms at Slow (precise=20ms, moderate=50ms) → moderate
        assert_eq!(
            t.accuracy_level(35.0, TempoRange::Slow),
            SpectrogramAccuracyLevel::Moderate
        );
    }

    #[test]
    fn test_accuracy_level_erratic() {
        let t = SpectrogramThresholds::default();
        // 60ms at Slow (moderate=50ms) → erratic
        assert_eq!(
            t.accuracy_level(60.0, TempoRange::Slow),
            SpectrogramAccuracyLevel::Erratic
        );
    }

    #[test]
    fn test_accuracy_level_from_percent() {
        let t = SpectrogramThresholds::default();
        // Slow: 16th note = 250ms. 4% of 250 = 10ms → precise (threshold=20ms)
        assert_eq!(
            t.accuracy_level_from_percent(4.0, TempoRange::Slow),
            SpectrogramAccuracyLevel::Precise
        );
        // 12% of 250 = 30ms → moderate (precise=20ms, moderate=50ms)
        assert_eq!(
            t.accuracy_level_from_percent(12.0, TempoRange::Slow),
            SpectrogramAccuracyLevel::Moderate
        );
        // 25% of 250 = 62.5ms → erratic (moderate=50ms)
        assert_eq!(
            t.accuracy_level_from_percent(25.0, TempoRange::Slow),
            SpectrogramAccuracyLevel::Erratic
        );
    }

    // --- SpectrogramData::compute tests ---

    #[test]
    fn test_compute_empty_data() {
        let data = SpectrogramData::compute(&[], &[], SpectrogramThresholds::default());
        assert!(data.is_empty());
        assert!(data.trained_ranges.is_empty());
    }

    #[test]
    fn test_compute_single_range_single_bucket() {
        let buckets = vec![make_bucket(0.0, 100.0, crate::BucketSize::Session)];
        let key_metrics = vec![
            (
                TempoRange::Slow,
                RhythmDirection::Early,
                vec![make_metric(50.0, 5.0)],
            ),
            (
                TempoRange::Slow,
                RhythmDirection::Late,
                vec![make_metric(60.0, 10.0)],
            ),
            (TempoRange::Slow, RhythmDirection::OnBeat, vec![]),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        assert_eq!(data.trained_ranges, vec![TempoRange::Slow]);
        assert_eq!(data.columns.len(), 1);
        assert_eq!(data.columns[0].cells.len(), 1);

        let cell = &data.columns[0].cells[0];
        assert!(cell.has_data());
        // mean of [5.0, 10.0] = 7.5
        assert!((cell.mean_accuracy_percent.unwrap() - 7.5).abs() < 0.01);
        assert_eq!(cell.early_stats.as_ref().unwrap().count, 1);
        assert!((cell.early_stats.as_ref().unwrap().mean_percent - 5.0).abs() < 0.01);
        assert_eq!(cell.late_stats.as_ref().unwrap().count, 1);
        assert!((cell.late_stats.as_ref().unwrap().mean_percent - 10.0).abs() < 0.01);
        assert_eq!(cell.record_count, 2);
    }

    #[test]
    fn test_compute_multi_range() {
        let buckets = vec![make_bucket(0.0, 100.0, crate::BucketSize::Session)];
        let key_metrics = vec![
            (
                TempoRange::Slow,
                RhythmDirection::Early,
                vec![make_metric(50.0, 5.0)],
            ),
            (TempoRange::Slow, RhythmDirection::Late, vec![]),
            (TempoRange::Slow, RhythmDirection::OnBeat, vec![]),
            (
                TempoRange::Fast,
                RhythmDirection::Early,
                vec![make_metric(50.0, 15.0)],
            ),
            (TempoRange::Fast, RhythmDirection::Late, vec![]),
            (TempoRange::Fast, RhythmDirection::OnBeat, vec![]),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        assert_eq!(
            data.trained_ranges,
            vec![TempoRange::Slow, TempoRange::Fast]
        );
        assert_eq!(data.columns[0].cells.len(), 2);

        // Slow cell
        assert!(data.columns[0].cells[0].has_data());
        // Fast cell
        assert!(data.columns[0].cells[1].has_data());
    }

    #[test]
    fn test_compute_only_trained_ranges_appear() {
        // Only Medium has data — Slow and Fast should be excluded.
        let buckets = vec![make_bucket(0.0, 100.0, crate::BucketSize::Day)];
        let key_metrics = vec![
            (
                TempoRange::Medium,
                RhythmDirection::Early,
                vec![make_metric(50.0, 8.0)],
            ),
            (TempoRange::Medium, RhythmDirection::Late, vec![]),
            (TempoRange::Medium, RhythmDirection::OnBeat, vec![]),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        assert_eq!(data.trained_ranges, vec![TempoRange::Medium]);
    }

    #[test]
    fn test_compute_metrics_filtered_by_bucket_time() {
        let buckets = vec![
            make_bucket(0.0, 100.0, crate::BucketSize::Session),
            make_bucket(100.0, 200.0, crate::BucketSize::Day),
        ];
        let key_metrics = vec![
            (
                TempoRange::Slow,
                RhythmDirection::Early,
                vec![
                    make_metric(50.0, 5.0),   // falls in bucket 0
                    make_metric(150.0, 15.0), // falls in bucket 1
                ],
            ),
            (TempoRange::Slow, RhythmDirection::Late, vec![]),
            (TempoRange::Slow, RhythmDirection::OnBeat, vec![]),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        assert_eq!(data.columns.len(), 2);

        let cell0 = &data.columns[0].cells[0];
        assert!((cell0.mean_accuracy_percent.unwrap() - 5.0).abs() < 0.01);

        let cell1 = &data.columns[1].cells[0];
        assert!((cell1.mean_accuracy_percent.unwrap() - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_empty_cell_for_range_without_data_in_bucket() {
        let buckets = vec![
            make_bucket(0.0, 50.0, crate::BucketSize::Session),
            make_bucket(50.0, 100.0, crate::BucketSize::Session),
        ];
        let key_metrics = vec![
            (
                TempoRange::Slow,
                RhythmDirection::Early,
                vec![make_metric(75.0, 10.0)], // only in bucket 1
            ),
            (TempoRange::Slow, RhythmDirection::Late, vec![]),
            (TempoRange::Slow, RhythmDirection::OnBeat, vec![]),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        assert!(!data.columns[0].cells[0].has_data()); // no data in bucket 0
        assert!(data.columns[1].cells[0].has_data()); // data in bucket 1
    }

    #[test]
    fn test_accuracy_level_classification() {
        let buckets = vec![make_bucket(0.0, 100.0, crate::BucketSize::Session)];
        let key_metrics = vec![
            (
                TempoRange::Slow,
                RhythmDirection::Early,
                vec![make_metric(50.0, 4.0)],
            ),
            (TempoRange::Slow, RhythmDirection::Late, vec![]),
            (TempoRange::Slow, RhythmDirection::OnBeat, vec![]),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        let cell = &data.columns[0].cells[0];
        // 4% of 16th at Slow (250ms) = 10ms → precise (threshold=20ms)
        assert_eq!(
            data.accuracy_level(cell, TempoRange::Slow),
            Some(SpectrogramAccuracyLevel::Precise)
        );
    }

    #[test]
    fn test_on_beat_contributes_to_mean_only() {
        let buckets = vec![make_bucket(0.0, 100.0, crate::BucketSize::Session)];
        let key_metrics = vec![
            (TempoRange::Slow, RhythmDirection::Early, vec![]),
            (TempoRange::Slow, RhythmDirection::Late, vec![]),
            (
                TempoRange::Slow,
                RhythmDirection::OnBeat,
                vec![make_metric(50.0, 2.0)],
            ),
        ];

        let data =
            SpectrogramData::compute(&buckets, &key_metrics, SpectrogramThresholds::default());
        let cell = &data.columns[0].cells[0];
        assert!(cell.has_data());
        assert!((cell.mean_accuracy_percent.unwrap() - 2.0).abs() < 0.01);
        assert!(cell.early_stats.is_none());
        assert!(cell.late_stats.is_none());
    }
}
