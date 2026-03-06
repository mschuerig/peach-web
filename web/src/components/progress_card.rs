use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use domain::{ProgressTimeline, TrainingMode, TrainingModeState, Trend};

use super::progress_chart::ProgressChart;

fn format_ewma(value: f64) -> String {
    format!("{value:.1}")
}

fn format_stddev(buckets: &[domain::TimeBucket]) -> String {
    if buckets.is_empty() {
        return String::new();
    }
    let last_stddev = buckets.last().map(|b| b.stddev).unwrap_or(0.0);
    format!("\u{00B1}{last_stddev:.1}")
}

fn trend_arrow(trend: Option<Trend>) -> (&'static str, &'static str) {
    match trend {
        Some(Trend::Improving) => ("\u{2198}", "text-green-600 dark:text-green-400"),
        Some(Trend::Stable) => ("\u{2192}", "text-gray-500 dark:text-gray-400"),
        Some(Trend::Declining) => ("\u{2197}", "text-amber-600 dark:text-amber-400"),
        None => ("", ""),
    }
}

fn trend_text(trend: Option<Trend>) -> &'static str {
    match trend {
        Some(Trend::Improving) => "improving",
        Some(Trend::Stable) => "stable",
        Some(Trend::Declining) => "declining",
        None => "",
    }
}

#[component]
pub fn ProgressCard(mode: TrainingMode) -> impl IntoView {
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline context");
    let is_profile_loaded: RwSignal<bool> = use_context().expect("is_profile_loaded context");

    let ptl = progress_timeline.clone();
    let config = mode.config();

    view! {
        {move || {
            if !is_profile_loaded.get() {
                return view! { <div /> }.into_any();
            }

            let tl = ptl.borrow();
            if tl.state(mode) == TrainingModeState::NoData {
                return view! { <div /> }.into_any();
            }

            let buckets = tl.buckets(mode);
            let ewma = tl.current_ewma(mode);
            let trend = tl.trend(mode);
            drop(tl);

            let ewma_str = ewma.map(format_ewma).unwrap_or_default();
            let stddev_str = format_stddev(&buckets);
            let (arrow, arrow_color) = trend_arrow(trend);
            let trend_label = trend_text(trend);
            let display_name = config.display_name;

            let card_aria = format!(
                "Progress for {display_name}: {ewma_str} cents, {trend_label}"
            );
            let value_aria = format!("{ewma_str} cents, trend {trend_label}");

            view! {
                <div
                    class="rounded-xl bg-gray-100 p-4 dark:bg-gray-800"
                    aria-label=card_aria
                >
                    // Headline row
                    <div class="flex items-baseline justify-between">
                        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                            {display_name}
                        </span>
                        <span
                            class="flex items-baseline gap-1.5"
                            aria-label=value_aria
                        >
                            <span class="text-xl font-bold dark:text-white">{ewma_str}</span>
                            <span class="text-xs text-gray-500 dark:text-gray-400">{stddev_str}</span>
                            <span class=format!("text-lg {arrow_color}") aria-hidden="true">{arrow}</span>
                        </span>
                    </div>
                    // Chart
                    <ProgressChart
                        buckets=buckets
                        optimal_baseline=config.optimal_baseline
                        unit_label=config.unit_label
                    />
                </div>
            }
            .into_any()
        }}
    }
}
