use leptos::prelude::*;
use leptos_fluent::move_tr;

use super::nav_bar::{NavBar, NavIconButton};
use crate::app::base_href;

#[component]
pub fn RhythmOffsetDetectionView() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-6 pt-4 pb-12">
            <NavBar
                title=move_tr!("compare-timing")
                left_content=ViewFn::from(move || view! {
                    <NavIconButton label=move_tr!("back") icon="\u{2190}".to_string() href=base_href("/") />
                })
            />
            <div class="flex flex-1 items-center justify-center py-16">
                <p class="text-gray-500 dark:text-gray-400 text-lg">{move_tr!("coming-soon")}</p>
            </div>
        </div>
    }
}

#[component]
pub fn ContinuousRhythmMatchingView() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-6 pt-4 pb-12">
            <NavBar
                title=move_tr!("fill-the-gap")
                left_content=ViewFn::from(move || view! {
                    <NavIconButton label=move_tr!("back") icon="\u{2190}".to_string() href=base_href("/") />
                })
            />
            <div class="flex flex-1 items-center justify-center py-16">
                <p class="text-gray-500 dark:text-gray-400 text-lg">{move_tr!("coming-soon")}</p>
            </div>
        </div>
    }
}
