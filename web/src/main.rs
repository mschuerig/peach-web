use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
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
