use domain::ports::UserSettings;
use domain::types::{Frequency, MIDINote, NoteDuration};
use domain::TuningSystem;

pub struct DefaultSettings;

impl UserSettings for DefaultSettings {
    fn note_range_min(&self) -> MIDINote {
        MIDINote::new(36) // C2
    }

    fn note_range_max(&self) -> MIDINote {
        MIDINote::new(84) // C6
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
