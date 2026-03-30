use std::cell::RefCell;
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use leptos_fluent::{move_tr, tr};
use send_wrapper::SendWrapper;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::audio_context::AudioContextManager;
use crate::adapters::audio_soundfont::SF2Preset;
use crate::adapters::csv_export_import;
use crate::adapters::csv_export_import::{ImportExportStatus, ResetStatus};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::adapters::sound_preview::SoundPreview;
use crate::app::{AudioNeedsGesture, SoundFontLoadStatus, WorkletAssets, WorkletConnecting};
use domain::ports::UserSettings;
use domain::types::{DetunedMIDINote, Frequency, MIDINote, SoundSourceID, StepPosition, TempoBPM};
use domain::{DirectedInterval, Direction, Interval, PerceptualProfile, TuningSystem};

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
fn SettingsSection(#[prop(into)] title: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="mt-6">
            <h2 class="px-4 mb-1 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                {move || title.get()}
            </h2>
            <div class="rounded-xl bg-gray-100 dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {children()}
            </div>
        </div>
    }
}

/// A single row inside a SettingsSection card.
#[component]
fn SettingsRow(#[prop(into)] label: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between px-4 py-3 min-h-[44px]">
            <span class="text-sm text-gray-900 dark:text-gray-100">{move || label.get()}</span>
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

/// Starts an auto-repeat loop: fires `callback` after `delay_ms`, then repeats with
/// accelerating speed (400→200→100→50ms). Stops when `holding` becomes false or
/// `disabled` becomes true.
fn start_auto_repeat(
    holding: RwSignal<bool>,
    disabled: Signal<bool>,
    callback: Callback<()>,
    delay_ms: u32,
) {
    spawn_local(async move {
        TimeoutFuture::new(delay_ms).await;
        if !holding.get_untracked() || disabled.get_untracked() {
            return;
        }
        callback.run(());
        let next_delay = match delay_ms {
            d if d >= 400 => 200,
            200 => 100,
            _ => 50,
        };
        start_auto_repeat(holding, disabled, callback, next_delay);
    });
}

/// iOS-style +/- stepper control with press-and-hold auto-repeat.
#[component]
fn Stepper(
    #[prop(into)] label: Signal<String>,
    on_decrement: Callback<()>,
    on_increment: Callback<()>,
    #[prop(into)] decrement_disabled: Signal<bool>,
    #[prop(into)] increment_disabled: Signal<bool>,
) -> impl IntoView {
    let dec_label = move || format!("Decrease {}", label.get());
    let inc_label = move || format!("Increase {}", label.get());

    let dec_holding = RwSignal::new(false);
    let inc_holding = RwSignal::new(false);

    on_cleanup(move || {
        dec_holding.set(false);
        inc_holding.set(false);
    });

    let on_dec_down = move |_: web_sys::PointerEvent| {
        if decrement_disabled.get_untracked() {
            return;
        }
        dec_holding.set(true);
        on_decrement.run(());
        start_auto_repeat(dec_holding, decrement_disabled, on_decrement, 400);
    };
    let on_dec_up = move |_: web_sys::PointerEvent| {
        dec_holding.set(false);
    };

    let on_inc_down = move |_: web_sys::PointerEvent| {
        if increment_disabled.get_untracked() {
            return;
        }
        inc_holding.set(true);
        on_increment.run(());
        start_auto_repeat(inc_holding, increment_disabled, on_increment, 400);
    };
    let on_inc_up = move |_: web_sys::PointerEvent| {
        inc_holding.set(false);
    };

    view! {
        <div class="inline-flex items-center rounded-lg bg-gray-200 dark:bg-gray-700" role="group">
            <button
                on:pointerdown=on_dec_down
                on:pointerup=on_dec_up
                on:pointerleave=on_dec_up
                on:pointercancel=on_dec_up
                disabled=decrement_disabled
                aria-label=dec_label
                class="min-h-[34px] min-w-[40px] flex items-center justify-center rounded-l-lg text-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-30 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-400 select-none touch-none"
            >
                "\u{2212}"
            </button>
            <div class="w-px h-5 bg-gray-300 dark:bg-gray-600"></div>
            <button
                on:pointerdown=on_inc_down
                on:pointerup=on_inc_up
                on:pointerleave=on_inc_up
                on:pointercancel=on_inc_up
                disabled=increment_disabled
                aria-label=inc_label
                class="min-h-[34px] min-w-[40px] flex items-center justify-center rounded-r-lg text-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-30 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-400 select-none touch-none"
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
    let note_gap = RwSignal::new(settings.note_gap().as_secs_f64());
    let note_gap_label = Signal::derive(
        move || tr!("note-gap-label", {"value" => format!("{:.1}", note_gap.get())}),
    );
    let tuning_system = RwSignal::new(
        match settings.tuning_system() {
            TuningSystem::EqualTemperament => "equalTemperament",
            TuningSystem::JustIntonation => "justIntonation",
        }
        .to_string(),
    );
    let sound_source = RwSignal::new(
        LocalStorageSettings::get_string("peach.sound_source")
            .unwrap_or_else(|| SoundSourceID::default().raw_value().to_string()),
    );
    let sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage> =
        use_context().expect("sf2_presets not provided");
    let sf2_load_status: RwSignal<SoundFontLoadStatus> =
        use_context().expect("sf2_load_status not provided");
    let selected_intervals = RwSignal::new(LocalStorageSettings::get_selected_intervals());

    // Rhythm settings
    let tempo_bpm = RwSignal::new(settings.tempo_bpm().bpm());
    let tempo_label =
        Signal::derive(move || tr!("tempo-label", {"value" => tempo_bpm.get().to_string()}));
    let gap_positions = RwSignal::new(LocalStorageSettings::get_enabled_gap_positions_static());

    // Reset training data
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile not provided");
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

        spawn_local(async move {
            let db_result = if let Some(store) = db_store.get_untracked() {
                store.delete_all().await
            } else {
                Ok(())
            };

            match db_result {
                Ok(()) => {
                    profile.borrow_mut().reset_all();
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
    let min_label = Signal::derive(move || tr!("lowest-note", {"note" => min_note_name.get()}));
    let max_label = Signal::derive(move || tr!("highest-note", {"note" => max_note_name.get()}));

    // Derived signals for sound settings display
    let duration_label = Signal::derive(
        move || tr!("duration-label", {"value" => format!("{:.1}", note_duration.get())}),
    );
    let pitch_label = Signal::derive(
        move || tr!("concert-pitch-label", {"value" => (reference_pitch.get().round() as i32).to_string()}),
    );

    view! {
        <div class="pt-4 pb-12">
            <NavBar title=move_tr!("settings-title") show_back=true>
                <NavIconButton label=Signal::derive(move || tr!("nav-help")) icon="?".to_string() on_click=Callback::new(move |_| is_help_open.set(true)) circled=true />
            </NavBar>
            <HelpModal title=move_tr!("settings-help-title") sections=SETTINGS_HELP is_open=is_help_open />

            // Language section
            <SettingsSection title=move_tr!("language-label")>
                <div class="px-4 py-3">
                    {move || {
                        let i18n = expect_context::<leptos_fluent::I18n>();
                        let languages = i18n.languages;
                        view! {
                            <select
                                class="w-full rounded-lg bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-400"
                                on:change=move |ev| {
                                    let val = target_value(&ev);
                                    if let Some(lang) = languages.iter().find(|l| val.as_str() == l.id.to_string().as_str()) {
                                        i18n.language.set(lang);
                                    }
                                }
                            >
                                {languages.iter().map(|lang| {
                                    let id = lang.id.to_string();
                                    let selected = i18n.language.get() == *lang;
                                    view! {
                                        <option value=id.clone() selected=selected>
                                            {lang.name}
                                        </option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                        }
                    }}
                </div>
            </SettingsSection>

            // Pitch Range section (AC: 1, 3)
            <SettingsSection title=move_tr!("pitch-range")>
                <SettingsRowDynamic label=min_label>
                    <Stepper
                        label=move_tr!("pitch-range")
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
                        label=move_tr!("pitch-range")
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
            <SettingsSection title=move_tr!("intervals-section")>
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
                                                    aria-label=move || format!("{} {}", interval.short_label(), tr!("ascending"))
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
                                                    aria-label=move || format!("{} {}", interval.short_label(), tr!("descending"))
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
                {move_tr!("intervals-hint")}
            </p>

            // Sound section (AC: 1, 5, 6)
            <SettingsSection title=move_tr!("sound-section")>
                <SettingsRow label=move_tr!("sound-label")>
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
                                                    view! { <option value={"oscillator:sine".to_string()}>{tr!("sine-oscillator")}</option> }
                                                ];
                                                let mut presets = sf2_presets.get();
                                                // Filter out percussion banks (≥120) — not useful as melodic sound sources
                                                presets.retain(|p| p.bank < 120);
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
                                            aria-label=move || if preview_playing.get() { tr!("stop-preview") } else { tr!("preview-sound") }
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
                        label=move_tr!("sound-section")
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
                        label=move_tr!("sound-section")
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
                <SettingsRow label=move_tr!("tuning-label")>
                    <select
                        class="text-sm text-right bg-transparent text-gray-500 dark:text-gray-400 border-none focus:outline-none focus:ring-0 cursor-pointer appearance-none pr-0"
                        prop:value=move || tuning_system.get()
                        on:change=move |ev| {
                            let val = target_value(&ev);
                            LocalStorageSettings::set("peach.tuning_system", &val);
                            tuning_system.set(val);
                        }
                    >
                        <option value="equalTemperament">{move_tr!("equal-temperament")}</option>
                        <option value="justIntonation">{move_tr!("just-intonation")}</option>
                    </select>
                    <span class="ml-1 text-gray-400 dark:text-gray-500 text-sm">{"\u{203A}"}</span>
                </SettingsRow>
            </SettingsSection>
            <p class="px-4 mt-1 text-xs text-gray-500 dark:text-gray-400">
                {move_tr!("tuning-hint")}
            </p>

            // Difficulty section (AC: 1, 7)
            <SettingsSection title=move_tr!("difficulty-section")>
                <div class="px-4 py-3 min-h-[44px]">
                    <div class="flex items-center justify-between mb-1">
                        <span class="text-sm text-gray-900 dark:text-gray-100">{move_tr!("loudness-variation")}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-gray-400 dark:text-gray-500">{move_tr!("off")}</span>
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
                            aria-label=move || tr!("loudness-variation-aria")
                        />
                        <span class="text-xs text-gray-400 dark:text-gray-500">{move_tr!("max")}</span>
                    </div>
                </div>
                <div class="border-t border-gray-200 dark:border-gray-700"></div>
                <SettingsRowDynamic label=note_gap_label>
                    <Stepper
                        label=move_tr!("note-gap-aria")
                        on_decrement=Callback::new(move |_| {
                            let val = note_gap.get();
                            let new_val = ((val * 10.0 - 1.0).round() / 10.0).max(0.0);
                            note_gap.set(new_val);
                            LocalStorageSettings::set("peach.note_gap", &format!("{new_val:.1}"));
                        })
                        on_increment=Callback::new(move |_| {
                            let val = note_gap.get();
                            let new_val = ((val * 10.0 + 1.0).round() / 10.0).min(5.0);
                            note_gap.set(new_val);
                            LocalStorageSettings::set("peach.note_gap", &format!("{new_val:.1}"));
                        })
                        decrement_disabled=Signal::derive(move || note_gap.get() <= 0.0)
                        increment_disabled=Signal::derive(move || note_gap.get() >= 5.0)
                    />
                </SettingsRowDynamic>
            </SettingsSection>

            // Rhythm section
            <SettingsSection title=move_tr!("rhythm-section")>
                <SettingsRowDynamic label=tempo_label>
                    <Stepper
                        label=move_tr!("tempo-aria")
                        on_decrement=Callback::new(move |_| {
                            let val = tempo_bpm.get();
                            if val > TempoBPM::MIN {
                                let new_val = val - 1;
                                tempo_bpm.set(new_val);
                                LocalStorageSettings::set_tempo_bpm(TempoBPM::new(new_val));
                            }
                        })
                        on_increment=Callback::new(move |_| {
                            let val = tempo_bpm.get();
                            if val < TempoBPM::MAX {
                                let new_val = val + 1;
                                tempo_bpm.set(new_val);
                                LocalStorageSettings::set_tempo_bpm(TempoBPM::new(new_val));
                            }
                        })
                        decrement_disabled=Signal::derive(move || tempo_bpm.get() <= TempoBPM::MIN)
                        increment_disabled=Signal::derive(move || tempo_bpm.get() >= TempoBPM::MAX)
                    />
                </SettingsRowDynamic>
                <div class="px-4 py-3">
                    <div class="flex items-center justify-between mb-2">
                        <span class="text-sm text-gray-900 dark:text-gray-100">{move_tr!("gap-positions-label")}</span>
                    </div>
                    <div class="flex gap-2">
                        {StepPosition::ALL.iter().map(|&pos| {
                            let label = match pos {
                                StepPosition::First => Signal::derive(move || tr!("gap-position-beat")),
                                StepPosition::Second => Signal::derive(move || tr!("gap-position-e")),
                                StepPosition::Third => Signal::derive(move || tr!("gap-position-and")),
                                StepPosition::Fourth => Signal::derive(move || tr!("gap-position-a")),
                            };
                            view! {
                                <button
                                    on:click=move |_| {
                                        gap_positions.update(|set| {
                                            if set.contains(&pos) {
                                                if set.len() > 1 {
                                                    set.remove(&pos);
                                                }
                                            } else {
                                                set.insert(pos);
                                            }
                                        });
                                        LocalStorageSettings::set_enabled_gap_positions(&gap_positions.get());
                                    }
                                    disabled=move || {
                                        let set = gap_positions.get();
                                        set.len() == 1 && set.contains(&pos)
                                    }
                                    aria-pressed=move || gap_positions.get().contains(&pos).to_string()
                                    aria-label=move || label.get()
                                    class=move || {
                                        if gap_positions.get().contains(&pos) {
                                            "flex-1 min-h-[34px] rounded-lg text-sm font-semibold bg-indigo-600 text-white dark:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400"
                                        } else {
                                            "flex-1 min-h-[34px] rounded-lg text-sm font-semibold bg-gray-200 text-gray-500 dark:bg-gray-700 dark:text-gray-400 hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-30 focus:outline-none focus:ring-2 focus:ring-indigo-400"
                                        }
                                    }
                                >
                                    {move || label.get()}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                <p class="px-4 mt-1 pb-2 text-xs text-gray-500 dark:text-gray-400">
                    {move_tr!("gap-positions-hint")}
                </p>
            </SettingsSection>

            // Data section (AC: 1, 8)
            <SettingsSection title=move_tr!("data-section")>
                {
                    let ie_status = RwSignal::new(ImportExportStatus::Idle);
                    let sr_announcement = RwSignal::new(String::new());
                    let import_dialog_ref = NodeRef::<leptos::html::Dialog>::new();
                    let import_data_signal: RwSignal<Option<csv_export_import::ParsedImportData>> = RwSignal::new(None);
                    let file_input_ref = NodeRef::<leptos::html::Input>::new();

                    // Sentinel placeholders for i18n template strings.
                    // tr!() is called inside each event handler (before spawn_local)
                    // so it reads the current language. spawn_local does not preserve
                    // the Leptos reactive owner, so tr!() must not be called after
                    // the async boundary.
                    const PH1: &str = "\x00";
                    const PH2: &str = "\x01";

                    let handle_export = {
                        move |_| {
                            let msg_db_unavailable = tr!("database-not-available");
                            let msg_exported = tr!("data-exported");
                            let msg_export_failed_tpl = tr!("export-failed", {"error" => PH1});
                            ie_status.set(ImportExportStatus::Exporting);
                            spawn_local(async move {
                                let result = if let Some(store) = db_store.get_untracked() {
                                    csv_export_import::export_all_data(&store).await
                                } else {
                                    Err(msg_db_unavailable)
                                };
                                match result {
                                    Ok(()) => {
                                        ie_status.set(ImportExportStatus::ExportSuccess);
                                        sr_announcement.set(msg_exported);
                                        TimeoutFuture::new(2000).await;
                                        ie_status.set(ImportExportStatus::Idle);
                                    }
                                    Err(e) => {
                                        let msg = msg_export_failed_tpl.replace(PH1, &e);
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

                    let handle_file_selected = {
                        move |_| {
                            let input = match file_input_ref.get() {
                                Some(i) => i,
                                None => return,
                            };
                            let file = match input.files().and_then(|fl| fl.get(0)) {
                                Some(f) => f,
                                None => return,
                            };

                            if file.size() > 10_000_000.0 {
                                let msg = tr!("file-too-large");
                                sr_announcement.set(msg.clone());
                                ie_status.set(ImportExportStatus::Error(msg));
                                spawn_local(async move {
                                    TimeoutFuture::new(3000).await;
                                    ie_status.set(ImportExportStatus::Idle);
                                });
                                input.set_value("");
                                return;
                            }

                            let msg_import_failed_tpl = tr!("import-failed", {"error" => PH1});
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
                                        let msg = msg_import_failed_tpl.replace(PH1, &e);
                                        sr_announcement.set(msg.clone());
                                        ie_status.set(ImportExportStatus::Error(msg));
                                        spawn_local(async move {
                                            TimeoutFuture::new(3000).await;
                                            ie_status.set(ImportExportStatus::Idle);
                                        });
                                    }
                                }
                            });
                        }
                    };

                    let handle_import_replace = {
                        move |_| {
                            let msg_db_unavailable = tr!("database-not-available");
                            let msg_records_imported_tpl = tr!("records-imported", {"count" => PH1});
                            let msg_import_failed_tpl = tr!("import-failed", {"error" => PH1});
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
                                        Err(msg_db_unavailable)
                                    };
                                    match result {
                                        Ok(count) => {
                                            let msg = msg_records_imported_tpl.replace(PH1, &count.to_string());
                                            sr_announcement.set(msg.clone());
                                            ie_status.set(ImportExportStatus::ImportSuccess(msg));
                                            TimeoutFuture::new(1500).await;
                                            csv_export_import::reload_page();
                                        }
                                        Err(e) => {
                                            let msg = msg_import_failed_tpl.replace(PH1, &e);
                                            sr_announcement.set(msg.clone());
                                            ie_status.set(ImportExportStatus::Error(msg));
                                            TimeoutFuture::new(3000).await;
                                            ie_status.set(ImportExportStatus::Idle);
                                        }
                                    }
                                }
                            });
                        }
                    };

                    let handle_import_merge = {
                        move |_| {
                            let msg_db_unavailable = tr!("database-not-available");
                            let msg_records_merged_tpl = tr!("records-merged", {"imported" => PH1, "skipped" => PH2});
                            let msg_import_failed_tpl = tr!("import-failed", {"error" => PH1});
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
                                        Err(msg_db_unavailable)
                                    };
                                    match result {
                                        Ok(r) => {
                                            let imported = r.discrimination_imported + r.pitch_matching_imported + r.rhythm_offset_imported + r.continuous_rhythm_imported;
                                            let skipped = r.discrimination_skipped + r.pitch_matching_skipped + r.rhythm_offset_skipped + r.continuous_rhythm_skipped;
                                            let msg = msg_records_merged_tpl
                                                .replace(PH1, &imported.to_string())
                                                .replace(PH2, &skipped.to_string());
                                            sr_announcement.set(msg.clone());
                                            ie_status.set(ImportExportStatus::ImportSuccess(msg));
                                            TimeoutFuture::new(1500).await;
                                            csv_export_import::reload_page();
                                        }
                                        Err(e) => {
                                            let msg = msg_import_failed_tpl.replace(PH1, &e);
                                            sr_announcement.set(msg.clone());
                                            ie_status.set(ImportExportStatus::Error(msg));
                                            TimeoutFuture::new(3000).await;
                                            ie_status.set(ImportExportStatus::Idle);
                                        }
                                    }
                                }
                            });
                        }
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
                                ImportExportStatus::Exporting => tr!("exporting"),
                                ImportExportStatus::ExportSuccess => tr!("exported"),
                                _ => tr!("export-training-data"),
                            }}
                        </button>
                        // Import row
                        <button
                            on:click=handle_import_click
                            disabled=move || !matches!(ie_status.get(), ImportExportStatus::Idle)
                            class="w-full text-left px-4 py-3 min-h-[44px] text-sm text-indigo-600 dark:text-indigo-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
                        >
                            {move || match ie_status.get() {
                                ImportExportStatus::Importing => tr!("importing"),
                                ImportExportStatus::ImportSuccess(ref msg) => msg.clone(),
                                _ => tr!("import-training-data"),
                            }}
                        </button>
                        // Delete row (destructive, red text)
                        <button
                            on:click=open_dialog
                            disabled=move || reset_status.get() == ResetStatus::Resetting
                            class="w-full text-left px-4 py-3 min-h-[44px] text-sm text-red-600 dark:text-red-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 transition-colors"
                        >
                            {move || match reset_status.get() {
                                ResetStatus::Resetting => tr!("resetting"),
                                ResetStatus::Success => tr!("data-reset"),
                                ResetStatus::Error => tr!("reset-failed"),
                                ResetStatus::Idle => tr!("delete-all-training-data"),
                            }}
                        </button>

                        // Hidden file input for import
                        <input
                            node_ref=file_input_ref
                            type="file"
                            accept=".csv"
                            on:change=handle_file_selected
                            class="sr-only"
                            aria-label=move || tr!("select-csv")
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
                            <h2 id="import-dialog-title" class="text-lg font-bold">{move_tr!("import-dialog-title")}</h2>
                            <p class="mt-3 text-sm text-gray-600 dark:text-gray-300">
                                {move || {
                                    if let Some(ref data) = import_data_signal.get() {
                                        let warnings_text = if data.warnings.is_empty() {
                                            String::new()
                                        } else {
                                            tr!("import-dialog-warnings", {"count" => data.warnings.len().to_string()})
                                        };
                                        tr!("import-dialog-found", {
                                            "discriminations" => data.pitch_discriminations.len().to_string(),
                                            "matchings" => data.pitch_matchings.len().to_string(),
                                            "warnings" => warnings_text,
                                        })
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
                                    {move_tr!("replace-all-data")}
                                </button>
                                <button
                                    on:click=handle_import_merge
                                    class="min-h-[44px] rounded-lg bg-indigo-600 px-4 py-2 font-semibold text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:bg-indigo-700 dark:hover:bg-indigo-800 dark:ring-offset-gray-900"
                                >
                                    {move_tr!("merge-with-existing")}
                                </button>
                                <button
                                    on:click=handle_import_cancel
                                    class="min-h-[44px] rounded-lg bg-gray-200 px-4 py-2 font-semibold text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600 dark:ring-offset-gray-900"
                                >
                                    {move_tr!("cancel")}
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
                <h2 id="reset-dialog-title" class="text-lg font-bold">{move_tr!("reset-dialog-title")}</h2>
                <p class="mt-3 text-sm text-gray-600 dark:text-gray-300">
                    {move_tr!("reset-dialog-message")}
                </p>
                <div class="mt-6 flex gap-3 justify-end">
                    <button
                        on:click=handle_cancel
                        disabled=move || reset_status.get() == ResetStatus::Resetting
                        class="min-h-[44px] rounded-lg bg-gray-200 px-4 py-2 font-semibold text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600 dark:ring-offset-gray-900 disabled:opacity-50"
                    >
                        {move_tr!("cancel")}
                    </button>
                    <button
                        on:click=handle_confirm
                        disabled=move || reset_status.get() == ResetStatus::Resetting
                        class="min-h-[44px] rounded-lg bg-red-600 px-4 py-2 font-semibold text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 dark:bg-red-700 dark:hover:bg-red-800 dark:ring-offset-gray-900 disabled:opacity-50"
                    >
                        {move_tr!("delete-all-data")}
                    </button>
                </div>
            </dialog>

        </div>
    }
}
