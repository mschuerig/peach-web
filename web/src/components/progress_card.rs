use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsValue;

use domain::{ProgressTimeline, TrainingMode, TrainingModeState, Trend};
use leptos_fluent::{I18n, tr};

fn format_decimal_1(value: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"minimumFractionDigits".into(), &1.into()).unwrap();
    js_sys::Reflect::set(&options, &"maximumFractionDigits".into(), &1.into()).unwrap();
    let formatter = js_sys::Intl::NumberFormat::new(&js_sys::Array::new(), &options);
    formatter
        .format()
        .call1(&JsValue::NULL, &JsValue::from_f64(value))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| format!("{value:.1}"))
}

fn format_stddev(buckets: &[domain::TimeBucket]) -> String {
    if buckets.is_empty() {
        return String::new();
    }
    let last_stddev = buckets.last().map(|b| b.stddev).unwrap_or(0.0);
    format!("\u{00B1}{}", format_decimal_1(last_stddev))
}

fn trend_arrow(trend: Option<Trend>) -> (&'static str, &'static str) {
    match trend {
        Some(Trend::Improving) => ("\u{2198}", "text-green-600 dark:text-green-400"),
        Some(Trend::Stable) => ("\u{2192}", "text-gray-500 dark:text-gray-400"),
        Some(Trend::Declining) => ("\u{2197}", "text-amber-600 dark:text-amber-400"),
        None => ("", ""),
    }
}

fn trend_text(trend: Option<Trend>) -> String {
    match trend {
        Some(Trend::Improving) => tr!("trend-improving"),
        Some(Trend::Stable) => tr!("trend-stable"),
        Some(Trend::Declining) => tr!("trend-declining"),
        None => String::new(),
    }
}

#[component]
pub fn ProgressCard(mode: TrainingMode) -> impl IntoView {
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline context");

    let ptl = progress_timeline.clone();
    let config = mode.config();
    let i18n = expect_context::<I18n>();

    view! {
        {move || {
            let tl = ptl.borrow();
            if tl.state(mode) == TrainingModeState::NoData {
                return view! { <div /> }.into_any();
            }

            let buckets = tl.display_buckets(mode);
            let ewma = tl.current_ewma(mode);
            let trend = tl.trend(mode);
            drop(tl);

            let ewma_str = ewma.map(format_decimal_1).unwrap_or_default();
            let stddev_str = format_stddev(&buckets);
            let (arrow, arrow_color) = trend_arrow(trend);
            let trend_label = trend_text(trend);
            let display_name = i18n.tr(config.display_name);

            let card_aria = tr!("progress-chart-for", {
                "name" => display_name.clone()
            });
            let value_aria = tr!("current-trend", {
                "ewma" => ewma_str.clone(),
                "trend" => trend_label
            });

            view! {
                <div
                    class="progress-card rounded-xl backdrop-blur-md bg-white/60 dark:bg-gray-900/60 border border-white/20 dark:border-gray-700/30 p-4"
                    role="group"
                    aria-label=card_aria
                    aria-description=value_aria
                >
                    // Headline row
                    <div class="flex items-baseline justify-between">
                        <span class="text-base font-semibold text-gray-700 dark:text-gray-300">
                            {display_name}
                        </span>
                        <span class="flex items-baseline gap-1.5">
                            <span class="text-xl font-bold dark:text-white">{ewma_str}</span>
                            <span class="text-xs text-gray-500 dark:text-gray-400">{stddev_str}</span>
                            <span class=format!("text-lg {arrow_color}") aria-hidden="true">{arrow}</span>
                        </span>
                    </div>
                    // Progress chart
                    <super::progress_chart::ProgressChart
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
