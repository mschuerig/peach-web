use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_router::components::A;
use send_wrapper::SendWrapper;

use domain::{MIDINote, PerceptualProfile};

use super::profile_visualization::{
    NoteData, BLACK_KEY_HEIGHT, BLACK_KEY_WIDTH, IS_WHITE, KEYBOARD_HEIGHT, KEYBOARD_Y, MIDI_MAX,
    MIDI_MIN, VIEWBOX_HEIGHT, VIEWBOX_WIDTH, WHITE_KEY_WIDTH, build_band_segments,
    build_mean_segments, note_x_center, white_key_x,
};

#[component]
pub fn ProfilePreview() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile context");
    let is_profile_loaded: RwSignal<bool> =
        use_context().expect("is_profile_loaded context");

    let profile_rc = profile.clone();

    view! {
        {move || {
            // Keyboard geometry (static — always renders)
            let white_keys: Vec<f64> = (MIDI_MIN..=MIDI_MAX)
                .filter_map(white_key_x)
                .collect();

            let black_keys: Vec<f64> = (MIDI_MIN..=MIDI_MAX)
                .filter(|&m| !IS_WHITE[(m % 12) as usize])
                .map(|m| note_x_center(m) - BLACK_KEY_WIDTH / 2.0)
                .collect();

            // Profile-dependent data
            let (aria_label, band_segments, mean_segments) = if is_profile_loaded.get() {
                let prof = profile_rc.borrow();
                let notes: Vec<NoteData> = (MIDI_MIN..=MIDI_MAX)
                    .map(|i| {
                        let stat = prof.note_stats(MIDINote::new(i));
                        NoteData {
                            midi: i,
                            mean: stat.mean(),
                            std_dev: stat.std_dev(),
                            is_trained: stat.is_trained(),
                        }
                    })
                    .collect();
                let trained_count = notes.iter().filter(|n| n.is_trained).count();
                let avg_threshold = prof.overall_mean();
                drop(prof);

                let label = if trained_count > 0 {
                    format!(
                        "Your pitch profile. Average threshold: {:.1} cents. Click to view details.",
                        avg_threshold.unwrap_or(0.0)
                    )
                } else {
                    "Your pitch profile. Click to view details.".to_string()
                };
                let bands = build_band_segments(&notes);
                let means = build_mean_segments(&notes);
                (label, bands, means)
            } else {
                (
                    "Your pitch profile. Click to view details.".to_string(),
                    Vec::new(),
                    Vec::new(),
                )
            };

            view! {
                <A href="/profile"
                    attr:aria-label=aria_label
                    attr:class="block rounded-lg overflow-hidden focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900">
                    <svg
                        viewBox=format!("0 0 {VIEWBOX_WIDTH} {VIEWBOX_HEIGHT}")
                        width="100%"
                        role="img"
                        aria-hidden="true"
                    >
                        // White keys
                        {white_keys
                            .iter()
                            .map(|x| {
                                view! {
                                    <rect
                                        x=format!("{x}")
                                        y=format!("{KEYBOARD_Y}")
                                        width=format!("{WHITE_KEY_WIDTH}")
                                        height=format!("{KEYBOARD_HEIGHT}")
                                        fill="var(--pv-key-white)"
                                        stroke="var(--pv-key-border)"
                                        stroke-width="0.5"
                                    />
                                }
                            })
                            .collect::<Vec<_>>()}

                        // Black keys
                        {black_keys
                            .iter()
                            .map(|x| {
                                view! {
                                    <rect
                                        x=format!("{x}")
                                        y=format!("{KEYBOARD_Y}")
                                        width=format!("{BLACK_KEY_WIDTH}")
                                        height=format!("{BLACK_KEY_HEIGHT}")
                                        fill="var(--pv-key-black)"
                                    />
                                }
                            })
                            .collect::<Vec<_>>()}

                        // Confidence band
                        {band_segments
                            .iter()
                            .map(|path_d| {
                                view! {
                                    <path d=path_d.clone() fill="var(--pv-band-fill)" />
                                }
                            })
                            .collect::<Vec<_>>()}

                        // Mean line
                        {mean_segments
                            .iter()
                            .map(|path_d| {
                                view! {
                                    <path
                                        d=path_d.clone()
                                        fill="none"
                                        stroke="var(--pv-band-stroke)"
                                        stroke-width="1.5"
                                    />
                                }
                            })
                            .collect::<Vec<_>>()}
                    </svg>
                </A>
            }
            .into_any()
        }}
    }
}
