/// Generic online accumulator implementing Welford's single-pass algorithm.
/// Computes running mean and variance in O(1) per update with numerical stability.
#[derive(Clone, Debug, PartialEq)]
pub struct WelfordAccumulator {
    count: usize,
    mean: f64,
    m2: f64,
}

impl WelfordAccumulator {
    pub fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    /// Add one measurement to the accumulator.
    ///
    /// # Panics
    /// Debug-panics if `value` is NaN or infinity — these poison the running
    /// mean and variance irreversibly.
    pub fn update(&mut self, value: f64) {
        debug_assert!(
            value.is_finite(),
            "WelfordAccumulator::update called with non-finite value: {value}"
        );
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Running mean. Returns `0.0` when the accumulator is empty (`count == 0`).
    ///
    /// Callers that need to distinguish "no data" from "mean is zero" should
    /// check `count() > 0` first.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Population standard deviation (None if fewer than 2 samples).
    pub fn population_std_dev(&self) -> Option<f64> {
        if self.count >= 2 {
            Some((self.m2 / self.count as f64).sqrt())
        } else {
            None
        }
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.count = 0;
        self.mean = 0.0;
        self.m2 = 0.0;
    }
}

impl Default for WelfordAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_accumulator() {
        let acc = WelfordAccumulator::new();
        assert_eq!(acc.count(), 0);
        assert_eq!(acc.population_std_dev(), None);
    }

    #[test]
    fn test_single_value() {
        let mut acc = WelfordAccumulator::new();
        acc.update(42.0);
        assert_eq!(acc.count(), 1);
        assert!((acc.mean() - 42.0).abs() < 1e-10);
        assert_eq!(acc.population_std_dev(), None); // needs >= 2
    }

    #[test]
    fn test_two_values() {
        let mut acc = WelfordAccumulator::new();
        acc.update(10.0);
        acc.update(20.0);
        assert_eq!(acc.count(), 2);
        assert!((acc.mean() - 15.0).abs() < 1e-10);
        // pop stddev of [10, 20] = sqrt(((10-15)^2+(20-15)^2)/2) = sqrt(25) = 5
        assert!((acc.population_std_dev().unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_three_values() {
        let mut acc = WelfordAccumulator::new();
        acc.update(10.0);
        acc.update(20.0);
        acc.update(30.0);
        assert_eq!(acc.count(), 3);
        assert!((acc.mean() - 20.0).abs() < 1e-10);
        let expected_std = (200.0_f64 / 3.0).sqrt();
        assert!((acc.population_std_dev().unwrap() - expected_std).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut acc = WelfordAccumulator::new();
        acc.update(50.0);
        acc.update(60.0);
        acc.reset();
        assert_eq!(acc.count(), 0);
        assert!((acc.mean() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_raw_mean_accessor() {
        let mut acc = WelfordAccumulator::new();
        acc.update(10.0);
        acc.update(30.0);
        assert!((acc.mean() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_population_std_dev() {
        let mut acc = WelfordAccumulator::new();
        acc.update(10.0);
        acc.update(20.0);
        let std = acc.population_std_dev().unwrap();
        assert!((std - 5.0).abs() < 1e-10);
    }
}
