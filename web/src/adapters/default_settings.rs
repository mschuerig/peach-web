use std::collections::HashSet;
use std::time::Duration;

use domain::TuningSystem;
use domain::ports::UserSettings;
use domain::types::{Frequency, MIDINote, NoteDuration, NoteRange, StepPosition, TempoBPM};

#[allow(dead_code)] // Planned fallback for Settings view (Epic 2, story 2.1)
pub struct DefaultSettings;

impl UserSettings for DefaultSettings {
    fn note_range(&self) -> NoteRange {
        NoteRange::new(MIDINote::new(36), MIDINote::new(84)) // C2-C6
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
