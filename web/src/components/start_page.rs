use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use super::page_nav::PageNav;
use super::ProfilePreview;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::interval_codes::encode_intervals;
use domain::Interval;

fn navigate_with_intervals(navigate: &impl Fn(&str, leptos_router::NavigateOptions), path: &str) {
    let intervals = LocalStorageSettings::get_selected_intervals();
    let has_non_prime = intervals
        .iter()
        .any(|di| di.interval != Interval::Prime);
    let code = if has_non_prime {
        encode_intervals(&intervals)
    } else {
        "P1".to_string()
    };
    navigate(
        &format!("{path}?intervals={code}"),
        Default::default(),
    );
}

#[component]
pub fn StartPage() -> impl IntoView {
    let navigate = use_navigate();
    let on_comparison = {
        let navigate = navigate.clone();
        move |_| {
            navigate("/training/comparison", Default::default());
        }
    };
    let on_pitch_matching = {
        let navigate = navigate.clone();
        move |_| {
            navigate("/training/pitch-matching", Default::default());
        }
    };
    let on_interval_comparison = {
        let navigate = navigate.clone();
        move |_| {
            navigate_with_intervals(&navigate, "/training/comparison");
        }
    };
    let on_interval_pitch_matching = {
        let navigate = navigate.clone();
        move |_| {
            navigate_with_intervals(&navigate, "/training/pitch-matching");
        }
    };

    view! {
        <div class="flex flex-col items-center gap-6 py-12">
            <PageNav current="start" />
            <h1 class="sr-only">"Peach"</h1>

            <ProfilePreview />

            <nav aria-label="Training modes" class="flex w-full flex-col items-center gap-6">
                <button
                    on:click=on_comparison
                    class="block w-full min-h-11 rounded-lg bg-indigo-600 px-6 py-4 text-center text-lg font-semibold text-white shadow-md hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-400"
                >
                    "Comparison"
                </button>

                <button
                    on:click=on_pitch_matching
                    class="block w-full min-h-11 rounded-lg bg-gray-200 px-6 py-3 text-center text-lg font-medium text-gray-800 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                >
                    "Pitch Matching"
                </button>

                <hr class="w-full border-gray-300 dark:border-gray-600" />

                <button
                    on:click=on_interval_comparison
                    class="block w-full min-h-11 rounded-lg bg-gray-200 px-6 py-3 text-center text-lg font-medium text-gray-800 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                >
                    "Interval Comparison"
                </button>

                <button
                    on:click=on_interval_pitch_matching
                    class="block w-full min-h-11 rounded-lg bg-gray-200 px-6 py-3 text-center text-lg font-medium text-gray-800 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                >
                    "Interval Pitch Matching"
                </button>
            </nav>

        </div>
    }
}
