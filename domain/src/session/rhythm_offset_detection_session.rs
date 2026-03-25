use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;

use crate::ports::{
    ProfileUpdating, ProgressTimelineUpdating, Resettable, TrainingRecordPersisting,
};
use crate::profile::PerceptualProfile;
use crate::records::{RhythmOffsetDetectionRecord, TrainingRecord};
use crate::statistics_key::StatisticsKey;
use crate::training::CompletedRhythmOffsetDetectionTrial;
use crate::training_discipline::TrainingDiscipline;
use crate::types::{RhythmDirection, RhythmOffset, TempoBPM, TempoRange};

/// Feedback display duration in seconds for rhythm training.
pub const RHYTHM_FEEDBACK_DURATION_SECS: f64 = 0.4;

/// Default initial difficulty percentage (of one sixteenth note).
const DEFAULT_INITIAL_DIFFICULTY_PCT: f64 = 10.0;

/// Minimum offset percentage (floor for adaptive difficulty).
const MIN_DIFFICULTY_PCT: f64 = 1.0;

/// Maximum offset percentage (ceiling for adaptive difficulty).
const MAX_DIFFICULTY_PCT: f64 = 20.0;

/// Factor to multiply difficulty by on correct answer (narrowing).
const NARROW_FACTOR: f64 = 0.85;

/// Factor to multiply difficulty by on incorrect answer (widening).
const WIDEN_FACTOR: f64 = 1.2;

/// State of the rhythm offset detection session state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RhythmOffsetDetectionSessionState {
    Idle,
    Playing,
    AwaitingAnswer,
    ShowingFeedback,
}

/// Per-direction adaptive difficulty tracker for rhythm offset detection.
///
/// Tracks independent difficulty percentages for early and late offsets.
/// On correct answer the offset narrows (gets harder); on incorrect it widens (gets easier).
#[derive(Clone, Debug)]
pub struct AdaptiveRhythmOffsetStrategy {
    early_difficulty_pct: f64,
    late_difficulty_pct: f64,
}

impl AdaptiveRhythmOffsetStrategy {
    /// Create a new strategy with default initial difficulty for both directions.
    pub fn new() -> Self {
        Self {
            early_difficulty_pct: DEFAULT_INITIAL_DIFFICULTY_PCT,
            late_difficulty_pct: DEFAULT_INITIAL_DIFFICULTY_PCT,
        }
    }

    /// Get the current difficulty percentage for the given direction.
    pub fn difficulty_pct(&self, direction: RhythmDirection) -> f64 {
        match direction {
            RhythmDirection::Early => self.early_difficulty_pct,
            RhythmDirection::Late => self.late_difficulty_pct,
            RhythmDirection::OnBeat => 0.0,
        }
    }

    /// Update difficulty after a trial result.
    /// Narrows (×0.85) on correct, widens (×1.2) on incorrect, clamped to [1%, 20%].
    pub fn update(&mut self, direction: RhythmDirection, is_correct: bool) {
        let difficulty = match direction {
            RhythmDirection::Early => &mut self.early_difficulty_pct,
            RhythmDirection::Late => &mut self.late_difficulty_pct,
            RhythmDirection::OnBeat => return,
        };

        if is_correct {
            *difficulty *= NARROW_FACTOR;
        } else {
            *difficulty *= WIDEN_FACTOR;
        }
        *difficulty = difficulty.clamp(MIN_DIFFICULTY_PCT, MAX_DIFFICULTY_PCT);
    }

    /// Reset both directions to default difficulty.
    pub fn reset(&mut self) {
        self.early_difficulty_pct = DEFAULT_INITIAL_DIFFICULTY_PCT;
        self.late_difficulty_pct = DEFAULT_INITIAL_DIFFICULTY_PCT;
    }
}

impl Default for AdaptiveRhythmOffsetStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters for the current trial, exposed to the web layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmOffsetDetectionTrialParams {
    pub tempo: TempoBPM,
    pub offset: RhythmOffset,
    pub direction: RhythmDirection,
}

/// Pure domain state machine for rhythm offset detection training sessions.
///
/// Manages the training loop: play pattern -> await answer -> show feedback -> repeat.
/// No browser dependencies — the web crate drives audio playback and timing.
pub struct RhythmOffsetDetectionSession {
    state: RhythmOffsetDetectionSessionState,
    profile: Rc<RefCell<PerceptualProfile>>,
    profile_port: Box<dyn ProfileUpdating>,
    record_port: Box<dyn TrainingRecordPersisting>,
    timeline_port: Box<dyn ProgressTimelineUpdating>,
    resettables: Vec<Box<dyn Resettable>>,

    // Adaptive difficulty
    strategy: AdaptiveRhythmOffsetStrategy,

    // Session-level state (snapshot from settings at start)
    session_tempo: TempoBPM,

    // Current trial state
    current_trial_params: Option<RhythmOffsetDetectionTrialParams>,
    last_completed: Option<CompletedRhythmOffsetDetectionTrial>,

    // Observable feedback state
    show_feedback: bool,
    is_last_answer_correct: bool,
}

impl RhythmOffsetDetectionSession {
    pub fn new(
        profile: Rc<RefCell<PerceptualProfile>>,
        profile_port: Box<dyn ProfileUpdating>,
        record_port: Box<dyn TrainingRecordPersisting>,
        timeline_port: Box<dyn ProgressTimelineUpdating>,
        resettables: Vec<Box<dyn Resettable>>,
    ) -> Self {
        Self {
            state: RhythmOffsetDetectionSessionState::Idle,
            profile,
            profile_port,
            record_port,
            timeline_port,
            resettables,
            strategy: AdaptiveRhythmOffsetStrategy::new(),
            session_tempo: TempoBPM::default(),
            current_trial_params: None,
            last_completed: None,
            show_feedback: false,
            is_last_answer_correct: false,
        }
    }

    // --- Observable state accessors ---

    pub fn state(&self) -> RhythmOffsetDetectionSessionState {
        self.state
    }

    pub fn show_feedback(&self) -> bool {
        self.show_feedback
    }

    pub fn is_last_answer_correct(&self) -> bool {
        self.is_last_answer_correct
    }

    pub fn current_trial_params(&self) -> Option<RhythmOffsetDetectionTrialParams> {
        self.current_trial_params
    }

    pub fn last_completed(&self) -> Option<&CompletedRhythmOffsetDetectionTrial> {
        self.last_completed.as_ref()
    }

    pub fn last_difficulty_pct(&self) -> Option<f64> {
        self.current_trial_params
            .map(|p| p.offset.percentage_of_sixteenth(p.tempo))
    }

    pub fn strategy(&self) -> &AdaptiveRhythmOffsetStrategy {
        &self.strategy
    }

    // --- State transitions ---

    /// Start a new trial: choose early/late, compute offset from difficulty %, transition to Playing.
    ///
    /// Panics if not in Idle state.
    pub fn start_trial(&mut self, tempo: TempoBPM) {
        assert_eq!(
            self.state,
            RhythmOffsetDetectionSessionState::Idle,
            "start_trial() requires Idle state"
        );

        self.session_tempo = tempo;

        // Choose direction: 50/50 early or late
        let direction = if rand::rng().random_bool(0.5) {
            RhythmDirection::Early
        } else {
            RhythmDirection::Late
        };

        let difficulty_pct = self.strategy.difficulty_pct(direction);
        let sixteenth_ms = tempo.sixteenth_note_duration_secs() * 1000.0;
        let offset_ms = sixteenth_ms * difficulty_pct / 100.0;

        // Early = negative offset, Late = positive offset
        let signed_offset_ms = match direction {
            RhythmDirection::Early => -offset_ms,
            RhythmDirection::Late => offset_ms,
            RhythmDirection::OnBeat => 0.0,
        };

        let offset = RhythmOffset::new(signed_offset_ms);
        self.current_trial_params = Some(RhythmOffsetDetectionTrialParams {
            tempo,
            offset,
            direction,
        });

        self.state = RhythmOffsetDetectionSessionState::Playing;
    }

    /// Called when the click pattern finishes playing. Transitions to AwaitingAnswer.
    pub fn pattern_finished(&mut self) {
        assert_eq!(
            self.state,
            RhythmOffsetDetectionSessionState::Playing,
            "pattern_finished() requires Playing state"
        );
        self.state = RhythmOffsetDetectionSessionState::AwaitingAnswer;
    }

    /// Handle the user's answer (early or late).
    ///
    /// Evaluates correctness, updates adaptive difficulty, calls port traits,
    /// transitions to ShowingFeedback.
    pub fn submit_answer(&mut self, is_early: bool, timestamp: String) {
        assert_eq!(
            self.state,
            RhythmOffsetDetectionSessionState::AwaitingAnswer,
            "submit_answer() requires AwaitingAnswer state"
        );

        let params = self
            .current_trial_params
            .expect("submit_answer() called without current trial params");

        // Evaluate correctness
        let actual_is_early = params.direction == RhythmDirection::Early;
        let is_correct = is_early == actual_is_early;

        // Update adaptive difficulty for this direction
        self.strategy.update(params.direction, is_correct);

        // Create completed trial
        let completed = CompletedRhythmOffsetDetectionTrial::new(
            params.tempo,
            params.offset,
            is_correct,
            timestamp,
        );

        // Map to generic port calls
        let tempo_range = TempoRange::from_bpm(params.tempo);
        let key = StatisticsKey::Rhythm(
            TrainingDiscipline::RhythmOffsetDetection,
            tempo_range,
            params.direction,
        );
        let metric = completed.metric_value();

        // Update profile
        self.profile_port
            .update_profile(key, completed.timestamp(), metric, is_correct);

        // Persist training record
        let record = RhythmOffsetDetectionRecord::from_completed(&completed);
        self.record_port
            .save_record(TrainingRecord::RhythmOffsetDetection(record));

        // Update progress timeline
        self.timeline_port.add_metric(
            TrainingDiscipline::RhythmOffsetDetection,
            completed.timestamp(),
            metric,
        );

        self.is_last_answer_correct = is_correct;
        self.show_feedback = true;
        self.last_completed = Some(completed);
        self.state = RhythmOffsetDetectionSessionState::ShowingFeedback;
    }

    /// Called when the feedback display period finishes. Transitions to Idle (ready for next trial).
    pub fn feedback_complete(&mut self) {
        assert_eq!(
            self.state,
            RhythmOffsetDetectionSessionState::ShowingFeedback,
            "feedback_complete() requires ShowingFeedback state"
        );
        self.show_feedback = false;
        self.state = RhythmOffsetDetectionSessionState::Idle;
    }

    /// Stop the session and return to Idle.
    pub fn stop(&mut self) {
        if self.state == RhythmOffsetDetectionSessionState::Idle {
            return;
        }
        self.state = RhythmOffsetDetectionSessionState::Idle;
        self.current_trial_params = None;
        self.last_completed = None;
        self.show_feedback = false;
        self.is_last_answer_correct = false;
    }

    /// Stop if running, reset profile, strategy, and all resettables.
    pub fn reset_training_data(&mut self) {
        self.stop();
        self.last_completed = None;
        self.strategy.reset();
        self.profile.borrow_mut().reset_all();
        for resettable in &mut self.resettables {
            resettable.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Adaptive strategy tests ---

    #[test]
    fn test_strategy_default_difficulty() {
        let strategy = AdaptiveRhythmOffsetStrategy::new();
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 10.0).abs() < 1e-10);
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_on_beat_returns_zero() {
        let strategy = AdaptiveRhythmOffsetStrategy::new();
        assert!((strategy.difficulty_pct(RhythmDirection::OnBeat) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_narrow_on_correct() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::Early, true);
        // 10.0 * 0.85 = 8.5
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 8.5).abs() < 1e-10);
        // Late unchanged
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_widen_on_incorrect() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::Late, false);
        // 10.0 * 1.2 = 12.0
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 12.0).abs() < 1e-10);
        // Early unchanged
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_independent_directions() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::Early, true); // 10 * 0.85 = 8.5
        strategy.update(RhythmDirection::Late, false); // 10 * 1.2 = 12.0
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 8.5).abs() < 1e-10);
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_clamp_to_min() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        // Narrow many times to hit floor
        for _ in 0..50 {
            strategy.update(RhythmDirection::Early, true);
        }
        assert!(
            (strategy.difficulty_pct(RhythmDirection::Early) - MIN_DIFFICULTY_PCT).abs() < 1e-10
        );
    }

    #[test]
    fn test_strategy_clamp_to_max() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        // Widen many times to hit ceiling
        for _ in 0..50 {
            strategy.update(RhythmDirection::Late, false);
        }
        assert!(
            (strategy.difficulty_pct(RhythmDirection::Late) - MAX_DIFFICULTY_PCT).abs() < 1e-10
        );
    }

    #[test]
    fn test_strategy_reset() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::Early, true);
        strategy.update(RhythmDirection::Late, false);
        strategy.reset();
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 10.0).abs() < 1e-10);
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_on_beat_update_is_noop() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::OnBeat, true);
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 10.0).abs() < 1e-10);
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_multiple_narrows() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::Early, true); // 10.0 * 0.85 = 8.5
        strategy.update(RhythmDirection::Early, true); // 8.5 * 0.85 = 7.225
        assert!((strategy.difficulty_pct(RhythmDirection::Early) - 7.225).abs() < 1e-10);
    }

    #[test]
    fn test_strategy_multiple_widens() {
        let mut strategy = AdaptiveRhythmOffsetStrategy::new();
        strategy.update(RhythmDirection::Late, false); // 10.0 * 1.2 = 12.0
        strategy.update(RhythmDirection::Late, false); // 12.0 * 1.2 = 14.4
        assert!((strategy.difficulty_pct(RhythmDirection::Late) - 14.4).abs() < 1e-10);
    }

    // --- Mock port implementations for session tests ---

    struct MockProfilePort {
        calls: Vec<(StatisticsKey, String, f64, bool)>,
    }

    impl MockProfilePort {
        fn new() -> Self {
            Self { calls: Vec::new() }
        }
    }

    impl ProfileUpdating for MockProfilePort {
        fn update_profile(
            &mut self,
            key: StatisticsKey,
            timestamp: &str,
            value: f64,
            is_correct: bool,
        ) {
            self.calls
                .push((key, timestamp.to_string(), value, is_correct));
        }
    }

    struct MockRecordPort {
        calls: Vec<TrainingRecord>,
    }

    impl MockRecordPort {
        fn new() -> Self {
            Self { calls: Vec::new() }
        }
    }

    impl TrainingRecordPersisting for MockRecordPort {
        fn save_record(&self, record: TrainingRecord) {
            // Use interior mutability for testing since trait takes &self
            // For simplicity in tests, we just verify the call compiles.
            // Real verification is done via session state.
            let _ = record;
        }
    }

    struct MockTimelinePort {
        calls: Vec<(TrainingDiscipline, String, f64)>,
    }

    impl MockTimelinePort {
        fn new() -> Self {
            Self { calls: Vec::new() }
        }
    }

    impl ProgressTimelineUpdating for MockTimelinePort {
        fn add_metric(&mut self, discipline: TrainingDiscipline, timestamp: &str, value: f64) {
            self.calls.push((discipline, timestamp.to_string(), value));
        }
    }

    fn make_session() -> RhythmOffsetDetectionSession {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        RhythmOffsetDetectionSession::new(
            profile,
            Box::new(MockProfilePort::new()),
            Box::new(MockRecordPort::new()),
            Box::new(MockTimelinePort::new()),
            Vec::new(),
        )
    }

    // --- Session state transition tests ---

    #[test]
    fn test_session_initial_state_is_idle() {
        let session = make_session();
        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Idle);
        assert!(!session.show_feedback());
        assert!(!session.is_last_answer_correct());
        assert!(session.current_trial_params().is_none());
    }

    #[test]
    fn test_start_trial_transitions_to_playing() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Playing);
        assert!(session.current_trial_params().is_some());
    }

    #[test]
    fn test_start_trial_generates_valid_params() {
        let mut session = make_session();
        let tempo = TempoBPM::new(80);
        session.start_trial(tempo);

        let params = session.current_trial_params().unwrap();
        assert_eq!(params.tempo, tempo);
        assert!(
            params.direction == RhythmDirection::Early || params.direction == RhythmDirection::Late
        );
        // Offset should match direction sign
        if params.direction == RhythmDirection::Early {
            assert!(params.offset.ms() < 0.0);
        } else {
            assert!(params.offset.ms() > 0.0);
        }
    }

    #[test]
    fn test_pattern_finished_transitions_to_awaiting_answer() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        assert_eq!(
            session.state(),
            RhythmOffsetDetectionSessionState::AwaitingAnswer
        );
    }

    #[test]
    fn test_submit_correct_answer_transitions_to_showing_feedback() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();

        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        assert_eq!(
            session.state(),
            RhythmOffsetDetectionSessionState::ShowingFeedback
        );
        assert!(session.show_feedback());
        assert!(session.is_last_answer_correct());
    }

    #[test]
    fn test_submit_incorrect_answer() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();

        let params = session.current_trial_params().unwrap();
        let is_early = params.direction != RhythmDirection::Early; // wrong answer
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        assert_eq!(
            session.state(),
            RhythmOffsetDetectionSessionState::ShowingFeedback
        );
        assert!(session.show_feedback());
        assert!(!session.is_last_answer_correct());
    }

    #[test]
    fn test_feedback_complete_transitions_to_idle() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());
        session.feedback_complete();

        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Idle);
        assert!(!session.show_feedback());
    }

    #[test]
    fn test_full_cycle_then_next_trial() {
        let mut session = make_session();

        // First trial
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());
        session.feedback_complete();

        // Second trial should work
        session.start_trial(TempoBPM::new(80));
        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Playing);
    }

    #[test]
    fn test_stop_returns_to_idle() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        session.stop();

        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Idle);
        assert!(session.current_trial_params().is_none());
        assert!(!session.show_feedback());
    }

    #[test]
    fn test_stop_when_idle_is_noop() {
        let mut session = make_session();
        session.stop(); // Should not panic
        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Idle);
    }

    #[test]
    fn test_reset_training_data_resets_strategy() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        session.reset_training_data();

        assert_eq!(session.state(), RhythmOffsetDetectionSessionState::Idle);
        assert!((session.strategy().difficulty_pct(RhythmDirection::Early) - 10.0).abs() < 1e-10);
        assert!((session.strategy().difficulty_pct(RhythmDirection::Late) - 10.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "start_trial() requires Idle state")]
    fn test_start_trial_from_playing_panics() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.start_trial(TempoBPM::new(80)); // should panic
    }

    #[test]
    #[should_panic(expected = "pattern_finished() requires Playing state")]
    fn test_pattern_finished_from_idle_panics() {
        let mut session = make_session();
        session.pattern_finished();
    }

    #[test]
    #[should_panic(expected = "submit_answer() requires AwaitingAnswer state")]
    fn test_submit_answer_from_idle_panics() {
        let mut session = make_session();
        session.submit_answer(true, "2026-03-25T12:00:00Z".to_string());
    }

    #[test]
    #[should_panic(expected = "feedback_complete() requires ShowingFeedback state")]
    fn test_feedback_complete_from_idle_panics() {
        let mut session = make_session();
        session.feedback_complete();
    }

    #[test]
    fn test_submit_answer_creates_completed_trial() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        let completed = session.last_completed().unwrap();
        assert_eq!(completed.tempo().bpm(), 80);
        assert!(completed.is_correct());
        assert_eq!(completed.timestamp(), "2026-03-25T12:00:00Z");
    }

    #[test]
    fn test_submit_answer_updates_strategy() {
        let mut session = make_session();
        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let direction = params.direction;
        let is_early = direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        // Correct answer should narrow difficulty for that direction
        let expected = 10.0 * NARROW_FACTOR; // 8.5
        assert!((session.strategy().difficulty_pct(direction) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_offset_magnitude_matches_difficulty() {
        let mut session = make_session();
        let tempo = TempoBPM::new(80);
        session.start_trial(tempo);

        let params = session.current_trial_params().unwrap();
        let sixteenth_ms = tempo.sixteenth_note_duration_secs() * 1000.0;
        let expected_offset_ms = sixteenth_ms * 10.0 / 100.0; // 10% of sixteenth
        assert!((params.offset.abs_ms() - expected_offset_ms).abs() < 1e-10);
    }

    // --- Port call verification using interior mutability ---

    use std::cell::Cell;

    struct TrackingProfilePort {
        call_count: Cell<usize>,
        last_key: Cell<Option<StatisticsKey>>,
    }

    impl TrackingProfilePort {
        fn new() -> Self {
            Self {
                call_count: Cell::new(0),
                last_key: Cell::new(None),
            }
        }
    }

    impl ProfileUpdating for TrackingProfilePort {
        fn update_profile(
            &mut self,
            key: StatisticsKey,
            _timestamp: &str,
            _value: f64,
            _is_correct: bool,
        ) {
            self.call_count.set(self.call_count.get() + 1);
            self.last_key.set(Some(key));
        }
    }

    struct TrackingRecordPort {
        call_count: Cell<usize>,
    }

    impl TrackingRecordPort {
        fn new() -> Self {
            Self {
                call_count: Cell::new(0),
            }
        }
    }

    impl TrainingRecordPersisting for TrackingRecordPort {
        fn save_record(&self, _record: TrainingRecord) {
            self.call_count.set(self.call_count.get() + 1);
        }
    }

    struct TrackingTimelinePort {
        call_count: Cell<usize>,
        last_discipline: Cell<Option<TrainingDiscipline>>,
    }

    impl TrackingTimelinePort {
        fn new() -> Self {
            Self {
                call_count: Cell::new(0),
                last_discipline: Cell::new(None),
            }
        }
    }

    impl ProgressTimelineUpdating for TrackingTimelinePort {
        fn add_metric(&mut self, discipline: TrainingDiscipline, _timestamp: &str, _value: f64) {
            self.call_count.set(self.call_count.get() + 1);
            self.last_discipline.set(Some(discipline));
        }
    }

    #[test]
    fn test_submit_answer_calls_all_ports() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let profile_port = Box::new(TrackingProfilePort::new());
        let record_port = Box::new(TrackingRecordPort::new());
        let timeline_port = Box::new(TrackingTimelinePort::new());

        let profile_port_ptr = &*profile_port as *const TrackingProfilePort;
        let record_port_ptr = &*record_port as *const TrackingRecordPort;
        let timeline_port_ptr = &*timeline_port as *const TrackingTimelinePort;

        let mut session = RhythmOffsetDetectionSession::new(
            profile,
            profile_port,
            record_port,
            timeline_port,
            Vec::new(),
        );

        session.start_trial(TempoBPM::new(80));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        // SAFETY: The session owns the ports, and we're in the same test function.
        // The pointers are valid because the session (and its ports) are still alive.
        unsafe {
            assert_eq!((*profile_port_ptr).call_count.get(), 1);
            assert_eq!((*record_port_ptr).call_count.get(), 1);
            assert_eq!((*timeline_port_ptr).call_count.get(), 1);

            // Verify correct StatisticsKey variant
            let key = (*profile_port_ptr).last_key.get().unwrap();
            match key {
                StatisticsKey::Rhythm(discipline, _tempo_range, _direction) => {
                    assert_eq!(discipline, TrainingDiscipline::RhythmOffsetDetection);
                }
                _ => panic!("Expected Rhythm StatisticsKey"),
            }

            // Verify timeline uses correct discipline
            assert_eq!(
                (*timeline_port_ptr).last_discipline.get().unwrap(),
                TrainingDiscipline::RhythmOffsetDetection
            );
        }
    }

    #[test]
    fn test_statistics_key_uses_correct_tempo_range() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let profile_port = Box::new(TrackingProfilePort::new());
        let profile_port_ptr = &*profile_port as *const TrackingProfilePort;

        let mut session = RhythmOffsetDetectionSession::new(
            profile,
            profile_port,
            Box::new(MockRecordPort::new()),
            Box::new(MockTimelinePort::new()),
            Vec::new(),
        );

        // Use 120 BPM = Fast tempo range
        session.start_trial(TempoBPM::new(120));
        session.pattern_finished();
        let params = session.current_trial_params().unwrap();
        let is_early = params.direction == RhythmDirection::Early;
        session.submit_answer(is_early, "2026-03-25T12:00:00Z".to_string());

        unsafe {
            let key = (*profile_port_ptr).last_key.get().unwrap();
            match key {
                StatisticsKey::Rhythm(_, tempo_range, _) => {
                    assert_eq!(tempo_range, TempoRange::Fast);
                }
                _ => panic!("Expected Rhythm StatisticsKey"),
            }
        }
    }
}
