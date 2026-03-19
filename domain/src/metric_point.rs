use crate::welford::WelfordMeasurement;

/// A timestamped measurement value for tracking training progress over time.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricPoint<M: WelfordMeasurement> {
    pub timestamp: f64,
    pub value: M,
}

impl<M: WelfordMeasurement> MetricPoint<M> {
    pub fn new(timestamp: f64, value: M) -> Self {
        Self { timestamp, value }
    }

    pub fn statistical_value(&self) -> f64 {
        self.value.statistical_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Cents;

    #[test]
    fn test_metric_point_construction() {
        let p = MetricPoint::new(1000.0, Cents::new(42.0));
        assert_eq!(p.timestamp, 1000.0);
        assert_eq!(p.value, Cents::new(42.0));
    }

    #[test]
    fn test_statistical_value() {
        let p = MetricPoint::new(1000.0, Cents::new(42.0));
        assert!((p.statistical_value() - 42.0).abs() < 1e-10);
    }
}
