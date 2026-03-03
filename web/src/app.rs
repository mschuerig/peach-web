use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};

use crate::components::{
    ComparisonView, InfoView, PitchMatchingView, ProfileView, SettingsView, StartPage,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:p-3 focus:bg-white focus:text-black dark:focus:bg-gray-900 dark:focus:text-white"
            >
                "Skip to main content"
            </a>
            <main id="main-content" class="min-h-screen bg-white dark:bg-gray-900">
                <div class="mx-auto max-w-lg px-4">
                    <Routes fallback=|| {
                        view! {
                            <div class="py-12 text-center">
                                <h1 class="text-2xl font-bold dark:text-white">"Page not found"</h1>
                                <A
                                    href="/"
                                    attr:class="mt-4 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                                >
                                    "Back to Start"
                                </A>
                            </div>
                        }
                    }>
                        <Route path=path!("/") view=StartPage />
                        <Route path=path!("/training/comparison") view=ComparisonView />
                        <Route path=path!("/training/pitch-matching") view=PitchMatchingView />
                        <Route path=path!("/profile") view=ProfileView />
                        <Route path=path!("/settings") view=SettingsView />
                        <Route path=path!("/info") view=InfoView />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}
