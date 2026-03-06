use leptos::prelude::*;

use super::help_content::HelpContent;
use super::page_nav::PageNav;
use crate::help_sections::{INFO_ACKNOWLEDGMENTS, INFO_HELP};

#[component]
pub fn InfoView() -> impl IntoView {
    view! {
        <div class="py-12">
            <PageNav current="info" />

            <h1 class="text-2xl font-bold dark:text-white">"Peach"</h1>
            <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Version 0.1.0"</p>

            <div class="mt-8 space-y-8">
                <HelpContent sections=INFO_HELP />

                <HelpContent sections=INFO_ACKNOWLEDGMENTS />

                <section>
                    <h2 class="text-lg font-semibold dark:text-white">"Developer"</h2>
                    <address class="mt-2 not-italic space-y-1 text-gray-700 dark:text-gray-300">
                        <p>"Michael Sch\u{00FC}rig"</p>
                        <p>
                            <a
                                href="mailto:michael@schuerig.de"
                                class="rounded text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                            >
                                "michael@schuerig.de"
                            </a>
                        </p>
                    </address>
                </section>

                <section>
                    <h2 class="text-lg font-semibold dark:text-white">"Project"</h2>
                    <dl class="mt-2 space-y-2 text-gray-700 dark:text-gray-300">
                        <div class="flex gap-2">
                            <dt>"GitHub:"</dt>
                            <dd>
                                <a
                                    href="https://github.com/mschuerig/peach-web"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="rounded text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                                >
                                    "mschuerig/peach-web"
                                </a>
                            </dd>
                        </div>
                        <div class="flex gap-2">
                            <dt>"License:"</dt>
                            <dd>"MIT"</dd>
                        </div>
                        <div class="flex gap-2">
                            <dt>"Copyright:"</dt>
                            <dd>"\u{00A9} 2026 Michael Sch\u{00FC}rig"</dd>
                        </div>
                    </dl>
                </section>
            </div>

        </div>
    }
}
