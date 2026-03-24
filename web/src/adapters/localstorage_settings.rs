use std::collections::HashSet;
use std::time::Duration;

use domain::TuningSystem;
use domain::ports::UserSettings;
use domain::types::{
    DirectedInterval, Direction, Frequency, Interval, MIDINote, NoteDuration, NoteRange,
    StepPosition, TempoBPM,
};

pub struct LocalStorageSettings;

impl LocalStorageSettings {
    /// Read a string value from localStorage. Used by Settings UI for keys
    /// not covered by the `UserSettings` trait (e.g. `peach.sound_source`).
    pub fn get_string(key: &str) -> Option<String> {
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

    /// Write a value to localStorage. Used by Settings UI.
    pub fn set(key: &str, value: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            && let Err(e) = storage.set_item(key, value)
        {
            log::error!("Failed to write localStorage key '{key}': {e:?}");
        }
    }

    /// Persist selected intervals to localStorage.
    pub fn set_selected_intervals(intervals: &HashSet<DirectedInterval>) {
        let mut sorted: Vec<DirectedInterval> = intervals.iter().copied().collect();
        sorted.sort_by_key(|d| (d.interval, d.direction));
        match serde_json::to_string(&sorted) {
            Ok(json) => Self::set("peach.intervals", &json),
            Err(e) => log::error!("Failed to serialize intervals: {e}"),
        }
    }

    fn get_u16(key: &str, default: u16) -> u16 {
        Self::get_string(key)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(default)
    }

    /// Persist tempo BPM to localStorage.
    pub fn set_tempo_bpm(tempo: TempoBPM) {
        Self::set("peach.tempo_bpm", &tempo.bpm().to_string());
    }

    /// Persist enabled gap positions to localStorage as comma-separated indices.
    pub fn set_enabled_gap_positions(positions: &HashSet<StepPosition>) {
        let mut indices: Vec<u8> = positions
            .iter()
            .map(|p| match p {
                StepPosition::First => 0,
                StepPosition::Second => 1,
                StepPosition::Third => 2,
                StepPosition::Fourth => 3,
            })
            .collect();
        indices.sort();
        let csv = indices
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Self::set("peach.gap_positions", &csv);
    }

    /// Read enabled gap positions from localStorage. Returns {Fourth} if absent or invalid.
    pub fn get_enabled_gap_positions_static() -> HashSet<StepPosition> {
        Self::get_string("peach.gap_positions")
            .map(|csv| {
                csv.split(',')
                    .filter_map(|s| match s.trim() {
                        "0" => Some(StepPosition::First),
                        "1" => Some(StepPosition::Second),
                        "2" => Some(StepPosition::Third),
                        "3" => Some(StepPosition::Fourth),
                        _ => None,
                    })
                    .collect::<HashSet<_>>()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| HashSet::from([StepPosition::Fourth]))
    }

    /// Read selected intervals from localStorage. Returns default {Prime/Up} if absent or invalid.
    pub fn get_selected_intervals() -> HashSet<DirectedInterval> {
        Self::get_string("peach.intervals")
            .and_then(|json| serde_json::from_str::<Vec<DirectedInterval>>(&json).ok())
            .map(|v| v.into_iter().collect())
            .filter(|s: &HashSet<DirectedInterval>| !s.is_empty())
            .unwrap_or_else(|| {
                let mut set = HashSet::new();
                set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
                set
            })
    }
}

impl UserSettings for LocalStorageSettings {
    fn note_range(&self) -> NoteRange {
        let min = MIDINote::try_new(Self::get_u8("peach.note_range_min", 36))
            .unwrap_or(MIDINote::new(36));
        let max = MIDINote::try_new(Self::get_u8("peach.note_range_max", 84))
            .unwrap_or(MIDINote::new(84));
        NoteRange::try_new(min, max).unwrap_or(NoteRange::new(MIDINote::new(36), MIDINote::new(84)))
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

    fn note_gap(&self) -> Duration {
        Duration::from_secs_f64(Self::get_f64("peach.note_gap", 0.0).clamp(0.0, 5.0))
    }

    fn tempo_bpm(&self) -> TempoBPM {
        TempoBPM::try_new(Self::get_u16("peach.tempo_bpm", 80)).unwrap_or_default()
    }

    fn enabled_gap_positions(&self) -> HashSet<StepPosition> {
        Self::get_enabled_gap_positions_static()
    }
}
