use std::collections::HashMap;

use crate::records::TrainingRecord;
use crate::training_discipline::TrainingDiscipline;

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

/// Internal per-discipline display bucket state.
/// Trend, EWMA, and discipline state live in `TrainingDisciplineStatistics` via `PerceptualProfile`.
#[derive(Clone, Debug)]
struct DisciplineState {
    discipline: TrainingDiscipline,
    display_buckets: Vec<TimeBucket>,
    record_count: usize,
}

impl DisciplineState {
    fn new(discipline: TrainingDiscipline) -> Self {
        Self {
            discipline,
            display_buckets: Vec::new(),
            record_count: 0,
        }
    }

    fn reset(&mut self) {
        let discipline = self.discipline;
        *self = Self::new(discipline);
    }

    /// Note: if `start_of_today` changes between calls (midnight crossover), previously
    /// added records retain their original zone assignment. Call `rebuild()` to recategorize.
    fn add_point(&mut self, timestamp: f64, metric: f64, start_of_today: f64) {
        self.record_count += 1;

        let session_gap = self.discipline.config().session_gap_secs;

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
        && timestamp - last.period_end < session_gap
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

/// Pure three-zone time-bucketing layer for UI progress display.
///
/// Buckets training metrics into Session (today), Day (last 7 days),
/// and Month (older) zones. Trend, EWMA, and discipline state are owned by
/// `TrainingDisciplineStatistics` via `PerceptualProfile`.
#[derive(Clone, Debug)]
pub struct ProgressTimeline {
    disciplines: HashMap<TrainingDiscipline, DisciplineState>,
}

impl ProgressTimeline {
    pub fn new() -> Self {
        let mut disciplines = HashMap::new();
        for discipline in TrainingDiscipline::ALL {
            disciplines.insert(discipline, DisciplineState::new(discipline));
        }
        Self { disciplines }
    }

    /// Rebuild all per-discipline data from stored records.
    pub fn rebuild(&mut self, records: &[TrainingRecord], start_of_today: f64) {
        // Reset all disciplines
        for state in self.disciplines.values_mut() {
            state.reset();
        }

        // Collect (timestamp, metric, discipline) tuples from all records
        let mut points: Vec<(f64, f64, TrainingDiscipline)> = Vec::new();

        for record in records {
            let ts = parse_iso8601_to_epoch(record.timestamp());
            for discipline in TrainingDiscipline::ALL {
                let metric = match record {
                    TrainingRecord::PitchDiscrimination(r) => {
                        discipline.extract_discrimination_metric(r)
                    }
                    TrainingRecord::PitchMatching(r) => discipline.extract_matching_metric(r),
                };
                if let Some(m) = metric {
                    points.push((ts, m, discipline));
                }
            }
        }

        // Sort by timestamp
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Group by discipline
        let mut discipline_points: HashMap<TrainingDiscipline, Vec<(f64, f64)>> = HashMap::new();
        for (ts, metric, discipline) in points {
            discipline_points
                .entry(discipline)
                .or_default()
                .push((ts, metric));
        }

        // Build each discipline
        let day_zone_start = start_of_today - 7.0 * SECS_PER_DAY;
        for (discipline, pts) in discipline_points {
            let state = self.disciplines.get_mut(&discipline).unwrap();
            let session_gap = discipline.config().session_gap_secs;

            // Build display buckets (three-zone)
            state.display_buckets =
                build_display_buckets(&pts, start_of_today, session_gap, day_zone_start);

            state.record_count = pts.len();
        }
    }

    /// Returns three-zone display buckets for the given discipline.
    pub fn display_buckets(&self, discipline: TrainingDiscipline) -> Vec<TimeBucket> {
        self.disciplines
            .get(&discipline)
            .map(|s| s.display_buckets.clone())
            .unwrap_or_default()
    }

    /// Returns the stddev of the last display bucket (for headline ±).
    pub fn latest_bucket_stddev(&self, discipline: TrainingDiscipline) -> Option<f64> {
        self.disciplines
            .get(&discipline)
            .and_then(|s| s.display_buckets.last())
            .map(|b| b.stddev)
    }

    /// Add a metric point for a specific discipline.
    pub fn add_metric_for_discipline(
        &mut self,
        discipline: TrainingDiscipline,
        timestamp_secs: f64,
        metric: f64,
        start_of_today: f64,
    ) {
        debug_assert!(
            self.disciplines.contains_key(&discipline),
            "ProgressTimeline has no entry for discipline {discipline:?}"
        );
        if let Some(state) = self.disciplines.get_mut(&discipline) {
            state.add_point(timestamp_secs, metric, start_of_today);
        }
    }

    /// Incrementally update from a new training record of any type.
    pub fn add_record(&mut self, record: &TrainingRecord, start_of_today: f64) {
        let ts = parse_iso8601_to_epoch(record.timestamp());
        for discipline in TrainingDiscipline::ALL {
            let metric = match record {
                TrainingRecord::PitchDiscrimination(r) => {
                    discipline.extract_discrimination_metric(r)
                }
                TrainingRecord::PitchMatching(r) => discipline.extract_matching_metric(r),
            };
            if let Some(m) = metric
                && let Some(state) = self.disciplines.get_mut(&discipline)
            {
                state.add_point(ts, m, start_of_today);
            }
        }
    }

    /// Whether a discipline has any bucketed data.
    pub fn has_data(&self, discipline: TrainingDiscipline) -> bool {
        self.disciplines
            .get(&discipline)
            .is_some_and(|s| s.record_count > 0)
    }

    /// Clear all per-discipline data.
    pub fn reset(&mut self) {
        for state in self.disciplines.values_mut() {
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
pub fn parse_iso8601_to_epoch(timestamp: &str) -> f64 {
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
    use crate::records::{PitchDiscriminationRecord, PitchMatchingRecord};

    fn make_discrimination(interval: u8, cent_offset: f64, timestamp: &str) -> TrainingRecord {
        TrainingRecord::PitchDiscrimination(PitchDiscriminationRecord {
            reference_note: 60,
            target_note: 60 + interval,
            cent_offset,
            is_correct: true,
            interval,
            tuning_system: "equalTemperament".to_string(),
            timestamp: timestamp.to_string(),
        })
    }

    fn make_matching(interval: u8, user_cent_error: f64, timestamp: &str) -> TrainingRecord {
        TrainingRecord::PitchMatching(PitchMatchingRecord {
            reference_note: 60,
            target_note: 60 + interval,
            initial_cent_offset: 20.0,
            user_cent_error,
            interval,
            tuning_system: "equalTemperament".to_string(),
            timestamp: timestamp.to_string(),
        })
    }

    // Start of today: 2026-03-06T00:00:00Z
    fn today_epoch() -> f64 {
        parse_iso8601_to_epoch("2026-03-06T00:00:00Z")
    }

    // Epoch for 2026-03-06T12:00:00Z
    fn now_epoch() -> f64 {
        parse_iso8601_to_epoch("2026-03-06T12:00:00Z")
    }

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

    // --- Constructor ---

    #[test]
    fn test_new_creates_empty_timeline() {
        let tl = ProgressTimeline::new();
        for discipline in TrainingDiscipline::ALL {
            assert!(!tl.has_data(discipline));
            assert!(tl.display_buckets(discipline).is_empty());
        }
    }

    // --- Rebuild from records ---

    #[test]
    fn test_rebuild_with_empty_records() {
        let mut tl = ProgressTimeline::new();
        tl.rebuild(&[], today_epoch());
        for discipline in TrainingDiscipline::ALL {
            assert!(!tl.has_data(discipline));
        }
    }

    #[test]
    fn test_rebuild_comparison_unison_active_others_nodata() {
        let mut tl = ProgressTimeline::new();
        let records = vec![
            make_discrimination(0, 15.0, "2026-03-06T11:00:00Z"),
            make_discrimination(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, today_epoch());

        assert!(tl.has_data(TrainingDiscipline::UnisonPitchDiscrimination));
        assert!(!tl.has_data(TrainingDiscipline::IntervalPitchDiscrimination));
        assert!(!tl.has_data(TrainingDiscipline::UnisonPitchMatching));
        assert!(!tl.has_data(TrainingDiscipline::IntervalPitchMatching));
    }

    #[test]
    fn test_rebuild_matching_interval_active() {
        let mut tl = ProgressTimeline::new();
        let records = vec![make_matching(7, 5.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&records, today_epoch());

        assert!(tl.has_data(TrainingDiscipline::IntervalPitchMatching));
        assert!(!tl.has_data(TrainingDiscipline::UnisonPitchMatching));
    }

    // --- Session gap grouping ---

    #[test]
    fn test_session_gap_grouping() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();
        let records = vec![
            make_discrimination(0, 20.0, "2026-03-06T11:00:00Z"),
            make_discrimination(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Session);
        assert_eq!(buckets[0].record_count, 2);
        assert!((buckets[0].mean - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_session_gap_splits_sessions() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();
        let records = vec![
            make_discrimination(0, 20.0, "2026-03-06T09:00:00Z"),
            make_discrimination(0, 10.0, "2026-03-06T11:30:00Z"),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Session);
        assert_eq!(buckets[1].bucket_size, BucketSize::Session);
    }

    // --- Three-zone bucketing ---

    #[test]
    fn test_three_zone_bucketing() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let two_months_ago = today - 60.0 * SECS_PER_DAY;
        let ts_month = epoch_to_iso8601(two_months_ago);
        let three_days_ago = today - 3.0 * SECS_PER_DAY + 3600.0;
        let ts_day = epoch_to_iso8601(three_days_ago);
        let ts_session = "2026-03-06T11:00:00Z";

        let records = vec![
            make_discrimination(0, 30.0, &ts_month),
            make_discrimination(0, 20.0, &ts_day),
            make_discrimination(0, 10.0, ts_session),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert_eq!(buckets[1].bucket_size, BucketSize::Day);
        assert_eq!(buckets[2].bucket_size, BucketSize::Session);
    }

    #[test]
    fn test_day_zone_calendar_day_snapping() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let three_days_ago_morning = today - 3.0 * SECS_PER_DAY + 3600.0;
        let three_days_ago_evening = today - 3.0 * SECS_PER_DAY + 15.0 * 3600.0;
        let ts1 = epoch_to_iso8601(three_days_ago_morning);
        let ts2 = epoch_to_iso8601(three_days_ago_evening);

        let records = vec![
            make_discrimination(0, 20.0, &ts1),
            make_discrimination(0, 10.0, &ts2),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Day);
        assert_eq!(buckets[0].record_count, 2);
    }

    #[test]
    fn test_day_zone_different_days() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let two_days_ago = today - 2.0 * SECS_PER_DAY + 3600.0;
        let four_days_ago = today - 4.0 * SECS_PER_DAY + 3600.0;
        let ts1 = epoch_to_iso8601(four_days_ago);
        let ts2 = epoch_to_iso8601(two_days_ago);

        let records = vec![
            make_discrimination(0, 20.0, &ts1),
            make_discrimination(0, 10.0, &ts2),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Day);
        assert_eq!(buckets[1].bucket_size, BucketSize::Day);
    }

    #[test]
    fn test_month_zone_calendar_month_grouping() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let ts1 = "2026-01-10T12:00:00Z";
        let ts2 = "2026-01-20T15:00:00Z";

        let records = vec![
            make_discrimination(0, 30.0, ts1),
            make_discrimination(0, 20.0, ts2),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert_eq!(buckets[0].record_count, 2);

        let jan1 = parse_iso8601_to_epoch("2026-01-01T00:00:00Z");
        assert!((buckets[0].period_start - jan1).abs() < 1.0);
    }

    #[test]
    fn test_month_zone_different_months() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let ts1 = "2026-01-15T12:00:00Z";
        let ts2 = "2026-02-15T12:00:00Z";

        let records = vec![
            make_discrimination(0, 30.0, ts1),
            make_discrimination(0, 20.0, ts2),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert_eq!(buckets[1].bucket_size, BucketSize::Month);
    }

    #[test]
    fn test_monthly_bucket_end_truncated_to_day_zone_boundary() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();
        let day_zone_start = today - 7.0 * SECS_PER_DAY;

        let ts = "2026-02-25T12:00:00Z";

        let records = vec![make_discrimination(0, 20.0, ts)];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_size, BucketSize::Month);
        assert!(
            buckets[0].period_end <= day_zone_start,
            "Monthly bucket end {} should be <= day_zone_start {}",
            buckets[0].period_end,
            day_zone_start
        );
    }

    // --- Population stddev ---

    #[test]
    fn test_population_stddev_in_display_buckets() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let records = vec![
            make_discrimination(0, 10.0, "2026-03-06T11:00:00Z"),
            make_discrimination(0, 20.0, "2026-03-06T11:05:00Z"),
            make_discrimination(0, 30.0, "2026-03-06T11:10:00Z"),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
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
        let today = today_epoch();

        let records = vec![make_discrimination(0, 20.0, "2026-03-06T11:00:00Z")];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert_eq!(buckets[0].stddev, 0.0);
    }

    // --- Incremental discrimination update ---

    #[test]
    fn test_add_discrimination_updates_correct_mode() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let record = make_discrimination(0, 15.0, "2026-03-06T11:55:00Z");
        tl.add_record(&record, today);

        assert!(tl.has_data(TrainingDiscipline::UnisonPitchDiscrimination));
        assert!(!tl.has_data(TrainingDiscipline::IntervalPitchDiscrimination));
    }

    // --- Incremental matching update ---

    #[test]
    fn test_add_matching_updates_correct_mode() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let record = make_matching(0, 5.0, "2026-03-06T11:55:00Z");
        tl.add_record(&record, today);

        assert!(tl.has_data(TrainingDiscipline::UnisonPitchMatching));
        assert!(!tl.has_data(TrainingDiscipline::IntervalPitchMatching));
    }

    #[test]
    fn test_incremental_session_stddev_matches_rebuild() {
        let today = today_epoch();
        let ts1 = "2026-03-06T11:00:00Z";
        let ts2 = "2026-03-06T11:05:00Z";
        let ts3 = "2026-03-06T11:10:00Z";

        // Rebuild path
        let mut tl_rebuild = ProgressTimeline::new();
        let records = vec![
            make_discrimination(0, 10.0, ts1),
            make_discrimination(0, 20.0, ts2),
            make_discrimination(0, 30.0, ts3),
        ];
        tl_rebuild.rebuild(&records, today);

        // Incremental path
        let mut tl_incr = ProgressTimeline::new();
        tl_incr.add_record(&make_discrimination(0, 10.0, ts1), today);
        tl_incr.add_record(&make_discrimination(0, 20.0, ts2), today);
        tl_incr.add_record(&make_discrimination(0, 30.0, ts3), today);

        let rebuild_buckets =
            tl_rebuild.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        let incr_buckets = tl_incr.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);

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

    #[test]
    fn test_incremental_cross_zone_matches_rebuild() {
        let today = today_epoch();

        let ts_month = "2026-01-15T12:00:00Z";
        let three_days_ago = today - 3.0 * SECS_PER_DAY + 3600.0;
        let ts_day = epoch_to_iso8601(three_days_ago);
        let ts_session = "2026-03-06T11:00:00Z";

        // Rebuild path
        let mut tl_rebuild = ProgressTimeline::new();
        let records = vec![
            make_discrimination(0, 30.0, ts_month),
            make_discrimination(0, 20.0, &ts_day),
            make_discrimination(0, 10.0, ts_session),
        ];
        tl_rebuild.rebuild(&records, today);

        // Incremental path
        let mut tl_incr = ProgressTimeline::new();
        tl_incr.add_record(&make_discrimination(0, 30.0, ts_month), today);
        tl_incr.add_record(&make_discrimination(0, 20.0, &ts_day), today);
        tl_incr.add_record(&make_discrimination(0, 10.0, ts_session), today);

        let rebuild_buckets =
            tl_rebuild.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        let incr_buckets = tl_incr.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);

        assert_eq!(
            rebuild_buckets.len(),
            incr_buckets.len(),
            "bucket count mismatch: rebuild={}, incremental={}",
            rebuild_buckets.len(),
            incr_buckets.len()
        );
        for (i, (rb, ib)) in rebuild_buckets.iter().zip(incr_buckets.iter()).enumerate() {
            assert_eq!(rb.bucket_size, ib.bucket_size, "bucket {} size mismatch", i);
            assert!(
                (rb.mean - ib.mean).abs() < 1e-10,
                "bucket {} mean mismatch: rebuild={}, incremental={}",
                i,
                rb.mean,
                ib.mean
            );
            assert!(
                (rb.stddev - ib.stddev).abs() < 1e-10,
                "bucket {} stddev mismatch: rebuild={}, incremental={}",
                i,
                rb.stddev,
                ib.stddev
            );
        }
    }

    // --- Reset ---

    #[test]
    fn test_reset_clears_all_data() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let records = vec![
            make_discrimination(0, 15.0, "2026-03-06T11:00:00Z"),
            make_discrimination(0, 10.0, "2026-03-06T11:05:00Z"),
        ];
        tl.rebuild(&records, today);
        assert!(tl.has_data(TrainingDiscipline::UnisonPitchDiscrimination));

        tl.reset();
        for discipline in TrainingDiscipline::ALL {
            assert!(!tl.has_data(discipline));
            assert!(tl.display_buckets(discipline).is_empty());
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

    // --- Empty zones ---

    #[test]
    fn test_empty_session_zone() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let records = vec![
            make_discrimination(0, 20.0, "2026-01-15T12:00:00Z"),
            make_discrimination(0, 15.0, "2026-02-15T12:00:00Z"),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert!(buckets.iter().all(|b| b.bucket_size == BucketSize::Month));
    }

    #[test]
    fn test_only_session_zone_data() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let records = vec![
            make_discrimination(0, 20.0, "2026-03-06T10:00:00Z"),
            make_discrimination(0, 15.0, "2026-03-06T11:00:00Z"),
        ];
        tl.rebuild(&records, today);

        let buckets = tl.display_buckets(TrainingDiscipline::UnisonPitchDiscrimination);
        assert!(buckets.iter().all(|b| b.bucket_size == BucketSize::Session));
    }

    // --- Public API ---

    #[test]
    fn test_latest_bucket_stddev() {
        let mut tl = ProgressTimeline::new();
        let today = today_epoch();

        let records = vec![
            make_discrimination(0, 10.0, "2026-03-06T11:00:00Z"),
            make_discrimination(0, 20.0, "2026-03-06T11:05:00Z"),
            make_discrimination(0, 30.0, "2026-03-06T11:10:00Z"),
        ];
        tl.rebuild(&records, today);

        let stddev = tl.latest_bucket_stddev(TrainingDiscipline::UnisonPitchDiscrimination);
        assert!(stddev.is_some());
        let expected = (200.0_f64 / 3.0).sqrt();
        assert!((stddev.unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_latest_bucket_stddev_no_data() {
        let tl = ProgressTimeline::new();
        assert_eq!(
            tl.latest_bucket_stddev(TrainingDiscipline::UnisonPitchDiscrimination),
            None
        );
    }

    // --- TimeBucket ---

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

    // --- Helpers ---

    #[test]
    fn test_floor_to_day() {
        let today = parse_iso8601_to_epoch("2026-03-06T00:00:00Z");
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
}
