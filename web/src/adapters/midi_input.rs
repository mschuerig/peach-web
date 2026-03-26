use wasm_bindgen::prelude::*;
use web_sys::{MidiAccess, MidiInput, MidiMessageEvent, MidiOptions};

/// Check whether the browser supports the Web MIDI API.
///
/// Uses JS reflection to test for `navigator.requestMIDIAccess` without
/// calling it (which would trigger a permission prompt).
pub fn is_midi_available() -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };
    let navigator = window.navigator();
    let key = JsValue::from_str("requestMIDIAccess");
    match js_sys::Reflect::get(&navigator, &key) {
        Ok(val) => val.is_function(),
        Err(_) => false,
    }
}

/// Return `true` if `data` is a MIDI note-on message with velocity > 0.
///
/// MIDI note-on: 3 bytes, status `0x90`-`0x9F` (channels 1-16), velocity > 0.
/// A note-on with velocity 0 is conventionally treated as note-off.
pub fn is_note_on(data: &[u8]) -> bool {
    if data.len() < 3 {
        return false;
    }
    let status = data[0];
    let velocity = data[2];
    (0x90..=0x9F).contains(&status) && velocity > 0
}

type MidiListener = (MidiInput, Closure<dyn FnMut(MidiMessageEvent)>);

/// Stores MIDI event listener closures so they can be removed on cleanup.
///
/// Dropping the handle automatically removes all listeners. Call [`cleanup`](Self::cleanup)
/// for explicit early cleanup with warning logs on failure.
pub struct MidiCleanupHandle {
    _midi_access: MidiAccess,
    listeners: Vec<MidiListener>,
}

impl MidiCleanupHandle {
    /// Remove all `midimessage` event listeners that were attached by
    /// [`setup_midi_listeners`].
    ///
    /// This is equivalent to dropping the handle, but makes the intent explicit
    /// at the call site. Safe to call even if the handle has already been cleaned up.
    pub fn cleanup(mut self) {
        self.remove_listeners();
    }

    fn remove_listeners(&mut self) {
        for (input, closure) in self.listeners.drain(..) {
            if let Err(e) = input.remove_event_listener_with_callback(
                "midimessage",
                closure.as_ref().unchecked_ref(),
            ) {
                web_sys::console::warn_1(&e);
            }
        }
    }
}

impl Drop for MidiCleanupHandle {
    fn drop(&mut self) {
        self.remove_listeners();
    }
}

/// Request MIDI access and attach `midimessage` listeners to every connected input.
///
/// For each note-on event, `on_note_on` is called with the event's
/// `DOMHighResTimeStamp` (in `performance.now()` coordinates - the same
/// coordinate space used by pointer and keyboard events).
///
/// Returns a [`MidiCleanupHandle`] that must be kept alive. Calling
/// [`MidiCleanupHandle::cleanup`] removes all listeners.
pub async fn setup_midi_listeners(
    on_note_on: impl Fn(f64) + 'static,
) -> Result<MidiCleanupHandle, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let navigator = window.navigator();

    let options = MidiOptions::new();
    options.set_sysex(false);

    let promise = navigator.request_midi_access_with_options(&options)?;
    let midi_access: MidiAccess = wasm_bindgen_futures::JsFuture::from(promise).await?.into();

    let input_map = midi_access.inputs();

    let on_note_on = std::rc::Rc::new(on_note_on);
    let mut listeners: Vec<MidiListener> = Vec::new();

    // Iterate the MidiInputMap via its JS iterator protocol.
    // Per-port errors are logged and skipped so one bad port doesn't block the rest.
    let values = input_map.values();
    loop {
        let next = match values.next() {
            Ok(n) => n,
            Err(_) => break,
        };
        if js_sys::Reflect::get(&next, &JsValue::from_str("done")).map_or(true, |v| v.is_truthy()) {
            break;
        }
        let port_result = (|| -> Result<(), JsValue> {
            let value = js_sys::Reflect::get(&next, &JsValue::from_str("value"))?;
            let midi_input: MidiInput = value.dyn_into()?;

            let cb = on_note_on.clone();
            let closure =
                Closure::<dyn FnMut(MidiMessageEvent)>::new(move |event: MidiMessageEvent| {
                    if let Ok(data) = event.data()
                        && is_note_on(&data)
                    {
                        cb(event.time_stamp());
                    }
                });

            midi_input.add_event_listener_with_callback(
                "midimessage",
                closure.as_ref().unchecked_ref(),
            )?;

            listeners.push((midi_input, closure));
            Ok(())
        })();

        if let Err(e) = port_result {
            web_sys::console::warn_1(&e);
        }
    }

    Ok(MidiCleanupHandle {
        _midi_access: midi_access,
        listeners,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_channel_1() {
        assert!(is_note_on(&[0x90, 60, 100]));
    }

    #[test]
    fn test_note_on_channel_16() {
        assert!(is_note_on(&[0x9F, 60, 100]));
    }

    #[test]
    fn test_velocity_zero_is_not_note_on() {
        assert!(!is_note_on(&[0x90, 60, 0]));
    }

    #[test]
    fn test_note_off_is_not_note_on() {
        assert!(!is_note_on(&[0x80, 60, 64]));
    }

    #[test]
    fn test_control_change_is_not_note_on() {
        assert!(!is_note_on(&[0xB0, 64, 127]));
    }

    #[test]
    fn test_truncated_message_is_not_note_on() {
        assert!(!is_note_on(&[0x90, 60]));
        assert!(!is_note_on(&[0x90]));
    }
}
