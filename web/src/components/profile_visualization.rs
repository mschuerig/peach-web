use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use domain::{MIDINote, PerceptualProfile};

/// Per-note data extracted from PerceptualProfile for rendering.
struct NoteData {
    midi: u8,
    mean: f64,
    std_dev: f64,
    is_trained: bool,
}

// Piano range: MIDI 21 (A0) to MIDI 108 (C8) — 88 keys, 52 white + 36 black.
const MIDI_MIN: u8 = 21;
const MIDI_MAX: u8 = 108;
const WHITE_KEY_WIDTH: f64 = 10.0;
const BLACK_KEY_WIDTH: f64 = 6.0;
const KEYBOARD_Y: f64 = 140.0;
const KEYBOARD_HEIGHT: f64 = 60.0;
const BLACK_KEY_HEIGHT: f64 = 39.0;
const CHART_TOP: f64 = 10.0;
const CHART_BOTTOM: f64 = 135.0;
const MAX_CENTS: f64 = 200.0;
const VIEWBOX_WIDTH: f64 = 520.0;
const VIEWBOX_HEIGHT: f64 = 210.0;

/// Note positions within an octave in white-key units.
/// C=0, C#=0.5, D=1, D#=1.5, E=2, F=3, F#=3.5, G=4, G#=4.5, A=5, A#=5.5, B=6
const NOTE_OFFSETS: [f64; 12] = [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0];

/// Whether a note (by chromatic index) is a white key.
const IS_WHITE: [bool; 12] = [
    true, false, true, false, true, true, false, true, false, true, false, true,
];

/// MIDI 21 (A0) absolute position: octave 1 * 7 + offset 5 = 12.
const FIRST_KEY_POSITION: f64 = 12.0;

/// Compute the x-center for any MIDI note on the keyboard layout.
fn note_x_center(midi: u8) -> f64 {
    let octave = (midi / 12) as f64;
    let note = (midi % 12) as usize;
    let abs_position = octave * 7.0 + NOTE_OFFSETS[note];
    (abs_position - FIRST_KEY_POSITION) * WHITE_KEY_WIDTH + WHITE_KEY_WIDTH / 2.0
}

/// Compute the white key's x position (left edge) relative to keyboard start.
/// Returns None for black keys.
/// Precondition: midi >= MIDI_MIN (21). Lower values cause u32 underflow.
fn white_key_x(midi: u8) -> Option<f64> {
    if !IS_WHITE[(midi % 12) as usize] {
        return None;
    }
    let octave = midi / 12;
    let note = midi % 12;
    let white_notes: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
    let pos = white_notes.iter().position(|&n| n == note)?;
    let abs_index = octave as u32 * 7 + pos as u32;
    // Subtract MIDI 21's white key index (12) for relative positioning
    Some((abs_index - 12) as f64 * WHITE_KEY_WIDTH)
}

/// Map a cent value to Y coordinate (inverted: lower cents = higher on screen).
fn cents_to_y(cents: f64) -> f64 {
    let clamped = cents.clamp(0.0, MAX_CENTS);
    CHART_TOP + (clamped / MAX_CENTS) * (CHART_BOTTOM - CHART_TOP)
}

/// Build SVG path data for confidence band segments (no interpolation across gaps).
fn build_band_segments(notes: &[NoteData]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut run: Vec<&NoteData> = Vec::new();

    for note in notes {
        if note.is_trained {
            run.push(note);
        } else if !run.is_empty() {
            segments.push(band_path(&run));
            run.clear();
        }
    }
    if !run.is_empty() {
        segments.push(band_path(&run));
    }
    segments
}

/// Build SVG path for a single band segment (consecutive trained notes).
fn band_path(run: &[&NoteData]) -> String {
    if run.len() == 1 {
        // Single note: thin rectangle
        let n = run[0];
        let cx = note_x_center(n.midi);
        let upper = cents_to_y((n.mean - n.std_dev).max(0.0));
        let lower = cents_to_y(n.mean + n.std_dev);
        let hw = 2.0;
        return format!(
            "M {:.1},{:.1} L {:.1},{:.1} L {:.1},{:.1} L {:.1},{:.1} Z",
            cx - hw,
            upper,
            cx + hw,
            upper,
            cx + hw,
            lower,
            cx - hw,
            lower
        );
    }

    let mut d = String::new();
    // Upper edge left to right: mean - std_dev
    for (i, n) in run.iter().enumerate() {
        let x = note_x_center(n.midi);
        let y = cents_to_y((n.mean - n.std_dev).max(0.0));
        if i == 0 {
            d.push_str(&format!("M {:.1},{:.1}", x, y));
        } else {
            d.push_str(&format!(" L {:.1},{:.1}", x, y));
        }
    }
    // Lower edge right to left: mean + std_dev
    for n in run.iter().rev() {
        let x = note_x_center(n.midi);
        let y = cents_to_y(n.mean + n.std_dev);
        d.push_str(&format!(" L {:.1},{:.1}", x, y));
    }
    d.push_str(" Z");
    d
}

/// Build SVG path data for mean line segments.
fn build_mean_segments(notes: &[NoteData]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut run: Vec<&NoteData> = Vec::new();

    for note in notes {
        if note.is_trained {
            run.push(note);
        } else if !run.is_empty() {
            segments.push(mean_path(&run));
            run.clear();
        }
    }
    if !run.is_empty() {
        segments.push(mean_path(&run));
    }
    segments
}

/// Build SVG path for a mean line across a run of trained notes.
fn mean_path(run: &[&NoteData]) -> String {
    if run.len() == 1 {
        // Single note: short horizontal tick mark
        let n = run[0];
        let x = note_x_center(n.midi);
        let y = cents_to_y(n.mean);
        let hw = 2.0;
        return format!("M {:.1},{:.1} L {:.1},{:.1}", x - hw, y, x + hw, y);
    }

    let mut d = String::new();
    for (i, n) in run.iter().enumerate() {
        let x = note_x_center(n.midi);
        let y = cents_to_y(n.mean);
        if i == 0 {
            d.push_str(&format!("M {:.1},{:.1}", x, y));
        } else {
            d.push_str(&format!(" L {:.1},{:.1}", x, y));
        }
    }
    d
}

/// Octave boundary notes to label: C2 through C7.
const OCTAVE_LABEL_MIDIS: [u8; 6] = [36, 48, 60, 72, 84, 96];

#[component]
pub fn ProfileVisualization() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile context");
    let is_profile_loaded: RwSignal<bool> =
        use_context().expect("is_profile_loaded context");

    let profile_rc = profile.clone();

    view! {
        {move || {
            // Keyboard geometry (static — always renders regardless of profile load state)
            let white_keys: Vec<f64> = (MIDI_MIN..=MIDI_MAX)
                .filter_map(white_key_x)
                .collect();

            let black_keys: Vec<f64> = (MIDI_MIN..=MIDI_MAX)
                .filter(|&m| !IS_WHITE[(m % 12) as usize])
                .map(|m| note_x_center(m) - BLACK_KEY_WIDTH / 2.0)
                .collect();

            let octave_labels: Vec<(f64, String)> = OCTAVE_LABEL_MIDIS
                .iter()
                .map(|&m| (note_x_center(m), MIDINote::new(m).name()))
                .collect();

            // Profile-dependent data (guarded to avoid RefCell conflicts during hydration)
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

                let label = if trained_count == 0 {
                    "Perceptual profile: no training data yet".to_string()
                } else {
                    format!(
                        "Perceptual profile: average detection threshold {:.1} cents across {} trained notes",
                        avg_threshold.unwrap_or(0.0),
                        trained_count
                    )
                };
                let bands = build_band_segments(&notes);
                let means = build_mean_segments(&notes);
                (label, bands, means)
            } else {
                (
                    "Perceptual profile: no training data yet".to_string(),
                    Vec::new(),
                    Vec::new(),
                )
            };
            let title_text = aria_label.clone();

            view! {
                <svg
                    viewBox=format!("0 0 {VIEWBOX_WIDTH} {VIEWBOX_HEIGHT}")
                    width="100%"
                    class="mt-6"
                    role="img"
                    aria-label=aria_label
                >
                    <title>{title_text}</title>

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

                    // Octave labels
                    {octave_labels
                        .iter()
                        .map(|(x, name)| {
                            view! {
                                <text
                                    x=format!("{x}")
                                    y=format!("{}", VIEWBOX_HEIGHT - 2.0)
                                    text-anchor="middle"
                                    font-size="7"
                                    fill="var(--pv-label-color)"
                                >
                                    {name.clone()}
                                </text>
                            }
                        })
                        .collect::<Vec<_>>()}
                </svg>
            }
            .into_any()
        }}
    }
}
