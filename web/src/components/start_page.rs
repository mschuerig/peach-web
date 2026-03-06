use leptos::prelude::*;

use super::nav_bar::{NavBar, NavIconButton};
use super::progress_sparkline::ProgressSparkline;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::app::SoundFontLoadStatus;
use crate::interval_codes::encode_intervals;
use domain::{Interval, TrainingMode};

fn interval_href(path: &str) -> String {
    let intervals = LocalStorageSettings::get_selected_intervals();
    let has_non_prime = intervals
        .iter()
        .any(|di| di.interval != Interval::Prime);
    let code = if has_non_prime {
        encode_intervals(&intervals)
    } else {
        "P1".to_string()
    };
    format!("{path}?intervals={code}")
}

#[component]
fn TrainingCard(
    label: &'static str,
    icon: &'static str,
    href: String,
    aria_label: &'static str,
    mode: TrainingMode,
    #[prop(into)] disabled: Signal<bool>,
) -> impl IntoView {
    let base_class = "flex w-full items-center gap-3 rounded-xl px-4 py-3 min-h-11 text-lg font-medium no-underline transition-opacity duration-150 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900";
    let enabled_class = " bg-gray-100 text-gray-800 active:opacity-70 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700";
    let disabled_class = " bg-gray-100 text-gray-400 cursor-not-allowed opacity-60 dark:bg-gray-800 dark:text-gray-500";

    view! {
        <a
            href=move || if disabled.get() { None } else { Some(href.clone()) }
            aria-label=aria_label
            aria-disabled=move || if disabled.get() { "true" } else { "false" }
            tabindex=move || if disabled.get() { "-1" } else { "0" }
            class=move || {
                let mut c = base_class.to_string();
                if disabled.get() {
                    c.push_str(disabled_class);
                } else {
                    c.push_str(enabled_class);
                }
                c
            }
            on:click=move |ev| {
                if disabled.get() {
                    ev.prevent_default();
                }
            }
        >
            <span class="text-xl" aria-hidden="true">{icon}</span>
            <div class="flex flex-col">
                <span>{label}</span>
                <ProgressSparkline mode=mode />
            </div>
        </a>
    }
}

#[component]
pub fn StartPage() -> impl IntoView {
    let interval_comparison_href = interval_href("/training/comparison");
    let interval_pitch_matching_href = interval_href("/training/pitch-matching");

    let sf2_status: RwSignal<SoundFontLoadStatus> = use_context()
        .expect("SoundFontLoadStatus signal must be provided");

    let can_start_training = Memo::new(move |_| {
        matches!(
            sf2_status.get(),
            SoundFontLoadStatus::NotNeeded | SoundFontLoadStatus::Ready | SoundFontLoadStatus::Failed(_)
        )
    });

    let disabled = Signal::derive(move || !can_start_training.get());

    // Auto-dismiss failure notification after 5 seconds
    let sf2_error_dismissed = RwSignal::new(false);
    Effect::new(move || {
        if matches!(sf2_status.get(), SoundFontLoadStatus::Failed(_)) && !sf2_error_dismissed.get() {
            let signal = sf2_error_dismissed;
            gloo_timers::callback::Timeout::new(5000, move || {
                signal.set(true);
            })
            .forget();
        }
    });

    view! {
        <div class="flex flex-col items-center gap-6 pt-4 pb-12">
            <NavBar
                title="Peach"
                left_content=ViewFn::from(move || view! {
                    <NavIconButton label="Info".to_string() icon="\u{24D8}".to_string() href="/info".to_string() />
                })
            >
                <NavIconButton label="Profile".to_string() icon="\u{1F4CA}".to_string() href="/profile".to_string() />
                <NavIconButton label="Settings".to_string() icon="\u{2699}\u{FE0F}".to_string() href="/settings".to_string() />
            </NavBar>

            // Loading indicator
            {move || {
                if matches!(sf2_status.get(), SoundFontLoadStatus::Fetching) {
                    view! {
                        <div
                            role="status"
                            aria-live="polite"
                            class="w-full rounded-lg bg-indigo-50 border border-indigo-200 px-4 py-3 text-center text-indigo-700 dark:bg-indigo-900/30 dark:border-indigo-700 dark:text-indigo-300"
                        >
                            <span class="inline-block animate-pulse">"Loading sounds\u{2026}"</span>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            <nav aria-label="Training modes" class="flex w-full flex-col gap-7 md:flex-row md:gap-8">
                // Single Notes section
                <section class="flex-1">
                    <h2 class="mb-2.5 text-sm font-medium text-gray-500 dark:text-gray-400">"Single Notes"</h2>
                    <div class="flex flex-col gap-2.5">
                        <TrainingCard
                            label="Hear & Compare"
                            icon="\u{1F442}"
                            href="/training/comparison".to_string()
                            aria_label="Hear and Compare, Single Notes"
                            mode=TrainingMode::UnisonPitchComparison
                            disabled=disabled
                        />
                        <TrainingCard
                            label="Tune & Match"
                            icon="\u{1F3AF}"
                            href="/training/pitch-matching".to_string()
                            aria_label="Tune and Match, Single Notes"
                            mode=TrainingMode::UnisonMatching
                            disabled=disabled
                        />
                    </div>
                </section>

                // Intervals section
                <section class="flex-1">
                    <h2 class="mb-2.5 text-sm font-medium text-gray-500 dark:text-gray-400">"Intervals"</h2>
                    <div class="flex flex-col gap-2.5">
                        <TrainingCard
                            label="Hear & Compare"
                            icon="\u{1F442}"
                            href=interval_comparison_href
                            aria_label="Hear and Compare, Intervals"
                            mode=TrainingMode::IntervalPitchComparison
                            disabled=disabled
                        />
                        <TrainingCard
                            label="Tune & Match"
                            icon="\u{1F3AF}"
                            href=interval_pitch_matching_href
                            aria_label="Tune and Match, Intervals"
                            mode=TrainingMode::IntervalMatching
                            disabled=disabled
                        />
                    </div>
                </section>
            </nav>

            // SoundFont load failure notification — non-blocking, auto-dismissing
            {move || {
                if let SoundFontLoadStatus::Failed(_) = sf2_status.get()
                    && !sf2_error_dismissed.get()
                {
                    return view! {
                        <div
                            class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-amber-100 border border-amber-400 text-amber-800 px-4 py-2 rounded-lg shadow-md text-sm dark:bg-amber-900 dark:border-amber-700 dark:text-amber-200"
                            role="alert"
                        >
                            "Selected sound could not be loaded. Using default sound."
                        </div>
                    }.into_any();
                }
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}
