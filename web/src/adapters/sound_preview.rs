use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::{Get, GetUntracked, RwSignal, Set};
use leptos::reactive::owner::LocalStorage;
use wasm_bindgen_futures::spawn_local;

use domain::ports::NotePlayer;
use domain::types::{AmplitudeDB, Frequency, MIDIVelocity, NoteDuration};

use super::audio_context::AudioContextManager;
use super::audio_soundfont::{SF2Preset, WorkletBridge};
use super::note_player::{UnifiedNotePlayer, create_note_player};
use crate::app::{WorkletAssets, ensure_worklet_connected};

/// Manages sound preview playback with auto-stop timer and cancellation.
///
/// Encapsulates AudioContext lifecycle, worklet connection, note player creation,
/// and timer-based auto-reset. Views only need to call `toggle()` and read
/// `playing_signal()`.
#[derive(Clone)]
pub struct SoundPreview {
    player: Rc<RefCell<Option<UnifiedNotePlayer>>>,
    playing: RwSignal<bool>,
    cancelled: Rc<Cell<bool>>,
    duration_secs: f64,
    audio_ctx: Rc<RefCell<AudioContextManager>>,
    audio_needs_gesture: RwSignal<bool>,
    worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>, LocalStorage>,
    worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, LocalStorage>,
    worklet_connecting: RwSignal<bool>,
    sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage>,
}

impl SoundPreview {
    pub fn new(
        duration_secs: f64,
        audio_ctx: Rc<RefCell<AudioContextManager>>,
        audio_needs_gesture: RwSignal<bool>,
        worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>, LocalStorage>,
        worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, LocalStorage>,
        worklet_connecting: RwSignal<bool>,
        sf2_presets: RwSignal<Vec<SF2Preset>, LocalStorage>,
    ) -> Self {
        Self {
            player: Rc::new(RefCell::new(None)),
            playing: RwSignal::new(false),
            cancelled: Rc::new(Cell::new(false)),
            duration_secs,
            audio_ctx,
            audio_needs_gesture,
            worklet_bridge,
            worklet_assets,
            worklet_connecting,
            sf2_presets,
        }
    }

    pub fn playing_signal(&self) -> RwSignal<bool> {
        self.playing
    }

    /// Stops any currently playing preview immediately.
    pub fn stop(&self) {
        if let Some(player) = self.player.borrow().as_ref() {
            player.stop_all();
        }
        self.cancelled.set(true);
        self.playing.set(false);
    }

    /// Toggles preview: stops if playing, starts if not.
    ///
    /// Must be called from a user gesture handler (click) so that
    /// AudioContext creation/resume satisfies browser autoplay policies.
    pub fn toggle(&self, sound_source: &str, frequency: Frequency) {
        if self.playing.get() {
            self.stop();
            return;
        }

        // AudioContext must be created/resumed synchronously within the user gesture
        let ctx_rc = match self.audio_ctx.borrow_mut().get_or_create() {
            Ok(ctx) => {
                let _ = ctx.borrow().resume();
                self.audio_needs_gesture.set(false);
                ctx
            }
            Err(e) => {
                log::error!("Failed to create AudioContext for preview: {e}");
                return;
            }
        };

        self.cancelled.set(false);
        self.playing.set(true);

        let source = sound_source.to_string();
        let ctx_manager = self.audio_ctx.clone();
        let pv = self.clone();
        let worklet_bridge = self.worklet_bridge;
        let worklet_assets = self.worklet_assets;
        let worklet_connecting = self.worklet_connecting;
        let sf2_presets = self.sf2_presets;

        spawn_local(async move {
            if source.starts_with("sf2:") {
                ensure_worklet_connected(
                    &ctx_rc,
                    worklet_bridge,
                    worklet_assets,
                    worklet_connecting,
                    sf2_presets,
                )
                .await;
            }

            // Bail out if user cancelled during async worklet init
            if pv.cancelled.get() {
                return;
            }

            let bridge = worklet_bridge.get_untracked();
            let player = create_note_player(&source, ctx_manager, bridge);
            if let Err(e) = player.play_for_duration(
                frequency,
                NoteDuration::new(pv.duration_secs),
                MIDIVelocity::new(63),
                AmplitudeDB::new(0.0),
            ) {
                log::error!("Preview playback failed: {e}");
                pv.playing.set(false);
                return;
            }

            *pv.player.borrow_mut() = Some(player);

            let cancelled = pv.cancelled.clone();
            let playing = pv.playing;
            let player_ref = pv.player.clone();
            let duration_ms = (pv.duration_secs * 1000.0) as u32;
            TimeoutFuture::new(duration_ms).await;
            if !cancelled.get() {
                playing.set(false);
            }
            *player_ref.borrow_mut() = None;
        });
    }
}
