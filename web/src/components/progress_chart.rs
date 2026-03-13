use leptos::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::wasm_bindgen::JsCast;

use domain::{BucketSize, TimeBucket};

const VIEWBOX_WIDTH: f64 = 300.0;
const VISIBLE_BUCKETS: usize = 8;
const MARGIN_LEFT: f64 = 10.0;
const MARGIN_RIGHT: f64 = 10.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM_BASE: f64 = 24.0;
const MARGIN_BOTTOM_YEARS: f64 = 40.0;
const Y_AXIS_VB_W: f64 = 40.0;

/// Compute nice Y-axis tick values from 0 to y_max.
fn compute_y_ticks(y_max: f64) -> Vec<f64> {
    if y_max <= 0.0 {
        return vec![0.0];
    }
    let rough_step = y_max / 3.0;
    let mag = 10_f64.powf(rough_step.log10().floor());
    let norm = rough_step / mag;
    let step = if norm <= 1.5 {
        mag
    } else if norm <= 3.5 {
        2.0 * mag
    } else {
        5.0 * mag
    };
    let mut ticks = vec![0.0];
    let mut v = step;
    while v < y_max {
        ticks.push(v);
        v += step;
    }
    ticks
}

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

fn format_annotation_date_monthly(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"month".into(), &"short".into()).unwrap();
    js_sys::Reflect::set(&options, &"year".into(), &"numeric".into()).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    let formatted = dtf
        .format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    formatted.trim_end_matches('.').to_string()
}

fn format_annotation_date_daily(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"weekday".into(), &"short".into()).unwrap();
    js_sys::Reflect::set(&options, &"month".into(), &"short".into()).unwrap();
    js_sys::Reflect::set(&options, &"day".into(), &"numeric".into()).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    let formatted = dtf
        .format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    formatted.trim_end_matches('.').to_string()
}

fn format_annotation_date_session(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"hour".into(), &"2-digit".into()).unwrap();
    js_sys::Reflect::set(&options, &"minute".into(), &"2-digit".into()).unwrap();
    js_sys::Reflect::set(&options, &"hour12".into(), &JsValue::FALSE).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    let formatted = dtf
        .format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    formatted.trim_end_matches('.').to_string()
}

fn format_annotation_date(bucket: &TimeBucket) -> String {
    match bucket.bucket_size {
        BucketSize::Month => format_annotation_date_monthly(bucket.period_start),
        BucketSize::Day => format_annotation_date_daily(bucket.period_start),
        BucketSize::Session => format_annotation_date_session(bucket.period_start),
    }
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

fn format_decimal_1_chart(value: f64) -> String {
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

    // Task 1: Selection state signal
    let selected_bucket = RwSignal::new(None::<usize>);

    // Scrollable container setup
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let is_scrollable = bucket_count > VISIBLE_BUCKETS;
    let svg_width = if is_scrollable {
        format!("{}%", bucket_count as f64 / VISIBLE_BUCKETS as f64 * 100.0)
    } else {
        "100%".to_string()
    };

    // Set initial scroll position to right edge after mount
    if is_scrollable {
        Effect::new(move |_| {
            if container_ref.get().is_some() {
                wasm_bindgen_futures::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(0).await;
                    if let Some(el) = container_ref.get() {
                        let element: &web_sys::Element = el.as_ref();
                        element
                            .set_scroll_left(element.scroll_width() - element.client_width());
                    }
                });
            }
        });
    }

    // Task 7: Dismiss annotation on scroll
    if is_scrollable {
        Effect::new(move |_| {
            if let Some(el) = container_ref.get() {
                let element: &web_sys::Element = el.as_ref();
                let target: &web_sys::EventTarget = element.as_ref();
                let closure =
                    wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                        selected_bucket.set(None);
                    });
                let _ = target.add_event_listener_with_callback(
                    "scroll",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }
        });
    }

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
    // Scale viewBox width proportionally so each bucket has the same coordinate
    // space as in static mode — prevents distortion with preserveAspectRatio="none"
    let viewbox_w = if is_scrollable {
        VIEWBOX_WIDTH * bucket_count as f64 / VISIBLE_BUCKETS as f64
    } else {
        VIEWBOX_WIDTH
    };
    let inner_w = viewbox_w - MARGIN_LEFT - MARGIN_RIGHT;
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
                        .any(|&t| i.abs_diff(t) <= 1);
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

    // --- Grid lines ---
    let y_ticks = compute_y_ticks(y_max);

    // Horizontal grid lines at each Y tick (rendered in main chart SVG)
    let chart_x_start = format!("{MARGIN_LEFT:.1}");
    let chart_x_end = format!("{:.1}", MARGIN_LEFT + inner_w);
    let h_grid_lines: Vec<_> = y_ticks
        .iter()
        .map(|&val| {
            let gy = y(val);
            let x1 = chart_x_start.clone();
            let x2 = chart_x_end.clone();
            view! {
                <line
                    x1=x1
                    y1=format!("{gy:.1}")
                    x2=x2
                    y2=format!("{gy:.1}")
                    stroke="currentColor"
                    opacity="0.15"
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect();

    // Vertical dashed grid lines at each bucket center
    let v_grid_lines: Vec<_> = (0..bucket_count)
        .map(|i| {
            let gx = x(i as f64);
            view! {
                <line
                    x1=format!("{gx:.1}")
                    y1=format!("{MARGIN_TOP:.1}")
                    x2=format!("{gx:.1}")
                    y2=format!("{:.1}", MARGIN_TOP + inner_h)
                    stroke="currentColor"
                    opacity="0.10"
                    stroke-width="1"
                    stroke-dasharray="2 3"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect();

    // --- Y-axis tick labels (right-side SVG) ---
    let y_axis_ticks: Vec<_> = y_ticks
        .iter()
        .map(|&val| {
            let ty = y(val);
            let label = format!("{}", val as i32);
            view! {
                <text
                    x="4"
                    y=format!("{:.1}", ty + 3.0)
                    text-anchor="start"
                    font-size="9"
                    fill="currentColor"
                    opacity="0.6"
                >
                    {label}
                </text>
            }
        })
        .collect();

    // Task 2: Click handler — convert click to bucket index
    let on_chart_click = move |ev: web_sys::MouseEvent| {
        let Some(target) = ev.current_target() else {
            return;
        };
        let Ok(element) = target.dyn_into::<web_sys::Element>() else {
            return;
        };
        let rect = element.get_bounding_client_rect();
        let client_x = ev.client_x() as f64;

        // getBoundingClientRect() on the SVG already reflects scroll position
        // (its left edge shifts as the container scrolls), so no scroll_left adjustment needed.
        let svg_x = (client_x - rect.left()) / rect.width() * viewbox_w;

        // Invert the x() function: raw_index = (svg_x - MARGIN_LEFT) / inner_w * bucket_count - 0.5
        let raw_index = (svg_x - MARGIN_LEFT) / inner_w * bucket_count as f64 - 0.5;
        let clicked_index = raw_index.round().max(0.0).min((bucket_count - 1) as f64) as usize;

        // Toggle logic
        if selected_bucket.get_untracked() == Some(clicked_index) {
            selected_bucket.set(None);
        } else {
            selected_bucket.set(Some(clicked_index));
        }
    };

    // --- Layer 7: Selection line and annotation popover ---
    // Pre-compute x positions and bucket data for reactive rendering
    let bucket_x_positions: Vec<f64> = (0..bucket_count).map(|i| x(i as f64)).collect();
    let buckets_for_annotation = buckets.clone();

    // Popover dimensions in SVG viewBox units
    let popover_w = 60.0_f64;
    let popover_h = 52.0_f64;

    let selection_line_and_popover = move || {
        selected_bucket.get().map(|idx| {
            let bx = bucket_x_positions[idx];
            let bucket = &buckets_for_annotation[idx];

            // Selection line
            let line_y1 = format!("{MARGIN_TOP:.1}");
            let line_y2 = format!("{:.1}", MARGIN_TOP + inner_h);

            // Popover position with overflow resolution (Task 6)
            let fo_x = (bx - popover_w / 2.0)
                .max(MARGIN_LEFT)
                .min(MARGIN_LEFT + inner_w - popover_w);
            let fo_y = MARGIN_TOP + 2.0;

            // Date formatting (Task 5)
            let date_str = format_annotation_date(bucket);
            let mean_str = format_decimal_1_chart(bucket.mean);
            let stddev_str = format!("\u{00B1}{}", format_decimal_1_chart(bucket.stddev));
            let records_str = untrack(|| {
                leptos_fluent::tr!("chart-annotation-records", {
                    "count" => bucket.record_count
                })
            });

            view! {
                // Selection line (Task 3)
                <line
                    x1=format!("{bx:.1}")
                    y1=line_y1
                    x2=format!("{bx:.1}")
                    y2=line_y2
                    class="chart-selection-line"
                    stroke="currentColor"
                    stroke-dasharray="5 3"
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
                // Annotation popover (Task 4)
                <foreignObject
                    x=format!("{fo_x:.1}")
                    y=format!("{fo_y:.1}")
                    width=format!("{popover_w:.0}")
                    height=format!("{popover_h:.0}")
                >
                    <div
                        class="backdrop-blur-md bg-white/60 dark:bg-gray-900/60 border border-white/20 dark:border-gray-700/30 rounded-[6px] p-[6px] space-y-[2px]"
                        style="font-size: 8px; line-height: 1.3;"
                    >
                        <div class="text-gray-500 dark:text-gray-400">{date_str}</div>
                        <div class="font-bold dark:text-white">{mean_str}</div>
                        <div class="text-gray-500 dark:text-gray-400">{stddev_str}</div>
                        <div class="text-gray-500 dark:text-gray-400">{records_str}</div>
                    </div>
                </foreignObject>
            }
        })
    };

    // Task 10: Accessibility — live region content
    let buckets_for_a11y = buckets.clone();
    let live_region_text = move || {
        selected_bucket.get().map(|idx| {
            let bucket = &buckets_for_a11y[idx];
            let date_str = format_annotation_date(bucket);
            let mean_str = format_decimal_1_chart(bucket.mean);
            let stddev_str = format_decimal_1_chart(bucket.stddev);
            untrack(|| {
                leptos_fluent::tr!("chart-annotation-summary", {
                    "date" => date_str,
                    "mean" => mean_str,
                    "unit" => unit_label,
                    "stddev" => stddev_str,
                    "count" => bucket.record_count
                })
            })
        }).unwrap_or_default()
    };

    view! {
        <div class="mt-1 text-xs text-gray-400 dark:text-gray-500">{unit_label}</div>
        <div class="flex h-[180px] md:h-[240px]">
        // Scrollable chart area
        <div
            node_ref=container_ref
            class=if is_scrollable { "flex-1 min-w-0 overflow-x-auto chart-scroll-container" } else { "flex-1 min-w-0" }
        >
        <svg
            viewBox=format!("0 0 {viewbox_w} {viewbox_height}")
            width=svg_width
            height="100%"
            role="img"
            aria-label=unit_label
            preserveAspectRatio="none"
            on:click=on_chart_click
        >
            // Grid lines (behind everything)
            {h_grid_lines}
            {v_grid_lines}
            // Layer 1: Zone backgrounds
            {zone_bgs}
            // Layer 2: Zone dividers
            {dividers}
            // Layer 3: Stddev band
            {band_path.map(|d| {
                view! {
                    <path
                        d=d
                        class="chart-stddev-band fill-blue-500 dark:fill-blue-400"
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
                            font-size="9"
                            fill="currentColor"
                            opacity="0.6"
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
                            font-size="9"
                            fill="currentColor"
                            opacity="0.5"
                        >
                            {label}
                        </text>
                    }
                })
                .collect::<Vec<_>>()}
            // Layer 7: Selection line and annotation popover
            {selection_line_and_popover}
        </svg>
        </div>
        // Fixed Y-axis (right side, non-scrolling)
        <svg
            viewBox=format!("0 0 {Y_AXIS_VB_W} {viewbox_height}")
            class="flex-none w-8"
            height="100%"
            aria-hidden="true"
            preserveAspectRatio="none"
        >
            {y_axis_ticks}
        </svg>
        </div>
        // Task 10: Accessibility — screen reader live region
        <div class="sr-only" role="status" aria-live="polite">
            {live_region_text}
        </div>
    }
    .into_any()
}
