use leptos::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::wasm_bindgen::JsCast;

use domain::{BucketSize, SpectrogramAccuracyLevel, SpectrogramData, TimeBucket};
use leptos_fluent::I18n;

use crate::components::progress_chart::{
    detect_zones, format_annotation_date, format_decimal_1_chart,
};

const VIEWBOX_WIDTH: f64 = 300.0;
const VISIBLE_BUCKETS: usize = 8;
const MARGIN_LEFT: f64 = 50.0;
const MARGIN_RIGHT: f64 = 10.0;
const MARGIN_TOP: f64 = 10.0;
const MARGIN_BOTTOM: f64 = 24.0;

fn cell_fill(level: Option<SpectrogramAccuracyLevel>) -> &'static str {
    match level {
        Some(SpectrogramAccuracyLevel::Excellent) => "rgb(45, 212, 191)",
        Some(SpectrogramAccuracyLevel::Precise) => "rgb(34, 197, 94)",
        Some(SpectrogramAccuracyLevel::Moderate) => "rgb(234, 179, 8)",
        Some(SpectrogramAccuracyLevel::Loose) => "rgb(249, 115, 22)",
        Some(SpectrogramAccuracyLevel::Erratic) => "rgb(239, 68, 68)",
        None => "rgb(156, 163, 175)",
    }
}

fn cell_opacity(level: Option<SpectrogramAccuracyLevel>) -> &'static str {
    match level {
        Some(_) => "0.7",
        None => "0.15",
    }
}

fn tempo_range_i18n_key(range: domain::TempoRange) -> &'static str {
    match range {
        domain::TempoRange::VerySlow => "tempo-range-very-slow",
        domain::TempoRange::Slow => "tempo-range-slow",
        domain::TempoRange::Moderate => "tempo-range-moderate",
        domain::TempoRange::Brisk => "tempo-range-brisk",
        domain::TempoRange::Fast => "tempo-range-fast",
        domain::TempoRange::VeryFast => "tempo-range-very-fast",
    }
}

#[component]
pub fn RhythmSpectrogramChart(
    data: SpectrogramData,
    #[prop(into)] unit_label: String,
    #[prop(into)] chart_label: String,
) -> impl IntoView {
    if data.is_empty() {
        return view! { <div /> }.into_any();
    }

    let i18n = expect_context::<I18n>();
    let col_count = data.columns.len();
    let row_count = data.trained_ranges.len();

    let selected_cell = RwSignal::new(None::<(usize, usize)>);

    let container_ref = NodeRef::<leptos::html::Div>::new();
    let is_scrollable = col_count > VISIBLE_BUCKETS;
    let svg_width = if is_scrollable {
        format!("{}%", col_count as f64 / VISIBLE_BUCKETS as f64 * 100.0)
    } else {
        "100%".to_string()
    };

    if is_scrollable {
        Effect::new(move |_| {
            if container_ref.get().is_some() {
                leptos::task::spawn_local_scoped_with_cancellation(async move {
                    gloo_timers::future::TimeoutFuture::new(0).await;
                    if let Some(el) = container_ref.get() {
                        let element: &web_sys::Element = el.as_ref();
                        element.set_scroll_left(element.scroll_width() - element.client_width());
                    }
                });
            }
        });
    }

    if is_scrollable {
        Effect::new(move |_| {
            if let Some(el) = container_ref.get() {
                let element: &web_sys::Element = el.as_ref();
                let target: &web_sys::EventTarget = element.as_ref();
                let closure = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                    selected_cell.set(None);
                });
                let scroll_fn: JsValue = closure.as_ref().clone();
                let _ =
                    target.add_event_listener_with_callback("scroll", scroll_fn.unchecked_ref());
                let _scroll_closure = StoredValue::new_local(closure);
                let target_owned: web_sys::EventTarget = target.clone();
                on_cleanup(move || {
                    let _ = target_owned
                        .remove_event_listener_with_callback("scroll", scroll_fn.unchecked_ref());
                });
            }
        });
    }

    let viewbox_height: f64 = 160.0;
    let viewbox_w = if is_scrollable {
        VIEWBOX_WIDTH * col_count as f64 / VISIBLE_BUCKETS as f64
    } else {
        VIEWBOX_WIDTH
    };
    let inner_w = viewbox_w - MARGIN_LEFT - MARGIN_RIGHT;
    let inner_h = viewbox_height - MARGIN_TOP - MARGIN_BOTTOM;
    let cell_w = inner_w / col_count as f64;
    let cell_h = inner_h / row_count as f64;

    // --- Static layers (computed once) ---

    let buckets: Vec<TimeBucket> = data.columns.iter().map(|c| c.bucket.clone()).collect();
    let zones = detect_zones(&buckets);
    let multi_zone = zones.len() > 1;

    // Zone backgrounds
    let zone_bgs: Vec<_> = if multi_zone {
        zones
            .iter()
            .map(|z| {
                let x1 = MARGIN_LEFT + z.start_index as f64 * cell_w;
                let x2 = MARGIN_LEFT + (z.end_index as f64 + 1.0) * cell_w;
                let width = x2 - x1;
                let fill_class = match z.zone {
                    BucketSize::Month | BucketSize::Session => "chart-zone-bg",
                    BucketSize::Day => "chart-zone-bg chart-zone-bg-secondary",
                };
                view! {
                    <rect
                        x=format!("{x1:.1}") y=format!("{MARGIN_TOP:.1}")
                        width=format!("{width:.1}") height=format!("{inner_h:.1}")
                        class=fill_class fill="currentColor" opacity="0.06"
                    />
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    // Zone dividers
    let dividers: Vec<_> = if multi_zone {
        (1..col_count)
            .filter(|&i| buckets[i].bucket_size != buckets[i - 1].bucket_size)
            .map(|i| {
                let dx = MARGIN_LEFT + i as f64 * cell_w;
                view! {
                    <line
                        x1=format!("{dx:.1}") y1=format!("{MARGIN_TOP:.1}")
                        x2=format!("{dx:.1}") y2=format!("{:.1}", MARGIN_TOP + inner_h)
                        class="chart-zone-divider" stroke="currentColor"
                        opacity="0.3" stroke-width="1" vector-effect="non-scaling-stroke"
                    />
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    // Heatmap cells
    let cell_gap = 1.0_f64;
    let mut heatmap_cells = Vec::new();
    for (ci, col) in data.columns.iter().enumerate() {
        for (ri, cell) in col.cells.iter().enumerate() {
            let range = data.trained_ranges[ri];
            let level = data.accuracy_level(cell, range);
            let fill = cell_fill(level);
            let opacity = cell_opacity(level);
            let rx = MARGIN_LEFT + ci as f64 * cell_w + cell_gap / 2.0;
            let ry = MARGIN_TOP + (row_count - 1 - ri) as f64 * cell_h + cell_gap / 2.0;
            let rw = (cell_w - cell_gap).max(1.0);
            let rh = (cell_h - cell_gap).max(1.0);
            heatmap_cells.push(view! {
                <rect
                    x=format!("{rx:.1}") y=format!("{ry:.1}")
                    width=format!("{rw:.1}") height=format!("{rh:.1}")
                    rx="2" fill=fill opacity=opacity
                />
            });
        }
    }

    // Y-axis labels
    let range_labels: Vec<_> = data
        .trained_ranges
        .iter()
        .enumerate()
        .map(|(ri, &range)| {
            let ly = MARGIN_TOP + (row_count - 1 - ri) as f64 * cell_h + cell_h / 2.0;
            let label = untrack(|| i18n.tr(tempo_range_i18n_key(range)));
            view! {
                <text
                    x=format!("{:.1}", MARGIN_LEFT - 4.0)
                    y=format!("{ly:.1}")
                    text-anchor="end" dominant-baseline="central"
                    font-size="11" fill="currentColor" opacity="0.7"
                >
                    {label}
                </text>
            }
        })
        .collect();

    // X-axis labels
    let today_label = untrack(|| i18n.tr("chart-today"));
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
                (MARGIN_LEFT + (i as f64 + 0.5) * cell_w, label)
            })
            .collect();
        let mut deduped = Vec::new();
        for (xpos, label) in raw {
            if label.is_empty() {
                continue;
            }
            if deduped
                .last()
                .map(|(_, l): &(f64, String)| l != &label)
                .unwrap_or(true)
            {
                deduped.push((xpos, label));
            }
        }
        deduped
    };
    let label_y = format!("{:.1}", viewbox_height - MARGIN_BOTTOM + 14.0);

    // --- Reactive layers ---

    // Click handler
    let on_chart_click = move |ev: web_sys::MouseEvent| {
        let Some(target) = ev.current_target() else {
            return;
        };
        let Ok(element) = target.dyn_into::<web_sys::Element>() else {
            return;
        };
        let rect = element.get_bounding_client_rect();
        let svg_x = (ev.client_x() as f64 - rect.left()) / rect.width() * viewbox_w;
        let svg_y = (ev.client_y() as f64 - rect.top()) / rect.height() * viewbox_height;

        let ci = ((svg_x - MARGIN_LEFT) / cell_w).floor() as isize;
        let row_from_top = ((svg_y - MARGIN_TOP) / cell_h).floor() as isize;
        let ri = row_count as isize - 1 - row_from_top;

        if ci < 0 || ci >= col_count as isize || ri < 0 || ri >= row_count as isize {
            selected_cell.set(None);
            return;
        }
        let pos = (ci as usize, ri as usize);
        if selected_cell.get_untracked() == Some(pos) {
            selected_cell.set(None);
        } else {
            selected_cell.set(Some(pos));
        }
    };

    // Pre-clone for reactive closures
    let columns_popover = data.columns.clone();
    let ranges_popover = data.trained_ranges.clone();
    let columns_a11y = data.columns;
    let ranges_a11y = data.trained_ranges;

    let popover_w = 100.0_f64;
    let popover_h = 88.0_f64;

    let selection_and_popover = move || {
        selected_cell.get().map(|(ci, ri)| {
            let col = &columns_popover[ci];
            let cell = &col.cells[ri];
            let range = ranges_popover[ri];
            let bucket = &col.bucket;

            let rx = MARGIN_LEFT + ci as f64 * cell_w;
            let ry = MARGIN_TOP + (row_count - 1 - ri) as f64 * cell_h;

            let fo_x = (rx + cell_w / 2.0 - popover_w / 2.0)
                .max(MARGIN_LEFT)
                .min(MARGIN_LEFT + inner_w - popover_w);
            let fo_y = (MARGIN_TOP + 2.0).min(MARGIN_TOP + inner_h - popover_h);

            let date_str = format_annotation_date(bucket);
            let range_label = untrack(|| i18n.tr(tempo_range_i18n_key(range)));
            let mean_str = cell
                .mean_accuracy_percent
                .map(format_decimal_1_chart)
                .unwrap_or_else(|| "\u{2013}".to_string());

            let early_label = untrack(|| i18n.tr("spectrogram-early"));
            let late_label = untrack(|| i18n.tr("spectrogram-late"));

            let early_text = cell.early_stats.as_ref().map(|s| {
                let rec = untrack(|| {
                    leptos_fluent::tr!("spectrogram-rec", { "count" => s.count })
                });
                format!(
                    "{}: {}%, {}",
                    early_label,
                    format_decimal_1_chart(s.mean_percent),
                    rec,
                )
            });
            let late_text = cell.late_stats.as_ref().map(|s| {
                let rec = untrack(|| {
                    leptos_fluent::tr!("spectrogram-rec", { "count" => s.count })
                });
                format!(
                    "{}: {}%, {}",
                    late_label,
                    format_decimal_1_chart(s.mean_percent),
                    rec,
                )
            });

            view! {
                <rect
                    x=format!("{rx:.1}") y=format!("{ry:.1}")
                    width=format!("{cell_w:.1}") height=format!("{cell_h:.1}")
                    fill="none" stroke="currentColor" stroke-width="2"
                    vector-effect="non-scaling-stroke" class="chart-selection-line"
                />
                <foreignObject
                    x=format!("{fo_x:.1}") y=format!("{fo_y:.1}")
                    width=format!("{popover_w:.0}") height=format!("{popover_h:.0}")
                >
                    <div
                        class="backdrop-blur-md bg-white/90 dark:bg-gray-900/90 border border-gray-200 dark:border-gray-700 rounded-[6px] p-[6px] space-y-[2px]"
                        style="font-size: 8px; line-height: 1.3;"
                    >
                        <div class="text-gray-500 dark:text-gray-400">{range_label}" \u{2014} "{date_str}</div>
                        <div class="font-bold dark:text-white">{mean_str}"%"</div>
                        {early_text.map(|t| view! { <div class="text-gray-500 dark:text-gray-400">{t}</div> })}
                        {late_text.map(|t| view! { <div class="text-gray-500 dark:text-gray-400">{t}</div> })}
                    </div>
                </foreignObject>
            }
        })
    };

    // A11y live region
    let unit_label_a11y = unit_label.clone();
    let live_region_text = move || {
        selected_cell
            .get()
            .map(|(ci, ri)| {
                let col = &columns_a11y[ci];
                let cell = &col.cells[ri];
                let range = ranges_a11y[ri];
                let date_str = format_annotation_date(&col.bucket);
                let range_label = untrack(|| i18n.tr(tempo_range_i18n_key(range)));
                let mean_str = cell
                    .mean_accuracy_percent
                    .map(format_decimal_1_chart)
                    .unwrap_or_else(|| "\u{2013}".to_string());
                let records = untrack(|| {
                    leptos_fluent::tr!("spectrogram-records", {
                        "count" => cell.record_count
                    })
                });
                format!(
                    "{} \u{2014} {}, {} {}, {}",
                    range_label, date_str, mean_str, unit_label_a11y, records
                )
            })
            .unwrap_or_default()
    };

    // Keyboard navigation
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let current = selected_cell.get_untracked();
        let new_sel = match (current, key.as_str()) {
            (None, "Enter" | " ") => Some((col_count - 1, 0)),
            (Some((ci, ri)), "ArrowRight") if ci + 1 < col_count => Some((ci + 1, ri)),
            (Some((ci, ri)), "ArrowLeft") if ci > 0 => Some((ci - 1, ri)),
            (Some((ci, ri)), "ArrowUp") if ri + 1 < row_count => Some((ci, ri + 1)),
            (Some((ci, ri)), "ArrowDown") if ri > 0 => Some((ci, ri - 1)),
            (Some(_), "Escape") => None,
            _ => return,
        };
        ev.prevent_default();
        selected_cell.set(new_sel);
    };

    view! {
        <div class="mt-1 text-xs text-gray-400 dark:text-gray-500">{unit_label}</div>
        <div class="flex h-[180px] md:h-[240px]">
        <div
            node_ref=container_ref
            class=if is_scrollable { "flex-1 min-w-0 overflow-x-auto chart-scroll-container" } else { "flex-1 min-w-0" }
        >
        <svg
            viewBox=format!("0 0 {viewbox_w} {viewbox_height}")
            width=svg_width height="100%"
            role="img" aria-label=chart_label
            preserveAspectRatio="none"
            on:click=on_chart_click
            on:keydown=on_keydown
            tabindex="0"
        >
            {zone_bgs}
            {dividers}
            {heatmap_cells}
            {range_labels}
            {x_labels
                .into_iter()
                .map(|(lx, label)| {
                    let ly = label_y.clone();
                    view! {
                        <text
                            x=format!("{lx:.1}") y=ly
                            text-anchor="middle" font-size="11"
                            fill="currentColor" opacity="0.6"
                        >
                            {label}
                        </text>
                    }
                })
                .collect::<Vec<_>>()}
            {selection_and_popover}
        </svg>
        </div>
        </div>
        <div class="sr-only" role="status" aria-live="polite">
            {live_region_text}
        </div>
    }
    .into_any()
}

fn format_month_label(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"month".into(), &"short".into()).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    dtf.format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_string()
}

fn format_weekday_label(epoch_secs: f64) -> String {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"weekday".into(), &"short".into()).unwrap();
    let dtf = js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &options);
    dtf.format()
        .call1(&JsValue::NULL, &JsValue::from_f64(epoch_secs * 1000.0))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_string()
}
