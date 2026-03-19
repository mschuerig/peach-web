use serde::{Deserialize, Serialize};

/// Direction of perceptual performance trend.
/// Computed per-mode in `TrainingModeStatistics` by comparing
/// the latest metric value against population statistics and EWMA.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Trend {
    Improving,
    Stable,
    Declining,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_enum_serde() {
        let json = serde_json::to_string(&Trend::Improving).unwrap();
        assert_eq!(json, "\"improving\"");

        let parsed: Trend = serde_json::from_str("\"declining\"").unwrap();
        assert_eq!(parsed, Trend::Declining);
    }

    #[test]
    fn test_trend_stable_serde() {
        let json = serde_json::to_string(&Trend::Stable).unwrap();
        assert_eq!(json, "\"stable\"");
    }
}
