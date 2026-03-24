/// A timestamped measurement value for tracking training progress over time.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricPoint {
    pub timestamp: f64,
    pub value: f64,
}

impl MetricPoint {
    pub fn new(timestamp: f64, value: f64) -> Self {
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
