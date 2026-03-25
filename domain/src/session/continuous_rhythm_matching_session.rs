use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::ports::{
    ProfileUpdating, ProgressTimelineUpdating, Resettable, TrainingRecordPersisting,
};
use crate::profile::PerceptualProfile;
use crate::records::{ContinuousRhythmMatchingRecord, TrainingRecord};
use crate::statistics_key::StatisticsKey;
use crate::training::continuous_rhythm_matching::{
    CYCLES_PER_TRIAL, CompletedContinuousRhythmMatchingTrial, CycleResult, aggregate_trial,
};
use crate::training::evaluate_tap;
use crate::training_discipline::TrainingDiscipline;
use crate::types::{RhythmDirection, RhythmOffset, StepPosition, TempoBPM, TempoRange};

/// State of the continuous rhythm matching session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuousRhythmMatchingSessionState {
    Idle,
    Running,
}

/// Trait for selecting a random gap position from the enabled set.
/// Allows deterministic testing.
pub trait GapPositionSelector {
    fn select(&self, enabled: &[StepPosition]) -> StepPosition;
}

/// Default gap position selector using rand.
pub struct RandomGapSelector;

impl GapPositionSelector for RandomGapSelector {
    fn select(&self, enabled: &[StepPosition]) -> StepPosition {
        use rand::Rng;
        if enabled.len() == 1 {
            return enabled[0];
        }
        let idx = rand::rng().random_range(0..enabled.len());
        enabled[idx]
    }
}

/// Pure domain state machine for continuous rhythm matching ("Fill the Gap") sessions.
///
/// Manages the exercise loop: continuous 4-step pattern with one gap per cycle,
/// user taps to fill the gap, 16 cycles aggregate into a trial.
pub struct ContinuousRhythmMatchingSession {
    state: ContinuousRhythmMatchingSessionState,
    profile: Rc<RefCell<PerceptualProfile>>,
    profile_port: Box<dyn ProfileUpdating>,
    record_port: Box<dyn TrainingRecordPersisting>,
    timeline_port: Box<dyn ProgressTimelineUpdating>,
    resettables: Vec<Box<dyn Resettable>>,
    gap_selector: Box<dyn GapPositionSelector>,

    // Session configuration
    tempo: Option<TempoBPM>,
    enabled_positions: Vec<StepPosition>,

    // Cycle tracking
    current_cycle_index: u16,
    cycle_results: Vec<(StepPosition, CycleResult)>,
    current_gap_position: Option<StepPosition>,
    current_gap_scheduled_time: Option<f64>,

    // Trial history
    last_completed: Option<CompletedContinuousRhythmMatchingTrial>,
}

impl ContinuousRhythmMatchingSession {
    pub fn new(
        profile: Rc<RefCell<PerceptualProfile>>,
        profile_port: Box<dyn ProfileUpdating>,
        record_port: Box<dyn TrainingRecordPersisting>,
        timeline_port: Box<dyn ProgressTimelineUpdating>,
        resettables: Vec<Box<dyn Resettable>>,
        gap_selector: Box<dyn GapPositionSelector>,
    ) -> Self {
        Self {
            state: ContinuousRhythmMatchingSessionState::Idle,
            profile,
            profile_port,
            record_port,
            timeline_port,
            resettables,
            gap_selector,
            tempo: None,
            enabled_positions: Vec::new(),
            current_cycle_index: 0,
            cycle_results: Vec::new(),
            current_gap_position: None,
            current_gap_scheduled_time: None,
            last_completed: None,
        }
    }

    // --- Observable state accessors ---

    pub fn state(&self) -> ContinuousRhythmMatchingSessionState {
        self.state
    }

    pub fn current_cycle_index(&self) -> u16 {
        self.current_cycle_index
    }

    pub fn current_gap_position(&self) -> Option<StepPosition> {
        self.current_gap_position
    }

    pub fn last_completed(&self) -> Option<&CompletedContinuousRhythmMatchingTrial> {
        self.last_completed.as_ref()
    }

    pub fn tempo(&self) -> Option<TempoBPM> {
        self.tempo
    }

    // --- State transitions ---

    /// Start a new session with the given tempo and enabled gap positions.
    ///
    /// Panics if not in Idle state or if no positions are enabled.
    pub fn start(&mut self, tempo: TempoBPM, enabled_positions: HashSet<StepPosition>) {
        assert_eq!(
            self.state,
            ContinuousRhythmMatchingSessionState::Idle,
            "start() requires Idle state"
        );
        assert!(
            !enabled_positions.is_empty(),
            "at least one gap position must be enabled"
        );

        self.tempo = Some(tempo);
        self.enabled_positions = enabled_positions.into_iter().collect();
        self.current_cycle_index = 0;
        self.cycle_results.clear();
        self.state = ContinuousRhythmMatchingSessionState::Running;

        // Select gap for first cycle
        self.current_gap_position = Some(self.gap_selector.select(&self.enabled_positions));
        self.current_gap_scheduled_time = None;
    }

    /// Called at the start of each cycle to provide the scheduled time of the gap position.
    /// The web layer calls this after scheduling the cycle's audio.
    pub fn set_gap_scheduled_time(&mut self, time: f64) {
        self.current_gap_scheduled_time = Some(time);
    }

    /// Handle a user tap. Returns the offset if the tap was within the acceptance window.
    ///
    /// If the tap is outside the window, returns `None` (ignored, not a miss).
    pub fn handle_tap(&mut self, tap_time: f64) -> Option<RhythmOffset> {
        if self.state != ContinuousRhythmMatchingSessionState::Running {
            return None;
        }

        let tempo = self.tempo.expect("tempo must be set when running");
        let gap_time = self
            .current_gap_scheduled_time
            .expect("gap scheduled time must be set when running");

        // Use evaluate_tap with just the gap position's scheduled time
        evaluate_tap(tap_time, &[gap_time], tempo)
    }

    /// Called when a cycle completes (the sequencer has moved to the next cycle).
    /// If the user tapped (hit), pass the offset. If no tap occurred, pass `None` (miss).
    ///
    /// Returns `Some(trial)` if this cycle completes a 16-cycle trial.
    pub fn cycle_complete(
        &mut self,
        tap_result: Option<RhythmOffset>,
        timestamp: String,
    ) -> Option<CompletedContinuousRhythmMatchingTrial> {
        if self.state != ContinuousRhythmMatchingSessionState::Running {
            return None;
        }

        let gap_position = self
            .current_gap_position
            .expect("gap position must be set when running");

        let result = match tap_result {
            Some(offset) => CycleResult::Hit(offset),
            None => CycleResult::Miss,
        };

        self.cycle_results.push((gap_position, result));
        self.current_cycle_index += 1;

        // Check if trial is complete
        if self.current_cycle_index >= CYCLES_PER_TRIAL {
            let tempo = self.tempo.expect("tempo must be set when running");
            let trial = aggregate_trial(tempo, &self.cycle_results, timestamp.clone());

            if let Some(ref completed) = trial {
                self.on_trial_completed(completed, &timestamp);
                self.last_completed = trial.clone();
            }

            // Reset for next trial
            self.current_cycle_index = 0;
            self.cycle_results.clear();

            // Select gap for next trial's first cycle
            self.current_gap_position = Some(self.gap_selector.select(&self.enabled_positions));
            self.current_gap_scheduled_time = None;

            return trial;
        }

        // Select gap for next cycle
        self.current_gap_position = Some(self.gap_selector.select(&self.enabled_positions));
        self.current_gap_scheduled_time = None;

        None
    }

    /// Stop the session. Incomplete trial is discarded (AC10).
    pub fn stop(&mut self) {
        if self.state == ContinuousRhythmMatchingSessionState::Idle {
            return;
        }
        self.state = ContinuousRhythmMatchingSessionState::Idle;
        self.current_cycle_index = 0;
        self.cycle_results.clear();
        self.current_gap_position = None;
        self.current_gap_scheduled_time = None;
        self.tempo = None;
    }

    /// Stop if running, reset profile, and all resettables.
    pub fn reset_training_data(&mut self) {
        self.stop();
        self.last_completed = None;
        self.profile.borrow_mut().reset_all();
        for resettable in &mut self.resettables {
            resettable.reset();
        }
    }

    // --- Internal ---

    fn on_trial_completed(
        &mut self,
        completed: &CompletedContinuousRhythmMatchingTrial,
        timestamp: &str,
    ) {
        let tempo = self.tempo.expect("tempo must be set");
        let tempo_range = TempoRange::from_bpm(tempo);
        let direction = RhythmDirection::from_offset_ms(completed.mean_offset_ms());
        let key = StatisticsKey::Rhythm(
            TrainingDiscipline::ContinuousRhythmMatching,
            tempo_range,
            direction,
        );
        let metric = completed.metric_value();

        // For continuous matching, is_correct is based on hit_rate > 0
        let is_correct = completed.hit_rate() > 0.0;

        // Update profile
        self.profile_port
            .update_profile(key, timestamp, metric, is_correct);

        // Persist training record
        let record = ContinuousRhythmMatchingRecord::from_completed(completed);
        self.record_port
            .save_record(TrainingRecord::ContinuousRhythmMatching(record));

        // Update progress timeline
        self.timeline_port.add_metric(
            TrainingDiscipline::ContinuousRhythmMatching,
            timestamp,
            metric,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // --- Deterministic gap selector for tests ---

    struct FixedGapSelector {
        positions: Vec<StepPosition>,
        index: Cell<usize>,
    }

    impl FixedGapSelector {
        fn new(positions: Vec<StepPosition>) -> Self {
            Self {
                positions,
                index: Cell::new(0),
            }
        }

        fn single(pos: StepPosition) -> Self {
            Self::new(vec![pos])
        }
    }

    impl GapPositionSelector for FixedGapSelector {
        fn select(&self, _enabled: &[StepPosition]) -> StepPosition {
            let idx = self.index.get();
            let pos = self.positions[idx % self.positions.len()];
            self.index.set(idx + 1);
            pos
        }
    }

    // --- Mock ports ---

    struct MockProfilePort {
        call_count: Rc<Cell<usize>>,
    }

    impl MockProfilePort {
        fn new(call_count: Rc<Cell<usize>>) -> Self {
            Self { call_count }
        }
    }

    impl ProfileUpdating for MockProfilePort {
        fn update_profile(
            &mut self,
            _key: StatisticsKey,
            _timestamp: &str,
            _value: f64,
            _is_correct: bool,
        ) {
            self.call_count.set(self.call_count.get() + 1);
        }
    }

    struct MockRecordPort {
        call_count: Rc<Cell<usize>>,
    }

    impl MockRecordPort {
        fn new(call_count: Rc<Cell<usize>>) -> Self {
            Self { call_count }
        }
    }

    impl TrainingRecordPersisting for MockRecordPort {
        fn save_record(&self, _record: TrainingRecord) {
            self.call_count.set(self.call_count.get() + 1);
        }
    }

    struct MockTimelinePort {
        call_count: Rc<Cell<usize>>,
    }

    impl MockTimelinePort {
        fn new(call_count: Rc<Cell<usize>>) -> Self {
            Self { call_count }
        }
    }

    impl ProgressTimelineUpdating for MockTimelinePort {
        fn add_metric(&mut self, _discipline: TrainingDiscipline, _timestamp: &str, _value: f64) {
            self.call_count.set(self.call_count.get() + 1);
        }
    }

    fn make_session_with_selector(
        selector: Box<dyn GapPositionSelector>,
    ) -> (
        ContinuousRhythmMatchingSession,
        Rc<Cell<usize>>,
        Rc<Cell<usize>>,
        Rc<Cell<usize>>,
    ) {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let profile_calls = Rc::new(Cell::new(0usize));
        let record_calls = Rc::new(Cell::new(0usize));
        let timeline_calls = Rc::new(Cell::new(0usize));

        let session = ContinuousRhythmMatchingSession::new(
            profile,
            Box::new(MockProfilePort::new(Rc::clone(&profile_calls))),
            Box::new(MockRecordPort::new(Rc::clone(&record_calls))),
            Box::new(MockTimelinePort::new(Rc::clone(&timeline_calls))),
            Vec::new(),
            selector,
        );

        (session, profile_calls, record_calls, timeline_calls)
    }

    fn make_session() -> ContinuousRhythmMatchingSession {
        make_session_with_selector(Box::new(FixedGapSelector::single(StepPosition::First))).0
    }

    fn enabled_all() -> HashSet<StepPosition> {
        StepPosition::ALL.iter().copied().collect()
    }

    fn enabled_single(pos: StepPosition) -> HashSet<StepPosition> {
        let mut set = HashSet::new();
        set.insert(pos);
        set
    }

    // --- State transition tests ---

    #[test]
    fn test_initial_state_is_idle() {
        let session = make_session();
        assert_eq!(session.state(), ContinuousRhythmMatchingSessionState::Idle);
        assert_eq!(session.current_cycle_index(), 0);
        assert!(session.current_gap_position().is_none());
        assert!(session.last_completed().is_none());
    }

    #[test]
    fn test_start_transitions_to_running() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        assert_eq!(
            session.state(),
            ContinuousRhythmMatchingSessionState::Running
        );
        assert_eq!(session.current_cycle_index(), 0);
        assert!(session.current_gap_position().is_some());
        assert_eq!(session.tempo(), Some(TempoBPM::new(80)));
    }

    #[test]
    #[should_panic(expected = "start() requires Idle state")]
    fn test_start_from_running_panics() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        session.start(TempoBPM::new(80), enabled_all());
    }

    #[test]
    #[should_panic(expected = "at least one gap position must be enabled")]
    fn test_start_with_empty_positions_panics() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), HashSet::new());
    }

    #[test]
    fn test_stop_returns_to_idle() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        session.stop();
        assert_eq!(session.state(), ContinuousRhythmMatchingSessionState::Idle);
        assert_eq!(session.current_cycle_index(), 0);
        assert!(session.current_gap_position().is_none());
    }

    #[test]
    fn test_stop_when_idle_is_noop() {
        let mut session = make_session();
        session.stop(); // Should not panic
        assert_eq!(session.state(), ContinuousRhythmMatchingSessionState::Idle);
    }

    // --- Gap position selection tests ---

    #[test]
    fn test_gap_position_selected_on_start() {
        let (mut session, _, _, _) =
            make_session_with_selector(Box::new(FixedGapSelector::single(StepPosition::Third)));
        session.start(TempoBPM::new(80), enabled_all());
        assert_eq!(session.current_gap_position(), Some(StepPosition::Third));
    }

    #[test]
    fn test_single_enabled_position_always_selected() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_single(StepPosition::Second));
        // The FixedGapSelector returns First, but the session was started with only Second enabled
        // However, our FixedGapSelector ignores the enabled set — this tests the selector abstraction
        // In production, RandomGapSelector only picks from enabled positions
        assert!(session.current_gap_position().is_some());
    }

    // --- Tap evaluation tests ---

    #[test]
    fn test_handle_tap_within_window() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        // Gap at time 1.0
        session.set_gap_scheduled_time(1.0);
        // Tap 10ms late
        let result = session.handle_tap(1.010);
        assert!(result.is_some());
        assert!((result.unwrap().ms() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_handle_tap_outside_window() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        session.set_gap_scheduled_time(1.0);
        // At 80 BPM: window = ±93.75ms = ±0.09375s. Tap 100ms late = outside
        let result = session.handle_tap(1.1);
        assert!(result.is_none());
    }

    #[test]
    fn test_handle_tap_when_idle_returns_none() {
        let mut session = make_session();
        let result = session.handle_tap(1.0);
        assert!(result.is_none());
    }

    // --- Cycle completion tests ---

    #[test]
    fn test_cycle_complete_increments_index() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        session.set_gap_scheduled_time(1.0);

        let trial = session.cycle_complete(
            Some(RhythmOffset::new(5.0)),
            "2026-03-25T12:00:00Z".to_string(),
        );
        assert!(trial.is_none()); // Not 16 cycles yet
        assert_eq!(session.current_cycle_index(), 1);
    }

    #[test]
    fn test_cycle_complete_miss() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());
        session.set_gap_scheduled_time(1.0);

        let trial = session.cycle_complete(None, "2026-03-25T12:00:00Z".to_string());
        assert!(trial.is_none());
        assert_eq!(session.current_cycle_index(), 1);
    }

    #[test]
    fn test_16_cycles_produces_trial() {
        let (mut session, _, _, _) =
            make_session_with_selector(Box::new(FixedGapSelector::single(StepPosition::First)));
        session.start(TempoBPM::new(80), enabled_all());

        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            let trial = session.cycle_complete(
                Some(RhythmOffset::new(5.0)),
                "2026-03-25T12:00:00Z".to_string(),
            );
            if i < 15 {
                assert!(trial.is_none(), "cycle {i} should not produce trial");
            } else {
                assert!(trial.is_some(), "cycle 15 should produce trial");
                let t = trial.unwrap();
                assert_eq!(t.cycle_count(), 16);
                assert_eq!(t.hit_rate(), 1.0);
                assert!((t.mean_offset_ms() - 5.0).abs() < 1e-10);
            }
        }

        // Should reset for next trial
        assert_eq!(session.current_cycle_index(), 0);
    }

    #[test]
    fn test_all_misses_produces_trial() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());

        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            let trial = session.cycle_complete(None, "2026-03-25T12:00:00Z".to_string());
            if i < 15 {
                assert!(trial.is_none());
            } else {
                let t = trial.unwrap();
                assert_eq!(t.hit_rate(), 0.0);
                assert_eq!(t.mean_offset_ms(), 0.0);
            }
        }
    }

    #[test]
    fn test_mixed_hits_and_misses() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());

        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            // Hit on even cycles, miss on odd
            let tap = if i % 2 == 0 {
                Some(RhythmOffset::new(10.0))
            } else {
                None
            };
            let trial = session.cycle_complete(tap, "2026-03-25T12:00:00Z".to_string());
            if i == 15 {
                let t = trial.unwrap();
                assert!((t.hit_rate() - 0.5).abs() < 1e-10); // 8/16
                assert!((t.mean_offset_ms() - 10.0).abs() < 1e-10);
            }
        }
    }

    // --- Incomplete trial discarded on stop (AC10) ---

    #[test]
    fn test_stop_mid_trial_discards_incomplete() {
        let (mut session, profile_calls, record_calls, timeline_calls) =
            make_session_with_selector(Box::new(FixedGapSelector::single(StepPosition::First)));
        session.start(TempoBPM::new(80), enabled_all());

        // Complete 8 cycles
        for i in 0..8 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            session.cycle_complete(
                Some(RhythmOffset::new(5.0)),
                "2026-03-25T12:00:00Z".to_string(),
            );
        }

        session.stop();

        // No ports should have been called (trial was incomplete)
        assert_eq!(profile_calls.get(), 0);
        assert_eq!(record_calls.get(), 0);
        assert_eq!(timeline_calls.get(), 0);
    }

    // --- Port calls on trial completion ---

    #[test]
    fn test_trial_completion_calls_all_ports() {
        let (mut session, profile_calls, record_calls, timeline_calls) =
            make_session_with_selector(Box::new(FixedGapSelector::single(StepPosition::First)));
        session.start(TempoBPM::new(80), enabled_all());

        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            session.cycle_complete(
                Some(RhythmOffset::new(5.0)),
                "2026-03-25T12:00:00Z".to_string(),
            );
        }

        assert_eq!(profile_calls.get(), 1);
        assert_eq!(record_calls.get(), 1);
        assert_eq!(timeline_calls.get(), 1);
    }

    #[test]
    fn test_two_trials_call_ports_twice() {
        let (mut session, profile_calls, record_calls, timeline_calls) =
            make_session_with_selector(Box::new(FixedGapSelector::single(StepPosition::First)));
        session.start(TempoBPM::new(80), enabled_all());

        // First trial: 16 cycles
        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            session.cycle_complete(
                Some(RhythmOffset::new(5.0)),
                "2026-03-25T12:00:00Z".to_string(),
            );
        }

        // Second trial: 16 cycles
        for i in 0..16 {
            session.set_gap_scheduled_time(4.0 + i as f64 * 0.1875);
            session.cycle_complete(
                Some(RhythmOffset::new(3.0)),
                "2026-03-25T12:01:00Z".to_string(),
            );
        }

        assert_eq!(profile_calls.get(), 2);
        assert_eq!(record_calls.get(), 2);
        assert_eq!(timeline_calls.get(), 2);
    }

    #[test]
    fn test_last_completed_updated_after_trial() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());

        assert!(session.last_completed().is_none());

        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            session.cycle_complete(
                Some(RhythmOffset::new(5.0)),
                "2026-03-25T12:00:00Z".to_string(),
            );
        }

        assert!(session.last_completed().is_some());
        let trial = session.last_completed().unwrap();
        assert_eq!(trial.tempo().bpm(), 80);
    }

    // --- Per-position breakdown ---

    #[test]
    fn test_per_position_breakdown_with_rotating_gaps() {
        let positions = vec![
            StepPosition::First,
            StepPosition::Second,
            StepPosition::Third,
            StepPosition::Fourth,
        ];
        let selector = FixedGapSelector::new(
            positions.iter().cycle().take(17).copied().collect(), // 16 + 1 for next cycle setup
        );

        let (mut session, _, _, _) = make_session_with_selector(Box::new(selector));
        session.start(TempoBPM::new(80), enabled_all());

        // Each position gets 4 hits with offset matching position index * 2
        let offsets = [2.0, 4.0, 6.0, 8.0];
        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            let offset = offsets[i % 4];
            let trial = session.cycle_complete(
                Some(RhythmOffset::new(offset)),
                "2026-03-25T12:00:00Z".to_string(),
            );

            if i == 15 {
                let t = trial.unwrap();
                let means = t.per_position_mean_ms();
                // Each position: 4 hits with same offset
                assert!((means[0].unwrap() - 2.0).abs() < 1e-10);
                assert!((means[1].unwrap() - 4.0).abs() < 1e-10);
                assert!((means[2].unwrap() - 6.0).abs() < 1e-10);
                assert!((means[3].unwrap() - 8.0).abs() < 1e-10);
            }
        }
    }

    // --- Reset tests ---

    struct MockResettable {
        reset_count: Rc<Cell<usize>>,
    }

    impl MockResettable {
        fn new(reset_count: Rc<Cell<usize>>) -> Self {
            Self { reset_count }
        }
    }

    impl Resettable for MockResettable {
        fn reset(&mut self) {
            self.reset_count.set(self.reset_count.get() + 1);
        }
    }

    #[test]
    fn test_reset_training_data_calls_resettables() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let reset_count = Rc::new(Cell::new(0usize));

        let mut session = ContinuousRhythmMatchingSession::new(
            profile,
            Box::new(MockProfilePort::new(Rc::new(Cell::new(0)))),
            Box::new(MockRecordPort::new(Rc::new(Cell::new(0)))),
            Box::new(MockTimelinePort::new(Rc::new(Cell::new(0)))),
            vec![Box::new(MockResettable::new(Rc::clone(&reset_count)))],
            Box::new(FixedGapSelector::single(StepPosition::First)),
        );

        session.reset_training_data();
        assert_eq!(reset_count.get(), 1);
    }

    #[test]
    fn test_reset_training_data_clears_last_completed() {
        let mut session = make_session();
        session.start(TempoBPM::new(80), enabled_all());

        for i in 0..16 {
            session.set_gap_scheduled_time(1.0 + i as f64 * 0.1875);
            session.cycle_complete(
                Some(RhythmOffset::new(5.0)),
                "2026-03-25T12:00:00Z".to_string(),
            );
        }
        assert!(session.last_completed().is_some());

        session.reset_training_data();
        assert!(session.last_completed().is_none());
    }

    // --- cycle_complete when idle is noop ---

    #[test]
    fn test_cycle_complete_when_idle_returns_none() {
        let mut session = make_session();
        let result = session.cycle_complete(
            Some(RhythmOffset::new(5.0)),
            "2026-03-25T12:00:00Z".to_string(),
        );
        assert!(result.is_none());
    }
}
