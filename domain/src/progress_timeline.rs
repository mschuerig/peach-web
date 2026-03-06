use std::collections::HashMap;

use crate::records::{PitchComparisonRecord, PitchMatchingRecord};
use crate::training_mode::TrainingMode;
use crate::trend::Trend;

const SECS_PER_DAY: f64 = 86_400.0;
const SECS_PER_WEEK: f64 = 604_800.0;
const SECS_PER_MONTH: f64 = 2_592_000.0;
const SESSION_GAP_SECS: f64 = 1_800.0;

/// Granularity of a time bucket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BucketSize {
    Session,
    Day,
    Week,
    Month,
}

/// A time-grouped aggregate of training metrics.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeBucket {
    pub period_start: f64,
    pub period_end: f64,
    pub bucket_size: BucketSize,
    pub mean: f64,
    pub stddev: f64,
    pub record_count: usize,
}

/// Internal per-mode tracking state.
#[derive(Clone, Debug)]
struct ModeState {
    buckets: Vec<TimeBucket>,
    ewma: Option<f64>,
    record_count: usize,
    computed_trend: Option<Trend>,
    all_metrics: Vec<(f64, f64)>, // (timestamp, metric)
    running_mean: f64,
    running_m2: f64,
}

impl ModeState {
    fn new() -> Self {
        Self {
            buckets: Vec::new(),
            ewma: None,
            record_count: 0,
            computed_trend: None,
            all_metrics: Vec::new(),
            running_mean: 0.0,
            running_m2: 0.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn add_point(&mut self, timestamp: f64, metric: f64, now: f64) {
        self.all_metrics.push((timestamp, metric));
        self.record_count += 1;

        // Update running mean/m2 via Welford's
        let n = self.record_count as f64;
        let delta = metric - self.running_mean;
        self.running_mean += delta / n;
        let delta2 = metric - self.running_mean;
        self.running_m2 += delta * delta2;

        // Update session bucket: extend last bucket if within gap, else new bucket
        let age = now - timestamp;
        if age < SECS_PER_DAY {
            // Session bucketing
            if let Some(last) = self.buckets.last_mut() {
                if last.bucket_size == BucketSize::Session
                    && (timestamp - last.period_end).abs() < SESSION_GAP_SECS
                {
                    // Extend existing session bucket using Welford's
                    let new_count = last.record_count + 1;
                    let delta = metric - last.mean;
                    let new_mean = last.mean + delta / new_count as f64;
                    let delta2 = metric - new_mean;
                    // We track stddev via the full recalc approach for simplicity
                    last.mean = new_mean;
                    last.record_count = new_count;
                    last.period_end = timestamp;
                    // Recompute stddev from scratch for the bucket (simple for small buckets)
                    // For incremental add, we accept approximate stddev
                    let old_m2 = last.stddev * last.stddev * (new_count - 1) as f64;
                    let new_m2 = old_m2 + delta * delta2;
                    last.stddev = if new_count > 1 {
                        (new_m2 / (new_count - 1) as f64).sqrt()
                    } else {
                        0.0
                    };
                } else {
                    self.buckets.push(TimeBucket {
                        period_start: timestamp,
                        period_end: timestamp,
                        bucket_size: BucketSize::Session,
                        mean: metric,
                        stddev: 0.0,
                        record_count: 1,
                    });
                }
            } else {
                self.buckets.push(TimeBucket {
                    period_start: timestamp,
                    period_end: timestamp,
                    bucket_size: BucketSize::Session,
                    mean: metric,
                    stddev: 0.0,
                    record_count: 1,
                });
            }
        } else {
            // For older records added incrementally, just append a single-record bucket
            let (size, start) = bucket_assignment(timestamp, now);
            self.buckets.push(TimeBucket {
                period_start: start,
                period_end: timestamp,
                bucket_size: size,
                mean: metric,
                stddev: 0.0,
                record_count: 1,
            });
        }

        self.recompute_ewma();
        self.recompute_trend();
    }

    fn recompute_ewma(&mut self) {
        if self.buckets.is_empty() {
            self.ewma = None;
            return;
        }

        let halflife = self.halflife_secs();
        let mut ewma = self.buckets[0].mean;

        for i in 1..self.buckets.len() {
            let dt = self.buckets[i].period_start - self.buckets[i - 1].period_start;
            let alpha = 1.0 - (-f64::ln(2.0) * dt / halflife).exp();
            ewma = alpha * self.buckets[i].mean + (1.0 - alpha) * ewma;
        }

        self.ewma = Some(ewma);
    }

    fn recompute_trend(&mut self) {
        if self.record_count < 2 {
            self.computed_trend = None;
            return;
        }

        let ewma = match self.ewma {
            Some(e) => e,
            None => {
                self.computed_trend = None;
                return;
            }
        };

        let latest = self.buckets.last().map(|b| b.mean).unwrap_or(0.0);
        let running_stddev = if self.record_count > 1 {
            (self.running_m2 / (self.record_count - 1) as f64).sqrt()
        } else {
            0.0
        };

        // Lower metric = better (closer to perfect pitch)
        // Declining if latest > running_mean + running_stddev
        // Improving if latest < ewma
        // Stable otherwise
        self.computed_trend = Some(if latest > self.running_mean + running_stddev {
            Trend::Declining
        } else if latest < ewma {
            Trend::Improving
        } else {
            Trend::Stable
        });
    }

    fn halflife_secs(&self) -> f64 {
        SECS_PER_WEEK // 7-day half-life
    }
}

/// Per-mode progress tracking with EWMA smoothing and adaptive time bucketing.
#[derive(Clone, Debug)]
pub struct ProgressTimeline {
    modes: HashMap<TrainingMode, ModeState>,
}

impl ProgressTimeline {
    pub fn new() -> Self {
        let mut modes = HashMap::new();
        for mode in TrainingMode::ALL {
            modes.insert(mode, ModeState::new());
        }
        Self { modes }
    }

    /// Rebuild all per-mode data from stored records.
    pub fn rebuild(
        &mut self,
        comparison_records: &[PitchComparisonRecord],
        matching_records: &[PitchMatchingRecord],
        now: f64,
    ) {
        // Reset all modes
        for state in self.modes.values_mut() {
            state.reset();
        }

        // Collect (timestamp, metric, mode) tuples from all records
        let mut points: Vec<(f64, f64, TrainingMode)> = Vec::new();

        for record in comparison_records {
            let ts = parse_iso8601_to_epoch(&record.timestamp);
            for mode in TrainingMode::ALL {
                if let Some(metric) = mode.extract_comparison_metric(record) {
                    points.push((ts, metric, mode));
                }
            }
        }

        for record in matching_records {
            let ts = parse_iso8601_to_epoch(&record.timestamp);
            for mode in TrainingMode::ALL {
                if let Some(metric) = mode.extract_matching_metric(record) {
                    points.push((ts, metric, mode));
                }
            }
        }

        // Sort by timestamp
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Group by mode
        let mut mode_points: HashMap<TrainingMode, Vec<(f64, f64)>> = HashMap::new();
        for (ts, metric, mode) in points {
            mode_points.entry(mode).or_default().push((ts, metric));
        }

        // Build each mode
        for (mode, pts) in mode_points {
            let state = self.modes.get_mut(&mode).unwrap();
            // Assign to adaptive buckets
            let buckets = build_adaptive_buckets(&pts, now);
            state.buckets = buckets;
            state.record_count = pts.len();

            // Compute running mean/m2 (Welford's across ALL metrics)
            for &(_ts, metric) in &pts {
                state.all_metrics.push((_ts, metric));
                let n = state.all_metrics.len() as f64;
                let delta = metric - state.running_mean;
                state.running_mean += delta / n;
                let delta2 = metric - state.running_mean;
                state.running_m2 += delta * delta2;
            }

            state.recompute_ewma();
            state.recompute_trend();
        }
    }

    /// Returns the training mode state: NoData or Active.
    pub fn state(&self, mode: TrainingMode) -> crate::training_mode::TrainingModeState {
        match self.modes.get(&mode) {
            Some(s) if s.record_count > 0 => crate::training_mode::TrainingModeState::Active,
            _ => crate::training_mode::TrainingModeState::NoData,
        }
    }

    /// Returns time-grouped buckets for the given mode.
    pub fn buckets(&self, mode: TrainingMode) -> Vec<TimeBucket> {
        self.modes.get(&mode).map(|s| s.buckets.clone()).unwrap_or_default()
    }

    /// Returns the current EWMA for the given mode.
    pub fn current_ewma(&self, mode: TrainingMode) -> Option<f64> {
        self.modes.get(&mode).and_then(|s| s.ewma)
    }

    /// Returns the trend for the given mode (None if < 2 records).
    pub fn trend(&self, mode: TrainingMode) -> Option<Trend> {
        self.modes.get(&mode).and_then(|s| s.computed_trend)
    }

    /// Incrementally update from a new comparison record.
    pub fn add_comparison(&mut self, record: &PitchComparisonRecord, now: f64) {
        let ts = parse_iso8601_to_epoch(&record.timestamp);
        for mode in TrainingMode::ALL {
            if let Some(metric) = mode.extract_comparison_metric(record)
                && let Some(state) = self.modes.get_mut(&mode)
            {
                state.add_point(ts, metric, now);
            }
        }
    }

    /// Incrementally update from a new pitch matching record.
    pub fn add_matching(&mut self, record: &PitchMatchingRecord, now: f64) {
        let ts = parse_iso8601_to_epoch(&record.timestamp);
        for mode in TrainingMode::ALL {
            if let Some(metric) = mode.extract_matching_metric(record)
                && let Some(state) = self.modes.get_mut(&mode)
            {
                state.add_point(ts, metric, now);
            }
        }
    }

    /// Clear all per-mode data.
    pub fn reset(&mut self) {
        for state in self.modes.values_mut() {
            state.reset();
        }
    }
}

impl Default for ProgressTimeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Determine bucket size and period start for a given timestamp.
fn bucket_assignment(timestamp: f64, now: f64) -> (BucketSize, f64) {
    let age = now - timestamp;
    if age < SECS_PER_DAY {
        (BucketSize::Session, timestamp)
    } else if age < SECS_PER_WEEK {
        let start = (timestamp / SECS_PER_DAY).floor() * SECS_PER_DAY;
        (BucketSize::Day, start)
    } else if age < SECS_PER_MONTH {
        let start = (timestamp / SECS_PER_WEEK).floor() * SECS_PER_WEEK;
        (BucketSize::Week, start)
    } else {
        let start = (timestamp / SECS_PER_MONTH).floor() * SECS_PER_MONTH;
        (BucketSize::Month, start)
    }
}

/// Build adaptive time buckets from sorted (timestamp, metric) pairs.
fn build_adaptive_buckets(points: &[(f64, f64)], now: f64) -> Vec<TimeBucket> {
    if points.is_empty() {
        return Vec::new();
    }

    type BucketGroup = (BucketSize, u64, Vec<(f64, f64)>);

    let mut groups: Vec<(BucketSize, Vec<(f64, f64)>)> = Vec::new();
    let mut current_group: Option<BucketGroup> = None;

    for &(ts, metric) in points {
        let age = now - ts;
        let (size, period_start) = if age < SECS_PER_DAY {
            (BucketSize::Session, ts)
        } else {
            bucket_assignment(ts, now)
        };

        let group_key = if size == BucketSize::Session {
            if let Some((ref cur_size, _, ref cur_points)) = current_group
                && *cur_size == BucketSize::Session
            {
                let last_ts = cur_points.last().map(|&(t, _)| t).unwrap_or(ts);
                if ts - last_ts < SESSION_GAP_SECS {
                    if let Some((_, _, ref mut pts)) = current_group {
                        pts.push((ts, metric));
                    }
                    continue;
                }
            }
            ts as u64
        } else {
            period_start as u64
        };

        if let Some((ref cur_size, cur_key, _)) = current_group
            && *cur_size == size && cur_key == group_key
        {
            if let Some((_, _, ref mut pts)) = current_group {
                pts.push((ts, metric));
            }
            continue;
        }

        if let Some((prev_size, _, prev_points)) = current_group.take() {
            groups.push((prev_size, prev_points));
        }

        current_group = Some((size, group_key, vec![(ts, metric)]));
    }

    if let Some((size, _, pts)) = current_group {
        groups.push((size, pts));
    }

    // Convert groups to TimeBuckets
    groups
        .into_iter()
        .map(|(size, pts)| {
            let n = pts.len();
            let first_ts = pts[0].0;
            let last_ts = pts[n - 1].0;

            // Welford's for mean and stddev
            let mut mean = 0.0;
            let mut m2 = 0.0;
            for (i, &(_, metric)) in pts.iter().enumerate() {
                let delta = metric - mean;
                mean += delta / (i + 1) as f64;
                let delta2 = metric - mean;
                m2 += delta * delta2;
            }
            let stddev = if n > 1 { (m2 / (n - 1) as f64).sqrt() } else { 0.0 };

            TimeBucket {
                period_start: first_ts,
                period_end: last_ts,
                bucket_size: size,
                mean,
                stddev,
                record_count: n,
            }
        })
        .collect()
}

/// Parse a simplified ISO 8601 timestamp to epoch seconds.
/// Expected format: "YYYY-MM-DDTHH:MM:SSZ" or similar.
fn parse_iso8601_to_epoch(timestamp: &str) -> f64 {
    // Simple parser for "YYYY-MM-DDTHH:MM:SSZ"
    if timestamp.len() < 19 {
        return 0.0;
    }

    let year: i64 = timestamp[0..4].parse().unwrap_or(1970);
    let month: i64 = timestamp[5..7].parse().unwrap_or(1);
    let day: i64 = timestamp[8..10].parse().unwrap_or(1);
    let hour: i64 = timestamp[11..13].parse().unwrap_or(0);
    let min: i64 = timestamp[14..16].parse().unwrap_or(0);
    let sec: i64 = timestamp[17..19].parse().unwrap_or(0);

    // Days from epoch (1970-01-01) — simplified, no leap second handling
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += month_days[m as usize];
        if m == 2 && is_leap(year) {
            days += 1;
        }
    }
    days += day - 1;

    (days * 86400 + hour * 3600 + min * 60 + sec) as f64
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::{PitchComparisonRecord, PitchMatchingRecord};
    use crate::training_mode::TrainingModeState;

    fn make_comparison(interval: u8, cent_offset: f64, timestamp: &str) -> PitchComparisonRecord {
        PitchComparisonRecord {
            reference_note: 60,
            target_note: 60 + interval,
            cent_offset,
            is_correct: true,
            interval,
            tuning_system: "equalTemperament".to_string(),
            timestamp: timestamp.to_string(),
        }
    }

    fn make_matching(interval: u8, user_cent_error: f64, timestamp: &str) -> PitchMatchingRecord {
        PitchMatchingRecord {
            reference_note: 60,
            target_note: 60 + interval,
            initial_cent_offset: 20.0,
            user_cent_error,
            interval,
            tuning_system: "equalTemperament".to_string(),
            timestamp: timestamp.to_string(),
        }
    }

    // Epoch for 2026-03-06T12:00:00Z
    fn now_epoch() -> f64 {
        parse_iso8601_to_epoch("2026-03-06T12:00:00Z")
    }

    // --- AC1: ProgressTimeline constructor ---

    #[test]
    fn test_new_creates_empty_timeline() {
        let tl = ProgressTimeline::new();
        for mode in TrainingMode::ALL {
            assert_eq!(tl.state(mode), TrainingModeState::NoData);
            assert!(tl.buckets(mode).is_empty());
            assert_eq!(tl.current_ewma(mode), None);
            assert_eq!(tl.trend(mode), None);
        }
    }

    // --- AC2: Rebuild from records ---

    #[test]
    fn test_rebuild_with_empty_records() {
        let mut tl = ProgressTimeline::new();
        tl.rebuild(&[], &[], now_epoch());
        for mode in TrainingMode::ALL {
            assert_eq!(tl.state(mode), TrainingModeState::NoData);
        }
    }

    #[test]
    fn test_rebuild_comparison_unison_active_others_nodata() {
        let mut tl = ProgressTimeline::new();
        let records = vec![
            make_comparison(0, 15.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, &[], now_epoch());

        assert_eq!(tl.state(TrainingMode::UnisonPitchComparison), TrainingModeState::Active);
        assert_eq!(tl.state(TrainingMode::IntervalPitchComparison), TrainingModeState::NoData);
        assert_eq!(tl.state(TrainingMode::UnisonMatching), TrainingModeState::NoData);
        assert_eq!(tl.state(TrainingMode::IntervalMatching), TrainingModeState::NoData);
    }

    #[test]
    fn test_rebuild_matching_interval_active() {
        let mut tl = ProgressTimeline::new();
        let records = vec![
            make_matching(7, 5.0, "2026-03-06T11:00:00Z"),
        ];
        tl.rebuild(&[], &records, now_epoch());

        assert_eq!(tl.state(TrainingMode::IntervalMatching), TrainingModeState::Active);
        assert_eq!(tl.state(TrainingMode::UnisonMatching), TrainingModeState::NoData);
    }

    // --- AC4: Adaptive bucketing ---

    #[test]
    fn test_session_gap_grouping() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        // Two records 5 min apart (< 1800s gap) -> same session bucket
        let records = vec![
            make_comparison(0, 20.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, &[], now);

        let buckets = tl.buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Session);
        assert_eq!(buckets[0].record_count, 2);
        assert!((buckets[0].mean - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_session_gap_splits_sessions() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        // Two records 2 hours apart (> 1800s gap) -> two session buckets
        let records = vec![
            make_comparison(0, 20.0, "2026-03-06T09:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:30:00Z"),
        ];
        tl.rebuild(&records, &[], now);

        let buckets = tl.buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Session);
        assert_eq!(buckets[1].bucket_size, BucketSize::Session);
    }

    #[test]
    fn test_day_bucketing_for_2_to_6_day_old_records() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        // Records from 3 days ago
        let three_days_ago = now - 3.0 * SECS_PER_DAY;
        let ts1 = epoch_to_iso8601(three_days_ago);
        let ts2 = epoch_to_iso8601(three_days_ago + 3600.0);

        let records = vec![
            make_comparison(0, 20.0, &ts1),
            make_comparison(0, 10.0, &ts2),
        ];
        tl.rebuild(&records, &[], now);

        let buckets = tl.buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Day);
        assert_eq!(buckets[0].record_count, 2);
    }

    // --- AC5: EWMA computation ---

    #[test]
    fn test_ewma_two_buckets() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        // Two separate sessions (>30min gap)
        let records = vec![
            make_comparison(0, 30.0, &epoch_to_iso8601(now - 4.0 * 3600.0)),
            make_comparison(0, 10.0, &epoch_to_iso8601(now - 1.0 * 3600.0)),
        ];
        tl.rebuild(&records, &[], now);

        let ewma = tl.current_ewma(TrainingMode::UnisonPitchComparison);
        assert!(ewma.is_some());
        let ewma_val = ewma.unwrap();
        // EWMA should be between the two bucket means, biased toward the more recent
        assert!(ewma_val > 10.0 && ewma_val < 30.0);
    }

    #[test]
    fn test_ewma_single_bucket() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let records = vec![
            make_comparison(0, 20.0, "2026-03-06T11:00:00Z"),
        ];
        tl.rebuild(&records, &[], now);

        let ewma = tl.current_ewma(TrainingMode::UnisonPitchComparison);
        assert_eq!(ewma, Some(20.0));
    }

    // --- AC6: Trend computation ---

    #[test]
    fn test_trend_none_with_less_than_2_records() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let records = vec![make_comparison(0, 20.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&records, &[], now);
        assert_eq!(tl.trend(TrainingMode::UnisonPitchComparison), None);
    }

    #[test]
    fn test_trend_improving() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        // Many high-value records followed by low-value records -> improving
        let mut records = Vec::new();
        for i in 0..10 {
            let ts = epoch_to_iso8601(now - (20 - i) as f64 * 3600.0);
            records.push(make_comparison(0, 50.0, &ts));
        }
        for i in 10..20 {
            let ts = epoch_to_iso8601(now - (20 - i) as f64 * 3600.0);
            records.push(make_comparison(0, 10.0, &ts));
        }
        tl.rebuild(&records, &[], now);
        // Latest bucket mean (10.0) < ewma -> Improving
        let trend = tl.trend(TrainingMode::UnisonPitchComparison);
        assert_eq!(trend, Some(Trend::Improving));
    }

    #[test]
    fn test_trend_declining() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        // Low-value records spread across days, then a very high recent bucket
        // -> latest > running_mean + running_stddev -> Declining
        let mut records = Vec::new();
        // 5 sessions over past days with low values
        for i in 0..5 {
            let ts = epoch_to_iso8601(now - (10 - i) as f64 * 2.0 * 3600.0);
            records.push(make_comparison(0, 10.0, &ts));
        }
        // Recent session with very high values (>= mean + stddev)
        let ts = epoch_to_iso8601(now - 300.0);
        records.push(make_comparison(0, 100.0, &ts));
        tl.rebuild(&records, &[], now);
        let trend = tl.trend(TrainingMode::UnisonPitchComparison);
        assert_eq!(trend, Some(Trend::Declining));
    }

    // --- AC7, AC8: TimeBucket and BucketSize ---

    #[test]
    fn test_time_bucket_fields() {
        let bucket = TimeBucket {
            period_start: 1000.0,
            period_end: 2000.0,
            bucket_size: BucketSize::Week,
            mean: 25.0,
            stddev: 5.0,
            record_count: 10,
        };
        assert_eq!(bucket.period_start, 1000.0);
        assert_eq!(bucket.period_end, 2000.0);
        assert_eq!(bucket.bucket_size, BucketSize::Week);
        assert_eq!(bucket.mean, 25.0);
        assert_eq!(bucket.stddev, 5.0);
        assert_eq!(bucket.record_count, 10);
    }

    #[test]
    fn test_bucket_size_variants() {
        let sizes = [BucketSize::Session, BucketSize::Day, BucketSize::Week, BucketSize::Month];
        assert_eq!(sizes.len(), 4);
    }

    // --- AC9: Incremental comparison update ---

    #[test]
    fn test_add_comparison_updates_correct_mode() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();

        let record = make_comparison(0, 15.0, "2026-03-06T11:55:00Z");
        tl.add_comparison(&record, now);

        assert_eq!(tl.state(TrainingMode::UnisonPitchComparison), TrainingModeState::Active);
        assert_eq!(tl.state(TrainingMode::IntervalPitchComparison), TrainingModeState::NoData);
        assert!(tl.current_ewma(TrainingMode::UnisonPitchComparison).is_some());
    }

    // --- AC10: Incremental matching update ---

    #[test]
    fn test_add_matching_updates_correct_mode() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();

        let record = make_matching(0, 5.0, "2026-03-06T11:55:00Z");
        tl.add_matching(&record, now);

        assert_eq!(tl.state(TrainingMode::UnisonMatching), TrainingModeState::Active);
        assert_eq!(tl.state(TrainingMode::IntervalMatching), TrainingModeState::NoData);
    }

    // --- AC11: Reset ---

    #[test]
    fn test_reset_clears_all_data() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();

        let records = vec![
            make_comparison(0, 15.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, &[], now);
        assert_eq!(tl.state(TrainingMode::UnisonPitchComparison), TrainingModeState::Active);

        tl.reset();
        for mode in TrainingMode::ALL {
            assert_eq!(tl.state(mode), TrainingModeState::NoData);
            assert!(tl.buckets(mode).is_empty());
            assert_eq!(tl.current_ewma(mode), None);
            assert_eq!(tl.trend(mode), None);
        }
    }

    // --- ISO 8601 parser ---

    #[test]
    fn test_parse_iso8601() {
        let epoch = parse_iso8601_to_epoch("1970-01-01T00:00:00Z");
        assert_eq!(epoch, 0.0);

        let epoch2 = parse_iso8601_to_epoch("1970-01-02T00:00:00Z");
        assert_eq!(epoch2, 86400.0);
    }

    // --- Helper to convert epoch back to ISO for test setup ---
    fn epoch_to_iso8601(epoch: f64) -> String {
        let mut remaining = epoch as i64;
        let secs = remaining % 60;
        remaining /= 60;
        let mins = remaining % 60;
        remaining /= 60;
        let hours = remaining % 24;
        remaining /= 24;

        let mut year = 1970i64;
        loop {
            let days_in_year = if is_leap(year) { 366 } else { 365 };
            if remaining < days_in_year {
                break;
            }
            remaining -= days_in_year;
            year += 1;
        }

        let month_days = if is_leap(year) {
            [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        let mut month = 1;
        for m in 1..=12 {
            if remaining < month_days[m] {
                month = m;
                break;
            }
            remaining -= month_days[m];
        }
        let day = remaining + 1;

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hours, mins, secs
        )
    }
}
