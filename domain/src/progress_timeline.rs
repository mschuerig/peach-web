use std::collections::HashMap;

use crate::records::{PitchComparisonRecord, PitchMatchingRecord};
use crate::training_mode::TrainingMode;
use crate::trend::Trend;

const SECS_PER_DAY: f64 = 86_400.0;

/// Granularity of a time bucket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BucketSize {
    Session,
    Day,
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
    mode: TrainingMode,
    display_buckets: Vec<TimeBucket>,
    ewma_buckets: Vec<TimeBucket>,
    ewma: Option<f64>,
    record_count: usize,
    computed_trend: Option<Trend>,
    running_mean: f64,
    running_m2: f64,
}

impl ModeState {
    fn new(mode: TrainingMode) -> Self {
        Self {
            mode,
            display_buckets: Vec::new(),
            ewma_buckets: Vec::new(),
            ewma: None,
            record_count: 0,
            computed_trend: None,
            running_mean: 0.0,
            running_m2: 0.0,
        }
    }

    fn reset(&mut self) {
        let mode = self.mode;
        *self = Self::new(mode);
    }

    fn add_point(&mut self, timestamp: f64, metric: f64, start_of_today: f64) {
        self.record_count += 1;

        // Update running mean/m2 via Welford's
        let n = self.record_count as f64;
        let delta = metric - self.running_mean;
        self.running_mean += delta / n;
        let delta2 = metric - self.running_mean;
        self.running_m2 += delta * delta2;

        let session_gap = self.mode.config().session_gap_secs;

        // Update EWMA session buckets (pure gap-based, no zone logic)
        update_session_bucket(&mut self.ewma_buckets, timestamp, metric, session_gap);

        // Update display buckets with zone-aware logic
        let day_zone_start = start_of_today - 7.0 * SECS_PER_DAY;
        if timestamp >= start_of_today {
            // Session zone
            update_session_bucket(&mut self.display_buckets, timestamp, metric, session_gap);
        } else if timestamp >= day_zone_start {
            // Day zone — group by calendar day
            let day_start = floor_to_day(timestamp, start_of_today);
            update_or_create_bucket(
                &mut self.display_buckets,
                timestamp,
                metric,
                BucketSize::Day,
                day_start,
            );
        } else {
            // Month zone — group by calendar month
            let month_start = epoch_to_month_start(timestamp);
            update_or_create_bucket(
                &mut self.display_buckets,
                timestamp,
                metric,
                BucketSize::Month,
                month_start,
            );
        }

        self.recompute_ewma();
        self.recompute_trend();
    }

    fn recompute_ewma(&mut self) {
        if self.ewma_buckets.is_empty() {
            self.ewma = None;
            return;
        }

        let halflife = self.halflife_secs();
        let mut ewma = self.ewma_buckets[0].mean;

        for i in 1..self.ewma_buckets.len() {
            let dt =
                self.ewma_buckets[i].period_start - self.ewma_buckets[i - 1].period_start;
            let alpha = 1.0 - (-f64::ln(2.0) * dt / halflife).exp();
            ewma = alpha * self.ewma_buckets[i].mean + (1.0 - alpha) * ewma;
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

        let latest = self.display_buckets.last().map(|b| b.mean).unwrap_or(0.0);
        // Population stddev: sqrt(m2 / n)
        let running_stddev = if self.record_count > 0 {
            (self.running_m2 / self.record_count as f64).sqrt()
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
        self.mode.config().ewma_halflife_secs
    }
}

/// Update an existing session bucket or create a new one.
fn update_session_bucket(
    buckets: &mut Vec<TimeBucket>,
    timestamp: f64,
    metric: f64,
    session_gap: f64,
) {
    if let Some(last) = buckets.last_mut()
        && last.bucket_size == BucketSize::Session
        && (timestamp - last.period_end).abs() < session_gap
    {
        welford_update_bucket(last, timestamp, metric);
        return;
    }
    buckets.push(TimeBucket {
        period_start: timestamp,
        period_end: timestamp,
        bucket_size: BucketSize::Session,
        mean: metric,
        stddev: 0.0,
        record_count: 1,
    });
}

/// Update an existing bucket matching (size, period_start) or create a new one.
fn update_or_create_bucket(
    buckets: &mut Vec<TimeBucket>,
    timestamp: f64,
    metric: f64,
    size: BucketSize,
    period_start: f64,
) {
    // Find existing bucket with matching size and period_start
    if let Some(bucket) = buckets
        .iter_mut()
        .find(|b| b.bucket_size == size && (b.period_start - period_start).abs() < 1.0)
    {
        welford_update_bucket(bucket, timestamp, metric);
        return;
    }
    buckets.push(TimeBucket {
        period_start,
        period_end: timestamp,
        bucket_size: size,
        mean: metric,
        stddev: 0.0,
        record_count: 1,
    });
}

/// Apply Welford's online update to an existing bucket (population stddev).
fn welford_update_bucket(bucket: &mut TimeBucket, timestamp: f64, metric: f64) {
    let old_count = bucket.record_count;
    let new_count = old_count + 1;
    let delta = metric - bucket.mean;
    let new_mean = bucket.mean + delta / new_count as f64;
    let delta2 = metric - new_mean;
    bucket.mean = new_mean;
    bucket.record_count = new_count;
    if timestamp > bucket.period_end {
        bucket.period_end = timestamp;
    }
    // Recover old m2 from population stddev: m2 = stddev² × old_count
    let old_m2 = bucket.stddev * bucket.stddev * old_count as f64;
    let new_m2 = old_m2 + delta * delta2;
    // Population stddev: sqrt(m2 / n)
    bucket.stddev = if new_count > 0 {
        (new_m2 / new_count as f64).sqrt()
    } else {
        0.0
    };
}

/// Per-mode progress tracking with EWMA smoothing and three-zone time bucketing.
#[derive(Clone, Debug)]
pub struct ProgressTimeline {
    modes: HashMap<TrainingMode, ModeState>,
}

impl ProgressTimeline {
    pub fn new() -> Self {
        let mut modes = HashMap::new();
        for mode in TrainingMode::ALL {
            modes.insert(mode, ModeState::new(mode));
        }
        Self { modes }
    }

    /// Rebuild all per-mode data from stored records.
    pub fn rebuild(
        &mut self,
        comparison_records: &[PitchComparisonRecord],
        matching_records: &[PitchMatchingRecord],
        _now: f64,
        start_of_today: f64,
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
        let day_zone_start = start_of_today - 7.0 * SECS_PER_DAY;
        for (mode, pts) in mode_points {
            let state = self.modes.get_mut(&mode).unwrap();
            let session_gap = mode.config().session_gap_secs;

            // Build display buckets (three-zone)
            state.display_buckets =
                build_display_buckets(&pts, start_of_today, session_gap, day_zone_start);

            // Build EWMA session buckets (pure gap-based)
            state.ewma_buckets = build_ewma_session_buckets(&pts, session_gap);

            state.record_count = pts.len();

            // Compute running mean/m2 (Welford's across ALL metrics)
            for (i, &(_ts, metric)) in pts.iter().enumerate() {
                let n = (i + 1) as f64;
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

    /// Returns three-zone display buckets for the given mode.
    pub fn display_buckets(&self, mode: TrainingMode) -> Vec<TimeBucket> {
        self.modes
            .get(&mode)
            .map(|s| s.display_buckets.clone())
            .unwrap_or_default()
    }

    /// Returns the stddev of the last display bucket (for headline ±).
    pub fn latest_bucket_stddev(&self, mode: TrainingMode) -> Option<f64> {
        self.modes
            .get(&mode)
            .and_then(|s| s.display_buckets.last())
            .map(|b| b.stddev)
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
    pub fn add_comparison(
        &mut self,
        record: &PitchComparisonRecord,
        _now: f64,
        start_of_today: f64,
    ) {
        let ts = parse_iso8601_to_epoch(&record.timestamp);
        for mode in TrainingMode::ALL {
            if let Some(metric) = mode.extract_comparison_metric(record)
                && let Some(state) = self.modes.get_mut(&mode)
            {
                state.add_point(ts, metric, start_of_today);
            }
        }
    }

    /// Incrementally update from a new pitch matching record.
    pub fn add_matching(
        &mut self,
        record: &PitchMatchingRecord,
        _now: f64,
        start_of_today: f64,
    ) {
        let ts = parse_iso8601_to_epoch(&record.timestamp);
        for mode in TrainingMode::ALL {
            if let Some(metric) = mode.extract_matching_metric(record)
                && let Some(state) = self.modes.get_mut(&mode)
            {
                state.add_point(ts, metric, start_of_today);
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

/// Build three-zone display buckets from sorted (timestamp, metric) pairs.
fn build_display_buckets(
    points: &[(f64, f64)],
    start_of_today: f64,
    session_gap_secs: f64,
    day_zone_start: f64,
) -> Vec<TimeBucket> {
    if points.is_empty() {
        return Vec::new();
    }

    // Separate points into three zones
    let mut month_points: Vec<(f64, f64)> = Vec::new();
    let mut day_points: Vec<(f64, f64)> = Vec::new();
    let mut session_points: Vec<(f64, f64)> = Vec::new();

    for &(ts, metric) in points {
        if ts >= start_of_today {
            session_points.push((ts, metric));
        } else if ts >= day_zone_start {
            day_points.push((ts, metric));
        } else {
            month_points.push((ts, metric));
        }
    }

    let mut all_buckets: Vec<TimeBucket> = Vec::new();

    // Month zone: group by (year, month)
    if !month_points.is_empty() {
        let mut month_groups: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        for &(ts, metric) in &month_points {
            let ms = epoch_to_month_start(ts);
            if let Some(last) = month_groups.last_mut()
                && (last.0 - ms).abs() < 1.0
            {
                last.1.push((ts, metric));
                continue;
            }
            month_groups.push((ms, vec![(ts, metric)]));
        }

        for (month_start, pts) in &month_groups {
            all_buckets.push(aggregate_bucket(pts, BucketSize::Month, *month_start));
        }

        // Truncate last monthly bucket's end to day_zone_start
        if let Some(last_month) = all_buckets.last_mut()
            && last_month.period_end > day_zone_start
        {
            last_month.period_end = day_zone_start;
        }
    }

    // Day zone: group by calendar day (floor to day boundary relative to start_of_today)
    if !day_points.is_empty() {
        let mut day_groups: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
        for &(ts, metric) in &day_points {
            let ds = floor_to_day(ts, start_of_today);
            if let Some(last) = day_groups.last_mut()
                && (last.0 - ds).abs() < 1.0
            {
                last.1.push((ts, metric));
                continue;
            }
            day_groups.push((ds, vec![(ts, metric)]));
        }

        for (day_start, pts) in &day_groups {
            all_buckets.push(aggregate_bucket(pts, BucketSize::Day, *day_start));
        }
    }

    // Session zone: group by 30-min gap rule
    if !session_points.is_empty() {
        let mut session_groups: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut current_group: Vec<(f64, f64)> = vec![session_points[0]];

        for &(ts, metric) in &session_points[1..] {
            let last_ts = current_group.last().unwrap().0;
            if ts - last_ts < session_gap_secs {
                current_group.push((ts, metric));
            } else {
                session_groups.push(std::mem::take(&mut current_group));
                current_group.push((ts, metric));
            }
        }
        session_groups.push(current_group);

        for group in &session_groups {
            let first_ts = group[0].0;
            all_buckets.push(aggregate_bucket(group, BucketSize::Session, first_ts));
        }
    }

    all_buckets
}

/// Build EWMA session buckets from sorted (timestamp, metric) pairs.
/// Pure 30-min gap grouping with NO zone logic.
fn build_ewma_session_buckets(
    points: &[(f64, f64)],
    session_gap_secs: f64,
) -> Vec<TimeBucket> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut groups: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current_group: Vec<(f64, f64)> = vec![points[0]];

    for &(ts, metric) in &points[1..] {
        let last_ts = current_group.last().unwrap().0;
        if ts - last_ts < session_gap_secs {
            current_group.push((ts, metric));
        } else {
            groups.push(std::mem::take(&mut current_group));
            current_group.push((ts, metric));
        }
    }
    groups.push(current_group);

    groups
        .iter()
        .map(|pts| {
            let first_ts = pts[0].0;
            aggregate_bucket(pts, BucketSize::Session, first_ts)
        })
        .collect()
}

/// Aggregate a group of points into a single TimeBucket using Welford's (population stddev).
fn aggregate_bucket(pts: &[(f64, f64)], size: BucketSize, period_start: f64) -> TimeBucket {
    let n = pts.len();
    let last_ts = pts[n - 1].0;

    let mut mean = 0.0;
    let mut m2 = 0.0;
    for (i, &(_, metric)) in pts.iter().enumerate() {
        let delta = metric - mean;
        mean += delta / (i + 1) as f64;
        let delta2 = metric - mean;
        m2 += delta * delta2;
    }
    // Population stddev: sqrt(m2 / n)
    let stddev = if n > 0 { (m2 / n as f64).sqrt() } else { 0.0 };

    TimeBucket {
        period_start,
        period_end: last_ts,
        bucket_size: size,
        mean,
        stddev,
        record_count: n,
    }
}

/// Floor a timestamp to the day boundary relative to start_of_today.
/// Days are aligned to the same time-of-day as start_of_today (local midnight).
fn floor_to_day(timestamp: f64, start_of_today: f64) -> f64 {
    let offset = start_of_today - timestamp;
    let days_back = (offset / SECS_PER_DAY).ceil();
    start_of_today - days_back * SECS_PER_DAY
}

/// Convert epoch seconds to the epoch of the first second of that month (UTC).
fn epoch_to_month_start(epoch: f64) -> f64 {
    let (year, month) = epoch_to_year_month(epoch);
    year_month_to_epoch(year, month)
}

/// Extract (year, month) from epoch seconds.
fn epoch_to_year_month(epoch: f64) -> (i64, i64) {
    let mut remaining = epoch as i64 / 86400;
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

    let mut month = 1i64;
    for (m, &days) in month_days.iter().enumerate().skip(1) {
        if remaining < days {
            month = m as i64;
            break;
        }
        remaining -= days;
    }
    (year, month)
}

/// Convert (year, month) to epoch seconds of the first second of that month.
fn year_month_to_epoch(year: i64, month: i64) -> f64 {
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
    (days * 86400) as f64
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

    // Start of today: 2026-03-06T00:00:00Z
    fn today_epoch() -> f64 {
        parse_iso8601_to_epoch("2026-03-06T00:00:00Z")
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
        for (m, &days) in month_days.iter().enumerate().skip(1) {
            if remaining < days {
                month = m;
                break;
            }
            remaining -= days;
        }
        let day = remaining + 1;

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hours, mins, secs
        )
    }

    // --- AC1: ProgressTimeline constructor ---

    #[test]
    fn test_new_creates_empty_timeline() {
        let tl = ProgressTimeline::new();
        for mode in TrainingMode::ALL {
            assert_eq!(tl.state(mode), TrainingModeState::NoData);
            assert!(tl.display_buckets(mode).is_empty());
            assert_eq!(tl.current_ewma(mode), None);
            assert_eq!(tl.trend(mode), None);
        }
    }

    // --- AC2: Rebuild from records ---

    #[test]
    fn test_rebuild_with_empty_records() {
        let mut tl = ProgressTimeline::new();
        tl.rebuild(&[], &[], now_epoch(), today_epoch());
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
        tl.rebuild(&records, &[], now_epoch(), today_epoch());

        assert_eq!(
            tl.state(TrainingMode::UnisonPitchComparison),
            TrainingModeState::Active
        );
        assert_eq!(
            tl.state(TrainingMode::IntervalPitchComparison),
            TrainingModeState::NoData
        );
        assert_eq!(
            tl.state(TrainingMode::UnisonMatching),
            TrainingModeState::NoData
        );
        assert_eq!(
            tl.state(TrainingMode::IntervalMatching),
            TrainingModeState::NoData
        );
    }

    #[test]
    fn test_rebuild_matching_interval_active() {
        let mut tl = ProgressTimeline::new();
        let records = vec![make_matching(7, 5.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&[], &records, now_epoch(), today_epoch());

        assert_eq!(
            tl.state(TrainingMode::IntervalMatching),
            TrainingModeState::Active
        );
        assert_eq!(
            tl.state(TrainingMode::UnisonMatching),
            TrainingModeState::NoData
        );
    }

    // --- Session gap grouping ---

    #[test]
    fn test_session_gap_grouping() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();
        // Two records 5 min apart (< 1800s gap) -> same session bucket
        let records = vec![
            make_comparison(0, 20.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Session);
        assert_eq!(buckets[0].record_count, 2);
        assert!((buckets[0].mean - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_session_gap_splits_sessions() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();
        // Two records 2 hours apart (> 1800s gap) -> two session buckets
        let records = vec![
            make_comparison(0, 20.0, "2026-03-06T09:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:30:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Session);
        assert_eq!(buckets[1].bucket_size, BucketSize::Session);
    }

    // --- Three-zone bucketing (AC: 2) ---

    #[test]
    fn test_three_zone_bucketing() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch(); // 2026-03-06T12:00:00Z
        let today = today_epoch(); // 2026-03-06T00:00:00Z

        // Month zone: 2+ months ago
        let two_months_ago = today - 60.0 * SECS_PER_DAY;
        let ts_month = epoch_to_iso8601(two_months_ago);

        // Day zone: 3 days ago (within 7 days before today, before start_of_today)
        let three_days_ago = today - 3.0 * SECS_PER_DAY + 3600.0; // +1h to be clearly in that day
        let ts_day = epoch_to_iso8601(three_days_ago);

        // Session zone: today
        let ts_session = "2026-03-06T11:00:00Z";

        let records = vec![
            make_comparison(0, 30.0, &ts_month),
            make_comparison(0, 20.0, &ts_day),
            make_comparison(0, 10.0, ts_session),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert_eq!(buckets[1].bucket_size, BucketSize::Day);
        assert_eq!(buckets[2].bucket_size, BucketSize::Session);
    }

    #[test]
    fn test_day_zone_calendar_day_snapping() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch(); // 2026-03-06T00:00:00Z

        // Two records on the same day (3 days ago), different times
        let three_days_ago_morning = today - 3.0 * SECS_PER_DAY + 3600.0; // 01:00
        let three_days_ago_evening = today - 3.0 * SECS_PER_DAY + 15.0 * 3600.0; // 15:00
        let ts1 = epoch_to_iso8601(three_days_ago_morning);
        let ts2 = epoch_to_iso8601(three_days_ago_evening);

        let records = vec![
            make_comparison(0, 20.0, &ts1),
            make_comparison(0, 10.0, &ts2),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Day);
        assert_eq!(buckets[0].record_count, 2);
    }

    #[test]
    fn test_day_zone_different_days() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Records on two different days within the day zone
        let two_days_ago = today - 2.0 * SECS_PER_DAY + 3600.0;
        let four_days_ago = today - 4.0 * SECS_PER_DAY + 3600.0;
        let ts1 = epoch_to_iso8601(four_days_ago);
        let ts2 = epoch_to_iso8601(two_days_ago);

        let records = vec![
            make_comparison(0, 20.0, &ts1),
            make_comparison(0, 10.0, &ts2),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Day);
        assert_eq!(buckets[1].bucket_size, BucketSize::Day);
    }

    #[test]
    fn test_month_zone_calendar_month_grouping() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Two records in January 2026 (well in the month zone)
        let ts1 = "2026-01-10T12:00:00Z";
        let ts2 = "2026-01-20T15:00:00Z";

        let records = vec![
            make_comparison(0, 30.0, ts1),
            make_comparison(0, 20.0, ts2),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert_eq!(buckets[0].record_count, 2);

        // period_start should be Jan 1 2026
        let jan1 = parse_iso8601_to_epoch("2026-01-01T00:00:00Z");
        assert!((buckets[0].period_start - jan1).abs() < 1.0);
    }

    #[test]
    fn test_month_zone_different_months() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Records in Jan and Feb (both in month zone)
        let ts1 = "2026-01-15T12:00:00Z";
        let ts2 = "2026-02-15T12:00:00Z";

        let records = vec![
            make_comparison(0, 30.0, ts1),
            make_comparison(0, 20.0, ts2),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert_eq!(buckets[1].bucket_size, BucketSize::Month);
    }

    #[test]
    fn test_monthly_bucket_end_truncated_to_day_zone_boundary() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch(); // 2026-03-06T00:00:00Z
        let day_zone_start = today - 7.0 * SECS_PER_DAY; // 2026-02-27T00:00:00Z

        // Record in Feb, close to the day zone boundary
        let ts = "2026-02-25T12:00:00Z"; // before day_zone_start

        let records = vec![make_comparison(0, 20.0, ts)];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        // period_end should not exceed day_zone_start
        assert!(
            buckets[0].period_end <= day_zone_start,
            "Monthly bucket end {} should be <= day_zone_start {}",
            buckets[0].period_end,
            day_zone_start
        );
    }

    // --- Population stddev (AC: 3) ---

    #[test]
    fn test_population_stddev_in_display_buckets() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Three values: 10, 20, 30 -> mean=20, pop_stddev = sqrt(((10-20)^2+(20-20)^2+(30-20)^2)/3) = sqrt(200/3) ≈ 8.165
        let records = vec![
            make_comparison(0, 10.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 20.0, "2026-03-06T11:05:00Z"),
            make_comparison(0, 30.0, "2026-03-06T11:10:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets.len(), 1);
        let expected_stddev = (200.0_f64 / 3.0).sqrt();
        assert!(
            (buckets[0].stddev - expected_stddev).abs() < 1e-10,
            "Expected population stddev {}, got {}",
            expected_stddev,
            buckets[0].stddev
        );
    }

    #[test]
    fn test_single_record_stddev_is_zero() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        let records = vec![make_comparison(0, 20.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(buckets[0].stddev, 0.0);
    }

    // --- Separate EWMA pipeline (AC: 4) ---

    #[test]
    fn test_ewma_uses_session_pipeline_not_display() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Records spread across zones — EWMA should use session-gap buckets across all
        // Two sessions 3 hours apart, but one is in day zone and one in session zone
        let yesterday = today - SECS_PER_DAY + 20.0 * 3600.0; // yesterday 20:00
        let today_morning = today + 9.0 * 3600.0; // today 09:00

        let ts1 = epoch_to_iso8601(yesterday);
        let ts2 = epoch_to_iso8601(today_morning);

        let records = vec![
            make_comparison(0, 30.0, &ts1),
            make_comparison(0, 10.0, &ts2),
        ];
        tl.rebuild(&records, &[], now, today);

        // Display buckets should have 2 buckets (one Day, one Session)
        let display = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert_eq!(display.len(), 2);
        assert_eq!(display[0].bucket_size, BucketSize::Day);
        assert_eq!(display[1].bucket_size, BucketSize::Session);

        // EWMA should still be computed (from session pipeline which has 2 session buckets)
        let ewma = tl.current_ewma(TrainingMode::UnisonPitchComparison);
        assert!(ewma.is_some());
        let ewma_val = ewma.unwrap();
        assert!(ewma_val > 10.0 && ewma_val < 30.0);
    }

    #[test]
    fn test_ewma_two_buckets() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();
        // Two separate sessions (>30min gap)
        let records = vec![
            make_comparison(0, 30.0, &epoch_to_iso8601(now - 4.0 * 3600.0)),
            make_comparison(0, 10.0, &epoch_to_iso8601(now - 1.0 * 3600.0)),
        ];
        tl.rebuild(&records, &[], now, today);

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
        let today = today_epoch();
        let records = vec![make_comparison(0, 20.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&records, &[], now, today);

        let ewma = tl.current_ewma(TrainingMode::UnisonPitchComparison);
        assert_eq!(ewma, Some(20.0));
    }

    // --- Trend computation (AC: 5) ---

    #[test]
    fn test_trend_none_with_less_than_2_records() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();
        let records = vec![make_comparison(0, 20.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&records, &[], now, today);
        assert_eq!(tl.trend(TrainingMode::UnisonPitchComparison), None);
    }

    #[test]
    fn test_trend_improving() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();
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
        tl.rebuild(&records, &[], now, today);
        // Latest bucket mean (10.0) < ewma -> Improving
        let trend = tl.trend(TrainingMode::UnisonPitchComparison);
        assert_eq!(trend, Some(Trend::Improving));
    }

    #[test]
    fn test_trend_declining() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();
        // Low-value records spread across days, then a very high recent bucket
        let mut records = Vec::new();
        for i in 0..5 {
            let ts = epoch_to_iso8601(now - (10 - i) as f64 * 2.0 * 3600.0);
            records.push(make_comparison(0, 10.0, &ts));
        }
        // Recent session with very high values
        let ts = epoch_to_iso8601(now - 300.0);
        records.push(make_comparison(0, 100.0, &ts));
        tl.rebuild(&records, &[], now, today);
        let trend = tl.trend(TrainingMode::UnisonPitchComparison);
        assert_eq!(trend, Some(Trend::Declining));
    }

    #[test]
    fn test_trend_population_stddev() {
        // Verify trend uses population stddev (n divisor)
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Two records with values 10 and 20
        // running_mean = 15, pop_stddev = sqrt(((10-15)^2 + (20-15)^2)/2) = sqrt(25) = 5
        // latest bucket mean = 20 (last session)
        // 20 > 15 + 5 = 20 → NOT declining (not strictly >)
        // 20 >= ewma → Stable
        let records = vec![
            make_comparison(0, 10.0, &epoch_to_iso8601(now - 4.0 * 3600.0)),
            make_comparison(0, 20.0, &epoch_to_iso8601(now - 1.0 * 3600.0)),
        ];
        tl.rebuild(&records, &[], now, today);
        let trend = tl.trend(TrainingMode::UnisonPitchComparison);
        assert_eq!(trend, Some(Trend::Stable));
    }

    // --- TimeBucket and BucketSize ---

    #[test]
    fn test_time_bucket_fields() {
        let bucket = TimeBucket {
            period_start: 1000.0,
            period_end: 2000.0,
            bucket_size: BucketSize::Month,
            mean: 25.0,
            stddev: 5.0,
            record_count: 10,
        };
        assert_eq!(bucket.period_start, 1000.0);
        assert_eq!(bucket.period_end, 2000.0);
        assert_eq!(bucket.bucket_size, BucketSize::Month);
        assert_eq!(bucket.mean, 25.0);
        assert_eq!(bucket.stddev, 5.0);
        assert_eq!(bucket.record_count, 10);
    }

    #[test]
    fn test_bucket_size_variants() {
        let sizes = [BucketSize::Session, BucketSize::Day, BucketSize::Month];
        assert_eq!(sizes.len(), 3);
    }

    // --- Incremental comparison update ---

    #[test]
    fn test_add_comparison_updates_correct_mode() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        let record = make_comparison(0, 15.0, "2026-03-06T11:55:00Z");
        tl.add_comparison(&record, now, today);

        assert_eq!(
            tl.state(TrainingMode::UnisonPitchComparison),
            TrainingModeState::Active
        );
        assert_eq!(
            tl.state(TrainingMode::IntervalPitchComparison),
            TrainingModeState::NoData
        );
        assert!(
            tl.current_ewma(TrainingMode::UnisonPitchComparison)
                .is_some()
        );
    }

    // --- Incremental matching update ---

    #[test]
    fn test_add_matching_updates_correct_mode() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        let record = make_matching(0, 5.0, "2026-03-06T11:55:00Z");
        tl.add_matching(&record, now, today);

        assert_eq!(
            tl.state(TrainingMode::UnisonMatching),
            TrainingModeState::Active
        );
        assert_eq!(
            tl.state(TrainingMode::IntervalMatching),
            TrainingModeState::NoData
        );
    }

    #[test]
    fn test_incremental_session_stddev_matches_rebuild() {
        // Verify that incremental add_comparison produces the same bucket stddev as rebuild
        let now = now_epoch();
        let today = today_epoch();
        let ts1 = "2026-03-06T11:00:00Z";
        let ts2 = "2026-03-06T11:05:00Z";
        let ts3 = "2026-03-06T11:10:00Z";

        // Rebuild path
        let mut tl_rebuild = ProgressTimeline::new();
        let records = vec![
            make_comparison(0, 10.0, ts1),
            make_comparison(0, 20.0, ts2),
            make_comparison(0, 30.0, ts3),
        ];
        tl_rebuild.rebuild(&records, &[], now, today);

        // Incremental path
        let mut tl_incr = ProgressTimeline::new();
        tl_incr.add_comparison(&make_comparison(0, 10.0, ts1), now, today);
        tl_incr.add_comparison(&make_comparison(0, 20.0, ts2), now, today);
        tl_incr.add_comparison(&make_comparison(0, 30.0, ts3), now, today);

        let rebuild_buckets = tl_rebuild.display_buckets(TrainingMode::UnisonPitchComparison);
        let incr_buckets = tl_incr.display_buckets(TrainingMode::UnisonPitchComparison);

        assert_eq!(rebuild_buckets.len(), incr_buckets.len());
        for (rb, ib) in rebuild_buckets.iter().zip(incr_buckets.iter()) {
            assert!((rb.mean - ib.mean).abs() < 1e-10, "mean mismatch");
            assert!(
                (rb.stddev - ib.stddev).abs() < 1e-10,
                "stddev mismatch: rebuild={}, incremental={}",
                rb.stddev,
                ib.stddev
            );
        }
    }

    // --- Reset ---

    #[test]
    fn test_reset_clears_all_data() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        let records = vec![
            make_comparison(0, 15.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);
        assert_eq!(
            tl.state(TrainingMode::UnisonPitchComparison),
            TrainingModeState::Active
        );

        tl.reset();
        for mode in TrainingMode::ALL {
            assert_eq!(tl.state(mode), TrainingModeState::NoData);
            assert!(tl.display_buckets(mode).is_empty());
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

    // --- Empty zones (AC: 5.7) ---

    #[test]
    fn test_empty_session_zone() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Only historical data, nothing today
        let records = vec![
            make_comparison(0, 20.0, "2026-01-15T12:00:00Z"),
            make_comparison(0, 15.0, "2026-02-15T12:00:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        // Should have month buckets but no session buckets
        assert!(buckets.iter().all(|b| b.bucket_size == BucketSize::Month));
    }

    #[test]
    fn test_only_session_zone_data() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        // Only data from today
        let records = vec![
            make_comparison(0, 20.0, "2026-03-06T10:00:00Z"),
            make_comparison(0, 15.0, "2026-03-06T11:00:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);

        let buckets = tl.display_buckets(TrainingMode::UnisonPitchComparison);
        assert!(buckets.iter().all(|b| b.bucket_size == BucketSize::Session));
    }

    // --- Public API (AC: 4) ---

    #[test]
    fn test_latest_bucket_stddev() {
        let mut tl = ProgressTimeline::new();
        let now = now_epoch();
        let today = today_epoch();

        let records = vec![
            make_comparison(0, 10.0, "2026-03-06T11:00:00Z"),
            make_comparison(0, 20.0, "2026-03-06T11:05:00Z"),
            make_comparison(0, 30.0, "2026-03-06T11:10:00Z"),
        ];
        tl.rebuild(&records, &[], now, today);

        let stddev = tl.latest_bucket_stddev(TrainingMode::UnisonPitchComparison);
        assert!(stddev.is_some());
        let expected = (200.0_f64 / 3.0).sqrt();
        assert!((stddev.unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_latest_bucket_stddev_no_data() {
        let tl = ProgressTimeline::new();
        assert_eq!(
            tl.latest_bucket_stddev(TrainingMode::UnisonPitchComparison),
            None
        );
    }

    // --- floor_to_day helper ---

    #[test]
    fn test_floor_to_day() {
        let today = parse_iso8601_to_epoch("2026-03-06T00:00:00Z");

        // Timestamp at 15:30 on March 4 -> should floor to March 4 00:00
        let ts = parse_iso8601_to_epoch("2026-03-04T15:30:00Z");
        let floored = floor_to_day(ts, today);
        let expected = parse_iso8601_to_epoch("2026-03-04T00:00:00Z");
        assert!(
            (floored - expected).abs() < 1.0,
            "Expected {}, got {}",
            expected,
            floored
        );
    }

    // --- epoch_to_month_start helper ---

    #[test]
    fn test_epoch_to_month_start() {
        let ts = parse_iso8601_to_epoch("2026-01-15T12:00:00Z");
        let ms = epoch_to_month_start(ts);
        let expected = parse_iso8601_to_epoch("2026-01-01T00:00:00Z");
        assert!(
            (ms - expected).abs() < 1.0,
            "Expected {}, got {}",
            expected,
            ms
        );
    }

    // --- Verify cargo test -p domain passes (AC: 7) ---
    // This test's existence and passing confirms pure domain with no browser deps.
}
