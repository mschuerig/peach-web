use crate::types::{RhythmOffset, StepPosition, TempoBPM};

/// Result of a single cycle within a continuous rhythm matching trial.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CycleResult {
    /// User tapped within the acceptance window; offset is signed ms from gap time.
    Hit(RhythmOffset),
    /// No tap occurred before the next cycle started.
    Miss,
}

/// A completed continuous rhythm matching trial aggregated from 16 cycles.
#[derive(Clone, Debug, PartialEq)]
pub struct CompletedContinuousRhythmMatchingTrial {
    tempo: TempoBPM,
    mean_offset_ms: f64,
    hit_rate: f64,
    per_position_mean_ms: [Option<f64>; 4],
    cycle_count: u16,
    timestamp: String,
}

impl CompletedContinuousRhythmMatchingTrial {
    pub fn new(
        tempo: TempoBPM,
        mean_offset_ms: f64,
        hit_rate: f64,
        per_position_mean_ms: [Option<f64>; 4],
        cycle_count: u16,
        timestamp: String,
    ) -> Self {
        assert!(!timestamp.is_empty(), "timestamp must not be empty");
        assert!(mean_offset_ms.is_finite(), "mean_offset_ms must be finite");
        assert!(
            hit_rate.is_finite() && (0.0..=1.0).contains(&hit_rate),
            "hit_rate must be between 0.0 and 1.0"
        );
        Self {
            tempo,
            mean_offset_ms,
            hit_rate,
            per_position_mean_ms,
            cycle_count,
            timestamp,
        }
    }

    pub fn tempo(&self) -> TempoBPM {
        self.tempo
    }

    pub fn mean_offset_ms(&self) -> f64 {
        self.mean_offset_ms
    }

    pub fn hit_rate(&self) -> f64 {
        self.hit_rate
    }

    pub fn per_position_mean_ms(&self) -> [Option<f64>; 4] {
        self.per_position_mean_ms
    }

    pub fn cycle_count(&self) -> u16 {
        self.cycle_count
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// Extract the metric value: absolute mean offset as percentage of one sixteenth note.
    pub fn metric_value(&self) -> f64 {
        let offset = RhythmOffset::new(self.mean_offset_ms);
        offset.percentage_of_sixteenth(self.tempo)
    }
}

/// Number of cycles per trial.
pub const CYCLES_PER_TRIAL: u16 = 16;

/// Aggregate 16 cycle results into a completed trial.
///
/// Returns `None` if there are no hits — all-miss trials are discarded
/// (matching iOS behavior where trials with zero hits are silently skipped).
/// The mean_offset_ms is the mean of signed hit offsets.
pub fn aggregate_trial(
    tempo: TempoBPM,
    cycles: &[(StepPosition, CycleResult)],
    timestamp: String,
) -> Option<CompletedContinuousRhythmMatchingTrial> {
    assert_eq!(
        cycles.len(),
        CYCLES_PER_TRIAL as usize,
        "trial must have exactly {} cycles",
        CYCLES_PER_TRIAL
    );

    let mut total_hits = 0u16;
    let mut total_offset_ms = 0.0f64;

    // Per-position accumulators: (sum_offset_ms, count)
    let mut per_position: [(f64, u16); 4] = [(0.0, 0); 4];

    for (position, result) in cycles {
        if let CycleResult::Hit(offset) = result {
            total_hits += 1;
            total_offset_ms += offset.ms();

            let idx = position_index(*position);
            per_position[idx].0 += offset.ms();
            per_position[idx].1 += 1;
        }
    }

    if total_hits == 0 {
        // All misses — discard trial (no meaningful metric to record)
        return None;
    }

    let mean_offset_ms = total_offset_ms / total_hits as f64;
    let hit_rate = total_hits as f64 / CYCLES_PER_TRIAL as f64;

    let per_position_mean_ms = [
        position_mean(&per_position[0]),
        position_mean(&per_position[1]),
        position_mean(&per_position[2]),
        position_mean(&per_position[3]),
    ];

    Some(CompletedContinuousRhythmMatchingTrial::new(
        tempo,
        mean_offset_ms,
        hit_rate,
        per_position_mean_ms,
        CYCLES_PER_TRIAL,
        timestamp,
    ))
}

fn position_index(pos: StepPosition) -> usize {
    match pos {
        StepPosition::First => 0,
        StepPosition::Second => 1,
        StepPosition::Third => 2,
        StepPosition::Fourth => 3,
    }
}

fn position_mean(acc: &(f64, u16)) -> Option<f64> {
    if acc.1 == 0 {
        None
    } else {
        Some(acc.0 / acc.1 as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completed_trial_fields_accessible() {
        let trial = CompletedContinuousRhythmMatchingTrial::new(
            TempoBPM::new(80),
            5.0,
            0.75,
            [Some(3.0), Some(7.0), None, Some(5.0)],
            16,
            "2026-03-25T12:00:00Z".to_string(),
        );
        assert_eq!(trial.tempo().bpm(), 80);
        assert_eq!(trial.mean_offset_ms(), 5.0);
        assert_eq!(trial.hit_rate(), 0.75);
        assert_eq!(trial.cycle_count(), 16);
        assert_eq!(trial.timestamp(), "2026-03-25T12:00:00Z");
        assert_eq!(trial.per_position_mean_ms()[0], Some(3.0));
        assert_eq!(trial.per_position_mean_ms()[2], None);
    }

    #[test]
    fn test_metric_value_at_80bpm() {
        // 9.375ms at 80 BPM = 5.0% of a sixteenth note
        let trial = CompletedContinuousRhythmMatchingTrial::new(
            TempoBPM::new(80),
            9.375,
            1.0,
            [None; 4],
            16,
            "2026-03-25T12:00:00Z".to_string(),
        );
        assert!((trial.metric_value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_negative_offset() {
        let trial = CompletedContinuousRhythmMatchingTrial::new(
            TempoBPM::new(80),
            -9.375,
            1.0,
            [None; 4],
            16,
            "2026-03-25T12:00:00Z".to_string(),
        );
        assert!((trial.metric_value() - 5.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "timestamp must not be empty")]
    fn test_empty_timestamp_panics() {
        CompletedContinuousRhythmMatchingTrial::new(
            TempoBPM::new(80),
            0.0,
            0.5,
            [None; 4],
            16,
            String::new(),
        );
    }

    #[test]
    #[should_panic(expected = "hit_rate must be between 0.0 and 1.0")]
    fn test_invalid_hit_rate_panics() {
        CompletedContinuousRhythmMatchingTrial::new(
            TempoBPM::new(80),
            0.0,
            1.5,
            [None; 4],
            16,
            "2026-03-25T12:00:00Z".to_string(),
        );
    }

    // --- CycleResult tests ---

    #[test]
    fn test_cycle_result_hit() {
        let result = CycleResult::Hit(RhythmOffset::new(5.0));
        match result {
            CycleResult::Hit(offset) => assert_eq!(offset.ms(), 5.0),
            CycleResult::Miss => panic!("Expected Hit"),
        }
    }

    #[test]
    fn test_cycle_result_miss() {
        let result = CycleResult::Miss;
        assert_eq!(result, CycleResult::Miss);
    }

    // --- aggregate_trial tests ---

    fn make_cycles(hits: &[(usize, StepPosition, f64)]) -> Vec<(StepPosition, CycleResult)> {
        let mut cycles = Vec::with_capacity(CYCLES_PER_TRIAL as usize);
        // Fill with misses first
        for i in 0..CYCLES_PER_TRIAL as usize {
            cycles.push((StepPosition::First, CycleResult::Miss));
            // Default gap position; will be overwritten for hits
            let _ = i;
        }
        // Place hits at specific indices
        for &(idx, pos, offset_ms) in hits {
            cycles[idx] = (pos, CycleResult::Hit(RhythmOffset::new(offset_ms)));
        }
        // Set positions for misses (distribute across positions)
        for (i, cycle) in cycles
            .iter_mut()
            .enumerate()
            .take(CYCLES_PER_TRIAL as usize)
        {
            if matches!(cycle.1, CycleResult::Miss) {
                cycle.0 = StepPosition::ALL[i % 4];
            }
        }
        cycles
    }

    #[test]
    fn test_aggregate_all_hits_same_offset() {
        let mut cycles = Vec::new();
        for i in 0..16 {
            cycles.push((
                StepPosition::ALL[i % 4],
                CycleResult::Hit(RhythmOffset::new(10.0)),
            ));
        }
        let trial = aggregate_trial(
            TempoBPM::new(80),
            &cycles,
            "2026-03-25T12:00:00Z".to_string(),
        )
        .unwrap();
        assert_eq!(trial.hit_rate(), 1.0);
        assert!((trial.mean_offset_ms() - 10.0).abs() < 1e-10);
        assert_eq!(trial.cycle_count(), 16);
        // Each position got 4 hits at 10.0ms
        for mean in trial.per_position_mean_ms() {
            assert!((mean.unwrap() - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_aggregate_all_misses_returns_none() {
        let cycles: Vec<(StepPosition, CycleResult)> = (0..16)
            .map(|i| (StepPosition::ALL[i % 4], CycleResult::Miss))
            .collect();
        let trial = aggregate_trial(
            TempoBPM::new(80),
            &cycles,
            "2026-03-25T12:00:00Z".to_string(),
        );
        assert!(trial.is_none(), "all-miss trials should be discarded");
    }

    #[test]
    fn test_aggregate_mixed_hits_and_misses() {
        // 4 hits out of 16 = 25% hit rate
        let hits = vec![
            (0, StepPosition::First, 5.0),
            (4, StepPosition::Second, 10.0),
            (8, StepPosition::Third, -5.0),
            (12, StepPosition::Fourth, 10.0),
        ];
        let cycles = make_cycles(&hits);
        let trial = aggregate_trial(
            TempoBPM::new(80),
            &cycles,
            "2026-03-25T12:00:00Z".to_string(),
        )
        .unwrap();
        assert!((trial.hit_rate() - 0.25).abs() < 1e-10);
        // mean = (5 + 10 + (-5) + 10) / 4 = 5.0
        assert!((trial.mean_offset_ms() - 5.0).abs() < 1e-10);
        assert!((trial.per_position_mean_ms()[0].unwrap() - 5.0).abs() < 1e-10);
        assert!((trial.per_position_mean_ms()[1].unwrap() - 10.0).abs() < 1e-10);
        assert!((trial.per_position_mean_ms()[2].unwrap() - (-5.0)).abs() < 1e-10);
        assert!((trial.per_position_mean_ms()[3].unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_aggregate_per_position_multiple_hits() {
        // Two hits on First position
        let mut cycles: Vec<(StepPosition, CycleResult)> = (0..16)
            .map(|i| (StepPosition::ALL[i % 4], CycleResult::Miss))
            .collect();
        cycles[0] = (
            StepPosition::First,
            CycleResult::Hit(RhythmOffset::new(4.0)),
        );
        cycles[4] = (
            StepPosition::First,
            CycleResult::Hit(RhythmOffset::new(8.0)),
        );

        let trial = aggregate_trial(
            TempoBPM::new(80),
            &cycles,
            "2026-03-25T12:00:00Z".to_string(),
        )
        .unwrap();
        // mean of First position: (4 + 8) / 2 = 6.0
        assert!((trial.per_position_mean_ms()[0].unwrap() - 6.0).abs() < 1e-10);
        // Other positions: None
        assert!(trial.per_position_mean_ms()[1].is_none());
        assert!(trial.per_position_mean_ms()[2].is_none());
        assert!(trial.per_position_mean_ms()[3].is_none());
    }

    #[test]
    #[should_panic(expected = "trial must have exactly 16 cycles")]
    fn test_aggregate_wrong_cycle_count_panics() {
        let cycles: Vec<(StepPosition, CycleResult)> = (0..10)
            .map(|_| (StepPosition::First, CycleResult::Miss))
            .collect();
        aggregate_trial(
            TempoBPM::new(80),
            &cycles,
            "2026-03-25T12:00:00Z".to_string(),
        );
    }
}
