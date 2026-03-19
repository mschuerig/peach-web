use crate::types::Cents;

/// Bridges domain types to f64 for Welford's algorithm.
/// Conforming types can be accumulated in a `WelfordAccumulator<M>`.
pub trait WelfordMeasurement: Copy + Clone + std::fmt::Debug + PartialEq {
    fn statistical_value(&self) -> f64;
    fn from_statistical_value(value: f64) -> Self;
}

impl WelfordMeasurement for Cents {
    fn statistical_value(&self) -> f64 {
        self.raw_value
    }

    fn from_statistical_value(value: f64) -> Self {
        Cents::new(value)
    }
}

/// Generic online accumulator implementing Welford's single-pass algorithm.
/// Computes running mean and variance in O(1) per update with numerical stability.
#[derive(Clone, Debug, PartialEq)]
pub struct WelfordAccumulator<M: WelfordMeasurement> {
    count: usize,
    mean: f64,
    m2: f64,
    _phantom: std::marker::PhantomData<M>,
}

impl<M: WelfordMeasurement> WelfordAccumulator<M> {
    pub fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add one measurement to the accumulator.
    pub fn update(&mut self, value: M) {
        self.count += 1;
        let v = value.statistical_value();
        let delta = v - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = v - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Raw mean as f64.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Mean wrapped in the original measurement type (None if no data).
    pub fn typed_mean(&self) -> Option<M> {
        if self.count > 0 {
            Some(M::from_statistical_value(self.mean))
        } else {
            None
        }
    }

    /// Population standard deviation (None if fewer than 2 samples).
    pub fn population_std_dev(&self) -> Option<f64> {
        if self.count >= 2 {
            Some((self.m2 / self.count as f64).sqrt())
        } else {
            None
        }
    }

    /// Population standard deviation wrapped in measurement type.
    pub fn typed_std_dev(&self) -> Option<M> {
        self.population_std_dev().map(M::from_statistical_value)
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.count = 0;
        self.mean = 0.0;
        self.m2 = 0.0;
    }
}

impl<M: WelfordMeasurement> Default for WelfordAccumulator<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_accumulator() {
        let acc = WelfordAccumulator::<Cents>::new();
        assert_eq!(acc.count(), 0);
        assert_eq!(acc.typed_mean(), None);
        assert_eq!(acc.typed_std_dev(), None);
    }

    #[test]
    fn test_single_value() {
        let mut acc = WelfordAccumulator::<Cents>::new();
        acc.update(Cents::new(42.0));
        assert_eq!(acc.count(), 1);
        assert_eq!(acc.typed_mean(), Some(Cents::new(42.0)));
        assert_eq!(acc.typed_std_dev(), None); // needs >= 2
    }

    #[test]
    fn test_two_values() {
        let mut acc = WelfordAccumulator::<Cents>::new();
        acc.update(Cents::new(10.0));
        acc.update(Cents::new(20.0));
        assert_eq!(acc.count(), 2);
        assert!((acc.mean() - 15.0).abs() < 1e-10);
        // pop stddev of [10, 20] = sqrt(((10-15)^2+(20-15)^2)/2) = sqrt(25) = 5
        assert!((acc.population_std_dev().unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_three_values() {
        let mut acc = WelfordAccumulator::<Cents>::new();
        acc.update(Cents::new(10.0));
        acc.update(Cents::new(20.0));
        acc.update(Cents::new(30.0));
        assert_eq!(acc.count(), 3);
        assert!((acc.mean() - 20.0).abs() < 1e-10);
        let expected_std = (200.0_f64 / 3.0).sqrt();
        assert!((acc.population_std_dev().unwrap() - expected_std).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut acc = WelfordAccumulator::<Cents>::new();
        acc.update(Cents::new(50.0));
        acc.update(Cents::new(60.0));
        acc.reset();
        assert_eq!(acc.count(), 0);
        assert_eq!(acc.typed_mean(), None);
    }

    #[test]
    fn test_raw_mean_accessor() {
        let mut acc = WelfordAccumulator::<Cents>::new();
        acc.update(Cents::new(10.0));
        acc.update(Cents::new(30.0));
        assert!((acc.mean() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_typed_std_dev() {
        let mut acc = WelfordAccumulator::<Cents>::new();
        acc.update(Cents::new(10.0));
        acc.update(Cents::new(20.0));
        let std = acc.typed_std_dev().unwrap();
        assert!((std.raw_value - 5.0).abs() < 1e-10);
    }
}
