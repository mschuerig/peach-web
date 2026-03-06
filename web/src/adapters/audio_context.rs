use std::cell::RefCell;
use std::rc::Rc;

use domain::AudioError;
use wasm_bindgen_futures::JsFuture;
use web_sys::AudioContext;

/// Manages the singleton AudioContext lifecycle.
///
/// No AudioContext is created until `get_or_create()` is called (lazy initialization).
/// The first call must happen inside a user gesture handler (click/keypress) to satisfy
/// browser autoplay policies.
#[derive(Default)]
pub struct AudioContextManager {
    context: Option<Rc<RefCell<AudioContext>>>,
}

impl AudioContextManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the shared AudioContext, creating it on first call.
    ///
    /// Must be called within a user gesture event handler the first time.
    pub fn get_or_create(&mut self) -> Result<Rc<RefCell<AudioContext>>, AudioError> {
        if let Some(ref ctx) = self.context {
            log::debug!(
                "[DIAG] AudioContext reused — state: {:?}",
                ctx.borrow().state()
            );
            return Ok(Rc::clone(ctx));
        }

        let ctx = AudioContext::new().map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;

        log::debug!(
            "[DIAG] AudioContext created — state: {:?}, sampleRate: {}",
            ctx.state(),
            ctx.sample_rate()
        );

        let shared = Rc::new(RefCell::new(ctx));
        self.context = Some(Rc::clone(&shared));
        Ok(shared)
    }

    /// Attaches a state change handler to the AudioContext.
    pub fn set_state_change_handler(&self, callback: &js_sys::Function) {
        if let Some(ctx) = &self.context {
            ctx.borrow().set_onstatechange(Some(callback));
        }
    }

    /// Removes the state change handler from the AudioContext.
    pub fn clear_state_change_handler(&self) {
        if let Some(ctx) = &self.context {
            ctx.borrow().set_onstatechange(None);
        }
    }
}

/// Ensures the AudioContext is in the `Running` state.
///
/// If suspended (common when created outside a user gesture), calls `resume()`
/// and awaits the promise. Returns `Err` if the context cannot be resumed.
///
/// Free function because `&self` on AudioContextManager cannot be held across await.
pub async fn ensure_running(
    ctx_rc: &Rc<RefCell<AudioContext>>,
) -> Result<(), AudioError> {
    use web_sys::AudioContextState;

    let state = ctx_rc.borrow().state();
    match state {
        AudioContextState::Running => Ok(()),
        AudioContextState::Suspended => {
            let promise = ctx_rc
                .borrow()
                .resume()
                .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;
            JsFuture::from(promise)
                .await
                .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;
            let new_state = ctx_rc.borrow().state();
            if new_state == AudioContextState::Running {
                Ok(())
            } else {
                Err(AudioError::EngineStartFailed(format!(
                    "AudioContext resume failed, state: {:?}",
                    new_state
                )))
            }
        }
        _ => Err(AudioError::EngineStartFailed(format!(
            "AudioContext in unexpected state: {:?}",
            state
        ))),
    }
}
