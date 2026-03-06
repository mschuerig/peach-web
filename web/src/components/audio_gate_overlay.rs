use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::adapters::audio_context::{AudioContextManager, provide_audio_gesture};

/// Overlay shown when the AudioContext needs a user gesture to resume.
///
/// Reads `audio_needs_gesture: RwSignal<bool>` from context. When `true`,
/// displays a "Tap to Start Training" button. Clicking it provides the
/// browser gesture via `provide_audio_gesture` and hides the overlay.
#[component]
pub fn AudioGateOverlay() -> impl IntoView {
    let crate::app::AudioNeedsGesture(audio_needs_gesture) =
        use_context().expect("audio_needs_gesture context");
    let audio_ctx: SendWrapper<Rc<RefCell<AudioContextManager>>> =
        use_context().expect("AudioContextManager context");

    let on_tap = {
        let manager = Rc::clone(&*audio_ctx);
        #[allow(clippy::redundant_closure)]
        let handler = SendWrapper::new(move |_: leptos::ev::MouseEvent| {
            provide_audio_gesture(&manager, audio_needs_gesture);
        });
        Callback::new(move |ev: leptos::ev::MouseEvent| handler(ev))
    };

    view! {
        <div class=move || {
            if audio_needs_gesture.get() {
                "flex flex-col items-center justify-center gap-6 py-16"
            } else {
                "hidden"
            }
        }>
            <button
                on:click=move |ev| on_tap.run(ev)
                class="rounded-2xl bg-blue-500 px-10 py-6 text-xl font-semibold text-white shadow-md hover:bg-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:ring-offset-2 dark:bg-blue-600 dark:hover:bg-blue-500 dark:focus:ring-offset-gray-900"
            >
                "Tap to Start Training"
            </button>
        </div>
    }
}
