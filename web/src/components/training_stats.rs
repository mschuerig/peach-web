use leptos::prelude::*;
use leptos_fluent::{move_tr, tr};

use domain::Trend;

pub fn format_cents(value: f64) -> String {
    format!("{value:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cents_one_decimal() {
        assert_eq!(format_cents(12.34), "12.3");
        assert_eq!(format_cents(0.0), "0.0");
        assert_eq!(format_cents(5.0), "5.0");
        assert_eq!(format_cents(99.99), "100.0");
    }
}

fn trend_arrow(trend: Trend) -> (&'static str, &'static str, String) {
    match trend {
        Trend::Improving => (
            "\u{2198}",
            "text-green-600 dark:text-green-400",
            tr!("trend-improving"),
        ),
        Trend::Stable => (
            "\u{2192}",
            "text-gray-500 dark:text-gray-400",
            tr!("trend-stable"),
        ),
        Trend::Declining => (
            "\u{2197}",
            "text-orange-500 dark:text-orange-400",
            tr!("trend-declining"),
        ),
    }
}

#[component]
pub fn TrainingStats(
    latest_value: Signal<Option<f64>>,
    session_best: Signal<Option<f64>>,
    trend: Signal<Option<Trend>>,
) -> impl IntoView {
    view! {
        <div class="text-left mb-4">
            // Latest value
            <div
                class=move || {
                    if latest_value.get().is_some() {
                        "flex items-baseline gap-1.5 text-sm text-gray-600 dark:text-gray-400"
                    } else {
                        "flex items-baseline gap-1.5 text-sm text-gray-600 dark:text-gray-400 opacity-0"
                    }
                }
            >
                <span>{move_tr!("latest")}</span>
                <span class="font-medium dark:text-gray-300">
                    {move || latest_value.get().map(|v| tr!("value-cents", {"value" => format_cents(v)})).unwrap_or_default()}
                </span>
                // Trend arrow
                {move || {
                    if let Some(t) = trend.get() {
                        let (arrow, color, label) = trend_arrow(t);
                        view! {
                            <span class=format!("text-base {color}") aria-label=label>{arrow}</span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>
            // Session best
            <div
                class=move || {
                    if session_best.get().is_some() {
                        "text-xs text-gray-500 dark:text-gray-500 mt-0.5"
                    } else {
                        "text-xs text-gray-500 dark:text-gray-500 mt-0.5 opacity-0"
                    }
                }
            >
                <span>{move_tr!("best")}</span>
                <span>
                    {move || session_best.get().map(|v| tr!("value-cents", {"value" => format_cents(v)})).unwrap_or_default()}
                </span>
            </div>
        </div>
    }
}
