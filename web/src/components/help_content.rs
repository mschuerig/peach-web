use leptos::prelude::*;

/// A single help section with a title and body text.
/// Body text supports simple inline markdown: **bold** and *italic*.
pub struct HelpSection {
    pub title: &'static str,
    pub body: &'static str,
}

/// Process simple inline markdown in trusted static text.
/// Converts **bold** to <strong>, *italic* to <em>, and \n\n to <br><br>.
fn process_markdown(text: &str) -> String {
    let mut result = text.to_string();

    // Replace \n\n with <br><br>
    result = result.replace("\n\n", "<br><br>");

    // Replace **bold** with <strong>bold</strong>
    // Use a simple loop-based approach to handle multiple occurrences
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let bold_text = &result[start + 2..start + 2 + end].to_string();
            let replacement = format!("<strong>{bold_text}</strong>");
            result = format!(
                "{}{}{}",
                &result[..start],
                replacement,
                &result[start + 2 + end + 2..]
            );
        } else {
            break;
        }
    }

    // Replace *italic* with <em>italic</em>
    // Must run after bold processing to avoid conflicts
    while let Some(start) = result.find('*') {
        if let Some(end) = result[start + 1..].find('*') {
            let italic_text = &result[start + 1..start + 1 + end].to_string();
            let replacement = format!("<em>{italic_text}</em>");
            result = format!(
                "{}{}{}",
                &result[..start],
                replacement,
                &result[start + 1 + end + 1..]
            );
        } else {
            break;
        }
    }

    result
}

/// Renders a list of help sections with titles and processed body text.
#[component]
pub fn HelpContent(
    sections: &'static [HelpSection],
    #[prop(optional)] use_h2: bool,
) -> impl IntoView {
    view! {
        <div class="space-y-5">
            {sections.iter().map(|section| {
                let body_html = process_markdown(section.body);
                let heading = if use_h2 {
                    view! { <h2 class="text-lg font-semibold dark:text-white">{section.title}</h2> }.into_any()
                } else {
                    view! { <h3 class="text-lg font-semibold dark:text-white">{section.title}</h3> }.into_any()
                };
                view! {
                    <div>
                        {heading}
                        <div
                            class="mt-2 text-gray-700 dark:text-gray-300"
                            inner_html=body_html
                        />
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// A modal dialog that displays help content.
/// Uses the native HTML <dialog> element for accessibility (focus trapping, Escape to close, backdrop).
#[component]
pub fn HelpModal(
    title: &'static str,
    sections: &'static [HelpSection],
    is_open: RwSignal<bool>,
    #[prop(optional)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();

    // Open/close the dialog when the signal changes
    Effect::new(move || {
        let open = is_open.get();
        if let Some(dialog) = dialog_ref.get() {
            if open {
                let _ = dialog.show_modal();
            } else {
                dialog.close();
            }
        }
    });

    let handle_close = move |_| {
        // Only set the signal — the Effect will call dialog.close(),
        // which fires the native close event, which calls on_close.
        is_open.set(false);
    };

    // Handle native dialog close event (e.g. Escape key)
    let on_dialog_close = move |_: leptos::ev::Event| {
        is_open.set(false);
        if let Some(cb) = on_close {
            cb.run(());
        }
    };

    view! {
        <dialog
            node_ref=dialog_ref
            role="dialog"
            aria-modal="true"
            aria-label=title
            on:close=on_dialog_close
            class="rounded-lg p-0 max-w-lg w-full mx-auto bg-white text-gray-900 backdrop:bg-black/50 dark:bg-gray-800 dark:text-gray-100 max-h-[85vh]"
        >
            <div class="flex flex-col h-full p-6">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-xl font-bold">{title}</h2>
                    <button
                        on:click=handle_close
                        class="min-h-11 min-w-11 flex items-center justify-center rounded-lg text-gray-600 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        "Done"
                    </button>
                </div>
                <div class="overflow-y-auto flex-1">
                    <HelpContent sections=sections />
                </div>
            </div>
        </dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_markdown_bold() {
        assert_eq!(
            process_markdown("This is **bold** text"),
            "This is <strong>bold</strong> text"
        );
    }

    #[test]
    fn test_process_markdown_italic() {
        assert_eq!(
            process_markdown("This is *italic* text"),
            "This is <em>italic</em> text"
        );
    }

    #[test]
    fn test_process_markdown_bold_and_italic() {
        assert_eq!(
            process_markdown("**bold** and *italic*"),
            "<strong>bold</strong> and <em>italic</em>"
        );
    }

    #[test]
    fn test_process_markdown_newlines() {
        assert_eq!(process_markdown("first\n\nsecond"), "first<br><br>second");
    }

    #[test]
    fn test_process_markdown_multiple_bold() {
        assert_eq!(
            process_markdown("**a** then **b**"),
            "<strong>a</strong> then <strong>b</strong>"
        );
    }

    #[test]
    fn test_process_markdown_no_markdown() {
        assert_eq!(process_markdown("plain text"), "plain text");
    }
}
