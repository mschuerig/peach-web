use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn PageNav(current: &'static str) -> impl IntoView {
    view! {
        <nav aria-label="Page navigation" class="flex gap-6 text-sm mb-6">
            {if current != "start" {
                Some(view! {
                    <A href="/"
                        attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                        "Start"
                    </A>
                })
            } else {
                None
            }}
            {if current != "settings" {
                Some(view! {
                    <A href="/settings"
                        attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                        "Settings"
                    </A>
                })
            } else {
                None
            }}
            {if current != "profile" {
                Some(view! {
                    <A href="/profile"
                        attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                        "Profile"
                    </A>
                })
            } else {
                None
            }}
            {if current == "start" {
                Some(view! {
                    <A href="/info"
                        attr:class="min-h-11 min-w-11 flex items-center justify-center rounded text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200">
                        "Info"
                    </A>
                })
            } else {
                None
            }}
        </nav>
    }
}
