use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use rand::prelude::IndexedRandom;

use crate::ports::{PitchMatchingObserver, Resettable, UserSettings};
use crate::profile::PerceptualProfile;
use crate::training::{CompletedPitchMatching, PitchMatchingChallenge};
use crate::tuning::TuningSystem;
use crate::types::{DirectedInterval, Direction, Frequency, MIDINote, NoteRange, NoteDuration};

/// MIDI velocity for pitch matching playback (fixed at 63).
pub const PITCH_MATCHING_VELOCITY: u8 = 63;

/// State of the pitch matching session state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchMatchingSessionState {
    Idle,
    PlayingReference,
    AwaitingSliderTouch,
    PlayingTunable,
    ShowingFeedback,
}

/// Data needed by the web layer to play the current challenge's notes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchMatchingPlaybackData {
    pub reference_frequency: Frequency,
    pub tunable_frequency: Frequency,
    pub duration: NoteDuration,
}

/// Pure domain state machine for pitch matching training sessions.
///
/// Manages the training loop state, challenge generation, pitch adjustment,
/// and observer notification. No browser dependencies — the web crate drives
/// the async loop and audio playback.
pub struct PitchMatchingSession {
    state: PitchMatchingSessionState,
    profile: Rc<RefCell<PerceptualProfile>>,
    observers: Vec<Box<dyn PitchMatchingObserver>>,
    resettables: Vec<Box<dyn Resettable>>,

    // Session-level state (snapshot from settings at start)
    session_intervals: HashSet<DirectedInterval>,
    session_tuning_system: TuningSystem,
    session_reference_pitch: Frequency,
    session_note_duration: NoteDuration,
    session_note_range: NoteRange,

    // Current challenge state
    current_challenge: Option<PitchMatchingChallenge>,
    current_playback_data: Option<PitchMatchingPlaybackData>,
    last_completed: Option<CompletedPitchMatching>,

    // Target frequency for pitch calculation (the "correct answer" frequency)
    target_frequency: Option<Frequency>,

    // Observable feedback state
    show_feedback: bool,
}

impl PitchMatchingSession {
    pub fn new(
        profile: Rc<RefCell<PerceptualProfile>>,
        observers: Vec<Box<dyn PitchMatchingObserver>>,
        resettables: Vec<Box<dyn Resettable>>,
    ) -> Self {
        Self {
            state: PitchMatchingSessionState::Idle,
            profile,
            observers,
            resettables,
            session_intervals: HashSet::new(),
            session_tuning_system: TuningSystem::EqualTemperament,
            session_reference_pitch: Frequency::CONCERT_440,
            session_note_duration: NoteDuration::new(1.0),
            session_note_range: NoteRange::new(MIDINote::new(36), MIDINote::new(84)),
            current_challenge: None,
            current_playback_data: None,
            last_completed: None,
            target_frequency: None,
            show_feedback: false,
        }
    }

    // --- Observable state accessors ---

    pub fn state(&self) -> PitchMatchingSessionState {
        self.state
    }

    pub fn show_feedback(&self) -> bool {
        self.show_feedback
    }

    pub fn last_completed(&self) -> Option<&CompletedPitchMatching> {
        self.last_completed.as_ref()
    }

    pub fn current_challenge(&self) -> Option<&PitchMatchingChallenge> {
        self.current_challenge.as_ref()
    }

    pub fn current_playback_data(&self) -> Option<PitchMatchingPlaybackData> {
        self.current_playback_data
    }

    pub fn current_interval(&self) -> Option<DirectedInterval> {
        self.current_challenge.as_ref().and_then(|c| {
            DirectedInterval::between(c.reference_note(), c.target_note()).ok()
        })
    }

    // --- State transitions ---

    /// Start a new pitch matching training session.
    ///
    /// Snapshots settings, generates the first challenge, transitions to PlayingReference.
    /// Panics if not idle or if intervals is empty.
    pub fn start(&mut self, intervals: HashSet<DirectedInterval>, settings: &dyn UserSettings) {
        assert_eq!(
            self.state,
            PitchMatchingSessionState::Idle,
            "start() requires Idle state"
        );
        assert!(!intervals.is_empty(), "start() requires at least one interval");

        // Snapshot settings for the session
        self.session_intervals = intervals;
        self.session_tuning_system = settings.tuning_system();
        self.session_reference_pitch = settings.reference_pitch();
        self.session_note_duration = settings.note_duration();
        self.session_note_range = settings.note_range();

        // Reset session-level transient state
        self.last_completed = None;
        self.show_feedback = false;

        // Generate first challenge
        self.generate_next_challenge();
        self.state = PitchMatchingSessionState::PlayingReference;
    }

    /// Called when reference note playback finishes.
    /// Computes tunable frequency with initial cent offset, transitions to AwaitingSliderTouch.
    pub fn on_reference_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchMatchingSessionState::PlayingReference,
            "on_reference_finished() requires PlayingReference state"
        );

        // Compute tunable frequency: target + initial cent offset
        let challenge = self.current_challenge.expect("challenge must exist in PlayingReference");
        let target_freq = self.target_frequency.expect("target_frequency must exist in PlayingReference");
        let initial_offset = challenge.initial_cent_offset();
        let tunable_frequency = Frequency::new(
            target_freq.raw_value() * 2.0_f64.powf(initial_offset / 1200.0),
        );

        // Update playback data with tunable frequency
        if let Some(ref mut data) = self.current_playback_data {
            data.tunable_frequency = tunable_frequency;
        }

        self.state = PitchMatchingSessionState::AwaitingSliderTouch;
    }

    /// Handle pitch adjustment from slider.
    ///
    /// If AwaitingSliderTouch: transitions to PlayingTunable and returns adjusted frequency.
    /// If PlayingTunable: returns adjusted frequency.
    /// Returns `None` if in wrong state.
    pub fn adjust_pitch(&mut self, value: f64) -> Option<Frequency> {
        match self.state {
            PitchMatchingSessionState::AwaitingSliderTouch => {
                self.state = PitchMatchingSessionState::PlayingTunable;
                Some(self.calculate_adjusted_frequency(value))
            }
            PitchMatchingSessionState::PlayingTunable => {
                Some(self.calculate_adjusted_frequency(value))
            }
            _ => None,
        }
    }

    /// Commit the pitch when slider is released.
    ///
    /// Calculates user_cent_error, creates CompletedPitchMatching, notifies observers,
    /// updates profile, transitions to ShowingFeedback.
    pub fn commit_pitch(&mut self, value: f64, timestamp: String) {
        assert_eq!(
            self.state,
            PitchMatchingSessionState::PlayingTunable,
            "commit_pitch() requires PlayingTunable state"
        );

        let challenge = self.current_challenge.expect("challenge must exist in PlayingTunable");
        let user_cent_error = value * 20.0;

        let completed = CompletedPitchMatching::new(
            challenge.reference_note(),
            challenge.target_note(),
            challenge.initial_cent_offset(),
            user_cent_error,
            self.session_tuning_system,
            timestamp,
        );

        // Notify observers with panic isolation
        self.notify_observers(&completed);

        // Update profile
        self.profile.borrow_mut().update_matching(
            completed.target_note(),
            completed.user_cent_error(),
        );

        self.show_feedback = true;
        self.last_completed = Some(completed);
        self.state = PitchMatchingSessionState::ShowingFeedback;
    }

    /// Called when the feedback display period finishes.
    /// Generates next challenge, transitions to PlayingReference.
    pub fn on_feedback_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchMatchingSessionState::ShowingFeedback,
            "on_feedback_finished() requires ShowingFeedback state"
        );
        self.show_feedback = false;
        self.generate_next_challenge();
        self.state = PitchMatchingSessionState::PlayingReference;
    }

    /// Stop the session and return to Idle.
    pub fn stop(&mut self) {
        if self.state == PitchMatchingSessionState::Idle {
            return;
        }
        self.state = PitchMatchingSessionState::Idle;
        self.current_challenge = None;
        self.current_playback_data = None;
        self.last_completed = None;
        self.target_frequency = None;
        self.show_feedback = false;
    }

    /// Stop if running, reset matching accumulators, and call reset on all resettables.
    pub fn reset_training_data(&mut self) {
        self.stop();
        self.last_completed = None;
        self.profile.borrow_mut().reset_matching();
        for resettable in &mut self.resettables {
            resettable.reset();
        }
    }

    // --- Private helpers ---

    fn generate_next_challenge(&mut self) {
        let interval = self.random_interval();
        let challenge = self.generate_challenge(interval);

        // Calculate playback data
        let reference_frequency = self
            .session_tuning_system
            .frequency_for_note(challenge.reference_note(), self.session_reference_pitch);
        let target_frequency = self
            .session_tuning_system
            .frequency_for_note(challenge.target_note(), self.session_reference_pitch);

        // Initial tunable frequency includes the random cent offset
        let initial_offset = challenge.initial_cent_offset();
        let tunable_frequency = Frequency::new(
            target_frequency.raw_value() * 2.0_f64.powf(initial_offset / 1200.0),
        );

        self.target_frequency = Some(target_frequency);
        self.current_challenge = Some(challenge);
        self.current_playback_data = Some(PitchMatchingPlaybackData {
            reference_frequency,
            tunable_frequency,
            duration: self.session_note_duration,
        });
    }

    fn generate_challenge(&self, interval: DirectedInterval) -> PitchMatchingChallenge {
        // Ensure transposed note stays in range
        let (min_raw, max_raw) = match interval.direction {
            Direction::Up => (
                self.session_note_range.min().raw_value(),
                self.session_note_range
                    .max()
                    .raw_value()
                    .saturating_sub(interval.interval.semitones()),
            ),
            Direction::Down => (
                self.session_note_range
                    .min()
                    .raw_value()
                    .saturating_add(interval.interval.semitones()),
                self.session_note_range.max().raw_value(),
            ),
        };

        let reference_note = MIDINote::random(min_raw..=max_raw);
        let target_note = reference_note
            .transposed(interval)
            .expect("range-adjusted reference ensures valid transposition");
        let initial_cent_offset = rand::random::<f64>() * 40.0 - 20.0; // [-20.0, +20.0]

        PitchMatchingChallenge::new(reference_note, target_note, initial_cent_offset)
    }

    fn random_interval(&self) -> DirectedInterval {
        let intervals_vec: Vec<_> = self.session_intervals.iter().collect();
        let mut rng = rand::rng();
        **intervals_vec
            .choose(&mut rng)
            .expect("session_intervals must not be empty")
    }

    fn calculate_adjusted_frequency(&self, value: f64) -> Frequency {
        let target_freq = self
            .target_frequency
            .expect("target_frequency must exist when adjusting pitch");
        let cent_offset = value * 20.0;
        Frequency::new(target_freq.raw_value() * 2.0_f64.powf(cent_offset / 1200.0))
    }

    fn notify_observers(&mut self, completed: &CompletedPitchMatching) {
        for observer in &mut self.observers {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                observer.pitch_matching_completed(completed);
            }));
            if let Err(e) = result {
                eprintln!("Observer panicked: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, Interval, NoteRange};
    use std::cell::Cell;

    // --- Mock types ---

    struct MockPitchMatchingObserver {
        calls: Rc<RefCell<Vec<CompletedPitchMatching>>>,
    }

    impl MockPitchMatchingObserver {
        fn new() -> (Self, Rc<RefCell<Vec<CompletedPitchMatching>>>) {
            let calls = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    calls: Rc::clone(&calls),
                },
                calls,
            )
        }
    }

    impl PitchMatchingObserver for MockPitchMatchingObserver {
        fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching) {
            self.calls.borrow_mut().push(completed.clone());
        }
    }

    struct PanickingPitchMatchingObserver;

    impl PitchMatchingObserver for PanickingPitchMatchingObserver {
        fn pitch_matching_completed(&mut self, _completed: &CompletedPitchMatching) {
            panic!("PanickingPitchMatchingObserver intentionally panicked");
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
    }

    fn default_intervals() -> HashSet<DirectedInterval> {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
        set
    }

    fn create_session() -> PitchMatchingSession {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        PitchMatchingSession::new(profile, vec![], vec![])
    }

    fn create_session_with_observer()
    -> (PitchMatchingSession, Rc<RefCell<Vec<CompletedPitchMatching>>>) {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (observer, calls) = MockPitchMatchingObserver::new();
        let session =
            PitchMatchingSession::new(profile, vec![Box::new(observer)], vec![]);
        (session, calls)
    }

    // --- AC1: Idle state tests ---

    #[test]
    fn test_idle_state_defaults() {
        let session = create_session();
        assert_eq!(session.state(), PitchMatchingSessionState::Idle);
        assert!(!session.show_feedback());
        assert!(session.last_completed().is_none());
        assert!(session.current_challenge().is_none());
        assert!(session.current_playback_data().is_none());
        assert!(session.current_interval().is_none());
    }

    // --- AC2: Start tests ---

    #[test]
    fn test_start_transitions_to_playing_reference() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingReference);
    }

    #[test]
    fn test_start_generates_challenge_with_playback_data() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        assert!(session.current_challenge().is_some());
        assert!(session.current_playback_data().is_some());
        assert!(session.current_interval().is_some());

        let data = session.current_playback_data().unwrap();
        assert!(data.reference_frequency.raw_value() > 0.0);
        assert!(data.tunable_frequency.raw_value() > 0.0);
        assert!((data.duration.raw_value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "start() requires Idle state")]
    fn test_start_panics_when_not_idle() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.start(default_intervals(), &DefaultTestSettings);
    }

    #[test]
    #[should_panic(expected = "start() requires at least one interval")]
    fn test_start_panics_with_empty_intervals() {
        let mut session = create_session();
        session.start(HashSet::new(), &DefaultTestSettings);
    }

    // --- AC3: Challenge generation ---

    #[test]
    fn test_challenge_generation_respects_note_range() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);

        // With Prime interval (Up), reference and target should be the same note
        let challenge = session.current_challenge().unwrap();
        assert!(challenge.reference_note().raw_value() >= 36);
        assert!(challenge.reference_note().raw_value() <= 84);
        assert!(challenge.target_note().raw_value() >= 36);
        assert!(challenge.target_note().raw_value() <= 84);
    }

    #[test]
    fn test_initial_cent_offset_within_range() {
        // Run multiple times to check the random offset stays in range
        for _ in 0..50 {
            let mut session = create_session();
            session.start(default_intervals(), &DefaultTestSettings);
            let challenge = session.current_challenge().unwrap();
            assert!(
                challenge.initial_cent_offset() >= -20.0,
                "offset {} below -20.0",
                challenge.initial_cent_offset()
            );
            assert!(
                challenge.initial_cent_offset() <= 20.0,
                "offset {} above 20.0",
                challenge.initial_cent_offset()
            );
        }
    }

    #[test]
    fn test_challenge_with_interval_transposition() {
        let mut intervals = HashSet::new();
        intervals.insert(DirectedInterval::new(Interval::PerfectFifth, Direction::Up));

        let mut session = create_session();
        session.start(intervals, &DefaultTestSettings);

        let challenge = session.current_challenge().unwrap();
        // Target should be 7 semitones above reference
        assert_eq!(
            challenge.target_note().raw_value() - challenge.reference_note().raw_value(),
            7
        );
        // Reference should be capped to allow room for the fifth
        assert!(challenge.reference_note().raw_value() <= 84 - 7);
    }

    // --- AC4: Reference finished transition ---

    #[test]
    fn test_on_reference_finished_transitions_to_awaiting_slider_touch() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        assert_eq!(session.state(), PitchMatchingSessionState::AwaitingSliderTouch);
    }

    #[test]
    fn test_on_reference_finished_provides_tunable_frequency_with_offset() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();

        let data = session.current_playback_data().unwrap();
        // Tunable frequency should be different from target due to offset
        // (unless offset is exactly 0, which is extremely unlikely)
        assert!(data.tunable_frequency.raw_value() > 0.0);
    }

    // --- AC5: First slider interaction ---

    #[test]
    fn test_adjust_pitch_from_awaiting_transitions_to_playing_tunable() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();

        let freq = session.adjust_pitch(0.0);
        assert!(freq.is_some());
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingTunable);
    }

    // --- AC6: Pitch adjustment ---

    #[test]
    fn test_adjust_pitch_returns_correct_frequency() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();

        // First touch
        session.adjust_pitch(0.0);

        // At value=0.0, adjusted frequency should equal target frequency
        let freq = session.adjust_pitch(0.0).unwrap();
        let target_freq = session.target_frequency.unwrap();
        assert!(
            (freq.raw_value() - target_freq.raw_value()).abs() < 1e-10,
            "At value=0.0, adjusted freq ({}) should equal target freq ({})",
            freq.raw_value(),
            target_freq.raw_value()
        );
    }

    #[test]
    fn test_adjust_pitch_at_boundaries() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.0); // transition to PlayingTunable

        let target_freq = session.target_frequency.unwrap().raw_value();

        // At +1.0: offset = +20 cents
        let freq_high = session.adjust_pitch(1.0).unwrap();
        let expected_high = target_freq * 2.0_f64.powf(20.0 / 1200.0);
        assert!(
            (freq_high.raw_value() - expected_high).abs() < 1e-10,
            "At +1.0: got {}, expected {}",
            freq_high.raw_value(),
            expected_high
        );

        // At -1.0: offset = -20 cents
        let freq_low = session.adjust_pitch(-1.0).unwrap();
        let expected_low = target_freq * 2.0_f64.powf(-20.0 / 1200.0);
        assert!(
            (freq_low.raw_value() - expected_low).abs() < 1e-10,
            "At -1.0: got {}, expected {}",
            freq_low.raw_value(),
            expected_low
        );
    }

    #[test]
    fn test_adjust_pitch_wrong_state_returns_none() {
        let mut session = create_session();
        assert!(session.adjust_pitch(0.0).is_none());

        session.start(default_intervals(), &DefaultTestSettings);
        assert!(session.adjust_pitch(0.0).is_none()); // PlayingReference
    }

    // --- AC7: Commit pitch ---

    #[test]
    fn test_commit_pitch_creates_completed_and_notifies_observers() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.5);
        session.commit_pitch(0.5, "2026-03-04T10:00:00Z".to_string());

        let completed_calls = calls.borrow();
        assert_eq!(completed_calls.len(), 1);
        assert_eq!(completed_calls[0].timestamp(), "2026-03-04T10:00:00Z");
    }

    #[test]
    fn test_commit_pitch_transitions_to_showing_feedback() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.0);
        session.commit_pitch(0.0, "2026-03-04T10:00:00Z".to_string());

        assert_eq!(session.state(), PitchMatchingSessionState::ShowingFeedback);
        assert!(session.show_feedback());
    }

    #[test]
    fn test_commit_pitch_user_cent_error_calculation() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.5);
        session.commit_pitch(0.5, "2026-03-04T10:00:00Z".to_string());

        let completed_calls = calls.borrow();
        // user_cent_error = value * 20.0 = 0.5 * 20.0 = 10.0
        assert!((completed_calls[0].user_cent_error() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_commit_pitch_negative_error() {
        let (mut session, calls) = create_session_with_observer();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(-0.75);
        session.commit_pitch(-0.75, "2026-03-04T10:00:00Z".to_string());

        let completed_calls = calls.borrow();
        // user_cent_error = -0.75 * 20.0 = -15.0
        assert!((completed_calls[0].user_cent_error() - (-15.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_commit_pitch_updates_profile() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let mut session = PitchMatchingSession::new(Rc::clone(&profile), vec![], vec![]);
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.5);
        session.commit_pitch(0.5, "2026-03-04T10:00:00Z".to_string());

        assert_eq!(profile.borrow().matching_count(), 1);
        assert!(profile.borrow().matching_mean().is_some());
    }

    // --- AC8: Feedback to next challenge ---

    #[test]
    fn test_on_feedback_finished_generates_next_challenge_and_loops() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.0);
        session.commit_pitch(0.0, "2026-03-04T10:00:00Z".to_string());

        let first_challenge = *session.current_challenge().unwrap();

        session.on_feedback_finished();
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingReference);
        assert!(!session.show_feedback());
        assert!(session.current_playback_data().is_some());
        assert!(session.current_challenge().is_some());

        // New challenge generated (may be different due to randomness, but exists)
        let _second_challenge = session.current_challenge().unwrap();
        // Just verify it's a valid challenge, not necessarily different (could be same due to Prime)
        let _ = first_challenge; // suppress unused warning
    }

    // --- Full lifecycle test ---

    #[test]
    fn test_full_lifecycle() {
        let mut session = create_session();

        // Idle
        assert_eq!(session.state(), PitchMatchingSessionState::Idle);

        // Start → PlayingReference
        session.start(default_intervals(), &DefaultTestSettings);
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingReference);

        // on_reference_finished → AwaitingSliderTouch
        session.on_reference_finished();
        assert_eq!(session.state(), PitchMatchingSessionState::AwaitingSliderTouch);

        // adjust_pitch (first touch) → PlayingTunable
        let freq = session.adjust_pitch(0.3);
        assert!(freq.is_some());
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingTunable);

        // adjust_pitch (continuous) → still PlayingTunable
        let freq2 = session.adjust_pitch(-0.2);
        assert!(freq2.is_some());
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingTunable);

        // commit_pitch → ShowingFeedback
        session.commit_pitch(-0.2, "2026-03-04T10:00:00Z".to_string());
        assert_eq!(session.state(), PitchMatchingSessionState::ShowingFeedback);
        assert!(session.show_feedback());

        // on_feedback_finished → PlayingReference (loop)
        session.on_feedback_finished();
        assert_eq!(session.state(), PitchMatchingSessionState::PlayingReference);
        assert!(!session.show_feedback());
    }

    // --- AC9: Stop ---

    #[test]
    fn test_stop_returns_to_idle_and_clears_state() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.0);

        session.stop();
        assert_eq!(session.state(), PitchMatchingSessionState::Idle);
        assert!(session.current_challenge().is_none());
        assert!(session.current_playback_data().is_none());
        assert!(session.last_completed().is_none());
        assert!(!session.show_feedback());
    }

    #[test]
    fn test_stop_from_idle_is_noop() {
        let mut session = create_session();
        session.stop(); // Should not panic
        assert_eq!(session.state(), PitchMatchingSessionState::Idle);
    }

    // --- AC10: Observer panic isolation ---

    #[test]
    fn test_observer_panic_isolation() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (normal_observer, normal_calls) = MockPitchMatchingObserver::new();
        let panicking_observer = PanickingPitchMatchingObserver;

        let mut session = PitchMatchingSession::new(
            profile,
            vec![
                Box::new(panicking_observer),
                Box::new(normal_observer),
            ],
            vec![],
        );

        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.0);
        session.commit_pitch(0.0, "2026-03-04T10:00:00Z".to_string());

        // Normal observer should still receive the event despite panicking observer
        assert_eq!(normal_calls.borrow().len(), 1);
        assert_eq!(session.state(), PitchMatchingSessionState::ShowingFeedback);
    }

    // --- AC14: Reset training data ---

    #[test]
    fn test_reset_training_data_stops_session_resets_matching_calls_resettables() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        profile.borrow_mut().update_matching(MIDINote::new(60), 5.0);
        assert!(profile.borrow().matching_mean().is_some());

        let (resettable, count) = MockResettable::new();
        let mut session = PitchMatchingSession::new(
            Rc::clone(&profile),
            vec![],
            vec![Box::new(resettable)],
        );

        session.start(default_intervals(), &DefaultTestSettings);
        session.reset_training_data();

        assert_eq!(session.state(), PitchMatchingSessionState::Idle);
        assert_eq!(profile.borrow().matching_mean(), None);
        assert_eq!(count.get(), 1);
    }

    // --- State guard tests ---

    #[test]
    #[should_panic(expected = "on_reference_finished() requires PlayingReference state")]
    fn test_on_reference_finished_from_idle_panics() {
        let mut session = create_session();
        session.on_reference_finished();
    }

    #[test]
    #[should_panic(expected = "commit_pitch() requires PlayingTunable state")]
    fn test_commit_pitch_from_idle_panics() {
        let mut session = create_session();
        session.commit_pitch(0.0, "2026-03-04T10:00:00Z".to_string());
    }

    #[test]
    #[should_panic(expected = "on_feedback_finished() requires ShowingFeedback state")]
    fn test_on_feedback_finished_from_idle_panics() {
        let mut session = create_session();
        session.on_feedback_finished();
    }

    #[test]
    #[should_panic(expected = "commit_pitch() requires PlayingTunable state")]
    fn test_commit_pitch_from_awaiting_slider_panics() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        // In AwaitingSliderTouch, commit without first adjust should panic
        session.commit_pitch(0.0, "2026-03-04T10:00:00Z".to_string());
    }

    #[test]
    #[should_panic(expected = "on_reference_finished() requires PlayingReference state")]
    fn test_on_reference_finished_from_showing_feedback_panics() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_finished();
        session.adjust_pitch(0.0);
        session.commit_pitch(0.0, "2026-03-04T10:00:00Z".to_string());
        // In ShowingFeedback — should panic
        session.on_reference_finished();
    }

    // --- Velocity constant ---

    #[test]
    fn test_pitch_matching_velocity_constant() {
        assert_eq!(PITCH_MATCHING_VELOCITY, 63);
    }
}
