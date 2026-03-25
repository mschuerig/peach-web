/// A timestamped measurement value for tracking training progress over time.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricPoint {
    pub timestamp: f64,
    pub value: f64,
}

impl MetricPoint {
    /// Create a new metric point.
    ///
    /// # Panics
    /// Debug-panics if `value` is negative — the statistics engine assumes
    /// absolute-error (non-negative) values.
    pub fn new(timestamp: f64, value: f64) -> Self {
        debug_assert!(
            value >= 0.0,
            "MetricPoint::new called with negative value: {value}"
        );
        Self { timestamp, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_point_construction() {
        let p = MetricPoint::new(1000.0, 42.0);
        assert_eq!(p.timestamp, 1000.0);
        assert!((p.value - 42.0).abs() < 1e-10);
    }
}
