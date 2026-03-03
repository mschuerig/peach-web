use domain::ports::UserSettings;
use domain::types::{Frequency, MIDINote, NoteRange, NoteDuration};
use domain::TuningSystem;

pub struct LocalStorageSettings;

impl LocalStorageSettings {
    fn get_string(key: &str) -> Option<String> {
        web_sys::window()?
            .local_storage()
            .ok()??
            .get_item(key)
            .ok()?
    }

    fn get_f64(key: &str, default: f64) -> f64 {
        Self::get_string(key)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(default)
    }

    fn get_u8(key: &str, default: u8) -> u8 {
        Self::get_string(key)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(default)
    }

    /// Write a value to localStorage. Used by Settings UI (story 2.1).
    #[allow(dead_code)] // Planned for Settings view (Epic 2, story 2.1)
    pub fn set(key: &str, value: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            && let Err(e) = storage.set_item(key, value)
        {
            log::error!("Failed to write localStorage key '{key}': {e:?}");
        }
    }
}

impl UserSettings for LocalStorageSettings {
    fn note_range(&self) -> NoteRange {
        let min = MIDINote::try_new(Self::get_u8("peach.note_range_min", 36))
            .unwrap_or(MIDINote::new(36));
        let max = MIDINote::try_new(Self::get_u8("peach.note_range_max", 84))
            .unwrap_or(MIDINote::new(84));
        NoteRange::try_new(min, max)
            .unwrap_or(NoteRange::new(MIDINote::new(36), MIDINote::new(84)))
    }

    fn note_duration(&self) -> NoteDuration {
        NoteDuration::new(Self::get_f64("peach.note_duration", 1.0))
    }

    fn reference_pitch(&self) -> Frequency {
        Frequency::try_new(Self::get_f64("peach.reference_pitch", 440.0))
            .unwrap_or(Frequency::CONCERT_440)
    }

    fn tuning_system(&self) -> TuningSystem {
        match Self::get_string("peach.tuning_system").as_deref() {
            Some("justIntonation") => TuningSystem::JustIntonation,
            _ => TuningSystem::EqualTemperament,
        }
    }

    fn vary_loudness(&self) -> f64 {
        Self::get_f64("peach.vary_loudness", 0.0)
    }
}
