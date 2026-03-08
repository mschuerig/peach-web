use leptos::prelude::*;
use leptos_router::components::A;

/// Reusable icon button for navigation bars.
/// Renders as `<A>` when href provided, `<button>` when on_click provided.
/// When `filled` is true, the button has a visible circle background (matching back button style).
/// When `circled` is true, the icon character gets a thin border circle (matching ⓘ style).
#[component]
pub fn NavIconButton(
    #[prop(into)] label: String,
    #[prop(into)] icon: String,
    #[prop(optional, into)] href: Option<String>,
    #[prop(optional, into)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
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
            <A href=href attr:class=class attr:aria-label=label.clone()>
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
                aria-label=label.clone()
            >
                <span class=icon_class aria-hidden="true">{icon}</span>
            </button>
        }
        .into_any()
    }
}

/// iOS-style navigation bar with back arrow (or custom left content), centered title,
/// and right-side action slots.
#[component]
pub fn NavBar(
    /// The page title displayed centered in the bar.
    title: &'static str,
    /// Optional href for back navigation. If None, no back button is shown (unless left_content provided).
    #[prop(optional, into)]
    back_href: Option<String>,
    /// Optional click handler for back button (for training views that need to stop sessions).
    #[prop(optional, into)]
    on_back: Option<Callback<leptos::ev::MouseEvent>>,
    /// Optional custom left-side content (replaces back button). Used by start page for info icon.
    #[prop(optional, into)]
    left_content: Option<ViewFn>,
    /// When true, right-side icons are wrapped in a pill-shaped container.
    #[prop(optional)]
    pill_group: bool,
    /// When true, title is left-aligned after the back button instead of centered.
    #[prop(optional)]
    title_left: bool,
    /// Right-side action icons passed as children.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let back_class = "min-h-11 min-w-11 flex items-center justify-center rounded-full bg-gray-100 text-gray-600 hover:bg-gray-200 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200 dark:focus:ring-offset-gray-900";

    let left = if let Some(left_fn) = left_content {
        left_fn.run()
    } else {
        match (back_href, on_back) {
            (Some(href), Some(on_back)) => view! {
                <A href=href attr:class=back_class attr:aria-label="Back"
                    on:click=move |ev| on_back.run(ev)
                >
                    <span aria-hidden="true">{"\u{2039}"}</span>
                </A>
            }
            .into_any(),
            (Some(href), None) => view! {
                <A href=href attr:class=back_class attr:aria-label="Back">
                    <span aria-hidden="true">{"\u{2039}"}</span>
                </A>
            }
            .into_any(),
            _ => view! { <span></span> }.into_any(),
        }
    };

    let title_class = if title_left {
        "flex-1 text-left text-lg font-bold truncate dark:text-white"
    } else {
        "flex-1 text-center text-lg font-bold truncate dark:text-white"
    };

    let right_class = if pill_group {
        "flex items-center gap-1 shrink-0 bg-gray-100 dark:bg-gray-800 rounded-full px-1"
    } else {
        "flex items-center gap-1 shrink-0 min-w-11 justify-end"
    };

    view! {
        <nav role="navigation" aria-label="Page navigation" class="flex w-full items-center gap-2 mb-4">
            // Left: back button, custom content, or spacer
            <div class="w-11 shrink-0">
                {left}
            </div>
            // Center/Left: title
            <h1 class=title_class>{title}</h1>
            // Right: action icons
            <div class=right_class>
                {children.map(|c| c())}
            </div>
        </nav>
    }
}
