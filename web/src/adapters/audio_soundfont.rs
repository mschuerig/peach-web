use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{AudioError, PlaybackHandle};
use domain::types::Frequency;
use wasm_bindgen::prelude::*;
use web_sys::{AudioWorkletNode, MessagePort};

/// A SoundFont preset descriptor (bank + program + display name).
#[derive(Clone, Debug)]
pub struct SF2Preset {
    pub name: String,
    pub bank: u16,
    pub program: u16,
}

/// Bridge to the AudioWorklet running OxiSynth WASM.
/// All communication happens via `postMessage` on the worklet's `MessagePort`.
pub struct WorkletBridge {
    _node: AudioWorkletNode,
    port: MessagePort,
}

impl WorkletBridge {
    pub fn new(node: AudioWorkletNode) -> Self {
        let port = node.port().expect("AudioWorkletNode must have a port");
        Self { _node: node, port }
    }

    pub fn send_note_on(&self, key: u8, vel: u8) -> Result<(), AudioError> {
        let msg = js_sys::Object::new();
        js_sys::Reflect::set(&msg, &"type".into(), &"noteOn".into())
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        js_sys::Reflect::set(&msg, &"key".into(), &JsValue::from(key))
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        js_sys::Reflect::set(&msg, &"vel".into(), &JsValue::from(vel))
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        self.port
            .post_message(&msg)
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))
    }

    pub fn send_note_off(&self, key: u8) -> Result<(), AudioError> {
        let msg = js_sys::Object::new();
        js_sys::Reflect::set(&msg, &"type".into(), &"noteOff".into())
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        js_sys::Reflect::set(&msg, &"key".into(), &JsValue::from(key))
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        self.port
            .post_message(&msg)
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))
    }

    pub fn send_pitch_bend(&self, value: u16) -> Result<(), AudioError> {
        let msg = js_sys::Object::new();
        js_sys::Reflect::set(&msg, &"type".into(), &"pitchBend".into())
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        js_sys::Reflect::set(&msg, &"value".into(), &JsValue::from(value))
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        self.port
            .post_message(&msg)
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))
    }

    pub fn send_all_notes_off(&self) -> Result<(), AudioError> {
        let msg = js_sys::Object::new();
        js_sys::Reflect::set(&msg, &"type".into(), &"allNotesOff".into())
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        self.port
            .post_message(&msg)
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))
    }

    pub fn send_select_program(&self, bank: u32, preset: u8) -> Result<(), AudioError> {
        let msg = js_sys::Object::new();
        js_sys::Reflect::set(&msg, &"type".into(), &"selectProgram".into())
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        js_sys::Reflect::set(&msg, &"bank".into(), &JsValue::from(bank))
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        js_sys::Reflect::set(&msg, &"preset".into(), &JsValue::from(preset))
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))?;
        self.port
            .post_message(&msg)
            .map_err(|e| AudioError::PlaybackFailed(format!("{e:?}")))
    }
}

// --- Frequency-to-MIDI conversion helpers ---

fn frequency_to_midi(freq: f64) -> u8 {
    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    midi.round().clamp(0.0, 127.0) as u8
}

fn midi_note_frequency(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

fn frequency_to_cents_from_midi(freq: f64, midi_note: u8) -> f64 {
    let midi_freq = midi_note_frequency(midi_note);
    1200.0 * (freq / midi_freq).log2()
}

/// Convert a cent offset to MIDI pitch bend value.
/// MIDI pitch bend: 0-16383, center = 8192.
/// Default OxiSynth pitch bend range: ±200 cents (2 semitones).
fn cents_to_pitch_bend(cents: f64) -> u16 {
    let bend = 8192.0 + (cents / 200.0) * 8192.0;
    bend.clamp(0.0, 16383.0) as u16
}

// --- SoundFontPlaybackHandle ---

/// Handle for a playing SoundFont note. Holds the MIDI key and bridge reference.
#[derive(Clone)]
pub struct SoundFontPlaybackHandle {
    bridge: Rc<RefCell<WorkletBridge>>,
    key: u8,
    stopped: bool,
}

impl SoundFontPlaybackHandle {
    fn new(bridge: Rc<RefCell<WorkletBridge>>, key: u8) -> Self {
        Self {
            bridge,
            key,
            stopped: false,
        }
    }
}

impl PlaybackHandle for SoundFontPlaybackHandle {
    fn stop(&mut self) {
        if !self.stopped {
            if let Err(e) = self.bridge.borrow().send_note_off(self.key) {
                log::warn!("Failed to send noteOff: {e}");
            }
            self.stopped = true;
        }
    }

    fn adjust_frequency(&mut self, frequency: Frequency) -> Result<(), AudioError> {
        if self.stopped {
            return Ok(());
        }
        let cents = frequency_to_cents_from_midi(frequency.raw_value(), self.key);
        let bend = cents_to_pitch_bend(cents);
        self.bridge.borrow().send_pitch_bend(bend)
    }
}

// --- SoundFontNotePlayer ---

use domain::ports::NotePlayer;
use domain::types::{AmplitudeDB, MIDIVelocity, NoteDuration};

/// Plays notes via OxiSynth running in an AudioWorklet.
pub struct SoundFontNotePlayer {
    bridge: Rc<RefCell<WorkletBridge>>,
    active_handles: RefCell<Vec<SoundFontPlaybackHandle>>,
}

impl SoundFontNotePlayer {
    pub fn new(bridge: Rc<RefCell<WorkletBridge>>) -> Self {
        Self {
            bridge,
            active_handles: RefCell::new(Vec::new()),
        }
    }

    fn prune_stopped(&self) {
        self.active_handles.borrow_mut().retain(|h| !h.stopped);
    }
}

impl NotePlayer for SoundFontNotePlayer {
    type Handle = SoundFontPlaybackHandle;

    fn play(
        &self,
        frequency: Frequency,
        velocity: MIDIVelocity,
        _amplitude_db: AmplitudeDB,
    ) -> Result<Self::Handle, AudioError> {
        self.prune_stopped();

        let key = frequency_to_midi(frequency.raw_value());
        let vel = velocity.raw_value();

        // Send NoteOn
        self.bridge.borrow().send_note_on(key, vel)?;

        // Set initial pitch bend for fractional cent offset
        let cents = frequency_to_cents_from_midi(frequency.raw_value(), key);
        if cents.abs() > 0.5 {
            let bend = cents_to_pitch_bend(cents);
            self.bridge.borrow().send_pitch_bend(bend)?;
        }

        let handle = SoundFontPlaybackHandle::new(Rc::clone(&self.bridge), key);
        self.active_handles.borrow_mut().push(handle.clone());
        Ok(handle)
    }

    fn play_for_duration(
        &self,
        frequency: Frequency,
        duration: NoteDuration,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<(), AudioError> {
        let mut handle = self.play(frequency, velocity, amplitude_db)?;

        // Schedule noteOff after duration
        let duration_ms = (duration.raw_value() * 1000.0) as u32;
        gloo_timers::callback::Timeout::new(duration_ms, move || {
            handle.stop();
        })
        .forget();

        Ok(())
    }

    fn stop_all(&self) {
        if let Err(e) = self.bridge.borrow().send_all_notes_off() {
            log::warn!("Failed to send allNotesOff: {e}");
        }
        let mut handles = self.active_handles.borrow_mut();
        for handle in handles.iter_mut() {
            handle.stopped = true;
        }
        handles.clear();
    }
}
