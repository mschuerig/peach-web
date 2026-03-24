use leptos::prelude::*;
use leptos_fluent::move_tr;

use super::nav_bar::{NavBar, NavIconButton};
use crate::app::base_href;

#[component]
pub fn ContinuousRhythmMatchingView() -> impl IntoView {
    view! {
        <div class="pt-4 pb-12">
            <NavBar
                title=move_tr!("fill-the-gap")
                left_content=ViewFn::from(move || view! {
                    <NavIconButton label=move_tr!("back") icon="\u{2190}".to_string() href=base_href("/") />
                })
            />
            <div class="flex flex-col items-center justify-center px-6 py-16 text-center">
                <h2 class="text-2xl font-semibold text-gray-400 dark:text-gray-500 mb-4">
                    {move_tr!("fill-the-gap")}
                </h2>
                <p class="text-gray-500 dark:text-gray-400 text-base max-w-sm leading-relaxed">
                    {move_tr!("continuous-rhythm-description")}
                </p>
                <p class="mt-8 text-sm text-gray-400 dark:text-gray-600 italic">
                    {move_tr!("coming-soon")}
                </p>
            </div>
        </div>
    }
}
