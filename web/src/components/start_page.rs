use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn StartPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-6 py-12">
            <h1 class="sr-only">"Peach"</h1>

            <nav aria-label="Training modes" class="flex w-full flex-col items-center gap-6">
                <A href="/training/comparison"
                    attr:class="block w-full min-h-11 rounded-lg bg-indigo-600 px-6 py-4 text-center text-lg font-semibold text-white shadow-md hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-400">
                    "Comparison"
                </A>

                <A href="/training/pitch-matching"
                    attr:class="block w-full min-h-11 rounded-lg bg-gray-200 px-6 py-3 text-center text-lg font-medium text-gray-800 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600">
                    "Pitch Matching"
                </A>

                <hr class="w-full border-gray-300 dark:border-gray-600" />

                <A href="/training/comparison"
                    attr:class="block w-full min-h-11 rounded-lg bg-gray-200 px-6 py-3 text-center text-lg font-medium text-gray-800 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600">
                    "Interval Comparison"
                </A>

                <A href="/training/pitch-matching"
                    attr:class="block w-full min-h-11 rounded-lg bg-gray-200 px-6 py-3 text-center text-lg font-medium text-gray-800 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600">
                    "Interval Pitch Matching"
                </A>
            </nav>

            <nav aria-label="Utility" class="flex gap-6 pt-4 text-sm">
                <A href="/settings"
                    attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                    "Settings"
                </A>
                <A href="/profile"
                    attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                    "Profile"
                </A>
                <A href="/info"
                    attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                    "Info"
                </A>
            </nav>
        </div>
    }
}
