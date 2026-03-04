use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use domain::{PerceptualProfile, Trend, TrendAnalyzer};

use super::page_nav::PageNav;
use super::profile_visualization::ProfileVisualization;

/// Format an optional f64 value to 1 decimal place with " cents" suffix, or em dash.
fn format_cents(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{:.1} cents", v),
        None => "\u{2014}".to_string(),
    }
}

#[component]
pub fn ProfileView() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile context");
    let trend_analyzer: SendWrapper<Rc<RefCell<TrendAnalyzer>>> =
        use_context().expect("TrendAnalyzer context");
    let is_profile_loaded: RwSignal<bool> =
        use_context().expect("is_profile_loaded context");

    let profile_rc = profile.clone();
    let trend_rc = trend_analyzer.clone();

    view! {
        <div class="py-12">
            <PageNav current="profile" />
            <h1 class="text-2xl font-bold dark:text-white">"Profile"</h1>

            <ProfileVisualization />

            {move || {
                if !is_profile_loaded.get() {
                    return view! {
                        <p class="mt-6 text-gray-500 dark:text-gray-400">"Loading\u{2026}"</p>
                    }.into_any();
                }

                let prof = profile_rc.borrow();
                let trend = trend_rc.borrow();

                let cold_start = prof.overall_mean().is_none() && prof.matching_mean().is_none();
                let overall_mean = format_cents(prof.overall_mean());
                let overall_std_dev = format_cents(prof.overall_std_dev());
                let matching_mean_str = format_cents(prof.matching_mean());
                let matching_std_dev_str = format_cents(prof.matching_std_dev());
                let matching_count = prof.matching_count();
                let matching_count_str = if matching_count > 0 {
                    matching_count.to_string()
                } else {
                    "\u{2014}".to_string()
                };
                let trend_value = trend.trend();

                // Drop borrows before view! macro
                drop(prof);
                drop(trend);

                view! {
                    <div class="mt-6">
                        {if cold_start {
                            Some(view! {
                                <p class="text-gray-500 dark:text-gray-400">
                                    "Start training to build your profile."
                                </p>
                            }.into_any())
                        } else {
                            Some(view! {
                                <div class="space-y-8">
                                    <section aria-labelledby="comparison-heading">
                                        <h2 id="comparison-heading" class="text-lg font-semibold dark:text-white">
                                            "Comparison Training"
                                        </h2>
                                        <dl class="mt-3 space-y-4">
                                            <div>
                                                <dt class="text-sm text-gray-500 dark:text-gray-400">
                                                    "Mean Detection Threshold"
                                                </dt>
                                                <dd class="text-2xl font-bold dark:text-white">
                                                    {overall_mean}
                                                </dd>
                                            </div>
                                            <div>
                                                <dt class="text-sm text-gray-500 dark:text-gray-400">
                                                    "Standard Deviation"
                                                </dt>
                                                <dd class="text-2xl font-bold dark:text-white">
                                                    {overall_std_dev}
                                                </dd>
                                            </div>
                                            {match trend_value {
                                                Some(t) => {
                                                    let (label, color) = match t {
                                                        Trend::Improving => ("Improving", "text-green-600 dark:text-green-400"),
                                                        Trend::Stable => ("Stable", "text-gray-600 dark:text-gray-400"),
                                                        Trend::Declining => ("Declining", "text-amber-600 dark:text-amber-400"),
                                                    };
                                                    Some(view! {
                                                        <div>
                                                            <dt class="text-sm text-gray-500 dark:text-gray-400">
                                                                "Trend"
                                                            </dt>
                                                            <dd
                                                                class=format!("text-2xl font-bold {color}")
                                                                aria-label=label
                                                            >
                                                                {label}
                                                            </dd>
                                                        </div>
                                                    })
                                                }
                                                None => None,
                                            }}
                                        </dl>
                                    </section>

                                    <section aria-labelledby="matching-heading">
                                        <h2 id="matching-heading" class="text-lg font-semibold dark:text-white">
                                            "Pitch Matching"
                                        </h2>
                                        <dl class="mt-3 space-y-4">
                                            <div>
                                                <dt class="text-sm text-gray-500 dark:text-gray-400">
                                                    "Mean Absolute Error"
                                                </dt>
                                                <dd class="text-2xl font-bold dark:text-white">
                                                    {matching_mean_str}
                                                </dd>
                                            </div>
                                            <div>
                                                <dt class="text-sm text-gray-500 dark:text-gray-400">
                                                    "Standard Deviation"
                                                </dt>
                                                <dd class="text-2xl font-bold dark:text-white">
                                                    {matching_std_dev_str}
                                                </dd>
                                            </div>
                                            <div>
                                                <dt class="text-sm text-gray-500 dark:text-gray-400">
                                                    "Sample Count"
                                                </dt>
                                                <dd class="text-2xl font-bold dark:text-white">
                                                    {matching_count_str}
                                                </dd>
                                            </div>
                                        </dl>
                                    </section>
                                </div>
                            }.into_any())
                        }}
                    </div>
                }.into_any()
            }}

        </div>
    }
}
