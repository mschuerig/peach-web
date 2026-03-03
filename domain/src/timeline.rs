use serde::{Deserialize, Serialize};

/// A single data point recorded during training.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimelineDataPoint {
    timestamp: String,
    cent_offset: f64,
    is_correct: bool,
    reference_note: u8,
}

impl TimelineDataPoint {
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    pub fn cent_offset(&self) -> f64 {
        self.cent_offset
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn reference_note(&self) -> u8 {
        self.reference_note
    }
}

/// Aggregated statistics for a time period (day).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeriodAggregate {
    period_start: String,
    mean_threshold: f64,
    comparison_count: u32,
    correct_count: u32,
}

impl PeriodAggregate {
    pub fn period_start(&self) -> &str {
        &self.period_start
    }

    pub fn mean_threshold(&self) -> f64 {
        self.mean_threshold
    }

    pub fn comparison_count(&self) -> u32 {
        self.comparison_count
    }

    pub fn correct_count(&self) -> u32 {
        self.correct_count
    }
}

/// Timeline of threshold data points with daily aggregation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdTimeline {
    data_points: Vec<TimelineDataPoint>,
}

impl ThresholdTimeline {
    pub fn new() -> Self {
        Self {
            data_points: Vec::new(),
        }
    }

    /// Record a new data point.
    pub fn push(
        &mut self,
        timestamp: &str,
        cent_offset: f64,
        is_correct: bool,
        reference_note: u8,
    ) {
        assert!(
            timestamp.len() >= 10,
            "timestamp must be ISO 8601 format (at least 10 chars), got: {timestamp}"
        );
        self.data_points.push(TimelineDataPoint {
            timestamp: timestamp.to_string(),
            cent_offset,
            is_correct,
            reference_note,
        });
    }

    /// Aggregate data points by day (first 10 characters of ISO 8601 timestamp).
    pub fn aggregate_daily(&self) -> Vec<PeriodAggregate> {
        use std::collections::BTreeMap;

        if self.data_points.is_empty() {
            return Vec::new();
        }

        // Group by date portion (first 10 chars of ISO 8601: "YYYY-MM-DD")
        let mut groups: BTreeMap<&str, Vec<&TimelineDataPoint>> = BTreeMap::new();

        for dp in &self.data_points {
            let date = &dp.timestamp[..10];
            groups.entry(date).or_default().push(dp);
        }

        groups
            .into_iter()
            .map(|(date, points)| {
                let count = points.len() as u32;
                let sum: f64 = points.iter().map(|p| p.cent_offset).sum();
                let correct: u32 = points.iter().filter(|p| p.is_correct).count() as u32;

                PeriodAggregate {
                    period_start: date.to_string(),
                    mean_threshold: sum / count as f64,
                    comparison_count: count,
                    correct_count: correct,
                }
            })
            .collect()
    }

    /// Clear all recorded data.
    pub fn reset(&mut self) {
        self.data_points.clear();
    }
}

impl Default for ThresholdTimeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- AC14: Data recording ---

    #[test]
    fn test_push_records_data_point() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("2026-03-03T14:00:00Z", 50.0, true, 60);

        let agg = timeline.aggregate_daily();
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].period_start(), "2026-03-03");
        assert!((agg[0].mean_threshold() - 50.0).abs() < 1e-10);
        assert_eq!(agg[0].comparison_count(), 1);
        assert_eq!(agg[0].correct_count(), 1);
    }

    #[test]
    fn test_daily_aggregation_single_day() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("2026-03-03T14:00:00Z", 40.0, true, 60);
        timeline.push("2026-03-03T15:00:00Z", 60.0, false, 72);
        timeline.push("2026-03-03T16:00:00Z", 50.0, true, 60);

        let agg = timeline.aggregate_daily();
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].period_start(), "2026-03-03");
        // Mean: (40 + 60 + 50) / 3 = 50
        assert!((agg[0].mean_threshold() - 50.0).abs() < 1e-10);
        assert_eq!(agg[0].comparison_count(), 3);
        assert_eq!(agg[0].correct_count(), 2);
    }

    #[test]
    fn test_daily_aggregation_multiple_days() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("2026-03-03T14:00:00Z", 40.0, true, 60);
        timeline.push("2026-03-03T15:00:00Z", 60.0, false, 72);
        timeline.push("2026-03-04T10:00:00Z", 30.0, true, 60);
        timeline.push("2026-03-04T11:00:00Z", 50.0, true, 65);

        let agg = timeline.aggregate_daily();
        assert_eq!(agg.len(), 2);

        // Day 1
        assert_eq!(agg[0].period_start(), "2026-03-03");
        assert!((agg[0].mean_threshold() - 50.0).abs() < 1e-10);
        assert_eq!(agg[0].comparison_count(), 2);
        assert_eq!(agg[0].correct_count(), 1);

        // Day 2
        assert_eq!(agg[1].period_start(), "2026-03-04");
        assert!((agg[1].mean_threshold() - 40.0).abs() < 1e-10);
        assert_eq!(agg[1].comparison_count(), 2);
        assert_eq!(agg[1].correct_count(), 2);
    }

    #[test]
    fn test_aggregate_empty_timeline() {
        let timeline = ThresholdTimeline::new();
        assert!(timeline.aggregate_daily().is_empty());
    }

    // --- Reset ---

    #[test]
    fn test_reset_clears_all_data() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("2026-03-03T14:00:00Z", 50.0, true, 60);
        timeline.push("2026-03-03T15:00:00Z", 30.0, false, 72);

        timeline.reset();
        assert!(timeline.aggregate_daily().is_empty());
    }

    // --- Serde ---

    #[test]
    fn test_timeline_data_point_serde_roundtrip() {
        let dp = TimelineDataPoint {
            timestamp: "2026-03-03T14:00:00Z".to_string(),
            cent_offset: 50.0,
            is_correct: true,
            reference_note: 60,
        };
        let json = serde_json::to_string(&dp).unwrap();
        let parsed: TimelineDataPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(dp, parsed);
    }

    #[test]
    fn test_period_aggregate_serde_roundtrip() {
        let agg = PeriodAggregate {
            period_start: "2026-03-03".to_string(),
            mean_threshold: 45.5,
            comparison_count: 10,
            correct_count: 7,
        };
        let json = serde_json::to_string(&agg).unwrap();
        let parsed: PeriodAggregate = serde_json::from_str(&json).unwrap();
        assert_eq!(agg, parsed);
    }

    #[test]
    fn test_threshold_timeline_serde_roundtrip() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("2026-03-03T14:00:00Z", 50.0, true, 60);

        let json = serde_json::to_string(&timeline).unwrap();
        let parsed: ThresholdTimeline = serde_json::from_str(&json).unwrap();
        assert_eq!(timeline, parsed);
    }

    // --- Default ---

    #[test]
    fn test_default() {
        let timeline = ThresholdTimeline::default();
        assert!(timeline.aggregate_daily().is_empty());
    }

    // --- Input validation ---

    #[test]
    #[should_panic(expected = "timestamp must be ISO 8601")]
    fn test_push_panics_on_short_timestamp() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("short", 50.0, true, 60);
    }

    // --- Non-chronological grouping ---

    #[test]
    fn test_aggregate_daily_non_chronological() {
        let mut timeline = ThresholdTimeline::new();
        timeline.push("2026-03-03T14:00:00Z", 40.0, true, 60);
        timeline.push("2026-03-04T10:00:00Z", 60.0, false, 72);
        timeline.push("2026-03-03T16:00:00Z", 50.0, true, 65);

        let agg = timeline.aggregate_daily();
        assert_eq!(agg.len(), 2);

        // Day 1: (40 + 50) / 2 = 45
        assert_eq!(agg[0].period_start(), "2026-03-03");
        assert_eq!(agg[0].comparison_count(), 2);
        assert!((agg[0].mean_threshold() - 45.0).abs() < 1e-10);

        // Day 2: 60
        assert_eq!(agg[1].period_start(), "2026-03-04");
        assert_eq!(agg[1].comparison_count(), 1);
    }
}
