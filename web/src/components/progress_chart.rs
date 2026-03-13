use leptos::prelude::*;
use wasm_bindgen::JsValue;

use domain::{BucketSize, TimeBucket};

const VIEWBOX_WIDTH: f64 = 300.0;
const MARGIN_LEFT: f64 = 35.0;
const MARGIN_RIGHT: f64 = 10.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM_BASE: f64 = 24.0;
const MARGIN_BOTTOM_YEARS: f64 = 40.0;

struct ZoneRange {
    zone: BucketSize,
    start_index: usize,
    end_index: usize, // inclusive
}

fn detect_zones(buckets: &[TimeBucket]) -> Vec<ZoneRange> {
    let mut zones = Vec::new();
    if buckets.is_empty() {
        return zones;
    }
    let mut current_zone = buckets[0].bucket_size;
    let mut start = 0;
    for (i, b) in buckets.iter().enumerate() {
        if b.bucket_size != current_zone {
            zones.push(ZoneRange {
                zone: current_zone,
                start_index: start,
                end_index: i - 1,
            });
            current_zone = b.bucket_size;
            start = i;
        }
    }
    zones.push(ZoneRange {
        zone: current_zone,
        start_index: start,
        end_index: buckets.len() - 1,
    });
    zones
}

struct BridgePoint {
    x: f64,
    mean: f64,
    stddev: f64,
}

fn compute_session_bridge(
    buckets: &[TimeBucket],
    first_session_index: usize,
) -> Option<BridgePoint> {
    let session_buckets: Vec<&TimeBucket> = buckets[first_session_index..]
        .iter()
        .filter(|b| b.bucket_size == BucketSize::Session)
        .collect();
    if session_buckets.is_empty() {
        return None;
    }

    let total_records: usize = session_buckets.iter().map(|b| b.record_count).sum();
    if total_records == 0 {
        return None;
    }

    let bridge_mean = session_buckets
        .iter()
        .map(|b| b.mean * b.record_count as f64)
        .sum::<f64>()
        / total_records as f64;

    let bridge_var = session_buckets
        .iter()
        .map(|b| b.stddev * b.stddev * b.record_count as f64)
        .sum::<f64>()
        / total_records as f64;

    Some(BridgePoint {
        x: first_session_index as f64 - 0.5,
        mean: bridge_mean,
        stddev: bridge_var.sqrt(),
    })
}

fn year_from_epoch(epoch_secs: f64) -> i32 {
    let date = js_sys::Date::new(&JsValue::from_f64(epoch_secs * 1000.0));
    date.get_full_year() as i32
}

fn format_month_label(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"month".into(), &"short".into()).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    let formatted = dtf
        .format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    formatted.trim_end_matches('.').to_string()
}

fn format_weekday_label(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"weekday".into(), &"short".into()).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    let formatted = dtf
        .format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    formatted.trim_end_matches('.').to_string()
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

    let bucket_count = buckets.len();

    // Zone detection
    let zones = detect_zones(&buckets);
    let multi_zone = zones.len() > 1;

    // Check if year labels are needed (monthly zone spanning multiple years)
    let has_year_labels = zones.iter().any(|z| {
        if z.zone != BucketSize::Month {
            return false;
        }
        let first_year = year_from_epoch(buckets[z.start_index].period_start);
        let last_year = year_from_epoch(buckets[z.end_index].period_start);
        first_year != last_year
    });

    let margin_bottom = if has_year_labels {
        MARGIN_BOTTOM_YEARS
    } else {
        MARGIN_BOTTOM_BASE
    };
    let viewbox_height: f64 = 160.0 + if has_year_labels { 16.0 } else { 0.0 };
    let inner_w = VIEWBOX_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
    let inner_h = viewbox_height - MARGIN_TOP - margin_bottom;

    // Y domain: 0 to max(1, max(bucket.mean + bucket.stddev)) — AC 1
    let y_max = buckets
        .iter()
        .map(|b| b.mean + b.stddev)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    // Index-based X mapping — AC 1
    let x = |index: f64| -> f64 {
        MARGIN_LEFT + (index + 0.5) / bucket_count as f64 * inner_w
    };
    let y = |value: f64| -> f64 { MARGIN_TOP + inner_h - (value / y_max) * inner_h };

    // Session detection
    let first_session_index = buckets
        .iter()
        .position(|b| b.bucket_size == BucketSize::Session);
    let has_non_session = buckets
        .iter()
        .any(|b| b.bucket_size != BucketSize::Session);
    let session_only = first_session_index == Some(0) && !has_non_session;

    // Session bridge — AC 6
    let bridge = if let Some(fsi) = first_session_index {
        if has_non_session {
            compute_session_bridge(&buckets, fsi)
        } else {
            None
        }
    } else {
        None
    };

    // Non-session buckets for band and line
    let non_session: Vec<(usize, &TimeBucket)> = buckets
        .iter()
        .enumerate()
        .filter(|(_, b)| b.bucket_size != BucketSize::Session)
        .collect();

    // --- Layer 1: Zone backgrounds (AC 2, 3) ---
    let zone_bgs = if multi_zone {
        zones
            .iter()
            .map(|z| {
                let x1 = x(z.start_index as f64 - 0.5);
                let x2 = x(z.end_index as f64 + 0.5);
                let width = x2 - x1;
                let fill_class = match z.zone {
                    BucketSize::Month | BucketSize::Session => "chart-zone-bg",
                    BucketSize::Day => "chart-zone-bg chart-zone-bg-secondary",
                };
                view! {
                    <rect
                        x=format!("{x1:.1}")
                        y=format!("{MARGIN_TOP:.1}")
                        width=format!("{width:.1}")
                        height=format!("{inner_h:.1}")
                        class=fill_class
                        fill="currentColor"
                        opacity="0.06"
                    />
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // --- Layer 2: Zone dividers and year boundaries (AC 4) ---
    let mut divider_xs: Vec<f64> = Vec::new();
    let mut zone_transition_indices: Vec<usize> = Vec::new();

    if multi_zone {
        for i in 1..buckets.len() {
            if buckets[i].bucket_size != buckets[i - 1].bucket_size {
                divider_xs.push(i as f64 - 0.5);
                zone_transition_indices.push(i);
            }
        }
    }

    // Year boundaries within monthly zone
    for z in &zones {
        if z.zone == BucketSize::Month {
            for i in (z.start_index + 1)..=z.end_index {
                let prev_year = year_from_epoch(buckets[i - 1].period_start);
                let curr_year = year_from_epoch(buckets[i].period_start);
                if curr_year != prev_year {
                    let boundary_idx = i as f64 - 0.5;
                    // Suppress if within 1 index of a zone transition
                    let near_transition = zone_transition_indices
                        .iter()
                        .any(|&t| (boundary_idx - t as f64).abs() <= 1.0);
                    if !near_transition {
                        divider_xs.push(boundary_idx);
                    }
                }
            }
        }
    }

    let dividers = divider_xs
        .iter()
        .map(|&idx| {
            let dx = x(idx);
            view! {
                <line
                    x1=format!("{dx:.1}")
                    y1=format!("{MARGIN_TOP:.1}")
                    x2=format!("{dx:.1}")
                    y2=format!("{:.1}", MARGIN_TOP + inner_h)
                    class="chart-zone-divider"
                    stroke="currentColor"
                    opacity="0.3"
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect::<Vec<_>>();

    // --- Layer 3: Stddev band (AC 5) ---
    let band_path = if !session_only && !non_session.is_empty() {
        let mut upper: Vec<(f64, f64)> = non_session
            .iter()
            .map(|(i, b)| (x(*i as f64), y((b.mean + b.stddev).min(y_max))))
            .collect();
        let mut lower: Vec<(f64, f64)> = non_session
            .iter()
            .map(|(i, b)| (x(*i as f64), y((b.mean - b.stddev).max(0.0))))
            .collect();

        if let Some(ref bp) = bridge {
            upper.push((x(bp.x), y((bp.mean + bp.stddev).min(y_max))));
            lower.push((x(bp.x), y((bp.mean - bp.stddev).max(0.0))));
        }

        let mut d = String::new();
        for (i, (px, py)) in upper.iter().enumerate() {
            if i == 0 {
                d.push_str(&format!("M {px:.1},{py:.1}"));
            } else {
                d.push_str(&format!(" L {px:.1},{py:.1}"));
            }
        }
        for (px, py) in lower.iter().rev() {
            d.push_str(&format!(" L {px:.1},{py:.1}"));
        }
        d.push_str(" Z");
        Some(d)
    } else {
        None
    };

    // --- Layer 4: Mean trend line (AC 8) ---
    let mean_points = if !session_only && !non_session.is_empty() {
        let mut pts: Vec<String> = non_session
            .iter()
            .map(|(i, b)| format!("{:.1},{:.1}", x(*i as f64), y(b.mean)))
            .collect();
        if let Some(ref bp) = bridge {
            pts.push(format!("{:.1},{:.1}", x(bp.x), y(bp.mean)));
        }
        Some(pts.join(" "))
    } else {
        None
    };

    // --- Layer 5: Session dots (AC 9) ---
    let session_dots: Vec<_> = buckets
        .iter()
        .enumerate()
        .filter(|(_, b)| b.bucket_size == BucketSize::Session)
        .map(|(i, b)| {
            let cx = x(i as f64);
            let cy = y(b.mean);
            // Area = 20 units -> radius = sqrt(20/pi) ≈ 2.52
            let r = (20.0_f64 / std::f64::consts::PI).sqrt();
            view! {
                <circle
                    cx=format!("{cx:.1}")
                    cy=format!("{cy:.1}")
                    r=format!("{r:.2}")
                    class="fill-blue-500 dark:fill-blue-400"
                />
            }
        })
        .collect();

    // --- Layer 6: Baseline (AC 10) ---
    let baseline_y_str = format!("{:.1}", y(optimal_baseline));
    let baseline_x1 = format!("{MARGIN_LEFT:.1}");
    let baseline_x2 = format!("{:.1}", MARGIN_LEFT + inner_w);

    // --- X-axis labels (AC 11) ---
    let today_label = untrack(|| leptos_fluent::tr!("chart-today"));
    let mut seen_first_session = false;
    let x_labels: Vec<(f64, String)> = {
        let raw: Vec<(f64, String)> = buckets
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let label = match b.bucket_size {
                    BucketSize::Month => format_month_label(b.period_start),
                    BucketSize::Day => format_weekday_label(b.period_start),
                    BucketSize::Session => {
                        if !seen_first_session {
                            seen_first_session = true;
                            today_label.clone()
                        } else {
                            String::new()
                        }
                    }
                };
                (x(i as f64), label)
            })
            .collect();
        // Deduplicate adjacent identical labels (skip empty)
        let mut deduped: Vec<(f64, String)> = Vec::new();
        for (xpos, label) in raw {
            if label.is_empty() {
                continue;
            }
            if deduped
                .last()
                .map(|(_, l)| l != &label)
                .unwrap_or(true)
            {
                deduped.push((xpos, label));
            }
        }
        deduped
    };

    let label_y = format!("{:.1}", viewbox_height - margin_bottom + 14.0);

    // --- Year labels (AC 12) ---
    let year_labels: Vec<(f64, String)> = if has_year_labels {
        let mut labels = Vec::new();
        for z in &zones {
            if z.zone != BucketSize::Month {
                continue;
            }
            // Group consecutive monthly buckets by year
            let mut year_start = z.start_index;
            let mut current_year = year_from_epoch(buckets[z.start_index].period_start);
            for (offset, b) in buckets[(z.start_index + 1)..=z.end_index]
                .iter()
                .enumerate()
            {
                let i = z.start_index + 1 + offset;
                let yr = year_from_epoch(b.period_start);
                if yr != current_year {
                    let center_x = x((year_start as f64 + (i - 1) as f64) / 2.0);
                    labels.push((center_x, format!("{current_year}")));
                    year_start = i;
                    current_year = yr;
                }
            }
            // Emit last year span
            let center_x = x((year_start as f64 + z.end_index as f64) / 2.0);
            labels.push((center_x, format!("{current_year}")));
        }
        labels
    } else {
        Vec::new()
    };

    let year_label_y = format!("{:.1}", viewbox_height - 4.0);

    // Y-axis unit label
    let y_axis_x = MARGIN_LEFT - 4.0;
    let y_axis_cy = MARGIN_TOP + inner_h / 2.0;

    view! {
        <svg
            viewBox=format!("0 0 {VIEWBOX_WIDTH} {viewbox_height}")
            width="100%"
            aria-hidden="true"
            class="mt-2 h-[180px] md:h-[240px]"
            preserveAspectRatio="none"
        >
            // Layer 1: Zone backgrounds
            {zone_bgs}
            // Layer 2: Zone dividers
            {dividers}
            // Layer 3: Stddev band
            {band_path.map(|d| {
                view! {
                    <path
                        d=d
                        class="chart-stddev-band"
                        fill="rgb(96, 165, 250)"
                        opacity="0.15"
                    />
                }
            })}
            // Layer 4: Mean trend line
            {mean_points.map(|pts| {
                view! {
                    <polyline
                        points=pts
                        fill="none"
                        stroke="rgb(59, 130, 246)"
                        stroke-width="2"
                        stroke-linejoin="round"
                        vector-effect="non-scaling-stroke"
                        class="dark:stroke-blue-400"
                    />
                }
            })}
            // Layer 5: Session dots
            {session_dots}
            // Layer 6: Baseline
            <line
                x1=baseline_x1
                y1=baseline_y_str.clone()
                x2=baseline_x2
                y2=baseline_y_str
                class="chart-baseline"
                stroke="rgb(34, 197, 94)"
                stroke-width="1"
                stroke-dasharray="5 3"
                opacity="0.60"
                vector-effect="non-scaling-stroke"
            />
            // Y-axis unit label
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
                .map(|(lx, label)| {
                    let ly = label_y.clone();
                    view! {
                        <text
                            x=format!("{lx:.1}")
                            y=ly
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
            // Year labels
            {year_labels
                .into_iter()
                .map(|(lx, label)| {
                    let ly = year_label_y.clone();
                    view! {
                        <text
                            x=format!("{lx:.1}")
                            y=ly
                            text-anchor="middle"
                            font-size="8"
                            fill="currentColor"
                            opacity="0.4"
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
