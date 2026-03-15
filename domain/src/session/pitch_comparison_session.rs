use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use rand::prelude::IndexedRandom;

use crate::ports::{PitchComparisonObserver, Resettable, UserSettings};
use crate::profile::PerceptualNote;
use crate::profile::PerceptualProfile;
use crate::strategy::{MIN_CENT_DIFFERENCE, TrainingSettings, next_pitch_comparison};
use crate::training::{CompletedPitchComparison, PitchComparison};
use crate::tuning::TuningSystem;
use crate::types::{
    AmplitudeDB, Cents, DetunedMIDINote, DirectedInterval, Frequency, MIDINote, NoteDuration,
    NoteRange,
};

/// Feedback display duration in seconds.
pub const FEEDBACK_DURATION_SECS: f64 = 0.4;

/// Scaling factor for amplitude variation (±10 dB at max vary_loudness).
pub const AMPLITUDE_VARY_SCALING: f64 = 10.0;

/// State of the comparison session state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchComparisonSessionState {
    Idle,
    PlayingReferenceNote,
    PlayingTargetNote,
    AwaitingAnswer,
    ShowingFeedback,
}

/// Data needed by the web layer to play the current comparison's notes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchComparisonPlaybackData {
    pub reference_frequency: Frequency,
    pub target_frequency: Frequency,
    pub duration: NoteDuration,
    pub target_amplitude_db: AmplitudeDB,
}

/// Pure domain state machine for comparison training sessions.
///
/// Manages the training loop state, comparison generation, answer processing,
/// and observer notification. No browser dependencies — the web crate drives
/// the async loop and audio playback.
pub struct PitchComparisonSession {
    state: PitchComparisonSessionState,
    profile: Rc<RefCell<PerceptualProfile>>,
    observers: Vec<Box<dyn PitchComparisonObserver>>,
    resettables: Vec<Box<dyn Resettable>>,

    // Session-level state (snapshot from settings at start)
    session_intervals: HashSet<DirectedInterval>,
    session_tuning_system: TuningSystem,
    session_reference_pitch: Frequency,
    session_note_duration: NoteDuration,
    session_vary_loudness: f64,
    session_note_range: NoteRange,

    // Current comparison state
    current_comparison: Option<PitchComparison>,
    current_playback_data: Option<PitchComparisonPlaybackData>,
    last_completed: Option<CompletedPitchComparison>,

    // Observable feedback state
    show_feedback: bool,
    is_last_answer_correct: bool,
    session_best_cent_difference: Option<f64>,
}

impl PitchComparisonSession {
    pub fn new(
        profile: Rc<RefCell<PerceptualProfile>>,
        observers: Vec<Box<dyn PitchComparisonObserver>>,
        resettables: Vec<Box<dyn Resettable>>,
    ) -> Self {
        Self {
            state: PitchComparisonSessionState::Idle,
            profile,
            observers,
            resettables,
            session_intervals: HashSet::new(),
            session_tuning_system: TuningSystem::EqualTemperament,
            session_reference_pitch: Frequency::CONCERT_440,
            session_note_duration: NoteDuration::new(1.0),
            session_vary_loudness: 0.0,
            session_note_range: NoteRange::new(MIDINote::new(36), MIDINote::new(84)),
            current_comparison: None,
            current_playback_data: None,
            last_completed: None,
            show_feedback: false,
            is_last_answer_correct: false,
            session_best_cent_difference: None,
        }
    }

    // --- Observable state accessors ---

    pub fn state(&self) -> PitchComparisonSessionState {
        self.state
    }

    pub fn show_feedback(&self) -> bool {
        self.show_feedback
    }

    pub fn is_last_answer_correct(&self) -> bool {
        self.is_last_answer_correct
    }

    pub fn session_best_cent_difference(&self) -> Option<f64> {
        self.session_best_cent_difference
    }

    pub fn last_cent_difference(&self) -> Option<f64> {
        self.last_completed
            .as_ref()
            .map(|c| c.pitch_comparison().target_note().offset.magnitude())
    }

    pub fn current_interval(&self) -> Option<DirectedInterval> {
        self.current_comparison
            .as_ref()
            .and_then(|c| DirectedInterval::between(c.reference_note(), c.target_note().note).ok())
    }

    pub fn current_playback_data(&self) -> Option<PitchComparisonPlaybackData> {
        self.current_playback_data
    }

    // --- State transitions ---

    /// Start a new comparison training session.
    ///
    /// Snapshots settings, generates the first comparison, transitions to PlayingReferenceNote.
    /// Panics if not idle or if intervals is empty.
    pub fn start(&mut self, intervals: HashSet<DirectedInterval>, settings: &dyn UserSettings) {
        assert_eq!(
            self.state,
            PitchComparisonSessionState::Idle,
            "start() requires Idle state"
        );
        assert!(
            !intervals.is_empty(),
            "start() requires at least one interval"
        );

        // Snapshot settings for the session
        self.session_intervals = intervals;
        self.session_tuning_system = settings.tuning_system();
        self.session_reference_pitch = settings.reference_pitch();
        self.session_note_duration = settings.note_duration();
        self.session_vary_loudness = settings.vary_loudness();
        self.session_note_range = settings.note_range();

        // Reset session-level transient state
        self.last_completed = None;
        self.show_feedback = false;
        self.is_last_answer_correct = false;
        self.session_best_cent_difference = None;

        // Generate first comparison
        self.generate_next_pitch_comparison();
        self.state = PitchComparisonSessionState::PlayingReferenceNote;
    }

    /// Called when reference note playback finishes. Transitions to PlayingTargetNote.
    pub fn on_reference_note_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchComparisonSessionState::PlayingReferenceNote,
            "on_reference_note_finished() requires PlayingReferenceNote state"
        );
        self.state = PitchComparisonSessionState::PlayingTargetNote;
    }

    /// Called when target note playback finishes. Transitions to AwaitingAnswer.
    pub fn on_target_note_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchComparisonSessionState::PlayingTargetNote,
            "on_target_note_finished() requires PlayingTargetNote state"
        );
        self.state = PitchComparisonSessionState::AwaitingAnswer;
    }

    /// Handle the user's answer (higher/lower).
    ///
    /// Valid from PlayingTargetNote (early answer) or AwaitingAnswer.
    /// Creates CompletedPitchComparison, notifies observers, transitions to ShowingFeedback.
    pub fn handle_answer(&mut self, is_higher: bool, timestamp: String) {
        assert!(
            self.state == PitchComparisonSessionState::PlayingTargetNote
                || self.state == PitchComparisonSessionState::AwaitingAnswer,
            "handle_answer() requires PlayingTargetNote or AwaitingAnswer state"
        );

        let comparison = self
            .current_comparison
            .expect("handle_answer() called without a current comparison");

        let completed = CompletedPitchComparison::new(
            comparison,
            is_higher,
            self.session_tuning_system,
            timestamp,
        );

        // Update session best cent difference (all attempts, not just correct)
        let cent_diff = comparison.target_note().offset.magnitude();
        match self.session_best_cent_difference {
            Some(best) if cent_diff < best => {
                self.session_best_cent_difference = Some(cent_diff);
            }
            None => {
                self.session_best_cent_difference = Some(cent_diff);
            }
            _ => {}
        }

        self.is_last_answer_correct = completed.is_correct();
        self.show_feedback = true;

        #[cfg(feature = "training-log")]
        log::info!(
            "Answer was {} (target was {})",
            if completed.is_correct() {
                "\u{2713} CORRECT"
            } else {
                "\u{2717} WRONG"
            },
            if comparison.is_target_higher() {
                "higher"
            } else {
                "lower"
            },
        );

        // Notify observers with panic isolation
        self.notify_observers(&completed);

        self.last_completed = Some(completed);
        self.state = PitchComparisonSessionState::ShowingFeedback;
    }

    /// Called when the feedback display period finishes.
    /// Generates next comparison, transitions to PlayingReferenceNote.
    pub fn on_feedback_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchComparisonSessionState::ShowingFeedback,
            "on_feedback_finished() requires ShowingFeedback state"
        );
        self.show_feedback = false;
        self.generate_next_pitch_comparison();
        self.state = PitchComparisonSessionState::PlayingReferenceNote;
    }

    /// Stop the session and return to Idle.
    pub fn stop(&mut self) {
        if self.state == PitchComparisonSessionState::Idle {
            return;
        }
        self.state = PitchComparisonSessionState::Idle;
        self.current_comparison = None;
        self.current_playback_data = None;
        self.last_completed = None;
        self.show_feedback = false;
        self.is_last_answer_correct = false;
        self.session_best_cent_difference = None;
    }

    /// Stop if running, reset profile, and call reset on all resettables.
    pub fn reset_training_data(&mut self) {
        self.stop();
        self.last_completed = None;
        self.session_best_cent_difference = None;
        self.profile.borrow_mut().reset();
        for resettable in &mut self.resettables {
            resettable.reset();
        }
    }

    // --- Private helpers ---

    fn generate_next_pitch_comparison(&mut self) {
        let interval = self.random_interval();
        let profile = self.profile.borrow();
        let training_settings = TrainingSettings::new(
            self.session_note_range,
            self.session_reference_pitch,
            Cents::new(MIN_CENT_DIFFERENCE),
            Cents::new(PerceptualNote::COLD_START_DIFFICULTY),
        );

        let comparison = next_pitch_comparison(
            &profile,
            &training_settings,
            self.last_completed.as_ref(),
            interval,
        );
        drop(profile); // Release borrow before computing frequencies

        // Calculate playback data
        let ref_detuned = DetunedMIDINote::from(comparison.reference_note());
        let reference_frequency = self
            .session_tuning_system
            .frequency(ref_detuned, self.session_reference_pitch);
        let target_frequency = self
            .session_tuning_system
            .frequency(comparison.target_note(), self.session_reference_pitch);
        let target_amplitude_db = calculate_target_amplitude(self.session_vary_loudness);

        self.current_comparison = Some(comparison);
        self.current_playback_data = Some(PitchComparisonPlaybackData {
            reference_frequency,
            target_frequency,
            duration: self.session_note_duration,
            target_amplitude_db,
        });

        #[cfg(feature = "training-log")]
        {
            let interval_str = DirectedInterval::between(
                comparison.reference_note(),
                comparison.target_note().note,
            )
            .map_or("?".to_string(), |di| {
                format!("{:?}{:?}", di.interval, di.direction)
            });
            let tuning_str = match self.session_tuning_system {
                TuningSystem::EqualTemperament => "ET",
                TuningSystem::JustIntonation => "JI",
            };
            log::info!(
                "PitchComparison: ref={} {:.2}Hz @0.0dB, target {:.2}Hz @{:.1}dB, offset={:.1}, interval={}, tuning={}, higher={}",
                comparison.reference_note().raw_value(),
                reference_frequency.raw_value(),
                target_frequency.raw_value(),
                target_amplitude_db.raw_value(),
                comparison.target_note().offset.raw_value,
                interval_str,
                tuning_str,
                comparison.is_target_higher(),
            );
        }
    }

    fn random_interval(&self) -> DirectedInterval {
        let intervals_vec: Vec<_> = self.session_intervals.iter().collect();
        let mut rng = rand::rng();
        **intervals_vec
            .choose(&mut rng)
            .expect("session_intervals must not be empty")
    }

    fn notify_observers(&mut self, completed: &CompletedPitchComparison) {
        for observer in &mut self.observers {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                observer.pitch_comparison_completed(completed);
            }));
            if let Err(e) = result {
                eprintln!("Observer panicked: {:?}", e);
            }
        }
    }
}

/// Calculate target amplitude variation based on vary_loudness setting.
fn calculate_target_amplitude(vary_loudness: f64) -> AmplitudeDB {
    if vary_loudness <= 0.0 {
        return AmplitudeDB::new(0.0);
    }
    let range = vary_loudness * AMPLITUDE_VARY_SCALING;
    let offset = rand::random::<f64>() * 2.0 * range - range;
    AmplitudeDB::new(offset as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, Interval, NoteRange};
    use std::cell::Cell;
    use std::time::Duration;

    // --- Mock types ---

    struct MockObserver {
        calls: Rc<RefCell<Vec<CompletedPitchComparison>>>,
    }

    impl MockObserver {
        fn new() -> (Self, Rc<RefCell<Vec<CompletedPitchComparison>>>) {
            let calls = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    calls: Rc::clone(&calls),
                },
                calls,
            )
        }
    }

    impl PitchComparisonObserver for MockObserver {
        fn pitch_comparison_completed(&mut self, completed: &CompletedPitchComparison) {
            self.calls.borrow_mut().push(completed.clone());
        }
    }

    struct PanickingObserver;

    impl PitchComparisonObserver for PanickingObserver {
        fn pitch_comparison_completed(&mut self, _completed: &CompletedPitchComparison) {
            panic!("PanickingObserver intentionally panicked");
        }
    }

    struct MockResettable {
        reset_count: Rc<Cell<u32>>,
    }

    impl MockResettable {
        fn new() -> (Self, Rc<Cell<u32>>) {
            let count = Rc::new(Cell::new(0));
            (
                Self {
                    reset_count: Rc::clone(&count),
                },
                count,
            )
        }
    }

    impl Resettable for MockResettable {
        fn reset(&mut self) {
            self.reset_count.set(self.reset_count.get() + 1);
        }
    }

    struct DefaultTestSettings;

    impl UserSettings for DefaultTestSettings {
        fn note_range(&self) -> NoteRange {
            NoteRange::new(MIDINote::new(36), MIDINote::new(84))
        }
        fn note_duration(&self) -> NoteDuration {
            NoteDuration::new(1.0)
        }
        fn reference_pitch(&self) -> Frequency {
            Frequency::CONCERT_440
        }
        fn tuning_system(&self) -> TuningSystem {
            TuningSystem::EqualTemperament
        }
        fn vary_loudness(&self) -> f64 {
            0.0
        }
        fn note_gap(&self) -> Duration {
            Duration::ZERO
        }
    }

    struct LoudnessTestSettings {
        vary_loudness: f64,
    }

    impl UserSettings for LoudnessTestSettings {
        fn note_range(&self) -> NoteRange {
            NoteRange::new(MIDINote::new(36), MIDINote::new(84))
        }
        fn note_duration(&self) -> NoteDuration {
            NoteDuration::new(1.0)
        }
        fn reference_pitch(&self) -> Frequency {
            Frequency::CONCERT_440
        }
        fn tuning_system(&self) -> TuningSystem {
            TuningSystem::EqualTemperament
        }
        fn vary_loudness(&self) -> f64 {
            self.vary_loudness
        }
        fn note_gap(&self) -> Duration {
            Duration::ZERO
        }
    }

    fn default_intervals() -> HashSet<DirectedInterval> {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
        set
    }

    fn create_session() -> PitchComparisonSession {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        PitchComparisonSession::new(profile, vec![], vec![])
    }

    fn create_session_with_observer() -> (
        PitchComparisonSession,
        Rc<RefCell<Vec<CompletedPitchComparison>>>,
    ) {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (observer, calls) = MockObserver::new();
        let session = PitchComparisonSession::new(profile, vec![Box::new(observer)], vec![]);
        (session, calls)
    }

    // --- AC1: Idle state tests ---

    #[test]
    fn test_idle_state_defaults() {
        let session = create_session();
        assert_eq!(session.state(), PitchComparisonSessionState::Idle);
        assert!(!session.show_feedback());
        assert!(!session.is_last_answer_correct());
        assert_eq!(session.session_best_cent_difference(), None);
        assert!(session.current_interval().is_none());
        assert!(session.current_playback_data().is_none());
    }

    // --- AC2: Start tests ---

    #[test]
    fn test_start_transitions_to_playing_reference_note() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::PlayingReferenceNote
        );
    }

    #[test]
    fn test_start_generates_comparison() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        assert!(session.current_playback_data().is_some());
        assert!(session.current_interval().is_some());
    }

    #[test]
    #[should_panic(expected = "start() requires Idle state")]
    fn test_start_panics_when_not_idle() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        // Already in PlayingReferenceNote, should panic
        session.start(default_intervals(), &DefaultTestSettings);
    }

    #[test]
    #[should_panic(expected = "start() requires at least one interval")]
    fn test_start_panics_with_empty_intervals() {
        let mut session = create_session();
        session.start(HashSet::new(), &DefaultTestSettings);
    }

    // --- AC3: Note playback data ---

    #[test]
    fn test_playback_data_has_frequencies_and_duration() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        let data = session.current_playback_data().unwrap();
        assert!(data.reference_frequency.raw_value() > 0.0);
        assert!(data.target_frequency.raw_value() > 0.0);
        assert!((data.duration.raw_value() - 1.0).abs() < f64::EPSILON);
    }

    // --- AC4: ReferenceNote to TargetNote transition ---

    #[test]
    fn test_on_reference_note_finished_transitions_to_playing_target_note() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::PlayingTargetNote
        );
    }

    // --- AC5: TargetNote to AwaitingAnswer ---

    #[test]
    fn test_on_target_note_finished_transitions_to_awaiting_answer() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        assert_eq!(session.state(), PitchComparisonSessionState::AwaitingAnswer);
    }

    // --- AC6: Handle answer ---

    #[test]
    fn test_handle_answer_from_awaiting_transitions_to_showing_feedback() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::ShowingFeedback
        );
        assert!(session.show_feedback());
    }

    // --- AC6 early answer: handle_answer from PlayingTargetNote ---

    #[test]
    fn test_early_answer_from_playing_target_note() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        // Answer while still playing target note (early answer)
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::ShowingFeedback
        );
    }

    // --- AC7: Feedback state ---

    #[test]
    fn test_on_feedback_finished_generates_next_pitch_comparison() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        session.on_feedback_finished();
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::PlayingReferenceNote
        );
        assert!(!session.show_feedback());
        assert!(session.current_playback_data().is_some());
    }

    // --- Full lifecycle test ---

    #[test]
    fn test_full_lifecycle() {
        let mut session = create_session();

        // Idle
        assert_eq!(session.state(), PitchComparisonSessionState::Idle);

        // Start → PlayingReferenceNote
        session.start(default_intervals(), &DefaultTestSettings);
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::PlayingReferenceNote
        );

        // on_reference_note_finished → PlayingTargetNote
        session.on_reference_note_finished();
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::PlayingTargetNote
        );

        // on_target_note_finished → AwaitingAnswer
        session.on_target_note_finished();
        assert_eq!(session.state(), PitchComparisonSessionState::AwaitingAnswer);

        // handle_answer → ShowingFeedback
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::ShowingFeedback
        );
        assert!(session.show_feedback());

        // on_feedback_finished → PlayingReferenceNote (next cycle)
        session.on_feedback_finished();
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::PlayingReferenceNote
        );
        assert!(!session.show_feedback());
    }

    // --- Guard tests ---

    #[test]
    #[should_panic(expected = "handle_answer() requires PlayingTargetNote or AwaitingAnswer state")]
    fn test_handle_answer_invalid_from_idle() {
        let mut session = create_session();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
    }

    #[test]
    #[should_panic(expected = "on_reference_note_finished() requires PlayingReferenceNote state")]
    fn test_on_reference_note_finished_from_idle_panics() {
        let mut session = create_session();
        session.on_reference_note_finished();
    }

    #[test]
    #[should_panic(expected = "on_target_note_finished() requires PlayingTargetNote state")]
    fn test_on_target_note_finished_from_idle_panics() {
        let mut session = create_session();
        session.on_target_note_finished();
    }

    #[test]
    #[should_panic(expected = "on_feedback_finished() requires ShowingFeedback state")]
    fn test_on_feedback_finished_from_idle_panics() {
        let mut session = create_session();
        session.on_feedback_finished();
    }

    #[test]
    #[should_panic(expected = "handle_answer() requires PlayingTargetNote or AwaitingAnswer state")]
    fn test_handle_answer_from_playing_reference_note_panics() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
    }

    #[test]
    #[should_panic(expected = "handle_answer() requires PlayingTargetNote or AwaitingAnswer state")]
    fn test_handle_answer_from_showing_feedback_panics() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        // Now in ShowingFeedback — should panic
        session.handle_answer(false, "2026-03-03T14:01:00Z".to_string());
    }

    // --- CompletedPitchComparison correctness ---

    #[test]
    fn test_completed_comparison_has_correct_timestamp() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        let timestamp = "2026-03-03T14:30:00Z".to_string();
        session.handle_answer(true, timestamp.clone());

        let completed_calls = calls.borrow();
        assert_eq!(completed_calls.len(), 1);
        assert_eq!(completed_calls[0].timestamp(), timestamp);
    }

    #[test]
    fn test_completed_comparison_has_snapshot_tuning_system() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());

        let completed_calls = calls.borrow();
        assert_eq!(
            completed_calls[0].tuning_system(),
            TuningSystem::EqualTemperament
        );
    }

    #[test]
    fn test_completed_comparison_is_correct_derivation() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        // Get the current comparison's target direction
        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();

        // Answer correctly
        session.handle_answer(is_higher, "2026-03-03T14:00:00Z".to_string());

        let completed_calls = calls.borrow();
        assert!(completed_calls[0].is_correct());
        assert!(session.is_last_answer_correct());
    }

    // --- Observer notification ---

    #[test]
    fn test_observer_receives_pitch_comparison_completed() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());

        assert_eq!(calls.borrow().len(), 1);
    }

    // --- Observer error isolation ---

    #[test]
    fn test_observer_panic_does_not_propagate() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (normal_observer, normal_calls) = MockObserver::new();
        let panicking_observer = PanickingObserver;

        let mut session = PitchComparisonSession::new(
            profile,
            vec![Box::new(panicking_observer), Box::new(normal_observer)],
            vec![],
        );

        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());

        // Normal observer should still receive the event despite panicking observer
        assert_eq!(normal_calls.borrow().len(), 1);
        assert_eq!(
            session.state(),
            PitchComparisonSessionState::ShowingFeedback
        );
    }

    // --- AC8: No loudness variation ---

    #[test]
    fn test_amplitude_zero_vary_loudness() {
        let result = calculate_target_amplitude(0.0);
        assert_eq!(result.raw_value(), 0.0);
    }

    // --- AC9: Loudness variation ---

    #[test]
    fn test_amplitude_with_vary_loudness() {
        let vary = 0.5;
        let max_range = vary * 10.0; // 5.0

        for _ in 0..100 {
            let result = calculate_target_amplitude(vary);
            assert!(
                result.raw_value() >= -max_range as f32,
                "amplitude {} below expected -{}",
                result.raw_value(),
                max_range
            );
            assert!(
                result.raw_value() <= max_range as f32,
                "amplitude {} above expected {}",
                result.raw_value(),
                max_range
            );
        }
    }

    #[test]
    fn test_playback_data_amplitude_zero_when_no_vary() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        let data = session.current_playback_data().unwrap();
        assert_eq!(data.target_amplitude_db.raw_value(), 0.0);
    }

    #[test]
    fn test_playback_data_amplitude_varies_when_configured() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let mut session = PitchComparisonSession::new(profile, vec![], vec![]);
        let settings = LoudnessTestSettings { vary_loudness: 0.5 };
        session.start(default_intervals(), &settings);
        let data = session.current_playback_data().unwrap();
        // With vary_loudness=0.5, max range is ±5.0 dB
        assert!(data.target_amplitude_db.raw_value() >= -5.0);
        assert!(data.target_amplitude_db.raw_value() <= 5.0);
    }

    // --- AC10: Stop ---

    #[test]
    fn test_stop_returns_to_idle() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.stop();
        assert_eq!(session.state(), PitchComparisonSessionState::Idle);
        assert!(session.current_playback_data().is_none());
        assert!(!session.show_feedback());
    }

    #[test]
    fn test_stop_clears_all_session_state() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        // Answer correctly to populate is_last_answer_correct and session_best
        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        session.handle_answer(is_higher, "2026-03-03T14:00:00Z".to_string());
        assert!(session.is_last_answer_correct());
        assert!(session.session_best_cent_difference().is_some());

        session.stop();
        assert!(!session.is_last_answer_correct());
        assert_eq!(session.session_best_cent_difference(), None);
        assert!(session.current_interval().is_none());
    }

    #[test]
    fn test_stop_from_idle_is_noop() {
        let mut session = create_session();
        session.stop(); // Should not panic
        assert_eq!(session.state(), PitchComparisonSessionState::Idle);
    }

    // --- Session best tracking ---

    #[test]
    fn test_session_best_updates_on_all_answers() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);

        session.on_reference_note_finished();
        session.on_target_note_finished();
        // Answer incorrectly — session best should still update
        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        session.handle_answer(!is_higher, "2026-03-03T14:00:00Z".to_string());
        assert!(session.session_best_cent_difference().is_some());
    }

    #[test]
    fn test_session_best_tracks_smallest_on_correct() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);

        // Run two correct cycles, best should track the smallest
        for i in 0..2 {
            let data = session.current_playback_data().unwrap();
            let is_higher =
                data.target_frequency.raw_value() > data.reference_frequency.raw_value();
            session.on_reference_note_finished();
            session.on_target_note_finished();
            session.handle_answer(is_higher, format!("2026-03-03T14:0{}:00Z", i));
            session.on_feedback_finished();
        }

        // Session best should exist
        assert!(session.session_best_cent_difference().is_some());
        assert!(session.session_best_cent_difference().unwrap() > 0.0);
    }

    // --- Last cent difference ---

    #[test]
    fn test_last_cent_difference_none_initially() {
        let session = create_session();
        assert_eq!(session.last_cent_difference(), None);
    }

    #[test]
    fn test_last_cent_difference_populated_after_answer() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        session.handle_answer(is_higher, "2026-03-03T14:00:00Z".to_string());

        let diff = session.last_cent_difference();
        assert!(diff.is_some());
        assert!(diff.unwrap() > 0.0);
    }

    #[test]
    fn test_last_cent_difference_cleared_on_stop() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        session.handle_answer(is_higher, "2026-03-03T14:00:00Z".to_string());
        assert!(session.last_cent_difference().is_some());

        session.stop();
        assert_eq!(session.last_cent_difference(), None);
    }

    // --- Reset training data ---

    #[test]
    fn test_reset_training_data_stops_session() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.reset_training_data();
        assert_eq!(session.state(), PitchComparisonSessionState::Idle);
    }

    #[test]
    fn test_reset_training_data_resets_profile() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        profile
            .borrow_mut()
            .update(MIDINote::new(60), Cents::new(50.0), true);
        assert!(profile.borrow().overall_mean().is_some());

        let mut session = PitchComparisonSession::new(Rc::clone(&profile), vec![], vec![]);
        session.reset_training_data();
        assert_eq!(profile.borrow().overall_mean(), None);
    }

    #[test]
    fn test_reset_training_data_calls_resettables() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (resettable, count) = MockResettable::new();
        let mut session = PitchComparisonSession::new(profile, vec![], vec![Box::new(resettable)]);
        session.reset_training_data();
        assert_eq!(count.get(), 1);
    }

    // --- Feedback duration constant ---

    #[test]
    fn test_feedback_duration_constant() {
        assert!((FEEDBACK_DURATION_SECS - 0.4).abs() < f64::EPSILON);
    }
}
