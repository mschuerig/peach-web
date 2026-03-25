use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use rand::prelude::IndexedRandom;

use crate::ports::{
    ProfileUpdating, ProgressTimelineUpdating, Resettable, TrainingRecordPersisting, UserSettings,
};
use crate::profile::{COLD_START_DIFFICULTY, PerceptualProfile};
use crate::records::{PitchDiscriminationRecord, TrainingRecord};
use crate::statistics_key::StatisticsKey;
use crate::strategy::{MIN_CENT_DIFFERENCE, TrainingSettings, next_pitch_discrimination_trial};
use crate::training::{CompletedPitchDiscriminationTrial, PitchDiscriminationTrial};
use crate::training_discipline::TrainingDiscipline;
use crate::tuning::TuningSystem;
use crate::types::{
    AmplitudeDB, Cents, DetunedMIDINote, DirectedInterval, Frequency, MIDINote, NoteDuration,
    NoteRange,
};

/// Feedback display duration in seconds.
pub const FEEDBACK_DURATION_SECS: f64 = 0.4;

/// Scaling factor for amplitude variation (±10 dB at max vary_loudness).
pub const AMPLITUDE_VARY_SCALING: f64 = 10.0;

/// State of the pitch discrimination session state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchDiscriminationSessionState {
    Idle,
    PlayingReferenceNote,
    PlayingTargetNote,
    AwaitingAnswer,
    ShowingFeedback,
}

/// Data needed by the web layer to play the current trial's notes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PitchDiscriminationPlaybackData {
    pub reference_frequency: Frequency,
    pub target_frequency: Frequency,
    pub duration: NoteDuration,
    pub target_amplitude_db: AmplitudeDB,
}

/// Pure domain state machine for pitch discrimination training sessions.
///
/// Manages the training loop state, trial generation, answer processing,
/// and observer notification. No browser dependencies — the web crate drives
/// the async loop and audio playback.
pub struct PitchDiscriminationSession {
    state: PitchDiscriminationSessionState,
    profile: Rc<RefCell<PerceptualProfile>>,
    profile_port: Box<dyn ProfileUpdating>,
    record_port: Box<dyn TrainingRecordPersisting>,
    timeline_port: Box<dyn ProgressTimelineUpdating>,
    resettables: Vec<Box<dyn Resettable>>,

    // Session-level state (snapshot from settings at start)
    session_intervals: HashSet<DirectedInterval>,
    session_tuning_system: TuningSystem,
    session_reference_pitch: Frequency,
    session_note_duration: NoteDuration,
    session_vary_loudness: f64,
    session_note_range: NoteRange,

    // Current trial state
    current_trial: Option<PitchDiscriminationTrial>,
    current_playback_data: Option<PitchDiscriminationPlaybackData>,
    last_completed: Option<CompletedPitchDiscriminationTrial>,

    // Observable feedback state
    show_feedback: bool,
    is_last_answer_correct: bool,
    session_best_cent_difference: Option<f64>,
}

impl PitchDiscriminationSession {
    pub fn new(
        profile: Rc<RefCell<PerceptualProfile>>,
        profile_port: Box<dyn ProfileUpdating>,
        record_port: Box<dyn TrainingRecordPersisting>,
        timeline_port: Box<dyn ProgressTimelineUpdating>,
        resettables: Vec<Box<dyn Resettable>>,
    ) -> Self {
        Self {
            state: PitchDiscriminationSessionState::Idle,
            profile,
            profile_port,
            record_port,
            timeline_port,
            resettables,
            session_intervals: HashSet::new(),
            session_tuning_system: TuningSystem::EqualTemperament,
            session_reference_pitch: Frequency::CONCERT_440,
            session_note_duration: NoteDuration::new(1.0),
            session_vary_loudness: 0.0,
            session_note_range: NoteRange::new(MIDINote::new(36), MIDINote::new(84)),
            current_trial: None,
            current_playback_data: None,
            last_completed: None,
            show_feedback: false,
            is_last_answer_correct: false,
            session_best_cent_difference: None,
        }
    }

    // --- Observable state accessors ---

    pub fn state(&self) -> PitchDiscriminationSessionState {
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
        self.last_completed.as_ref().map(|c| {
            c.pitch_discrimination_trial()
                .target_note()
                .offset
                .magnitude()
        })
    }

    pub fn current_interval(&self) -> Option<DirectedInterval> {
        self.current_trial
            .as_ref()
            .and_then(|c| DirectedInterval::between(c.reference_note(), c.target_note().note).ok())
    }

    pub fn current_playback_data(&self) -> Option<PitchDiscriminationPlaybackData> {
        self.current_playback_data
    }

    // --- State transitions ---

    /// Start a new pitch discrimination training session.
    ///
    /// Snapshots settings, generates the first trial, transitions to PlayingReferenceNote.
    /// Panics if not idle or if intervals is empty.
    pub fn start(&mut self, intervals: HashSet<DirectedInterval>, settings: &dyn UserSettings) {
        assert_eq!(
            self.state,
            PitchDiscriminationSessionState::Idle,
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

        // Generate first trial
        self.generate_next_pitch_discrimination_trial();
        self.state = PitchDiscriminationSessionState::PlayingReferenceNote;
    }

    /// Called when reference note playback finishes. Transitions to PlayingTargetNote.
    pub fn on_reference_note_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchDiscriminationSessionState::PlayingReferenceNote,
            "on_reference_note_finished() requires PlayingReferenceNote state"
        );
        self.state = PitchDiscriminationSessionState::PlayingTargetNote;
    }

    /// Called when target note playback finishes. Transitions to AwaitingAnswer.
    pub fn on_target_note_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchDiscriminationSessionState::PlayingTargetNote,
            "on_target_note_finished() requires PlayingTargetNote state"
        );
        self.state = PitchDiscriminationSessionState::AwaitingAnswer;
    }

    /// Handle the user's answer (higher/lower).
    ///
    /// Valid from PlayingTargetNote (early answer) or AwaitingAnswer.
    /// Creates CompletedPitchDiscriminationTrial, notifies observers, transitions to ShowingFeedback.
    pub fn handle_answer(&mut self, is_higher: bool, timestamp: String) {
        assert!(
            self.state == PitchDiscriminationSessionState::PlayingTargetNote
                || self.state == PitchDiscriminationSessionState::AwaitingAnswer,
            "handle_answer() requires PlayingTargetNote or AwaitingAnswer state"
        );

        let trial = self
            .current_trial
            .expect("handle_answer() called without a current trial");

        let completed = CompletedPitchDiscriminationTrial::new(
            trial,
            is_higher,
            self.session_tuning_system,
            timestamp,
        );

        // Update session best cent difference (all attempts, not just correct)
        let cent_diff = trial.target_note().offset.magnitude();
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
            if trial.is_target_higher() {
                "higher"
            } else {
                "lower"
            },
        );

        // Map trial result to generic port calls (AC7: mapping logic lives in session)
        let ref_note = trial.reference_note().raw_value();
        let target_note = trial.target_note().note.raw_value();
        let interval = target_note.abs_diff(ref_note);
        let discipline = if interval == 0 {
            TrainingDiscipline::UnisonPitchDiscrimination
        } else {
            TrainingDiscipline::IntervalPitchDiscrimination
        };
        let key = StatisticsKey::Pitch(discipline);
        let metric = trial.target_note().offset.raw_value.abs();

        // Update profile (only for correct answers — add_point early-returns on is_correct=false)
        self.profile_port.update_profile(
            key,
            completed.timestamp(),
            metric,
            completed.is_correct(),
        );

        // Persist training record
        let record = PitchDiscriminationRecord::from_completed(&completed);
        self.record_port
            .save_record(TrainingRecord::PitchDiscrimination(record));

        // Update progress timeline
        self.timeline_port
            .add_metric(discipline, completed.timestamp(), metric);

        self.last_completed = Some(completed);
        self.state = PitchDiscriminationSessionState::ShowingFeedback;
    }

    /// Called when the feedback display period finishes.
    /// Generates next trial, transitions to PlayingReferenceNote.
    pub fn on_feedback_finished(&mut self) {
        assert_eq!(
            self.state,
            PitchDiscriminationSessionState::ShowingFeedback,
            "on_feedback_finished() requires ShowingFeedback state"
        );
        self.show_feedback = false;
        self.generate_next_pitch_discrimination_trial();
        self.state = PitchDiscriminationSessionState::PlayingReferenceNote;
    }

    /// Stop the session and return to Idle.
    pub fn stop(&mut self) {
        if self.state == PitchDiscriminationSessionState::Idle {
            return;
        }
        self.state = PitchDiscriminationSessionState::Idle;
        self.current_trial = None;
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
        self.profile.borrow_mut().reset_all();
        for resettable in &mut self.resettables {
            resettable.reset();
        }
    }

    // --- Private helpers ---

    fn generate_next_pitch_discrimination_trial(&mut self) {
        let interval = self.random_interval();
        let profile = self.profile.borrow();
        let training_settings = TrainingSettings::new(
            self.session_note_range,
            self.session_reference_pitch,
            Cents::new(MIN_CENT_DIFFERENCE),
            Cents::new(COLD_START_DIFFICULTY),
        );

        let trial = next_pitch_discrimination_trial(
            &profile,
            &training_settings,
            self.last_completed.as_ref(),
            interval,
        );
        drop(profile); // Release borrow before computing frequencies

        // Calculate playback data
        let ref_detuned = DetunedMIDINote::from(trial.reference_note());
        let reference_frequency = self
            .session_tuning_system
            .frequency(ref_detuned, self.session_reference_pitch);
        let target_frequency = self
            .session_tuning_system
            .frequency(trial.target_note(), self.session_reference_pitch);
        let target_amplitude_db = calculate_target_amplitude(self.session_vary_loudness);

        self.current_trial = Some(trial);
        self.current_playback_data = Some(PitchDiscriminationPlaybackData {
            reference_frequency,
            target_frequency,
            duration: self.session_note_duration,
            target_amplitude_db,
        });

        #[cfg(feature = "training-log")]
        {
            let interval_str =
                DirectedInterval::between(trial.reference_note(), trial.target_note().note)
                    .map_or("?".to_string(), |di| {
                        format!("{:?}{:?}", di.interval, di.direction)
                    });
            let tuning_str = match self.session_tuning_system {
                TuningSystem::EqualTemperament => "ET",
                TuningSystem::JustIntonation => "JI",
            };
            log::info!(
                "PitchDiscriminationTrial: ref={} {:.2}Hz @0.0dB, target {:.2}Hz @{:.1}dB, offset={:.1}, interval={}, tuning={}, higher={}",
                trial.reference_note().raw_value(),
                reference_frequency.raw_value(),
                target_frequency.raw_value(),
                target_amplitude_db.raw_value(),
                trial.target_note().offset.raw_value,
                interval_str,
                tuning_str,
                trial.is_target_higher(),
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
    use crate::types::{Direction, Interval, NoteRange, StepPosition, TempoBPM};
    use std::cell::Cell;
    use std::time::Duration;

    // --- Mock types ---

    use crate::records::TrainingRecord;

    type ProfileUpdateLog = Rc<RefCell<Vec<(StatisticsKey, String, f64, bool)>>>;
    type TimelineMetricLog = Rc<RefCell<Vec<(TrainingDiscipline, String, f64)>>>;

    struct MockProfilePort {
        updates: ProfileUpdateLog,
    }

    impl MockProfilePort {
        fn new() -> (Self, ProfileUpdateLog) {
            let updates = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    updates: Rc::clone(&updates),
                },
                updates,
            )
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
            self.updates
                .borrow_mut()
                .push((key, timestamp.to_string(), value, is_correct));
        }
    }

    struct MockRecordPort {
        records: Rc<RefCell<Vec<TrainingRecord>>>,
    }

    impl MockRecordPort {
        fn new() -> (Self, Rc<RefCell<Vec<TrainingRecord>>>) {
            let records = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    records: Rc::clone(&records),
                },
                records,
            )
        }
    }

    impl TrainingRecordPersisting for MockRecordPort {
        fn save_record(&self, record: TrainingRecord) {
            self.records.borrow_mut().push(record);
        }
    }

    struct MockTimelinePort {
        metrics: TimelineMetricLog,
    }

    impl MockTimelinePort {
        fn new() -> (Self, TimelineMetricLog) {
            let metrics = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    metrics: Rc::clone(&metrics),
                },
                metrics,
            )
        }
    }

    impl ProgressTimelineUpdating for MockTimelinePort {
        fn add_metric(&mut self, discipline: TrainingDiscipline, timestamp: &str, value: f64) {
            self.metrics
                .borrow_mut()
                .push((discipline, timestamp.to_string(), value));
        }
    }

    struct NoOpProfilePort;
    impl ProfileUpdating for NoOpProfilePort {
        fn update_profile(
            &mut self,
            _key: StatisticsKey,
            _timestamp: &str,
            _value: f64,
            _is_correct: bool,
        ) {
        }
    }

    struct NoOpRecordPort;
    impl TrainingRecordPersisting for NoOpRecordPort {
        fn save_record(&self, _record: TrainingRecord) {}
    }

    struct NoOpTimelinePort;
    impl ProgressTimelineUpdating for NoOpTimelinePort {
        fn add_metric(&mut self, _discipline: TrainingDiscipline, _timestamp: &str, _value: f64) {}
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
        fn tempo_bpm(&self) -> TempoBPM {
            TempoBPM::default()
        }
        fn enabled_gap_positions(&self) -> HashSet<StepPosition> {
            HashSet::from([StepPosition::Fourth])
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
        fn tempo_bpm(&self) -> TempoBPM {
            TempoBPM::default()
        }
        fn enabled_gap_positions(&self) -> HashSet<StepPosition> {
            HashSet::from([StepPosition::Fourth])
        }
    }

    fn default_intervals() -> HashSet<DirectedInterval> {
        let mut set = HashSet::new();
        set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
        set
    }

    fn create_session() -> PitchDiscriminationSession {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        PitchDiscriminationSession::new(
            profile,
            Box::new(NoOpProfilePort),
            Box::new(NoOpRecordPort),
            Box::new(NoOpTimelinePort),
            vec![],
        )
    }

    struct MockPorts {
        profile_updates: ProfileUpdateLog,
        records: Rc<RefCell<Vec<TrainingRecord>>>,
        timeline_metrics: TimelineMetricLog,
    }

    fn create_session_with_ports() -> (PitchDiscriminationSession, MockPorts) {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (profile_port, profile_updates) = MockProfilePort::new();
        let (record_port, records) = MockRecordPort::new();
        let (timeline_port, timeline_metrics) = MockTimelinePort::new();
        let session = PitchDiscriminationSession::new(
            profile,
            Box::new(profile_port),
            Box::new(record_port),
            Box::new(timeline_port),
            vec![],
        );
        (
            session,
            MockPorts {
                profile_updates,
                records,
                timeline_metrics,
            },
        )
    }

    // --- AC1: Idle state tests ---

    #[test]
    fn test_idle_state_defaults() {
        let session = create_session();
        assert_eq!(session.state(), PitchDiscriminationSessionState::Idle);
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
            PitchDiscriminationSessionState::PlayingReferenceNote
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
            PitchDiscriminationSessionState::PlayingTargetNote
        );
    }

    // --- AC5: TargetNote to AwaitingAnswer ---

    #[test]
    fn test_on_target_note_finished_transitions_to_awaiting_answer() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::AwaitingAnswer
        );
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
            PitchDiscriminationSessionState::ShowingFeedback
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
            PitchDiscriminationSessionState::ShowingFeedback
        );
    }

    // --- AC7: Feedback state ---

    #[test]
    fn test_on_feedback_finished_generates_next_pitch_discrimination_trial() {
        let mut session = create_session();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        session.on_feedback_finished();
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::PlayingReferenceNote
        );
        assert!(!session.show_feedback());
        assert!(session.current_playback_data().is_some());
    }

    // --- Full lifecycle test ---

    #[test]
    fn test_full_lifecycle() {
        let mut session = create_session();

        // Idle
        assert_eq!(session.state(), PitchDiscriminationSessionState::Idle);

        // Start → PlayingReferenceNote
        session.start(default_intervals(), &DefaultTestSettings);
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::PlayingReferenceNote
        );

        // on_reference_note_finished → PlayingTargetNote
        session.on_reference_note_finished();
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::PlayingTargetNote
        );

        // on_target_note_finished → AwaitingAnswer
        session.on_target_note_finished();
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::AwaitingAnswer
        );

        // handle_answer → ShowingFeedback
        session.handle_answer(true, "2026-03-03T14:00:00Z".to_string());
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::ShowingFeedback
        );
        assert!(session.show_feedback());

        // on_feedback_finished → PlayingReferenceNote (next cycle)
        session.on_feedback_finished();
        assert_eq!(
            session.state(),
            PitchDiscriminationSessionState::PlayingReferenceNote
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

    // --- Port notification tests ---

    #[test]
    fn test_handle_answer_calls_all_ports() {
        let (mut session, ports) = create_session_with_ports();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        session.handle_answer(is_higher, "2026-03-03T14:30:00Z".to_string());

        // Profile port called with is_correct=true
        assert_eq!(ports.profile_updates.borrow().len(), 1);
        let (key, ts, _, is_correct) = &ports.profile_updates.borrow()[0];
        assert!(matches!(key, StatisticsKey::Pitch(_)));
        assert_eq!(ts, "2026-03-03T14:30:00Z");
        assert!(*is_correct);

        // Record port called
        assert_eq!(ports.records.borrow().len(), 1);
        assert!(matches!(
            &ports.records.borrow()[0],
            TrainingRecord::PitchDiscrimination(_)
        ));

        // Timeline port called
        assert_eq!(ports.timeline_metrics.borrow().len(), 1);
    }

    #[test]
    fn test_incorrect_answer_skips_profile_port() {
        let (mut session, ports) = create_session_with_ports();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        // Answer incorrectly
        session.handle_answer(!is_higher, "2026-03-03T14:00:00Z".to_string());

        // Profile called with is_correct=false for incorrect answers
        assert_eq!(ports.profile_updates.borrow().len(), 1);
        let (_, _, _, is_correct) = &ports.profile_updates.borrow()[0];
        assert!(!*is_correct);
        // Record and timeline still called
        assert_eq!(ports.records.borrow().len(), 1);
        assert_eq!(ports.timeline_metrics.borrow().len(), 1);
    }

    #[test]
    fn test_completed_comparison_has_correct_state() {
        let (mut session, _ports) = create_session_with_ports();
        session.start(default_intervals(), &DefaultTestSettings);
        session.on_reference_note_finished();
        session.on_target_note_finished();

        let data = session.current_playback_data().unwrap();
        let is_higher = data.target_frequency.raw_value() > data.reference_frequency.raw_value();
        session.handle_answer(is_higher, "2026-03-03T14:00:00Z".to_string());

        assert!(session.is_last_answer_correct());
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
        let mut session = PitchDiscriminationSession::new(
            profile,
            Box::new(NoOpProfilePort),
            Box::new(NoOpRecordPort),
            Box::new(NoOpTimelinePort),
            vec![],
        );
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
        assert_eq!(session.state(), PitchDiscriminationSessionState::Idle);
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
        assert_eq!(session.state(), PitchDiscriminationSessionState::Idle);
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
        assert_eq!(session.state(), PitchDiscriminationSessionState::Idle);
    }

    #[test]
    fn test_reset_training_data_resets_profile() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        profile.borrow_mut().add_point(
            crate::StatisticsKey::Pitch(crate::TrainingDiscipline::UnisonPitchDiscrimination),
            crate::MetricPoint::new(1000.0, 50.0),
            true,
        );
        assert!(
            profile
                .borrow()
                .has_data(crate::TrainingDiscipline::UnisonPitchDiscrimination)
        );

        let mut session = PitchDiscriminationSession::new(
            Rc::clone(&profile),
            Box::new(NoOpProfilePort),
            Box::new(NoOpRecordPort),
            Box::new(NoOpTimelinePort),
            vec![],
        );
        session.reset_training_data();
        assert!(
            !profile
                .borrow()
                .has_data(crate::TrainingDiscipline::UnisonPitchDiscrimination)
        );
    }

    #[test]
    fn test_reset_training_data_calls_resettables() {
        let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
        let (resettable, count) = MockResettable::new();
        let mut session = PitchDiscriminationSession::new(
            profile,
            Box::new(NoOpProfilePort),
            Box::new(NoOpRecordPort),
            Box::new(NoOpTimelinePort),
            vec![Box::new(resettable)],
        );
        session.reset_training_data();
        assert_eq!(count.get(), 1);
    }

    // --- Feedback duration constant ---

    #[test]
    fn test_feedback_duration_constant() {
        assert!((FEEDBACK_DURATION_SECS - 0.4).abs() < f64::EPSILON);
    }
}
