use leptos::prelude::*;

use super::page_nav::PageNav;
use super::progress_sparkline::ProgressSparkline;
use crate::adapters::localstorage_settings::LocalStorageSettings;
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
) -> impl IntoView {
    view! {
        <a
            href=href
            aria-label=aria_label
            class="flex w-full items-center gap-3 rounded-xl bg-gray-100 px-4 py-3 min-h-11 text-lg font-medium text-gray-800 no-underline transition-opacity duration-150 ease-in-out active:opacity-70 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700 dark:focus:ring-offset-gray-900"
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

    view! {
        <div class="flex flex-col items-center gap-6 py-12">
            <PageNav current="start" />
            <h1 class="sr-only">"Peach"</h1>

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
                        />
                        <TrainingCard
                            label="Tune & Match"
                            icon="\u{1F3AF}"
                            href="/training/pitch-matching".to_string()
                            aria_label="Tune and Match, Single Notes"
                            mode=TrainingMode::UnisonMatching
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
                        />
                        <TrainingCard
                            label="Tune & Match"
                            icon="\u{1F3AF}"
                            href=interval_pitch_matching_href
                            aria_label="Tune and Match, Intervals"
                            mode=TrainingMode::IntervalMatching
                        />
                    </div>
                </section>
            </nav>
        </div>
    }
}
