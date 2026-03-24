use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped_with_cancellation;
use leptos_fluent::move_tr;
use leptos_router::components::A;

use super::nav_bar::{NavBar, NavIconButton};
use super::progress_sparkline::ProgressSparkline;
use crate::adapters::localstorage_settings::LocalStorageSettings;
use crate::app::{SoundFontLoadStatus, base_href};
use crate::interval_codes::encode_intervals;
use domain::{Interval, TrainingDiscipline};

fn interval_href(path: &str) -> String {
    let intervals = LocalStorageSettings::get_selected_intervals();
    let has_non_prime = intervals.iter().any(|di| di.interval != Interval::Prime);
    let code = if has_non_prime {
        encode_intervals(&intervals)
    } else {
        "P1".to_string()
    };
    format!("{}?intervals={code}", base_href(path))
}

#[component]
fn TrainingCard(
    #[prop(into)] label: Signal<String>,
    icon: &'static str,
    href: String,
    #[prop(into)] aria_label: Signal<String>,
    mode: TrainingDiscipline,
    #[prop(into)] disabled: Signal<bool>,
) -> impl IntoView {
    let base_class = "flex w-full items-center gap-3 rounded-xl px-4 py-3 min-h-[4.5rem] text-lg font-medium no-underline transition-opacity duration-150 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900";
    let enabled_class = " bg-gray-100 text-gray-800 active:opacity-70 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700";
    let disabled_class = " bg-gray-100 text-gray-400 cursor-not-allowed opacity-60 dark:bg-gray-800 dark:text-gray-500";

    let enabled_cls = format!("{base_class}{enabled_class}");
    let disabled_cls = format!("{base_class}{disabled_class}");

    // Stable DOM: always render <A>, toggle class + click prevention reactively.
    // This avoids DOM rebuilds that eat the first click.
    view! {
        <A
            href=href
            attr:aria-label=move || aria_label.get()
            attr:class=move || if disabled.get() { disabled_cls.clone() } else { enabled_cls.clone() }
            attr:aria-disabled=move || if disabled.get() { Some("true") } else { None }
            attr:tabindex=move || if disabled.get() { Some("-1") } else { None }
            on:click=move |ev| {
                if disabled.get_untracked() {
                    ev.prevent_default();
                }
            }
        >
            <span class="text-xl" aria-hidden="true">{icon}</span>
            <div class="flex flex-col">
                <span>{move || label.get()}</span>
                <ProgressSparkline mode=mode />
            </div>
        </A>
    }
}

#[component]
pub fn StartPage() -> impl IntoView {
    let interval_comparison_href = interval_href("/training/comparison");
    let interval_pitch_matching_href = interval_href("/training/pitch-matching");

    let sf2_status: RwSignal<SoundFontLoadStatus> =
        use_context().expect("SoundFontLoadStatus signal must be provided");

    let can_start_training = Memo::new(move |_| {
        matches!(
            sf2_status.get(),
            SoundFontLoadStatus::Ready | SoundFontLoadStatus::Failed(_)
        )
    });

    let disabled = Signal::derive(move || !can_start_training.get());

    // Auto-dismiss failure notification after 5 seconds
    let sf2_error_dismissed = RwSignal::new(false);
    Effect::new(move || {
        if matches!(sf2_status.get(), SoundFontLoadStatus::Failed(_)) && !sf2_error_dismissed.get()
        {
            spawn_local_scoped_with_cancellation(async move {
                TimeoutFuture::new(5000).await;
                sf2_error_dismissed.set(true);
            });
        }
    });

    view! {
        <div class="flex flex-col items-center gap-6 pt-4 pb-12">
            <NavBar
                title=move_tr!("app-name")
                pill_group=true
                left_content=ViewFn::from(move || view! {
                    <NavIconButton label=move_tr!("nav-info") icon="\u{24D8}".to_string() href=base_href("/info") filled=true />
                })
            >
                <NavIconButton label=move_tr!("nav-profile") icon="\u{1F4CA}".to_string() href=base_href("/profile") />
                <NavIconButton label=move_tr!("nav-settings") icon="\u{2699}\u{FE0F}".to_string() href=base_href("/settings") />
            </NavBar>

            // Loading indicator
            {move || {
                if matches!(sf2_status.get(), SoundFontLoadStatus::Fetching) {
                    view! {
                        <div
                            role="status"
                            aria-live="polite"
                            class="w-full rounded-lg bg-indigo-50 border border-indigo-200 px-4 py-3 text-center text-indigo-700 dark:bg-indigo-900/30 dark:border-indigo-700 dark:text-indigo-300"
                        >
                            <span class="inline-block animate-pulse">{move_tr!("loading-sounds")}</span>
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}

            <nav aria-label="Training modes" class="flex w-full flex-col gap-7 md:flex-row md:gap-8">
                // Single Notes section
                <section class="flex-1">
                    <h2 class="mb-2.5 text-center text-sm font-medium text-gray-500 dark:text-gray-400">{move_tr!("single-notes")}</h2>
                    <div class="flex flex-col gap-2.5">
                        <TrainingCard
                            label=move_tr!("hear-and-compare")
                            icon="\u{1F442}"
                            href=base_href("/training/comparison")
                            aria_label=move_tr!("hear-compare-single-aria")
                            mode=TrainingDiscipline::UnisonPitchDiscrimination
                            disabled=disabled
                        />
                        <TrainingCard
                            label=move_tr!("tune-and-match")
                            icon="\u{1F3AF}"
                            href=base_href("/training/pitch-matching")
                            aria_label=move_tr!("tune-match-single-aria")
                            mode=TrainingDiscipline::UnisonPitchMatching
                            disabled=disabled
                        />
                    </div>
                </section>

                // Intervals section
                <section class="flex-1">
                    <h2 class="mb-2.5 text-center text-sm font-medium text-gray-500 dark:text-gray-400">{move_tr!("intervals")}</h2>
                    <div class="flex flex-col gap-2.5">
                        <TrainingCard
                            label=move_tr!("hear-and-compare")
                            icon="\u{1F442}"
                            href=interval_comparison_href
                            aria_label=move_tr!("hear-compare-intervals-aria")
                            mode=TrainingDiscipline::IntervalPitchDiscrimination
                            disabled=disabled
                        />
                        <TrainingCard
                            label=move_tr!("tune-and-match")
                            icon="\u{1F3AF}"
                            href=interval_pitch_matching_href
                            aria_label=move_tr!("tune-match-intervals-aria")
                            mode=TrainingDiscipline::IntervalPitchMatching
                            disabled=disabled
                        />
                    </div>
                </section>
            </nav>

            // SoundFont load failure notification — non-blocking, auto-dismissing
            {move || {
                if let SoundFontLoadStatus::Failed(_) = sf2_status.get()
                    && !sf2_error_dismissed.get()
                {
                    return view! {
                        <div
                            class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-amber-100 border border-amber-400 text-amber-800 px-4 py-2 rounded-lg shadow-md text-sm dark:bg-amber-900 dark:border-amber-700 dark:text-amber-200"
                            role="alert"
                        >
                            {move_tr!("sound-load-failed")}
                        </div>
                    }.into_any();
                }
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}
