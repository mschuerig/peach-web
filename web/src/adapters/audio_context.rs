use std::cell::RefCell;
use std::rc::Rc;

use domain::AudioError;
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
            return Ok(Rc::clone(ctx));
        }

        let ctx = AudioContext::new().map_err(|e| {
            AudioError::EngineStartFailed(format!("{:?}", e))
        })?;

        let shared = Rc::new(RefCell::new(ctx));
        self.context = Some(Rc::clone(&shared));
        Ok(shared)
    }
}
