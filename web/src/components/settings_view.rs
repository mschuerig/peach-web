use std::cell::RefCell;
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::audio_soundfont::SF2Preset;
use crate::adapters::data_portability;
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

#[derive(Clone, PartialEq)]
enum ImportExportStatus {
    Idle,
    Exporting,
    ExportSuccess,
    Importing,
    ImportSuccess(String),
    Error(String),
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

            // Data Management (Export / Import)
            {
                let ie_status = RwSignal::new(ImportExportStatus::Idle);
                let sr_announcement = RwSignal::new(String::new());
                let import_dialog_ref = NodeRef::<leptos::html::Dialog>::new();
                let import_data_signal: RwSignal<Option<data_portability::ParsedImportData>> = RwSignal::new(None);
                let file_input_ref = NodeRef::<leptos::html::Input>::new();

                let handle_export = {
                    move |_| {
                        ie_status.set(ImportExportStatus::Exporting);
                        spawn_local(async move {
                            let result = if let Some(store) = db_store.get_untracked() {
                                data_portability::export_all_data(&store).await
                            } else {
                                Err("Database not available".to_string())
                            };
                            match result {
                                Ok(()) => {
                                    ie_status.set(ImportExportStatus::ExportSuccess);
                                    sr_announcement.set("Data exported".into());
                                    TimeoutFuture::new(2000).await;
                                    ie_status.set(ImportExportStatus::Idle);
                                }
                                Err(e) => {
                                    let msg = format!("Export failed: {e}");
                                    sr_announcement.set(msg.clone());
                                    ie_status.set(ImportExportStatus::Error(msg));
                                    TimeoutFuture::new(3000).await;
                                    ie_status.set(ImportExportStatus::Idle);
                                }
                            }
                        });
                    }
                };

                let handle_import_click = move |_| {
                    if let Some(input) = file_input_ref.get() {
                        input.click();
                    }
                };

                let handle_file_selected = move |_| {
                    let input = match file_input_ref.get() {
                        Some(i) => i,
                        None => return,
                    };
                    let file = match input.files().and_then(|fl| fl.get(0)) {
                        Some(f) => f,
                        None => return,
                    };

                    // Reject files larger than 10 MB to prevent browser OOM
                    if file.size() > 10_000_000.0 {
                        let msg = "File too large (max 10 MB)".to_string();
                        sr_announcement.set(msg.clone());
                        ie_status.set(ImportExportStatus::Error(msg));
                        spawn_local(async move {
                            TimeoutFuture::new(3000).await;
                            ie_status.set(ImportExportStatus::Idle);
                        });
                        input.set_value("");
                        return;
                    }

                    let reader = match web_sys::FileReader::new() {
                        Ok(r) => r,
                        Err(e) => {
                            let msg = format!("Failed to create FileReader: {e:?}");
                            sr_announcement.set(msg.clone());
                            ie_status.set(ImportExportStatus::Error(msg));
                            return;
                        }
                    };

                    let reader_clone = reader.clone();
                    let onload = Closure::once(move |_event: web_sys::Event| {
                        let content = match reader_clone.result() {
                            Ok(val) => val.as_string().unwrap_or_default(),
                            Err(e) => {
                                let msg = format!("Failed to read file: {e:?}");
                                sr_announcement.set(msg.clone());
                                ie_status.set(ImportExportStatus::Error(msg));
                                return;
                            }
                        };

                        match data_portability::parse_import_file(&content) {
                            Ok(parsed) => {
                                for warning in &parsed.warnings {
                                    web_sys::console::warn_1(&warning.into());
                                }
                                import_data_signal.set(Some(parsed));
                                if let Some(dialog) = import_dialog_ref.get() {
                                    let _ = dialog.show_modal();
                                }
                            }
                            Err(e) => {
                                let msg = format!("Import failed: {e}");
                                sr_announcement.set(msg.clone());
                                ie_status.set(ImportExportStatus::Error(msg));
                                spawn_local(async move {
                                    TimeoutFuture::new(3000).await;
                                    ie_status.set(ImportExportStatus::Idle);
                                });
                            }
                        }
                    });

                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    let _ = reader.read_as_text(&file);
                    // Reset input so the same file can be selected again
                    input.set_value("");
                };

                let handle_import_replace = move |_| {
                    ie_status.set(ImportExportStatus::Importing);
                    if let Some(dialog) = import_dialog_ref.get() {
                        dialog.close();
                    }
                    let data = import_data_signal.get_untracked();
                    spawn_local(async move {
                        if let Some(data) = data.as_ref() {
                            let result = if let Some(store) = db_store.get_untracked() {
                                data_portability::import_replace(&store, data).await
                            } else {
                                Err("Database not available".to_string())
                            };
                            match result {
                                Ok(count) => {
                                    let msg = format!("{count} records imported");
                                    sr_announcement.set(msg.clone());
                                    ie_status.set(ImportExportStatus::ImportSuccess(msg));
                                    TimeoutFuture::new(1500).await;
                                    data_portability::reload_page();
                                }
                                Err(e) => {
                                    let msg = format!("Import failed: {e}");
                                    sr_announcement.set(msg.clone());
                                    ie_status.set(ImportExportStatus::Error(msg));
                                    TimeoutFuture::new(3000).await;
                                    ie_status.set(ImportExportStatus::Idle);
                                }
                            }
                        }
                    });
                };

                let handle_import_merge = move |_| {
                    ie_status.set(ImportExportStatus::Importing);
                    if let Some(dialog) = import_dialog_ref.get() {
                        dialog.close();
                    }
                    let data = import_data_signal.get_untracked();
                    spawn_local(async move {
                        if let Some(data) = data.as_ref() {
                            let result = if let Some(store) = db_store.get_untracked() {
                                data_portability::import_merge(&store, data).await
                            } else {
                                Err("Database not available".to_string())
                            };
                            match result {
                                Ok(r) => {
                                    let imported = r.comparison_imported + r.pitch_matching_imported;
                                    let skipped = r.comparison_skipped + r.pitch_matching_skipped;
                                    let msg = format!("{imported} records imported, {skipped} duplicates skipped");
                                    sr_announcement.set(msg.clone());
                                    ie_status.set(ImportExportStatus::ImportSuccess(msg));
                                    TimeoutFuture::new(1500).await;
                                    data_portability::reload_page();
                                }
                                Err(e) => {
                                    let msg = format!("Import failed: {e}");
                                    sr_announcement.set(msg.clone());
                                    ie_status.set(ImportExportStatus::Error(msg));
                                    TimeoutFuture::new(3000).await;
                                    ie_status.set(ImportExportStatus::Idle);
                                }
                            }
                        }
                    });
                };

                let handle_import_cancel = move |_| {
                    if let Some(dialog) = import_dialog_ref.get() {
                        dialog.close();
                    }
                    import_data_signal.set(None);
                };

                view! {
                    <fieldset class="mt-8 border-t border-gray-200 pt-6 dark:border-gray-700">
                        <legend class="text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Data Management"
                        </legend>
                        <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                            "Export your training data as CSV or import data from another device. Compatible with the iOS app."
                        </p>

                        <div class="mt-4 flex gap-3">
                            <button
                                on:click=handle_export
                                disabled=move || !matches!(ie_status.get(), ImportExportStatus::Idle)
                                class="flex-1 min-h-[44px] rounded-lg bg-indigo-600 px-4 py-3 font-semibold text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:bg-indigo-700 dark:hover:bg-indigo-800 dark:ring-offset-gray-900 disabled:opacity-50"
                            >
                                {move || match ie_status.get() {
                                    ImportExportStatus::Exporting => "Exporting\u{2026}".to_string(),
                                    ImportExportStatus::ExportSuccess => "Exported!".to_string(),
                                    _ => "Export Data".to_string(),
                                }}
                            </button>
                            <button
                                on:click=handle_import_click
                                disabled=move || !matches!(ie_status.get(), ImportExportStatus::Idle)
                                class="flex-1 min-h-[44px] rounded-lg bg-indigo-600 px-4 py-3 font-semibold text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:bg-indigo-700 dark:hover:bg-indigo-800 dark:ring-offset-gray-900 disabled:opacity-50"
                            >
                                {move || match ie_status.get() {
                                    ImportExportStatus::Importing => "Importing\u{2026}".to_string(),
                                    ImportExportStatus::ImportSuccess(ref msg) => msg.clone(),
                                    _ => "Import Data".to_string(),
                                }}
                            </button>
                        </div>

                        // Hidden file input for import
                        <input
                            node_ref=file_input_ref
                            type="file"
                            accept=".csv"
                            on:change=handle_file_selected
                            class="sr-only"
                            aria-label="Select CSV file to import"
                        />

                        // Status message
                        {move || {
                            if let ImportExportStatus::Error(ref msg) = ie_status.get() {
                                Some(view! {
                                    <p class="mt-2 text-sm text-red-600 dark:text-red-400" role="alert">
                                        {msg.clone()}
                                    </p>
                                })
                            } else {
                                None
                            }
                        }}

                        // Screen reader announcement
                        <div aria-live="polite" aria-atomic="true" class="sr-only">
                            {move || sr_announcement.get()}
                        </div>

                        // Import mode dialog
                        <dialog
                            node_ref=import_dialog_ref
                            aria-labelledby="import-dialog-title"
                            class="rounded-lg p-6 max-w-md mx-auto bg-white text-gray-900 backdrop:bg-black/50 dark:bg-gray-800 dark:text-gray-100"
                        >
                            <h2 id="import-dialog-title" class="text-lg font-bold">"Import Training Data"</h2>
                            <p class="mt-3 text-sm text-gray-600 dark:text-gray-300">
                                {move || {
                                    if let Some(ref data) = import_data_signal.get() {
                                        let warnings_text = if data.warnings.is_empty() {
                                            String::new()
                                        } else {
                                            format!(" ({} rows skipped with warnings)", data.warnings.len())
                                        };
                                        format!(
                                            "Found {} comparison records and {} pitch matching records.{} How would you like to import?",
                                            data.pitch_comparisons.len(),
                                            data.pitch_matchings.len(),
                                            warnings_text,
                                        )
                                    } else {
                                        String::new()
                                    }
                                }}
                            </p>
                            <div class="mt-6 flex flex-col gap-3">
                                <button
                                    on:click=handle_import_replace
                                    class="min-h-[44px] rounded-lg bg-red-600 px-4 py-2 font-semibold text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 dark:bg-red-700 dark:hover:bg-red-800 dark:ring-offset-gray-900"
                                >
                                    "Replace All Data"
                                </button>
                                <button
                                    on:click=handle_import_merge
                                    class="min-h-[44px] rounded-lg bg-indigo-600 px-4 py-2 font-semibold text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:bg-indigo-700 dark:hover:bg-indigo-800 dark:ring-offset-gray-900"
                                >
                                    "Merge with Existing"
                                </button>
                                <button
                                    on:click=handle_import_cancel
                                    class="min-h-[44px] rounded-lg bg-gray-200 px-4 py-2 font-semibold text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600 dark:ring-offset-gray-900"
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </dialog>
                    </fieldset>
                }
            }

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
                        class="min-h-[44px] rounded-lg bg-gray-200 px-4 py-2 font-semibold text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600 dark:ring-offset-gray-900 disabled:opacity-50"
                    >
                        "Cancel"
                    </button>
                    <button
                        on:click=handle_confirm
                        disabled=move || reset_status.get() == ResetStatus::Resetting
                        class="min-h-[44px] rounded-lg bg-red-600 px-4 py-2 font-semibold text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 dark:bg-red-700 dark:hover:bg-red-800 dark:ring-offset-gray-900 disabled:opacity-50"
                    >
                        "Delete All Data"
                    </button>
                </div>
            </dialog>

        </div>
    }
}
