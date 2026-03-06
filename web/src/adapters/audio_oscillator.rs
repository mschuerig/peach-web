use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{AudioError, NotePlayer, PlaybackHandle};
use domain::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteDuration};
use web_sys::{AudioContext, OscillatorType};

use super::audio_context::AudioContextManager;

/// Shared inner state for a playing oscillator note.
struct OscillatorHandleInner {
    oscillator: web_sys::OscillatorNode,
    gain: web_sys::GainNode,
    stopped: bool,
}

/// A handle to a playing oscillator note. Can be cloned (shares inner state via Rc).
#[derive(Clone)]
pub struct OscillatorPlaybackHandle {
    inner: Rc<RefCell<OscillatorHandleInner>>,
}

impl OscillatorPlaybackHandle {
    fn new(oscillator: web_sys::OscillatorNode, gain: web_sys::GainNode) -> Self {
        Self {
            inner: Rc::new(RefCell::new(OscillatorHandleInner {
                oscillator,
                gain,
                stopped: false,
            })),
        }
    }

    fn is_stopped(&self) -> bool {
        self.inner.borrow().stopped
    }
}

impl PlaybackHandle for OscillatorPlaybackHandle {
    fn stop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        if !inner.stopped {
            if let Err(e) = inner.oscillator.stop() {
                log::warn!("Failed to stop oscillator: {e:?}");
            }
            if let Err(e) = inner.gain.disconnect() {
                log::warn!("Failed to disconnect gain node: {e:?}");
            }
            inner.stopped = true;
        }
    }

    fn adjust_frequency(&mut self, frequency: Frequency) -> Result<(), AudioError> {
        let inner = self.inner.borrow();
        if inner.stopped {
            return Ok(());
        }
        inner
            .oscillator
            .frequency()
            .set_value(frequency.raw_value() as f32);
        Ok(())
    }
}

/// Plays notes using Web Audio OscillatorNode + GainNode.
pub struct OscillatorNotePlayer {
    context_manager: Rc<RefCell<AudioContextManager>>,
    active_handles: RefCell<Vec<OscillatorPlaybackHandle>>,
}

impl OscillatorNotePlayer {
    pub fn new(context_manager: Rc<RefCell<AudioContextManager>>) -> Self {
        Self {
            context_manager,
            active_handles: RefCell::new(Vec::new()),
        }
    }

    fn get_context(&self) -> Result<Rc<RefCell<AudioContext>>, AudioError> {
        self.context_manager.borrow_mut().get_or_create()
    }

    fn prune_stopped(&self) {
        self.active_handles.borrow_mut().retain(|h| !h.is_stopped());
    }

    fn create_and_start(
        &self,
        ctx_rc: &Rc<RefCell<AudioContext>>,
        frequency: Frequency,
        _velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<OscillatorPlaybackHandle, AudioError> {
        self.prune_stopped();

        let ctx = ctx_rc.borrow();

        let osc = ctx
            .create_oscillator()
            .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;
        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(frequency.raw_value() as f32);

        let gain_node = ctx
            .create_gain()
            .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;
        let gain_linear = 10_f32.powf(amplitude_db.raw_value() / 20.0);
        gain_node.gain().set_value(gain_linear);

        osc.connect_with_audio_node(&gain_node)
            .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;
        gain_node
            .connect_with_audio_node(&ctx.destination())
            .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;

        osc.start()
            .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;

        let handle = OscillatorPlaybackHandle::new(osc, gain_node);
        self.active_handles.borrow_mut().push(handle.clone());
        Ok(handle)
    }
}

impl NotePlayer for OscillatorNotePlayer {
    type Handle = OscillatorPlaybackHandle;

    fn play(
        &self,
        frequency: Frequency,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<Self::Handle, AudioError> {
        let ctx_rc = self.get_context()?;
        log::debug!(
            "[DIAG] OscillatorNotePlayer::play — ctx.state: {:?}, freq: {}",
            ctx_rc.borrow().state(),
            frequency.raw_value()
        );
        self.create_and_start(&ctx_rc, frequency, velocity, amplitude_db)
    }

    fn play_for_duration(
        &self,
        frequency: Frequency,
        duration: NoteDuration,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<(), AudioError> {
        let ctx_rc = self.get_context()?;
        log::debug!(
            "[DIAG] OscillatorNotePlayer::play_for_duration — ctx.state: {:?}, freq: {}, dur: {}s",
            ctx_rc.borrow().state(),
            frequency.raw_value(),
            duration.raw_value()
        );
        let handle = self.create_and_start(&ctx_rc, frequency, velocity, amplitude_db)?;

        // Schedule automatic stop using Web Audio's high-precision clock.
        let ctx = ctx_rc.borrow();
        let stop_time = ctx.current_time() + duration.raw_value();
        handle
            .inner
            .borrow()
            .oscillator
            .stop_with_when(stop_time)
            .map_err(|e| AudioError::PlaybackFailed(format!("{:?}", e)))?;

        // Keep handle in active_handles so stop_all() can stop it early
        // (e.g., when the user answers during target note playback).
        // Handles are pruned on next create_and_start, or cleared by stop_all().

        Ok(())
    }

    fn stop_all(&self) {
        let mut handles = self.active_handles.borrow_mut();
        for handle in handles.iter_mut() {
            handle.stop();
        }
        handles.clear();
    }
}
