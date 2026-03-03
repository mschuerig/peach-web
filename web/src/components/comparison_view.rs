use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn ComparisonView() -> impl IntoView {
    view! {
        <div class="py-12">
            <h1 class="text-2xl font-bold dark:text-white">"Comparison Training"</h1>
            <A href="/"
                attr:class="mt-4 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300">
                "Back to Start"
            </A>
        </div>
    }
}
