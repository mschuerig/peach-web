use leptos::prelude::*;

fn init_logging() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Warn
    };
    console_log::init_with_level(level).expect("error initializing logger");
}

fn main() {
    console_error_panic_hook::set_once();
    init_logging();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <main class="flex min-h-screen items-center justify-center">
            <h1 class="text-4xl font-bold">"Peach"</h1>
        </main>
    }
}
