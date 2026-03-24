use crate::types::{RhythmOffset, TempoBPM};

/// Evaluate a tap against scheduled beat times.
///
/// Returns the signed offset (in ms) from the nearest scheduled beat, or `None`
/// if the tap falls outside the acceptance window (±50% of one sixteenth note
/// at the given tempo).
///
/// All times are in seconds (AudioContext.currentTime scale).
pub fn evaluate_tap(
    tap_time: f64,
    scheduled_times: &[f64],
    tempo: TempoBPM,
) -> Option<RhythmOffset> {
    if scheduled_times.is_empty() {
        return None;
    }

    // Find the nearest scheduled beat
    let mut nearest_time = scheduled_times[0];
    let mut min_abs_diff = (tap_time - nearest_time).abs();

    for &t in &scheduled_times[1..] {
        let abs_diff = (tap_time - t).abs();
        if abs_diff < min_abs_diff {
            min_abs_diff = abs_diff;
            nearest_time = t;
        }
    }

    // Acceptance window: ±50% of one sixteenth note
    let sixteenth_secs = tempo.sixteenth_note_duration_secs();
    let window_secs = sixteenth_secs * 0.5;

    if min_abs_diff > window_secs {
        return None;
    }

    // Signed offset: positive = late, negative = early
    let offset_ms = (tap_time - nearest_time) * 1000.0;
    Some(RhythmOffset::new(offset_ms))
}

/// A completed rhythm offset detection trial with all context for persistence.
#[derive(Clone, Debug, PartialEq)]
pub struct CompletedRhythmOffsetDetectionTrial {
    tempo: TempoBPM,
    offset: RhythmOffset,
    is_correct: bool,
    timestamp: String,
}

impl CompletedRhythmOffsetDetectionTrial {
    pub fn new(tempo: TempoBPM, offset: RhythmOffset, is_correct: bool, timestamp: String) -> Self {
        assert!(!timestamp.is_empty(), "timestamp must not be empty");
        Self {
            tempo,
            offset,
            is_correct,
            timestamp,
        }
    }

    pub fn tempo(&self) -> TempoBPM {
        self.tempo
    }

    pub fn offset(&self) -> RhythmOffset {
        self.offset
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// Extract the metric value: absolute offset as percentage of one sixteenth note.
    pub fn metric_value(&self) -> f64 {
        self.offset.percentage_of_sixteenth(self.tempo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trial(
        bpm: u16,
        offset_ms: f64,
        is_correct: bool,
    ) -> CompletedRhythmOffsetDetectionTrial {
        CompletedRhythmOffsetDetectionTrial::new(
            TempoBPM::new(bpm),
            RhythmOffset::new(offset_ms),
            is_correct,
            "2026-03-24T12:00:00Z".to_string(),
        )
    }

    #[test]
    fn test_fields_accessible() {
        let trial = make_trial(80, -9.375, true);
        assert_eq!(trial.tempo().bpm(), 80);
        assert_eq!(trial.offset().ms(), -9.375);
        assert!(trial.is_correct());
        assert_eq!(trial.timestamp(), "2026-03-24T12:00:00Z");
    }

    #[test]
    fn test_metric_value_at_80bpm() {
        // 9.375ms at 80 BPM = 5.0% of a sixteenth note
        let trial = make_trial(80, 9.375, true);
        assert!((trial.metric_value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_negative_offset() {
        // Negative offset should give same metric (uses abs internally)
        let trial = make_trial(80, -9.375, false);
        assert!((trial.metric_value() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_zero_offset() {
        let trial = make_trial(120, 0.0, true);
        assert!((trial.metric_value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_metric_value_at_120bpm() {
        // 12.5ms at 120 BPM: sixteenth = 125ms → 10.0%
        let trial = make_trial(120, 12.5, true);
        assert!((trial.metric_value() - 10.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "timestamp must not be empty")]
    fn test_empty_timestamp_panics() {
        CompletedRhythmOffsetDetectionTrial::new(
            TempoBPM::new(80),
            RhythmOffset::new(0.0),
            true,
            String::new(),
        );
    }

    // --- evaluate_tap tests ---

    // At 80 BPM: sixteenth = 0.1875s = 187.5ms, window = ±93.75ms = ±0.09375s

    #[test]
    fn test_evaluate_tap_exact_hit() {
        let times = vec![1.0, 1.1875, 1.375, 1.5625];
        let tempo = TempoBPM::new(80);
        let result = evaluate_tap(1.1875, &times, tempo);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_tap_early() {
        let times = vec![1.0, 1.1875, 1.375, 1.5625];
        let tempo = TempoBPM::new(80);
        // Tap 20ms early relative to second beat
        let result = evaluate_tap(1.1875 - 0.020, &times, tempo);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - (-20.0)).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_tap_late() {
        let times = vec![1.0, 1.1875, 1.375, 1.5625];
        let tempo = TempoBPM::new(80);
        // Tap 30ms late relative to first beat
        let result = evaluate_tap(1.0 + 0.030, &times, tempo);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_tap_outside_window() {
        let times = vec![1.0, 1.1875, 1.375, 1.5625];
        let tempo = TempoBPM::new(80);
        // Tap 100ms after last beat — window is ±93.75ms, so outside
        let result = evaluate_tap(1.5625 + 0.100, &times, tempo);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_tap_at_window_boundary_inside() {
        let times = vec![1.0];
        let tempo = TempoBPM::new(80);
        // Exactly at boundary: 93.75ms = 0.09375s
        let result = evaluate_tap(1.0 + 0.09375, &times, tempo);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - 93.75).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_tap_just_outside_window() {
        let times = vec![1.0];
        let tempo = TempoBPM::new(80);
        // Slightly beyond boundary
        let result = evaluate_tap(1.0 + 0.09376, &times, tempo);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_tap_empty_scheduled_times() {
        let tempo = TempoBPM::new(80);
        let result = evaluate_tap(1.0, &[], tempo);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_tap_finds_nearest_beat() {
        let times = vec![1.0, 1.1875, 1.375, 1.5625];
        let tempo = TempoBPM::new(80);
        // Tap closer to third beat (1.375) than second (1.1875)
        let result = evaluate_tap(1.30, &times, tempo);
        assert!(result.is_some());
        // offset = 1.30 - 1.375 = -0.075s = -75ms
        assert!((result.unwrap().ms() - (-75.0)).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_tap_at_120bpm() {
        // At 120 BPM: sixteenth = 0.125s = 125ms, window = ±62.5ms
        let times = vec![2.0, 2.125, 2.250, 2.375];
        let tempo = TempoBPM::new(120);
        // Tap 50ms late on first beat — within window
        let result = evaluate_tap(2.050, &times, tempo);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_tap_single_scheduled_time() {
        let times = vec![5.0];
        let tempo = TempoBPM::new(80);
        let result = evaluate_tap(5.010, &times, tempo);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - 10.0).abs() < 1e-10);
    }
}
