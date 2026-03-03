use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::{AudioError, NotePlayer, PlaybackHandle};
use domain::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteDuration};
use web_sys::{AudioContext, OscillatorType};

use super::audio_context::AudioContextManager;

/// Shared inner state for a playing oscillator note.
struct OscillatorHandleInner {
    oscillator: web_sys::OscillatorNode,
    stopped: bool,
}

/// A handle to a playing oscillator note. Can be cloned (shares inner state).
pub struct OscillatorPlaybackHandle {
    inner: Rc<RefCell<OscillatorHandleInner>>,
}

impl OscillatorPlaybackHandle {
    fn new(oscillator: web_sys::OscillatorNode) -> Self {
        Self {
            inner: Rc::new(RefCell::new(OscillatorHandleInner {
                oscillator,
                stopped: false,
            })),
        }
    }

    fn clone_shared(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
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
            let _ = inner.oscillator.stop();
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
        self.active_handles
            .borrow_mut()
            .retain(|h| !h.is_stopped());
    }

    fn create_and_start(
        &self,
        frequency: Frequency,
        _velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<OscillatorPlaybackHandle, AudioError> {
        self.prune_stopped();

        let ctx_rc = self.get_context()?;
        let ctx = ctx_rc.borrow();

        let osc = ctx.create_oscillator().map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;
        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(frequency.raw_value() as f32);

        let gain_node = ctx.create_gain().map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;
        let gain_linear = 10_f32.powf(amplitude_db.raw_value() / 20.0);
        gain_node.gain().set_value(gain_linear);

        osc.connect_with_audio_node(&gain_node).map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;
        gain_node
            .connect_with_audio_node(&ctx.destination())
            .map_err(|e| {
                AudioError::EngineStartFailed(format!("{:?}", e))
            })?;

        osc.start().map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;

        let handle = OscillatorPlaybackHandle::new(osc);
        self.active_handles
            .borrow_mut()
            .push(handle.clone_shared());
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
        self.create_and_start(frequency, velocity, amplitude_db)
    }

    fn play_for_duration(
        &self,
        frequency: Frequency,
        duration: NoteDuration,
        velocity: MIDIVelocity,
        amplitude_db: AmplitudeDB,
    ) -> Result<(), AudioError> {
        let handle = self.create_and_start(frequency, velocity, amplitude_db)?;

        // Schedule automatic stop using Web Audio's high-precision clock.
        let ctx_rc = self.get_context()?;
        let ctx = ctx_rc.borrow();
        let stop_time = ctx.current_time() + duration.raw_value();
        let inner = handle.inner.borrow();
        inner.oscillator.stop_with_when(stop_time).map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;

        Ok(())
    }

    fn stop_all(&mut self) {
        let mut handles = self.active_handles.borrow_mut();
        for handle in handles.iter_mut() {
            handle.stop();
        }
        handles.clear();
    }
}
