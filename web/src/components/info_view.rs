use leptos::prelude::*;
use leptos_fluent::move_tr;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;

use super::help_content::HelpContent;
use super::nav_bar::NavBar;
use crate::app::go_back;
use crate::help_sections::{INFO_ACKNOWLEDGMENTS, INFO_HELP};

#[component]
pub fn InfoView() -> impl IntoView {
    // Escape key handler — navigates back
    let keydown_handler = Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
        if ev.key() == "Escape" {
            ev.prevent_default();
            go_back();
        }
    });

    let document = web_sys::window().unwrap().document().unwrap();
    let keydown_fn: JsValue = keydown_handler.as_ref().clone();
    document
        .add_event_listener_with_callback("keydown", keydown_fn.unchecked_ref())
        .unwrap();

    // Keep closure alive for component lifetime
    let _keydown_closure = StoredValue::new_local(keydown_handler);

    // Clean up listener on component unmount
    on_cleanup(move || {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            let _ =
                document.remove_event_listener_with_callback("keydown", keydown_fn.unchecked_ref());
        }
    });

    view! {
        <div class="pt-4 pb-12">
            <NavBar title=move_tr!("app-name") left_content=ViewFn::from({
                move || {
                    view! {
                        <button
                            on:click=move |_| go_back()
                            class="min-h-11 min-w-11 px-3 flex items-center justify-center rounded-full bg-gray-100 text-gray-600 hover:bg-gray-200 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200 dark:focus:ring-offset-gray-900"
                            aria-label=move || move_tr!("done").get()
                        >
                            {move_tr!("done")}
                        </button>
                    }.into_any()
                }
            })>
            </NavBar>
            <p class="text-sm text-gray-500 dark:text-gray-400 text-center -mt-2 mb-4">{move_tr!("version-label", {"number" => "0.1.0"})}</p>

            <div class="mt-8 space-y-8">
                <HelpContent sections=INFO_HELP use_h2=true />

                <HelpContent sections=INFO_ACKNOWLEDGMENTS use_h2=true />

                <section>
                    <h2 class="text-lg font-semibold dark:text-white">{move_tr!("developer")}</h2>
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
                    <h2 class="text-lg font-semibold dark:text-white">{move_tr!("project")}</h2>
                    <dl class="mt-2 space-y-2 text-gray-700 dark:text-gray-300">
                        <div class="flex gap-2">
                            <dt>{move_tr!("github-label")}</dt>
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
                            <dt>{move_tr!("license-label")}</dt>
                            <dd>"MIT"</dd>
                        </div>
                        <div class="flex gap-2">
                            <dt>{move_tr!("copyright-label")}</dt>
                            <dd>{move_tr!("copyright-text")}</dd>
                        </div>
                    </dl>
                </section>
            </div>

        </div>
    }
}
