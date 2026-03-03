use leptos::prelude::*;
use leptos_router::components::A;

use crate::adapters::localstorage_settings::LocalStorageSettings;
use domain::ports::UserSettings;
use domain::types::MIDINote;
use domain::{DirectedInterval, Direction, Interval, TuningSystem};

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

/// Human-readable label for an interval with direction.
fn interval_label(interval: Interval, direction: Direction) -> String {
    let name = match interval {
        Interval::Prime => return "Prime".to_string(),
        Interval::MinorSecond => "Minor Second",
        Interval::MajorSecond => "Major Second",
        Interval::MinorThird => "Minor Third",
        Interval::MajorThird => "Major Third",
        Interval::PerfectFourth => "Perfect Fourth",
        Interval::Tritone => "Tritone",
        Interval::PerfectFifth => "Perfect Fifth",
        Interval::MinorSixth => "Minor Sixth",
        Interval::MajorSixth => "Major Sixth",
        Interval::MinorSeventh => "Minor Seventh",
        Interval::MajorSeventh => "Major Seventh",
        Interval::Octave => "Octave",
    };
    let dir = match direction {
        Direction::Up => "Up",
        Direction::Down => "Down",
    };
    format!("{name} {dir}")
}

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
    let selected_intervals = RwSignal::new(LocalStorageSettings::get_selected_intervals());

    view! {
        <div class="py-12">
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
                        <option value="oscillator:sine">"Sine Oscillator"</option>
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
                <div>
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Interval Selection"
                    </span>
                    <div class="mt-2 space-y-1">
                        {all_directed_intervals().into_iter().map(|di| {
                            let label_text = interval_label(di.interval, di.direction);
                            view! {
                                <label class="flex items-center gap-3 min-h-[44px] cursor-pointer">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_intervals.get().contains(&di)
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
                                            if let Ok(json) = serde_json::to_string(&intervals) {
                                                LocalStorageSettings::set("peach.intervals", &json);
                                            }
                                        }
                                        class="h-5 w-5 accent-indigo-600 dark:accent-indigo-400"
                                    />
                                    <span class="text-gray-700 dark:text-gray-300">{label_text}</span>
                                </label>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>

            <A
                href="/"
                attr:class="mt-8 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:ring-offset-gray-900 dark:text-indigo-400 dark:hover:text-indigo-300"
            >
                "Back to Start"
            </A>
        </div>
    }
}
