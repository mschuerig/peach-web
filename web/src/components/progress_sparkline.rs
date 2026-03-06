use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use domain::{ProgressTimeline, TrainingMode, TrainingModeState, Trend};

use super::training_stats::format_cents;

fn trend_stroke_color(trend: Option<Trend>) -> &'static str {
    match trend {
        Some(Trend::Improving) => "#16a34a", // green-600
        Some(Trend::Stable) => "#f59e0b",    // amber-500
        _ => "#9ca3af",                       // gray-400
    }
}

fn trend_label(trend: Option<Trend>) -> &'static str {
    match trend {
        Some(Trend::Improving) => "improving",
        Some(Trend::Stable) => "stable",
        Some(Trend::Declining) => "declining",
        None => "",
    }
}

fn compute_points(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }
    if values.len() == 1 {
        return "0,12 60,12".to_string();
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    let count = values.len();
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = i as f64 / (count - 1) as f64 * 60.0;
            let y = if range < 0.1 {
                12.0
            } else {
                24.0 * (1.0 - (v - min) / range)
            };
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn ProgressSparkline(mode: TrainingMode) -> impl IntoView {
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline context");

    let ptl = progress_timeline.clone();

    view! {
        {move || {
            let tl = ptl.borrow();
            if tl.state(mode) == TrainingModeState::NoData {
                return view! { <div /> }.into_any();
            }

            let buckets = tl.buckets(mode);
            let ewma = tl.current_ewma(mode);
            let trend = tl.trend(mode);
            drop(tl);

            let values: Vec<f64> = buckets.iter().map(|b| b.mean).collect();
            let points = compute_points(&values);
            let stroke_color = trend_stroke_color(trend);

            let ewma_str = ewma.map(|v| format!("{} cents", format_cents(v))).unwrap_or_default();
            let trend_str = trend_label(trend);
            let mode_name = mode.config().display_name;

            let aria = format!("{mode_name}: {ewma_str}, {trend_str}");

            view! {
                <div
                    class="inline-flex items-center gap-1.5"
                    aria-label=aria
                >
                    <svg
                        width="60"
                        height="24"
                        viewBox="0 0 60 24"
                        aria-hidden="true"
                    >
                        <polyline
                            points=points
                            fill="none"
                            stroke-width="1.5"
                            stroke=stroke_color
                        />
                    </svg>
                    <span class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                        {ewma_str}
                    </span>
                </div>
            }
            .into_any()
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_points_empty() {
        assert_eq!(compute_points(&[]), "");
    }

    #[test]
    fn test_compute_points_single_value() {
        assert_eq!(compute_points(&[5.0]), "0,12 60,12");
    }

    #[test]
    fn test_compute_points_flat_line_small_range() {
        // Range < 0.1 -> all points at y=12
        let points = compute_points(&[10.0, 10.05, 10.0]);
        assert!(points.contains("12.0"));
    }

    #[test]
    fn test_compute_points_two_values() {
        // min=0, max=24, range=24
        // Point 0: x=0, y=24*(1 - (0-0)/24) = 24
        // Point 1: x=60, y=24*(1 - (24-0)/24) = 0
        let points = compute_points(&[0.0, 24.0]);
        assert_eq!(points, "0.0,24.0 60.0,0.0");
    }

    #[test]
    fn test_compute_points_three_values() {
        // min=10, max=30, range=20
        // Point 0: x=0, y=24*(1-(10-10)/20)=24
        // Point 1: x=30, y=24*(1-(20-10)/20)=12
        // Point 2: x=60, y=24*(1-(30-10)/20)=0
        let points = compute_points(&[10.0, 20.0, 30.0]);
        assert_eq!(points, "0.0,24.0 30.0,12.0 60.0,0.0");
    }

    #[test]
    fn test_trend_stroke_color_improving() {
        assert_eq!(trend_stroke_color(Some(Trend::Improving)), "#16a34a");
    }

    #[test]
    fn test_trend_stroke_color_stable() {
        assert_eq!(trend_stroke_color(Some(Trend::Stable)), "#f59e0b");
    }

    #[test]
    fn test_trend_stroke_color_declining() {
        assert_eq!(trend_stroke_color(Some(Trend::Declining)), "#9ca3af");
    }

    #[test]
    fn test_trend_stroke_color_none() {
        assert_eq!(trend_stroke_color(None), "#9ca3af");
    }

    #[test]
    fn test_trend_label_values() {
        assert_eq!(trend_label(Some(Trend::Improving)), "improving");
        assert_eq!(trend_label(Some(Trend::Stable)), "stable");
        assert_eq!(trend_label(Some(Trend::Declining)), "declining");
        assert_eq!(trend_label(None), "");
    }
}
