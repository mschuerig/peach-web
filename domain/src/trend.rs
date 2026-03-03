use serde::{Deserialize, Serialize};

/// Minimum number of data points required for trend analysis.
const MINIMUM_RECORD_COUNT: usize = 20;

/// Percentage change threshold to determine improving/declining vs stable.
const CHANGE_THRESHOLD: f64 = 0.05;

/// Direction of perceptual performance trend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Trend {
    Improving,
    Stable,
    Declining,
}

/// Analyzes trends in absolute cent offsets over time using half-split comparison.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendAnalyzer {
    abs_offsets: Vec<f64>,
}

impl TrendAnalyzer {
    pub fn new() -> Self {
        Self {
            abs_offsets: Vec::new(),
        }
    }

    /// Append an absolute offset value.
    pub fn push(&mut self, abs_offset: f64) {
        self.abs_offsets.push(abs_offset);
    }

    /// Analyze the trend by splitting data in half and comparing means.
    /// Returns `None` if fewer than `MINIMUM_RECORD_COUNT` data points.
    pub fn trend(&self) -> Option<Trend> {
        if self.abs_offsets.len() < MINIMUM_RECORD_COUNT {
            return None;
        }

        let mid = self.abs_offsets.len() / 2;
        let first_half = &self.abs_offsets[..mid];
        let second_half = &self.abs_offsets[mid..];

        let first_mean = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_mean = second_half.iter().sum::<f64>() / second_half.len() as f64;

        if first_mean == 0.0 {
            return Some(Trend::Stable);
        }

        let change_ratio = (second_mean - first_mean) / first_mean;

        if change_ratio < -CHANGE_THRESHOLD {
            Some(Trend::Improving) // lower thresholds = better discrimination
        } else if change_ratio > CHANGE_THRESHOLD {
            Some(Trend::Declining)
        } else {
            Some(Trend::Stable)
        }
    }

    /// Clear all recorded data.
    pub fn reset(&mut self) {
        self.abs_offsets.clear();
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- AC12: Insufficient data ---

    #[test]
    fn test_trend_none_with_zero_data() {
        let analyzer = TrendAnalyzer::new();
        assert_eq!(analyzer.trend(), None);
    }

    #[test]
    fn test_trend_none_with_19_points() {
        let mut analyzer = TrendAnalyzer::new();
        for i in 0..19 {
            analyzer.push(i as f64);
        }
        assert_eq!(analyzer.trend(), None);
    }

    #[test]
    fn test_trend_some_with_20_points() {
        let mut analyzer = TrendAnalyzer::new();
        for _ in 0..20 {
            analyzer.push(50.0);
        }
        assert!(analyzer.trend().is_some());
    }

    // --- AC13: Improving ---

    #[test]
    fn test_trend_improving() {
        let mut analyzer = TrendAnalyzer::new();
        // First half: high values (poor performance)
        for _ in 0..10 {
            analyzer.push(50.0);
        }
        // Second half: >5% lower (improved)
        for _ in 0..10 {
            analyzer.push(40.0);
        }
        assert_eq!(analyzer.trend(), Some(Trend::Improving));
    }

    // --- Stable ---

    #[test]
    fn test_trend_stable() {
        let mut analyzer = TrendAnalyzer::new();
        // Both halves similar
        for _ in 0..10 {
            analyzer.push(50.0);
        }
        for _ in 0..10 {
            analyzer.push(49.0);
        }
        assert_eq!(analyzer.trend(), Some(Trend::Stable));
    }

    // --- Declining ---

    #[test]
    fn test_trend_declining() {
        let mut analyzer = TrendAnalyzer::new();
        // First half: low values (good performance)
        for _ in 0..10 {
            analyzer.push(30.0);
        }
        // Second half: >5% higher (worse)
        for _ in 0..10 {
            analyzer.push(45.0);
        }
        assert_eq!(analyzer.trend(), Some(Trend::Declining));
    }

    // --- Reset ---

    #[test]
    fn test_reset_clears_data() {
        let mut analyzer = TrendAnalyzer::new();
        for _ in 0..20 {
            analyzer.push(50.0);
        }
        assert!(analyzer.trend().is_some());

        analyzer.reset();
        assert_eq!(analyzer.trend(), None);
    }

    // --- Serde ---

    #[test]
    fn test_trend_enum_serde() {
        let json = serde_json::to_string(&Trend::Improving).unwrap();
        assert_eq!(json, "\"improving\"");

        let parsed: Trend = serde_json::from_str("\"declining\"").unwrap();
        assert_eq!(parsed, Trend::Declining);
    }

    #[test]
    fn test_trend_analyzer_serde_roundtrip() {
        let mut analyzer = TrendAnalyzer::new();
        analyzer.push(10.0);
        analyzer.push(20.0);

        let json = serde_json::to_string(&analyzer).unwrap();
        let parsed: TrendAnalyzer = serde_json::from_str(&json).unwrap();
        assert_eq!(analyzer, parsed);
    }

    // --- Default ---

    #[test]
    fn test_default() {
        let analyzer = TrendAnalyzer::default();
        assert_eq!(analyzer.trend(), None);
    }
}
