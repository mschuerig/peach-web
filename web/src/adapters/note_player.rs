use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{AudioError, NotePlayer, PlaybackHandle};
use domain::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteDuration};

use super::audio_context::AudioContextManager;
use super::audio_oscillator::{OscillatorNotePlayer, OscillatorPlaybackHandle};
use super::audio_soundfont::{SoundFontNotePlayer, SoundFontPlaybackHandle, WorkletBridge};

/// Unified playback handle wrapping oscillator or SoundFont variants.
#[derive(Clone)]
pub enum UnifiedPlaybackHandle {
    Oscillator(OscillatorPlaybackHandle),
    SoundFont(SoundFontPlaybackHandle),
}

impl PlaybackHandle for UnifiedPlaybackHandle {
    fn stop(&mut self) {
        match self {
            Self::Oscillator(h) => h.stop(),
            Self::SoundFont(h) => h.stop(),
        }
    }

    fn adjust_frequency(&mut self, frequency: Frequency) -> Result<(), AudioError> {
        match self {
            Self::Oscillator(h) => h.adjust_frequency(frequency),
            Self::SoundFont(h) => h.adjust_frequency(frequency),
        }
    }
}

/// Unified note player wrapping oscillator or SoundFont variants.
pub enum UnifiedNotePlayer {
    Oscillator(OscillatorNotePlayer),
    SoundFont(SoundFontNotePlayer),
}

impl NotePlayer for UnifiedNotePlayer {
    type Handle = UnifiedPlaybackHandle;

    fn play(
        &self,
        frequency: Frequency,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<Self::Handle, AudioError> {
        match self {
            Self::Oscillator(p) => p
                .play(frequency, velocity, amplitude_db)
                .map(UnifiedPlaybackHandle::Oscillator),
            Self::SoundFont(p) => p
                .play(frequency, velocity, amplitude_db)
                .map(UnifiedPlaybackHandle::SoundFont),
        }
    }

    fn play_for_duration(
        &self,
        frequency: Frequency,
        duration: NoteDuration,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<(), AudioError> {
        match self {
            Self::Oscillator(p) => p.play_for_duration(frequency, duration, velocity, amplitude_db),
            Self::SoundFont(p) => p.play_for_duration(frequency, duration, velocity, amplitude_db),
        }
    }

    fn stop_all(&self) {
        match self {
            Self::Oscillator(p) => p.stop_all(),
            Self::SoundFont(p) => p.stop_all(),
        }
    }
}

/// Factory function to create the appropriate NotePlayer based on sound source setting
/// and worklet bridge availability.
pub fn create_note_player(
    sound_source: &str,
    audio_ctx: Rc<RefCell<AudioContextManager>>,
    worklet_bridge: Option<Rc<RefCell<WorkletBridge>>>,
    sf_gain_node: Option<Rc<web_sys::GainNode>>,
) -> UnifiedNotePlayer {
    match (sound_source, worklet_bridge, sf_gain_node) {
        (s, Some(bridge), Some(gain)) if s.starts_with("sf2:") => {
            // Parse bank:preset from "sf2:<bank>:<preset>"
            let parts: Vec<&str> = s.splitn(3, ':').collect();
            if let (Some(bank_str), Some(preset_str)) = (parts.get(1), parts.get(2))
                && let (Ok(bank), Ok(preset)) = (bank_str.parse::<u32>(), preset_str.parse::<u8>())
                && let Err(e) = bridge.borrow().send_select_program(bank, preset)
            {
                log::warn!("Failed to select SoundFont program: {e}");
            }
            UnifiedNotePlayer::SoundFont(SoundFontNotePlayer::new(bridge, gain))
        }
        _ => UnifiedNotePlayer::Oscillator(OscillatorNotePlayer::new(audio_ctx)),
    }
}
