use leptos::prelude::*;
use wasm_bindgen::JsCast;

const FINE_STEP: f64 = 0.05;

fn value_from_pointer(ev: &web_sys::PointerEvent) -> f64 {
    let Some(target) = ev.current_target() else {
        return 0.0;
    };
    let el: web_sys::Element = target.unchecked_into();
    let rect = el.get_bounding_client_rect();
    let height = rect.height();
    if height <= 0.0 {
        return 0.0;
    }
    let relative = ((ev.client_y() as f64 - rect.top()) / height).clamp(0.0, 1.0);
    1.0 - 2.0 * relative
}

#[component]
pub fn VerticalPitchSlider(
    /// Whether the slider accepts input. When false, appears dimmed.
    enabled: Signal<bool>,
    /// Called continuously during drag with normalized value [-1.0, +1.0].
    /// Up = positive (sharper), down = negative (flatter).
    on_change: Callback<f64>,
    /// Called when the user releases the slider (commit). Also fired on Enter/Space.
    on_commit: Callback<f64>,
    /// Increment this signal to reset the slider to center (0.0).
    reset_trigger: Signal<u32>,
) -> impl IntoView {
    let value = RwSignal::new(0.0_f64);
    let dragging = RwSignal::new(false);

    // Reset to center when reset_trigger changes
    Effect::new(move || {
        let _ = reset_trigger.get();
        value.set(0.0);
    });

    // Reset to center when enabled transitions false → true
    let prev_enabled = RwSignal::new(false);
    Effect::new(move || {
        let is_enabled = enabled.get();
        if is_enabled && !prev_enabled.get_untracked() {
            value.set(0.0);
        }
        prev_enabled.set(is_enabled);
    });

    // Thumb position: +1.0 → 0%, 0.0 → 50%, -1.0 → 100%
    let thumb_pct = move || (1.0 - value.get()) / 2.0 * 100.0;

    let on_pointerdown = move |ev: web_sys::PointerEvent| {
        if !enabled.get_untracked() {
            return;
        }
        let new_value = value_from_pointer(&ev);
        value.set(new_value);
        dragging.set(true);
        if let Some(target) = ev.current_target() {
            let el: web_sys::Element = target.unchecked_into();
            if let Err(e) = el.set_pointer_capture(ev.pointer_id()) {
                web_sys::console::warn_1(&format!("setPointerCapture failed: {e:?}").into());
            }
        }
        on_change.run(new_value);
    };

    let on_pointermove = move |ev: web_sys::PointerEvent| {
        if !dragging.get_untracked() || !enabled.get_untracked() {
            return;
        }
        let new_value = value_from_pointer(&ev);
        value.set(new_value);
        on_change.run(new_value);
    };

    let on_pointerup = move |ev: web_sys::PointerEvent| {
        if !dragging.get_untracked() {
            return;
        }
        dragging.set(false);
        if let Some(target) = ev.current_target() {
            let el: web_sys::Element = target.unchecked_into();
            if let Err(e) = el.release_pointer_capture(ev.pointer_id()) {
                web_sys::console::warn_1(
                    &format!("releasePointerCapture failed: {e:?}").into(),
                );
            }
        }
        if enabled.get_untracked() {
            on_commit.run(value.get_untracked());
        }
    };

    let on_pointercancel = move |_ev: web_sys::PointerEvent| {
        dragging.set(false);
        // Pointer capture is implicitly released by the browser on cancel
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if !enabled.get_untracked() {
            return;
        }
        let key = ev.key();
        match key.as_str() {
            "ArrowUp" => {
                ev.prevent_default();
                let new_value = (value.get_untracked() + FINE_STEP).clamp(-1.0, 1.0);
                value.set(new_value);
                on_change.run(new_value);
            }
            "ArrowDown" => {
                ev.prevent_default();
                let new_value = (value.get_untracked() - FINE_STEP).clamp(-1.0, 1.0);
                value.set(new_value);
                on_change.run(new_value);
            }
            "Enter" | " " => {
                ev.prevent_default();
                on_commit.run(value.get_untracked());
            }
            _ => {}
        }
    };

    view! {
        <div
            class=move || {
                let base = "relative mx-auto flex h-[60vh] w-16 items-center justify-center select-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900";
                if enabled.get() {
                    if dragging.get() {
                        format!("{base} touch-none cursor-grabbing opacity-100")
                    } else {
                        format!("{base} touch-none cursor-grab opacity-100")
                    }
                } else {
                    format!("{base} pointer-events-none cursor-not-allowed opacity-40")
                }
            }
            role="slider"
            aria-label="Pitch adjustment"
            aria-orientation="vertical"
            aria-valuemin="-1"
            aria-valuemax="1"
            aria-valuenow=move || format!("{:.2}", value.get())
            tabindex="0"
            on:pointerdown=on_pointerdown
            on:pointermove=on_pointermove
            on:pointerup=on_pointerup
            on:pointercancel=on_pointercancel
            on:keydown=on_keydown
        >
            <div class="h-full w-2 rounded-full bg-gray-200 dark:bg-gray-700" />
            <div
                class="pointer-events-none absolute left-1/2 h-12 w-12 -translate-x-1/2 -translate-y-1/2 rounded-full bg-indigo-600 shadow-md dark:bg-indigo-400"
                style=move || format!("top: {:.1}%", thumb_pct())
            />
        </div>
    }
}
