use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::AudioContext;

/// Converts a `PointerEvent.timeStamp` or `KeyboardEvent.timeStamp` (ms, in
/// `performance.now()` coordinates) to audio-clock time (seconds) using
/// `AudioContext.getOutputTimestamp()`.
///
/// Returns `None` when `getOutputTimestamp()` is unsupported (Safari) or returns
/// unusable data (zero `contextTime`).
pub fn bridge_event_to_audio_time(
    ctx: &Rc<RefCell<AudioContext>>,
    event_timestamp_ms: f64,
) -> Option<f64> {
    let ctx_ref = ctx.borrow();

    // Call getOutputTimestamp() via JS interop (not in web-sys)
    let ots = js_sys::Reflect::get(ctx_ref.as_ref(), &JsValue::from_str("getOutputTimestamp"))
        .ok()
        .filter(|v| v.is_function())?;

    let func: &js_sys::Function = ots.dyn_ref()?;
    let result = func.call0(ctx_ref.as_ref()).ok()?;

    let context_time = js_sys::Reflect::get(&result, &JsValue::from_str("contextTime"))
        .ok()
        .and_then(|v| v.as_f64())?;

    let performance_time = js_sys::Reflect::get(&result, &JsValue::from_str("performanceTime"))
        .ok()
        .and_then(|v| v.as_f64())?;

    // Feature detection: contextTime 0.0 means the context hasn't started
    // producing output yet, or the API is not truly supported.
    if context_time <= 0.0 || performance_time <= 0.0 {
        return None;
    }

    // Bridge: convert event timestamp from performance.now() ms to audio seconds
    let audio_time = context_time + (event_timestamp_ms - performance_time) / 1000.0;

    // Reject nonsensical results (e.g. synthetic events with zero timestamp)
    if audio_time <= 0.0 {
        return None;
    }

    Some(audio_time)
}
