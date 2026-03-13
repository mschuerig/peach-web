use leptos::prelude::*;

use domain::{BucketSize, TimeBucket};

const VIEWBOX_WIDTH: f64 = 300.0;
const MARGIN_LEFT: f64 = 35.0;
const MARGIN_RIGHT: f64 = 10.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM: f64 = 24.0;

fn format_x_label(epoch_secs: f64, bucket_size: BucketSize) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(epoch_secs * 1000.0));
    match bucket_size {
        BucketSize::Session => {
            let now = js_sys::Date::now() / 1000.0;
            let diff_secs = (now - epoch_secs).max(0.0);
            let diff_mins = (diff_secs / 60.0) as u64;
            if diff_mins < 60 {
                format!("{}m", diff_mins.max(1))
            } else {
                format!("{}h", diff_mins / 60)
            }
        }
        BucketSize::Day => {
            const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            WEEKDAYS[date.get_day() as usize].to_string()
        }
        BucketSize::Month => {
            const MONTHS: [&str; 12] = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            MONTHS[date.get_month() as usize].to_string()
        }
    }
}

#[component]
pub fn ProgressChart(
    buckets: Vec<TimeBucket>,
    optimal_baseline: f64,
    unit_label: &'static str,
) -> impl IntoView {
    if buckets.len() <= 1 {
        return view! { <div /> }.into_any();
    }

    // Use a fixed viewBox; actual rendered height controlled by CSS classes on the SVG
    let viewbox_height: f64 = 160.0;
    let inner_w = VIEWBOX_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
    let inner_h = viewbox_height - MARGIN_TOP - MARGIN_BOTTOM;

    // Y-axis range: 0 to max(mean + stddev, baseline) * 1.15
    let y_max = buckets
        .iter()
        .map(|b| b.mean + b.stddev)
        .fold(0.0_f64, f64::max)
        .max(optimal_baseline)
        * 1.15;
    let y_max = y_max.max(1.0);

    // X-axis range
    let x_min = buckets.first().unwrap().period_start;
    let x_max = buckets.last().unwrap().period_start;
    let x_range = if (x_max - x_min).abs() < 1e-6 {
        1.0
    } else {
        x_max - x_min
    };

    let map_x = |epoch: f64| -> f64 { MARGIN_LEFT + ((epoch - x_min) / x_range) * inner_w };
    let map_y = |value: f64| -> f64 { MARGIN_TOP + inner_h - (value / y_max) * inner_h };

    // Stddev band path (upper = mean + stddev, lower = mean - stddev, clamped to 0)
    let mut band_d = String::new();
    for (i, b) in buckets.iter().enumerate() {
        let x = map_x(b.period_start);
        let y = map_y((b.mean + b.stddev).min(y_max));
        if i == 0 {
            band_d.push_str(&format!("M {x:.1},{y:.1}"));
        } else {
            band_d.push_str(&format!(" L {x:.1},{y:.1}"));
        }
    }
    for b in buckets.iter().rev() {
        let x = map_x(b.period_start);
        let y = map_y((b.mean - b.stddev).max(0.0));
        band_d.push_str(&format!(" L {x:.1},{y:.1}"));
    }
    band_d.push_str(" Z");

    // Mean line polyline points
    let mean_points: String = buckets
        .iter()
        .map(|b| format!("{:.1},{:.1}", map_x(b.period_start), map_y(b.mean)))
        .collect::<Vec<_>>()
        .join(" ");

    // Baseline y
    let baseline_y = format!("{:.1}", map_y(optimal_baseline));

    // X-axis labels (max ~6, evenly spaced)
    let max_labels = 6.min(buckets.len());
    let step = if buckets.len() <= max_labels {
        1
    } else {
        buckets.len() / max_labels
    };
    let x_labels: Vec<(f64, String)> = {
        let raw: Vec<(f64, String)> = buckets
            .iter()
            .enumerate()
            .filter(|(i, _)| *i % step == 0 || *i == buckets.len() - 1)
            .map(|(_, b)| {
                (
                    map_x(b.period_start),
                    format_x_label(b.period_start, b.bucket_size),
                )
            })
            .collect();
        // Deduplicate adjacent identical labels
        let mut deduped: Vec<(f64, String)> = Vec::new();
        for (x, label) in raw {
            if deduped.last().map(|(_, l)| l != &label).unwrap_or(true) {
                deduped.push((x, label));
            }
        }
        deduped
    };

    let label_y = format!("{:.1}", viewbox_height - 4.0);
    let y_axis_x = MARGIN_LEFT - 4.0;
    let y_axis_cy = MARGIN_TOP + inner_h / 2.0;
    let margin_left_str = format!("{MARGIN_LEFT}");
    let x2_str = format!("{}", MARGIN_LEFT + inner_w);

    view! {
        <svg
            viewBox=format!("0 0 {VIEWBOX_WIDTH} {viewbox_height}")
            width="100%"
            aria-hidden="true"
            class="mt-2 h-[180px] md:h-[240px]"
            preserveAspectRatio="none"
        >
            // Stddev band
            <path d=band_d fill="rgba(96, 165, 250, 0.2)" />
            // Mean line
            <polyline
                points=mean_points
                fill="none"
                stroke="rgb(59, 130, 246)"
                stroke-width="2"
                stroke-linejoin="round"
                vector-effect="non-scaling-stroke"
            />
            // Baseline (dashed green)
            <line
                x1=margin_left_str.clone()
                y1=baseline_y.clone()
                x2=x2_str
                y2=baseline_y
                stroke="rgb(34, 197, 94)"
                stroke-width="1"
                stroke-dasharray="4 3"
                vector-effect="non-scaling-stroke"
            />
            // Y-axis unit label (rotated)
            <text
                x=format!("{y_axis_x:.1}")
                y=format!("{y_axis_cy:.1}")
                text-anchor="middle"
                font-size="9"
                fill="currentColor"
                opacity="0.5"
                transform=format!("rotate(-90, {y_axis_x:.1}, {y_axis_cy:.1})")
            >
                {unit_label}
            </text>
            // X-axis labels
            {x_labels
                .into_iter()
                .map(|(x, label)| {
                    let lbl_y = label_y.clone();
                    view! {
                        <text
                            x=format!("{x:.1}")
                            y=lbl_y
                            text-anchor="middle"
                            font-size="8"
                            fill="currentColor"
                            opacity="0.5"
                        >
                            {label}
                        </text>
                    }
                })
                .collect::<Vec<_>>()}
        </svg>
    }
    .into_any()
}
