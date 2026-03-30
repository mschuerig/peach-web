use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_router::components::A;

use crate::app::{go_back, nav_push};

/// Reusable icon button for navigation bars.
/// Renders as `<A>` when href provided, `<button>` when on_click provided.
/// When `filled` is true, the button has a visible circle background (matching back button style).
/// When `circled` is true, the icon character gets a thin border circle (matching ⓘ style).
#[component]
pub fn NavIconButton(
    #[prop(into)] label: Signal<String>,
    #[prop(into)] icon: String,
    #[prop(optional, into)] href: Option<String>,
    #[prop(optional, into)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    /// Optional callback fired before navigation when `href` is used.
    /// Use for cleanup (e.g. stopping a training session) before navigating away.
    #[prop(optional, into)]
    before_nav: Option<Callback<()>>,
    #[prop(optional)] filled: bool,
    #[prop(optional)] circled: bool,
) -> impl IntoView {
    let class = if filled {
        "min-h-11 min-w-11 flex items-center justify-center rounded-full bg-gray-200 text-gray-600 hover:bg-gray-300 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-700 dark:text-gray-400 dark:hover:bg-gray-600 dark:hover:text-gray-200 dark:focus:ring-offset-gray-900"
    } else {
        "min-h-11 min-w-11 flex items-center justify-center rounded-full text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:bg-gray-800 dark:focus:ring-offset-gray-900"
    };

    let icon_class = if circled {
        "flex items-center justify-center w-5 h-5 rounded-full border border-current text-sm"
    } else {
        ""
    };

    if let Some(href) = href {
        view! {
            <A href=href attr:class=class attr:aria-label=move || label.get()
                on:click=move |_| {
                    if let Some(cb) = before_nav {
                        cb.run(());
                    }
                    nav_push();
                }
            >
                <span class=icon_class aria-hidden="true">{icon}</span>
            </A>
        }
        .into_any()
    } else {
        let on_click = on_click.unwrap_or_else(|| Callback::new(|_| {}));
        view! {
            <button
                on:click=move |ev| on_click.run(ev)
                class=class
                aria-label=move || label.get()
            >
                <span class=icon_class aria-hidden="true">{icon}</span>
            </button>
        }
        .into_any()
    }
}

/// iOS-style navigation bar with back arrow (or custom left content), centered title,
/// and right-side action slots. Both sides use `flex-1` so they share space equally,
/// keeping the title page-centered. When the title grows long, the smaller side shrinks
/// first (iOS-like shift), then the title truncates with an ellipsis.
#[component]
pub fn NavBar(
    /// The page title displayed centered in the bar.
    #[prop(into)]
    title: Signal<String>,
    /// When true, a back button is shown. Uses `go_back()` for navigation.
    #[prop(optional)]
    show_back: bool,
    /// Optional click handler called before `go_back()` (for training views that need to stop sessions).
    #[prop(optional, into)]
    on_back: Option<Callback<()>>,
    /// Optional custom left-side content (replaces back button). Used by start page for info icon.
    #[prop(optional, into)]
    left_content: Option<ViewFn>,
    /// When true, right-side icons are wrapped in a pill-shaped container.
    #[prop(optional)]
    pill_group: bool,
    /// Right-side action icons passed as children.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let back_class = "min-h-11 min-w-11 flex items-center justify-center rounded-full bg-gray-100 text-gray-600 hover:bg-gray-200 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200 dark:focus:ring-offset-gray-900";

    let left = if let Some(left_fn) = left_content {
        left_fn.run()
    } else if show_back {
        let on_back = on_back.unwrap_or_else(|| Callback::new(|()| {}));
        view! {
            <button
                on:click=move |_| {
                    on_back.run(());
                    go_back();
                }
                class=back_class
                aria-label=move || tr!("back")
            >
                <span aria-hidden="true">{"\u{2039}"}</span>
            </button>
        }
        .into_any()
    } else {
        view! { <span></span> }.into_any()
    };

    let right_class = if pill_group {
        "flex items-center gap-1 bg-gray-100 dark:bg-gray-800 rounded-full px-1"
    } else {
        "flex items-center gap-1"
    };

    view! {
        <nav role="navigation" aria-label=move || tr!("page-navigation") class="flex w-full items-center gap-2 mb-4">
            // Left: back button, custom content, or spacer — flex-1 for equal side widths
            <div class="flex-1 flex items-center">
                {left}
            </div>
            // Center: title — shrinks and truncates when space is tight
            <h1 class="shrink min-w-0 text-center text-lg font-bold truncate dark:text-white">{move || title.get()}</h1>
            // Right: action icons — flex-1 mirrors left for centering
            <div class="flex-1 flex justify-end">
                <div class=right_class>
                    {children.map(|c| c())}
                </div>
            </div>
        </nav>
    }
}
