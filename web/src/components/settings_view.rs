use std::cell::RefCell;
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use send_wrapper::SendWrapper;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::audio_context::AudioContextManager;
use crate::adapters::audio_soundfont::SF2Preset;
use crate::adapters::csv_export_import;
use crate::adapters::csv_export_import::{ImportExportStatus, ResetStatus};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::sound_preview::SoundPreview;
use crate::app::base_href;
use crate::app::{AudioNeedsGesture, SoundFontLoadStatus, WorkletAssets, WorkletConnecting};
use domain::ports::UserSettings;
use domain::types::{DetunedMIDINote, Frequency, MIDINote};
use domain::{
    DirectedInterval, Direction, Interval, PerceptualProfile, ThresholdTimeline, TrendAnalyzer,
    TuningSystem,
};

use super::help_content::HelpModal;
use super::nav_bar::{NavBar, NavIconButton};
use crate::help_sections::SETTINGS_HELP;

/// Duration of the sound source preview in seconds.
const PREVIEW_DURATION_SECS: f64 = 2.0;

/// Extract the `.value` property from an event's target element.
fn target_value(ev: &web_sys::Event) -> String {
    ev.target()
        .and_then(|t| js_sys::Reflect::get(&t, &wasm_bindgen::JsValue::from_str("value")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// iOS-style grouped settings section with a muted header and rounded card.
#[component]
fn SettingsSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="mt-6">
            <h2 class="px-4 mb-1 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                {title}
            </h2>
            <div class="rounded-xl bg-gray-100 dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {children()}
            </div>
        </div>
    }
}

/// A single row inside a SettingsSection card.
#[component]
fn SettingsRow(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between px-4 py-3 min-h-[44px]">
            <span class="text-sm text-gray-900 dark:text-gray-100">{label}</span>
            <div class="flex items-center">
                {children()}
            </div>
        </div>
    }
}

/// A row with a dynamic (reactive) label.
#[component]
fn SettingsRowDynamic(label: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between px-4 py-3 min-h-[44px]">
            <span class="text-sm text-gray-900 dark:text-gray-100">{move || label.get()}</span>
            <div class="flex items-center">
                {children()}
            </div>
        </div>
    }
}

/// iOS-style +/- stepper control.
#[component]
fn Stepper(
    label: &'static str,
    on_decrement: Callback<()>,
    on_increment: Callback<()>,
    #[prop(into)] decrement_disabled: Signal<bool>,
    #[prop(into)] increment_disabled: Signal<bool>,
) -> impl IntoView {
    let dec_label = format!("Decrease {label}");
    let inc_label = format!("Increase {label}");
    view! {
        <div class="inline-flex items-center rounded-lg bg-gray-200 dark:bg-gray-700" role="group">
            <button
                on:click=move |_| on_decrement.run(())
                disabled=decrement_disabled
                aria-label=dec_label
                class="min-h-[34px] min-w-[40px] flex items-center justify-center rounded-l-lg text-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-30 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-400"
            >
                "\u{2212}"
            </button>
            <div class="w-px h-5 bg-gray-300 dark:bg-gray-600"></div>
            <button
                on:click=move |_| on_increment.run(())
                disabled=increment_disabled
                aria-label=inc_label
                class="min-h-[34px] min-w-[40px] flex items-center justify-center rounded-r-lg text-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-30 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-400"
            >
                "+"
            </button>
        </div>
    }
}

const TOGGLE_ACTIVE: &str = "w-7 h-7 rounded text-[10px] font-semibold bg-indigo-600 text-white dark:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400";
const TOGGLE_INACTIVE: &str = "w-7 h-7 rounded text-[10px] font-semibold bg-gray-200 text-gray-500 dark:bg-gray-700 dark:text-gray-400 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-30 focus:outline-none focus:ring-2 focus:ring-indigo-400";

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
    let sf2_load_status: RwSignal<SoundFontLoadStatus> =
        use_context().expect("sf2_load_status not provided");
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

    // Sound source preview
    let audio_ctx_manager: SendWrapper<Rc<RefCell<AudioContextManager>>> =
        use_context().expect("AudioContextManager not provided");
    let AudioNeedsGesture(audio_needs_gesture) =
        use_context().expect("AudioNeedsGesture not provided");
    let worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, LocalStorage> =
        use_context().expect("worklet_assets not provided");
    let WorkletConnecting(worklet_connecting) =
        use_context().expect("WorkletConnecting not provided");
    let worklet_bridge = use_context().expect("worklet_bridge not provided");
    let sf_gain_node: RwSignal<Option<Rc<web_sys::GainNode>>, LocalStorage> =
        use_context().expect("sf_gain_node not provided");
    let preview = SendWrapper::new(SoundPreview::new(
        PREVIEW_DURATION_SECS,
        Rc::clone(&*audio_ctx_manager),
        audio_needs_gesture,
        worklet_bridge,
        worklet_assets,
        worklet_connecting,
        sf2_presets,
        sf_gain_node,
    ));

    // Stop preview on sound source change (Task 4)
    let preview_for_effect = SendWrapper::new((*preview).clone());
    Effect::new(move |prev: Option<String>| {
        let current = sound_source.get();
        if let Some(prev_val) = prev
            && prev_val != current
        {
            preview_for_effect.stop();
        }
        current
    });

    // Stop preview on navigation away (Task 5)
    let preview_for_cleanup = SendWrapper::new((*preview).clone());
    on_cleanup(move || {
        preview_for_cleanup.stop();
    });

    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
    let reset_status = RwSignal::new(ResetStatus::Idle);

    let open_dialog = move |_| {
        if let Some(dialog) = dialog_ref.get()
            && let Err(e) = dialog.show_modal()
        {
            web_sys::console::warn_1(&format!("Failed to open reset dialog: {e:?}").into());
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

    let is_help_open = RwSignal::new(false);

    // Derived signals for pitch range display
    let min_note_name = Signal::derive(move || MIDINote::new(note_range_min.get()).name());
    let max_note_name = Signal::derive(move || MIDINote::new(note_range_max.get()).name());
    let min_label = Signal::derive(move || format!("Lowest Note: {}", min_note_name.get()));
    let max_label = Signal::derive(move || format!("Highest Note: {}", max_note_name.get()));

    // Derived signals for sound settings display
    let duration_label = Signal::derive(move || format!("Duration: {:.1}s", note_duration.get()));
    let pitch_label = Signal::derive(move || {
        format!("Concert Pitch: {} Hz", reference_pitch.get().round() as i32)
    });

    view! {
        <div class="pt-4 pb-12">
            <NavBar title="Settings" back_href=base_href("/")>
                <NavIconButton label="Help".to_string() icon="?".to_string() on_click=Callback::new(move |_| is_help_open.set(true)) circled=true />
            </NavBar>
            <HelpModal title="Settings Help" sections=SETTINGS_HELP is_open=is_help_open />

            // Pitch Range section (AC: 1, 3)
            <SettingsSection title="Pitch Range">
                <SettingsRowDynamic label=min_label>
                    <Stepper
                        label="lowest note"
                        on_decrement=Callback::new(move |_| {
                            let val = note_range_min.get();
                            if val > 21 {
                                let new_val = val - 1;
                                note_range_min.set(new_val);
                                LocalStorageSettings::set("peach.note_range_min", &new_val.to_string());
                            }
                        })
                        on_increment=Callback::new(move |_| {
                            let val = note_range_min.get();
                            let max = note_range_max.get();
                            if val < max - 1 {
                                let new_val = val + 1;
                                note_range_min.set(new_val);
                                LocalStorageSettings::set("peach.note_range_min", &new_val.to_string());
                            }
                        })
                        decrement_disabled=Signal::derive(move || note_range_min.get() <= 21)
                        increment_disabled=Signal::derive(move || note_range_min.get() >= note_range_max.get() - 1)
                    />
                </SettingsRowDynamic>
                <SettingsRowDynamic label=max_label>
                    <Stepper
                        label="highest note"
                        on_decrement=Callback::new(move |_| {
                            let val = note_range_max.get();
                            let min = note_range_min.get();
                            if val > min + 1 {
                                let new_val = val - 1;
                                note_range_max.set(new_val);
                                LocalStorageSettings::set("peach.note_range_max", &new_val.to_string());
                            }
                        })
                        on_increment=Callback::new(move |_| {
                            let val = note_range_max.get();
                            if val < 108 {
                                let new_val = val + 1;
                                note_range_max.set(new_val);
                                LocalStorageSettings::set("peach.note_range_max", &new_val.to_string());
                            }
                        })
                        decrement_disabled=Signal::derive(move || note_range_max.get() <= note_range_min.get() + 1)
                        increment_disabled=Signal::derive(move || note_range_max.get() >= 108)
                    />
                </SettingsRowDynamic>
            </SettingsSection>

            // Intervals section (AC: 1, 4, 9)
            <SettingsSection title="Intervals">
                <div class="px-4 py-3">
                    <div class="overflow-x-auto">
                        <table class="w-full text-center text-xs">
                            <thead>
                                <tr>
                                    <th class="w-8"></th>
                                    {Interval::all_chromatic().iter().map(|&interval| {
                                        view! {
                                            <th class="px-0.5 py-1 font-medium text-gray-500 dark:text-gray-400">
                                                {interval.short_label()}
                                            </th>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tr>
                            </thead>
                            <tbody>
                                // Ascending row
                                <tr>
                                    <td class="text-gray-400 dark:text-gray-500 pr-1">{"\u{2191}"}</td>
                                    {Interval::all_chromatic().iter().map(|&interval| {
                                        let di = DirectedInterval::new(interval, Direction::Up);
                                        view! {
                                            <td class="px-0.5 py-1">
                                                <button
                                                    on:click=move |_| {
                                                        selected_intervals.update(|set| {
                                                            if set.contains(&di) {
                                                                if set.len() > 1 {
                                                                    set.remove(&di);
                                                                }
                                                            } else {
                                                                set.insert(di);
                                                            }
                                                        });
                                                        LocalStorageSettings::set_selected_intervals(&selected_intervals.get());
                                                    }
                                                    disabled=move || {
                                                        let set = selected_intervals.get();
                                                        set.len() == 1 && set.contains(&di)
                                                    }
                                                    aria-pressed=move || selected_intervals.get().contains(&di).to_string()
                                                    aria-label=format!("{} ascending", interval.short_label())
                                                    class=move || {
                                                        if selected_intervals.get().contains(&di) { TOGGLE_ACTIVE } else { TOGGLE_INACTIVE }
                                                    }
                                                >
                                                    {interval.short_label()}
                                                </button>
                                            </td>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tr>
                                // Descending row
                                <tr>
                                    <td class="text-gray-400 dark:text-gray-500 pr-1">{"\u{2193}"}</td>
                                    {Interval::all_chromatic().iter().map(|&interval| {
                                        if interval == Interval::Prime {
                                            // P1 descending is hidden (same as ascending)
                                            return view! { <td class="px-0.5 py-1"></td> }.into_any();
                                        }
                                        let di = DirectedInterval::new(interval, Direction::Down);
                                        view! {
                                            <td class="px-0.5 py-1">
                                                <button
                                                    on:click=move |_| {
                                                        selected_intervals.update(|set| {
                                                            if set.contains(&di) {
                                                                if set.len() > 1 {
                                                                    set.remove(&di);
                                                                }
                                                            } else {
                                                                set.insert(di);
                                                            }
                                                        });
                                                        LocalStorageSettings::set_selected_intervals(&selected_intervals.get());
                                                    }
                                                    disabled=move || {
                                                        let set = selected_intervals.get();
                                                        set.len() == 1 && set.contains(&di)
                                                    }
                                                    aria-pressed=move || selected_intervals.get().contains(&di).to_string()
                                                    aria-label=format!("{} descending", interval.short_label())
                                                    class=move || {
                                                        if selected_intervals.get().contains(&di) { TOGGLE_ACTIVE } else { TOGGLE_INACTIVE }
                                                    }
                                                >
                                                    {interval.short_label()}
                                                </button>
                                            </td>
                                        }.into_any()
                                    }).collect::<Vec<_>>()}
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </SettingsSection>
            <p class="px-4 mt-1 text-xs text-gray-500 dark:text-gray-400">
                "Select the intervals you want to practice. At least one must remain active."
            </p>

            // Sound section (AC: 1, 5, 6)
            <SettingsSection title="Sound">
                <SettingsRow label="Sound">
                    {move || {
                        let status = sf2_load_status.get();
                        match status {
                            SoundFontLoadStatus::Fetching => view! {
                                <span class="text-sm text-gray-400 dark:text-gray-500 italic">"Loading sounds\u{2026}"</span>
                            }.into_any(),
                            _ => {
                                let preview_for_click = (*preview).clone();
                                let preview_playing = preview.playing_signal();
                                view! {
                                    <div class="flex items-center gap-2">
                                        <select
                                            class="text-sm text-right bg-transparent text-gray-500 dark:text-gray-400 border-none focus:outline-none focus:ring-0 cursor-pointer appearance-none pr-0"
                                            prop:value=move || sound_source.get()
                                            on:change=move |ev| {
                                                let val = target_value(&ev);
                                                LocalStorageSettings::set("peach.sound_source", &val);
                                                sound_source.set(val);
                                            }
                                        >
                                            {move || {
                                                let mut options = vec![
                                                    view! { <option value={"oscillator:sine".to_string()}>{"Sine Oscillator".to_string()}</option> }
                                                ];
                                                let mut presets = sf2_presets.get();
                                                presets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                                                options.extend(presets.into_iter().map(|p| {
                                                    let value = format!("sf2:{}:{}", p.bank, p.program);
                                                    let label = p.name.clone();
                                                    view! { <option value={value}>{label}</option> }
                                                }));
                                                options
                                            }}
                                        </select>
                                        <button
                                            aria-label=move || if preview_playing.get() { "Stop preview" } else { "Preview sound" }
                                            class="min-h-[34px] min-w-[34px] flex items-center justify-center rounded-lg text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-indigo-400"
                                            on:click=move |_| {
                                                let source = sound_source.get_untracked();
                                                let pitch = reference_pitch.get_untracked();
                                                let frequency = TuningSystem::EqualTemperament.frequency(
                                                    DetunedMIDINote::from(MIDINote::new(69)),
                                                    Frequency::new(pitch),
                                                );
                                                preview_for_click.toggle(&source, frequency);
                                            }
                                        >
                                            {move || if preview_playing.get() {
                                                // Stop icon (square)
                                                "\u{25A0}"
                                            } else {
                                                // Speaker icon
                                                "\u{1F50A}"
                                            }}
                                        </button>
                                    </div>
                                }.into_any()
                            },
                        }
                    }}
                </SettingsRow>
                <SettingsRowDynamic label=duration_label>
                    <Stepper
                        label="duration"
                        on_decrement=Callback::new(move |_| {
                            let val = note_duration.get();
                            let new_val = ((val * 10.0 - 1.0).round() / 10.0).max(0.3);
                            note_duration.set(new_val);
                            LocalStorageSettings::set("peach.note_duration", &format!("{new_val:.1}"));
                        })
                        on_increment=Callback::new(move |_| {
                            let val = note_duration.get();
                            let new_val = ((val * 10.0 + 1.0).round() / 10.0).min(3.0);
                            note_duration.set(new_val);
                            LocalStorageSettings::set("peach.note_duration", &format!("{new_val:.1}"));
                        })
                        decrement_disabled=Signal::derive(move || note_duration.get() <= 0.3)
                        increment_disabled=Signal::derive(move || note_duration.get() >= 3.0)
                    />
                </SettingsRowDynamic>
                <SettingsRowDynamic label=pitch_label>
                    <Stepper
                        label="concert pitch"
                        on_decrement=Callback::new(move |_| {
                            let val = reference_pitch.get();
                            let new_val = val - 1.0;
                            if new_val >= 400.0 {
                                reference_pitch.set(new_val);
                                LocalStorageSettings::set("peach.reference_pitch", &new_val.to_string());
                            }
                        })
                        on_increment=Callback::new(move |_| {
                            let val = reference_pitch.get();
                            let new_val = val + 1.0;
                            if new_val <= 460.0 {
                                reference_pitch.set(new_val);
                                LocalStorageSettings::set("peach.reference_pitch", &new_val.to_string());
                            }
                        })
                        decrement_disabled=Signal::derive(move || reference_pitch.get() <= 400.0)
                        increment_disabled=Signal::derive(move || reference_pitch.get() >= 460.0)
                    />
                </SettingsRowDynamic>
                <SettingsRow label="Tuning">
                    <select
                        class="text-sm text-right bg-transparent text-gray-500 dark:text-gray-400 border-none focus:outline-none focus:ring-0 cursor-pointer appearance-none pr-0"
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
                    <span class="ml-1 text-gray-400 dark:text-gray-500 text-sm">{"\u{203A}"}</span>
                </SettingsRow>
            </SettingsSection>
            <p class="px-4 mt-1 text-xs text-gray-500 dark:text-gray-400">
                "Select the tuning for intervals. Equal temperament divides the octave into 12 equal steps. Just intonation uses pure frequency ratios."
            </p>

            // Difficulty section (AC: 1, 7)
            <SettingsSection title="Difficulty">
                <div class="px-4 py-3 min-h-[44px]">
                    <div class="flex items-center justify-between mb-1">
                        <span class="text-sm text-gray-900 dark:text-gray-100">"Loudness Variation"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-gray-400 dark:text-gray-500">"Off"</span>
                        <input
                            type="range"
                            min="0"
                            max="100"
                            step="1"
                            class="flex-1 accent-indigo-600 dark:accent-indigo-400"
                            prop:value=move || vary_loudness_pct.get().to_string()
                            on:input=move |ev| {
                                if let Ok(val) = target_value(&ev).parse::<i32>() {
                                    vary_loudness_pct.set(val);
                                    let float_val = val as f64 / 100.0;
                                    LocalStorageSettings::set("peach.vary_loudness", &float_val.to_string());
                                }
                            }
                            aria-label="Loudness variation"
                        />
                        <span class="text-xs text-gray-400 dark:text-gray-500">"Max"</span>
                    </div>
                </div>
            </SettingsSection>

            // Data section (AC: 1, 8)
            <SettingsSection title="Data">
                {
                    let ie_status = RwSignal::new(ImportExportStatus::Idle);
                    let sr_announcement = RwSignal::new(String::new());
                    let import_dialog_ref = NodeRef::<leptos::html::Dialog>::new();
                    let import_data_signal: RwSignal<Option<csv_export_import::ParsedImportData>> = RwSignal::new(None);
                    let file_input_ref = NodeRef::<leptos::html::Input>::new();

                    let handle_export = {
                        move |_| {
                            ie_status.set(ImportExportStatus::Exporting);
                            spawn_local(async move {
                                let result = if let Some(store) = db_store.get_untracked() {
                                    csv_export_import::export_all_data(&store).await
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

                        input.set_value("");
                        spawn_local(async move {
                            let content = match csv_export_import::read_file_as_text(file).await {
                                Ok(c) => c,
                                Err(msg) => {
                                    sr_announcement.set(msg.clone());
                                    ie_status.set(ImportExportStatus::Error(msg));
                                    return;
                                }
                            };

                            match csv_export_import::parse_import_file(&content) {
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
                                    csv_export_import::import_replace(&store, data).await
                                } else {
                                    Err("Database not available".to_string())
                                };
                                match result {
                                    Ok(count) => {
                                        let msg = format!("{count} records imported");
                                        sr_announcement.set(msg.clone());
                                        ie_status.set(ImportExportStatus::ImportSuccess(msg));
                                        TimeoutFuture::new(1500).await;
                                        csv_export_import::reload_page();
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
                                    csv_export_import::import_merge(&store, data).await
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
                                        csv_export_import::reload_page();
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
                        // Export row
                        <button
                            on:click=handle_export
                            disabled=move || !matches!(ie_status.get(), ImportExportStatus::Idle)
                            class="w-full text-left px-4 py-3 min-h-[44px] text-sm text-indigo-600 dark:text-indigo-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
                        >
                            {move || match ie_status.get() {
                                ImportExportStatus::Exporting => "Exporting\u{2026}".to_string(),
                                ImportExportStatus::ExportSuccess => "Exported!".to_string(),
                                _ => "Export Training Data".to_string(),
                            }}
                        </button>
                        // Import row
                        <button
                            on:click=handle_import_click
                            disabled=move || !matches!(ie_status.get(), ImportExportStatus::Idle)
                            class="w-full text-left px-4 py-3 min-h-[44px] text-sm text-indigo-600 dark:text-indigo-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
                        >
                            {move || match ie_status.get() {
                                ImportExportStatus::Importing => "Importing\u{2026}".to_string(),
                                ImportExportStatus::ImportSuccess(ref msg) => msg.clone(),
                                _ => "Import Training Data".to_string(),
                            }}
                        </button>
                        // Delete row (destructive, red text)
                        <button
                            on:click=open_dialog
                            disabled=move || reset_status.get() == ResetStatus::Resetting
                            class="w-full text-left px-4 py-3 min-h-[44px] text-sm text-red-600 dark:text-red-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
                        >
                            {move || match reset_status.get() {
                                ResetStatus::Resetting => "Resetting\u{2026}",
                                ResetStatus::Success => "Data Reset",
                                ResetStatus::Error => "Reset Failed",
                                ResetStatus::Idle => "Delete All Training Data",
                            }}
                        </button>

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
                                    <p class="px-4 py-2 text-sm text-red-600 dark:text-red-400" role="alert">
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
                    }
                }
            </SettingsSection>

            // Reset confirmation dialog
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
