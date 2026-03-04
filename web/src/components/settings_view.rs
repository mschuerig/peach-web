use std::cell::RefCell;
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use send_wrapper::SendWrapper;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::audio_soundfont::SF2Preset;
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use domain::ports::UserSettings;
use domain::types::MIDINote;
use domain::{
    DirectedInterval, Direction, Interval, PerceptualProfile, ThresholdTimeline, TrendAnalyzer,
    TuningSystem,
};

/// Extract the `.value` property from an event's target element.
fn target_value(ev: &web_sys::Event) -> String {
    ev.target()
        .and_then(|t| {
            js_sys::Reflect::get(&t, &wasm_bindgen::JsValue::from_str("value")).ok()
        })
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// Extract the `checked` property from an event's target element.
fn target_checked(ev: &web_sys::Event) -> bool {
    ev.target()
        .and_then(|t| {
            js_sys::Reflect::get(&t, &wasm_bindgen::JsValue::from_str("checked")).ok()
        })
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

use super::page_nav::PageNav;
use crate::interval_codes::interval_label;

/// All 25 directed intervals in display order.
fn all_directed_intervals() -> Vec<DirectedInterval> {
    let intervals = [
        Interval::Prime,
        Interval::MinorSecond,
        Interval::MajorSecond,
        Interval::MinorThird,
        Interval::MajorThird,
        Interval::PerfectFourth,
        Interval::Tritone,
        Interval::PerfectFifth,
        Interval::MinorSixth,
        Interval::MajorSixth,
        Interval::MinorSeventh,
        Interval::MajorSeventh,
        Interval::Octave,
    ];
    let mut result = vec![DirectedInterval::new(Interval::Prime, Direction::Up)];
    for &interval in &intervals[1..] {
        result.push(DirectedInterval::new(interval, Direction::Up));
        result.push(DirectedInterval::new(interval, Direction::Down));
    }
    result
}

#[derive(Clone, Copy, PartialEq)]
enum ResetStatus {
    Idle,
    Resetting,
    Success,
    Error,
}

#[component]
pub fn SettingsView() -> impl IntoView {
    let settings = LocalStorageSettings;
    let range = settings.note_range();

    let note_range_min = RwSignal::new(range.min().raw_value().clamp(21, 108));
    let note_range_max = RwSignal::new(range.max().raw_value().clamp(21, 108));
    let note_duration = RwSignal::new(settings.note_duration().raw_value());
    let reference_pitch = RwSignal::new(settings.reference_pitch().raw_value());
    let vary_loudness_pct = RwSignal::new((settings.vary_loudness() * 100.0).round() as i32);
    let tuning_system = RwSignal::new(
        match settings.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        }
        .to_string(),
    );
    let sound_source = RwSignal::new(
        LocalStorageSettings::get_string("peach.sound_source")
            .unwrap_or_else(|| "oscillator:sine".to_string()),
    );
    let sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage> =
        use_context().expect("sf2_presets not provided");
    let selected_intervals = RwSignal::new(LocalStorageSettings::get_selected_intervals());

    // Reset training data
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile not provided");
    let trend: SendWrapper<Rc<RefCell<TrendAnalyzer>>> =
        use_context().expect("TrendAnalyzer not provided");
    let timeline: SendWrapper<Rc<RefCell<ThresholdTimeline>>> =
        use_context().expect("ThresholdTimeline not provided");
    let db_store: RwSignal<Option<Rc<IndexedDbStore>>, LocalStorage> =
        use_context().expect("db_store not provided");

    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
    let reset_status = RwSignal::new(ResetStatus::Idle);

    let open_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get()
            && let Err(e) = dialog.show_modal()
        {
            web_sys::console::warn_1(
                &format!("Failed to open reset dialog: {e:?}").into(),
            );
        }
    };

    let handle_cancel = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            dialog.close();
        }
    };

    let handle_confirm = move |_| {
        reset_status.set(ResetStatus::Resetting);
        let profile = Rc::clone(&*profile);
        let trend = Rc::clone(&*trend);
        let timeline = Rc::clone(&*timeline);

        spawn_local(async move {
            let db_result = if let Some(store) = db_store.get_untracked() {
                store.delete_all().await
            } else {
                // IndexedDB not loaded — no stored data to delete
                Ok(())
            };

            match db_result {
                Ok(()) => {
                    {
                        let mut p = profile.borrow_mut();
                        p.reset();
                        p.reset_matching();
                    }
                    trend.borrow_mut().reset();
                    timeline.borrow_mut().reset();
                    if let Some(dialog) = dialog_ref.get() {
                        dialog.close();
                    }
                    reset_status.set(ResetStatus::Success);
                    TimeoutFuture::new(2000).await;
                    reset_status.set(ResetStatus::Idle);
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("Failed to delete training data: {e:?}").into(),
                    );
                    if let Some(dialog) = dialog_ref.get() {
                        dialog.close();
                    }
                    reset_status.set(ResetStatus::Error);
                    TimeoutFuture::new(3000).await;
                    reset_status.set(ResetStatus::Idle);
                }
            }
        });
    };

    view! {
        <div class="py-12">
            <PageNav current="settings" />
            <h1 class="text-2xl font-bold dark:text-white">"Settings"</h1>

            <div class="mt-6 space-y-6">
                // Note Range — Lower Bound (AC2)
                <div>
                    <label for="note-range-min" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Note Range — Lower Bound"
                    </label>
                    <select
                        id="note-range-min"
                        class="mt-1 block w-full min-h-11 rounded border border-gray-300 bg-white px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                        prop:value=move || note_range_min.get().to_string()
                        on:change=move |ev| {
                            if let Ok(val) = target_value(&ev).parse::<u8>() {
                                note_range_min.set(val);
                                LocalStorageSettings::set("peach.note_range_min", &val.to_string());
                            }
                        }
                    >
                        {move || {
                            let max = note_range_max.get();
                            (21u8..=max)
                                .map(|i| {
                                    let note = MIDINote::new(i);
                                    let label = format!("{} ({})", note.name(), i);
                                    view! { <option value={i.to_string()}>{label}</option> }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </select>
                </div>

                // Note Range — Upper Bound (AC3)
                <div>
                    <label for="note-range-max" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Note Range — Upper Bound"
                    </label>
                    <select
                        id="note-range-max"
                        class="mt-1 block w-full min-h-11 rounded border border-gray-300 bg-white px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                        prop:value=move || note_range_max.get().to_string()
                        on:change=move |ev| {
                            if let Ok(val) = target_value(&ev).parse::<u8>() {
                                note_range_max.set(val);
                                LocalStorageSettings::set("peach.note_range_max", &val.to_string());
                            }
                        }
                    >
                        {move || {
                            let min = note_range_min.get();
                            (min..=108u8)
                                .map(|i| {
                                    let note = MIDINote::new(i);
                                    let label = format!("{} ({})", note.name(), i);
                                    view! { <option value={i.to_string()}>{label}</option> }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </select>
                </div>

                // Note Duration (AC4)
                <div>
                    <label for="note-duration" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {move || format!("Note Duration — {:.1}s", note_duration.get())}
                    </label>
                    <input
                        type="range"
                        id="note-duration"
                        min="0.3"
                        max="3.0"
                        step="0.1"
                        class="mt-1 block w-full min-h-11 accent-indigo-600 dark:accent-indigo-400"
                        prop:value=move || format!("{:.1}", note_duration.get())
                        on:input=move |ev| {
                            if let Ok(val) = target_value(&ev).parse::<f64>() {
                                let rounded = (val * 10.0).round() / 10.0;
                                note_duration.set(rounded);
                                LocalStorageSettings::set("peach.note_duration", &format!("{rounded:.1}"));
                            }
                        }
                    />
                </div>

                // Reference Pitch (AC5)
                <div>
                    <label for="reference-pitch" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Reference Pitch"
                    </label>
                    <select
                        id="reference-pitch"
                        class="mt-1 block w-full min-h-11 rounded border border-gray-300 bg-white px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                        prop:value=move || reference_pitch.get().to_string()
                        on:change=move |ev| {
                            if let Ok(val) = target_value(&ev).parse::<f64>() {
                                reference_pitch.set(val);
                                LocalStorageSettings::set("peach.reference_pitch", &val.to_string());
                            }
                        }
                    >
                        <option value="440">"440 Hz (Concert)"</option>
                        <option value="442">"442 Hz"</option>
                        <option value="432">"432 Hz"</option>
                        <option value="415">"415 Hz (Baroque)"</option>
                    </select>
                </div>

                // Sound Source (AC6)
                <div>
                    <label for="sound-source" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Sound Source"
                    </label>
                    <select
                        id="sound-source"
                        class="mt-1 block w-full min-h-11 rounded border border-gray-300 bg-white px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                        prop:value=move || sound_source.get()
                        on:change=move |ev| {
                            let val = target_value(&ev);
                            LocalStorageSettings::set("peach.sound_source", &val);
                            sound_source.set(val);
                        }
                    >
                        {move || {
                            let mut presets = sf2_presets.get();
                            if presets.is_empty() {
                                // Fallback when SoundFont is unavailable
                                vec![view! { <option value={"oscillator:sine".to_string()}>{"Sine Oscillator".to_string()}</option> }]
                            } else {
                                presets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                                presets.into_iter().map(|p| {
                                    let value = format!("sf2:{}:{}", p.bank, p.program);
                                    let label = p.name.clone();
                                    view! { <option value={value}>{label}</option> }
                                }).collect::<Vec<_>>()
                            }
                        }}
                    </select>
                </div>

                // Loudness Variation (AC7)
                <div>
                    <label for="vary-loudness" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {move || format!("Loudness Variation — {}%", vary_loudness_pct.get())}
                    </label>
                    <input
                        type="range"
                        id="vary-loudness"
                        min="0"
                        max="100"
                        step="1"
                        class="mt-1 block w-full min-h-11 accent-indigo-600 dark:accent-indigo-400"
                        prop:value=move || vary_loudness_pct.get().to_string()
                        on:input=move |ev| {
                            if let Ok(val) = target_value(&ev).parse::<i32>() {
                                vary_loudness_pct.set(val);
                                let float_val = val as f64 / 100.0;
                                LocalStorageSettings::set("peach.vary_loudness", &float_val.to_string());
                            }
                        }
                    />
                </div>

                // Tuning System (AC8)
                <div>
                    <label for="tuning-system" class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Tuning System"
                    </label>
                    <select
                        id="tuning-system"
                        class="mt-1 block w-full min-h-11 rounded border border-gray-300 bg-white px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
                        prop:value=move || tuning_system.get()
                        on:change=move |ev| {
                            let val = target_value(&ev);
                            LocalStorageSettings::set("peach.tuning_system", &val);
                            tuning_system.set(val);
                        }
                    >
                        <option value="equalTemperament">"Equal Temperament"</option>
                        <option value="justIntonation">"Just Intonation"</option>
                    </select>
                </div>

                // Interval Selection
                <fieldset>
                    <legend class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Interval Selection"
                    </legend>
                    <div class="mt-2 space-y-1">
                        {all_directed_intervals().into_iter().map(|di| {
                            let label_text = interval_label(di.interval, di.direction);
                            view! {
                                <label class="flex items-center gap-3 min-h-[44px] cursor-pointer">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_intervals.get().contains(&di)
                                        disabled=move || {
                                            let set = selected_intervals.get();
                                            set.len() == 1 && set.contains(&di)
                                        }
                                        on:change=move |ev| {
                                            let checked = target_checked(&ev);
                                            selected_intervals.update(|set| {
                                                if checked {
                                                    set.insert(di);
                                                } else if set.len() > 1 {
                                                    set.remove(&di);
                                                }
                                            });
                                            let mut intervals: Vec<DirectedInterval> =
                                                selected_intervals.get().into_iter().collect();
                                            intervals.sort_by_key(|d| (d.interval, d.direction));
                                            match serde_json::to_string(&intervals) {
                                                Ok(json) => LocalStorageSettings::set("peach.intervals", &json),
                                                Err(e) => log::error!("Failed to serialize intervals: {e}"),
                                            }
                                        }
                                        class="h-5 w-5 accent-indigo-600 dark:accent-indigo-400"
                                    />
                                    <span class="text-gray-700 dark:text-gray-300">{label_text}</span>
                                </label>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </fieldset>
            </div>

            // Reset Training Data
            <fieldset class="mt-8 border-t border-gray-200 pt-6 dark:border-gray-700">
                <legend class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    "Danger Zone"
                </legend>
                <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                    "Permanently delete all training data, including your perceptual profile and comparison history. Your settings will be preserved."
                </p>
                <button
                    on:click=open_dialog
                    disabled=move || reset_status.get() == ResetStatus::Resetting
                    class="mt-4 w-full min-h-[44px] rounded-lg bg-red-600 px-4 py-3 font-semibold text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 dark:bg-red-700 dark:hover:bg-red-800 dark:ring-offset-gray-900 disabled:opacity-50"
                >
                    {move || match reset_status.get() {
                        ResetStatus::Resetting => "Resetting\u{2026}",
                        ResetStatus::Success => "Data Reset",
                        ResetStatus::Error => "Reset Failed",
                        ResetStatus::Idle => "Reset all training data",
                    }}
                </button>
            </fieldset>

            // Confirmation dialog
            <dialog
                node_ref=dialog_ref
                aria-labelledby="reset-dialog-title"
                class="rounded-lg p-6 max-w-md mx-auto bg-white text-gray-900 backdrop:bg-black/50 dark:bg-gray-800 dark:text-gray-100"
            >
                <h2 id="reset-dialog-title" class="text-lg font-bold">"Reset Training Data?"</h2>
                <p class="mt-3 text-sm text-gray-600 dark:text-gray-300">
                    "This will permanently delete all training data, including your perceptual profile and comparison history. This cannot be undone."
                </p>
                <div class="mt-6 flex gap-3 justify-end">
                    <button
                        on:click=handle_cancel
                        disabled=move || reset_status.get() == ResetStatus::Resetting
                        class="min-h-[44px] rounded-lg bg-gray-200 px-4 py-2 font-semibold text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600 disabled:opacity-50"
                    >
                        "Cancel"
                    </button>
                    <button
                        on:click=handle_confirm
                        disabled=move || reset_status.get() == ResetStatus::Resetting
                        class="min-h-[44px] rounded-lg bg-red-600 px-4 py-2 font-semibold text-white hover:bg-red-700 dark:bg-red-700 dark:hover:bg-red-800 disabled:opacity-50"
                    >
                        "Delete All Data"
                    </button>
                </div>
            </dialog>

        </div>
    }
}
