use std::cell::{Cell, RefCell};
use std::rc::Rc;

use domain::AudioError;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, AudioContextOptions};

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

        let opts = AudioContextOptions::new();
        opts.set_latency_hint(&wasm_bindgen::JsValue::from(0.0));
        let ctx = AudioContext::new_with_context_options(&opts)
            .or_else(|_| AudioContext::new())
            .map_err(|e| AudioError::EngineStartFailed(format!("{:?}", e)))?;

        // baseLatency is not exposed by web-sys; read via JS interop
        let base_latency_str = js_sys::Reflect::get(
            ctx.as_ref(),
            &wasm_bindgen::JsValue::from_str("baseLatency"),
        )
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| format!("{:.4}s", v))
        .unwrap_or_else(|| "unavailable".to_string());

        log::debug!(
            "[DIAG] AudioContext created — state: {:?}, sampleRate: {}, baseLatency: {}",
            ctx.state(),
            ctx.sample_rate(),
            base_latency_str
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

const GESTURE_POLL_MS: u32 = 100;

/// Ensures the AudioContext is created and in the `Running` state, waiting for
/// a user gesture if needed.
///
/// This is the single entry point for training views to obtain a running
/// AudioContext. If `resume()` fails (no prior user gesture), it sets
/// `audio_needs_gesture` to `true` and polls until a UI gesture handler
/// clears it, then retries.
///
/// Returns the shared `Rc<RefCell<AudioContext>>` on success.
pub async fn ensure_audio_ready(
    manager: &Rc<RefCell<AudioContextManager>>,
    audio_needs_gesture: RwSignal<bool>,
    cancelled: &Rc<Cell<bool>>,
) -> Result<Rc<RefCell<AudioContext>>, AudioError> {
    let ctx_rc = manager.borrow_mut().get_or_create()?;

    if ensure_running(&ctx_rc).await.is_err() {
        log::info!("AudioContext needs user gesture to start");
        audio_needs_gesture.set(true);
        while audio_needs_gesture.get() {
            if cancelled.get() {
                return Err(AudioError::EngineStartFailed("cancelled".into()));
            }
            TimeoutFuture::new(GESTURE_POLL_MS).await;
        }
        ensure_running(&ctx_rc).await?;
    }

    Ok(ctx_rc)
}

/// Provides the user gesture needed to resume the AudioContext.
///
/// Called from a click handler (which counts as a user gesture for the browser).
/// Tries to resume the AudioContext and clears the `audio_needs_gesture` signal.
pub fn provide_audio_gesture(
    manager: &Rc<RefCell<AudioContextManager>>,
    audio_needs_gesture: RwSignal<bool>,
) {
    if let Ok(ctx) = manager.borrow_mut().get_or_create() {
        let _ = ctx.borrow().resume();
    }
    audio_needs_gesture.set(false);
}

/// Ensures the AudioContext is in the `Running` state.
///
/// If suspended (common when created outside a user gesture), calls `resume()`
/// and awaits the promise. Returns `Err` if the context cannot be resumed.
///
/// Free function because `&self` on AudioContextManager cannot be held across await.
pub async fn ensure_running(ctx_rc: &Rc<RefCell<AudioContext>>) -> Result<(), AudioError> {
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
