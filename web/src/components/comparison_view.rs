use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_router::components::A;

use crate::adapters::audio_context::AudioContextManager;
use crate::adapters::audio_oscillator::OscillatorNotePlayer;
use domain::ports::NotePlayer;
use domain::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteDuration};

#[component]
pub fn ComparisonView() -> impl IntoView {
    // TODO: Remove test button in story 1.7
    let context_manager = Rc::new(RefCell::new(AudioContextManager::new()));
    let player = Rc::new(RefCell::new(OscillatorNotePlayer::new(Rc::clone(
        &context_manager,
    ))));

    let on_test_click = move |_| {
        let result = player.borrow().play_for_duration(
            Frequency::new(440.0),
            NoteDuration::new(1.0),
            MIDIVelocity::new(80),
            AmplitudeDB::new(0.0),
        );
        if let Err(e) = result {
            log::error!("Audio playback failed: {e}");
        }
    };

    view! {
        <div class="py-12">
            <h1 class="text-2xl font-bold dark:text-white">"Comparison Training"</h1>
            // TODO: Remove in story 1.7
            <button
                on:click=on_test_click
                class="mt-4 rounded bg-indigo-600 px-4 py-2 text-white hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2"
            >
                "Play Test Note (440 Hz)"
            </button>
            <A href="/"
                attr:class="mt-4 ml-4 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300">
                "Back to Start"
            </A>
        </div>
    }
}
