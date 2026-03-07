use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::MessagePort;

use crate::adapters::audio_context::AudioContextManager;

// Newtype wrappers for RwSignal<bool> contexts — Leptos uses types for context
// lookup, so multiple RwSignal<bool> values would shadow each other.
#[derive(Clone, Copy)]
pub struct IsProfileLoaded(pub RwSignal<bool>);

#[derive(Clone, Copy)]
pub struct WorkletConnecting(pub RwSignal<bool>);

#[derive(Clone, Copy)]
pub struct AudioNeedsGesture(pub RwSignal<bool>);
use crate::adapters::audio_soundfont::{SF2Preset, WorkletBridge};
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::components::{
    PitchComparisonView, InfoView, PitchMatchingView, ProfileView, SettingsView, StartPage,
};
use domain::types::MIDINote;
use domain::{PerceptualProfile, ProgressTimeline, ThresholdTimeline, TrendAnalyzer};

#[derive(Clone, Debug, PartialEq)]
pub enum SoundFontLoadStatus {
    Fetching,
    Ready,
    Failed(String),
}

#[component]
pub fn App() -> impl IntoView {
    // SendWrapper is required because Leptos 0.8 provide_context requires Send + Sync,
    // but Rc<RefCell<T>> doesn't implement those traits. SendWrapper is safe here because
    // WASM is single-threaded — the Send + Sync bounds are never actually exercised.
    let profile = SendWrapper::new(Rc::new(RefCell::new(PerceptualProfile::new())));
    let audio_ctx_manager = SendWrapper::new(Rc::new(RefCell::new(AudioContextManager::new())));
    let trend_analyzer = SendWrapper::new(Rc::new(RefCell::new(TrendAnalyzer::new())));
    let timeline = SendWrapper::new(Rc::new(RefCell::new(ThresholdTimeline::new())));
    let progress_timeline = SendWrapper::new(Rc::new(RefCell::new(ProgressTimeline::new())));
    let is_profile_loaded = RwSignal::new(false);
    let db_store = RwSignal::new_local(None::<Rc<IndexedDbStore>>);
    let worklet_bridge = RwSignal::new_local(None::<Rc<RefCell<WorkletBridge>>>);
    let sf2_presets = RwSignal::new_local(Vec::<SF2Preset>::new());
    let worklet_assets = RwSignal::new_local(None::<Rc<WorkletAssets>>);
    let worklet_connecting = RwSignal::new(false);
    let audio_needs_gesture = RwSignal::new(false);

    // Always fetch SF2 so presets are available in settings.
    // Initial status is Fetching — views that need SF2 show loading indicator.
    let sf2_load_status = RwSignal::new(SoundFontLoadStatus::Fetching);

    provide_context(sf2_load_status);
    provide_context(profile.clone());
    provide_context(audio_ctx_manager.clone());
    provide_context(trend_analyzer.clone());
    provide_context(timeline.clone());
    provide_context(progress_timeline.clone());
    provide_context(IsProfileLoaded(is_profile_loaded));
    provide_context(db_store);
    provide_context(worklet_bridge);
    provide_context(sf2_presets);
    provide_context(worklet_assets);
    provide_context(WorkletConnecting(worklet_connecting));
    provide_context(AudioNeedsGesture(audio_needs_gesture));

    // Async hydration — runs after mount
    let profile_for_hydration = Rc::clone(&*profile);
    let trend_for_hydration = Rc::clone(&*trend_analyzer);
    let timeline_for_hydration = Rc::clone(&*timeline);
    let ptl_for_hydration = Rc::clone(&*progress_timeline);

    spawn_local(async move {
        match IndexedDbStore::open().await {
            Ok(store) => {
                let store = Rc::new(store);

                let comparison_records = match store.fetch_all_pitch_comparisons().await {
                    Ok(records) => {
                        let mut prof = profile_for_hydration.borrow_mut();
                        let mut trend = trend_for_hydration.borrow_mut();
                        let mut tl = timeline_for_hydration.borrow_mut();
                        let mut skipped = 0u32;

                        for record in &records {
                            let note = match MIDINote::try_new(record.reference_note) {
                                Ok(n) => n,
                                Err(_) => {
                                    skipped += 1;
                                    continue;
                                }
                            };

                            prof.update(
                                note,
                                domain::Cents::new(record.cent_offset.abs()),
                                record.is_correct,
                            );

                            trend.push(record.cent_offset.abs());

                            tl.push(
                                &record.timestamp,
                                record.cent_offset.abs(),
                                record.is_correct,
                                record.reference_note,
                            );
                        }

                        if skipped > 0 {
                            log::warn!("Skipped {skipped} records with invalid MIDI note values during hydration");
                        }
                        log::info!("Profile comparison hydrated from {} records", records.len() - skipped as usize);
                        records
                    }
                    Err(e) => {
                        log::error!("Failed to fetch records for hydration: {e}");
                        Vec::new()
                    }
                };

                // Pitch matching hydration
                let matching_records = match store.fetch_all_pitch_matchings().await {
                    Ok(records) => {
                        let mut prof = profile_for_hydration.borrow_mut();
                        let mut skipped = 0u32;

                        for record in &records {
                            let note = match MIDINote::try_new(record.reference_note) {
                                Ok(n) => n,
                                Err(_) => {
                                    skipped += 1;
                                    continue;
                                }
                            };

                            prof.update_matching(note, domain::Cents::new(record.user_cent_error));
                        }

                        if skipped > 0 {
                            log::warn!("Skipped {skipped} pitch matching records with invalid MIDI note values during hydration");
                        }
                        log::info!(
                            "Profile pitch matching hydrated from {} records",
                            records.len() - skipped as usize
                        );
                        records
                    }
                    Err(e) => {
                        log::error!("Failed to fetch pitch matching records for hydration: {e}");
                        Vec::new()
                    }
                };

                // ProgressTimeline hydration — rebuild from all records
                {
                    let now = js_sys::Date::now() / 1000.0;
                    ptl_for_hydration.borrow_mut().rebuild(
                        &comparison_records,
                        &matching_records,
                        now,
                    );
                    log::info!("ProgressTimeline hydrated");
                }

                db_store.set(Some(store));
            }
            Err(e) => {
                log::error!("Failed to open IndexedDB: {e}");
            }
        }

        is_profile_loaded.set(true);
    });

    // Phase 1: Fetch and compile worklet assets (no AudioContext needed)
    // Always fetch so SF2 presets are available in settings even for oscillator users.
    // Training buttons are only gated (disabled) when the user has SF2 selected.
    spawn_local(async move {
        match fetch_worklet_assets().await {
            Ok(assets) => {
                log::info!("Worklet assets fetched (WASM + SF2)");
                let presets = parse_sf2_preset_headers(&assets.sf2_buffer);
                log::info!("Parsed {} SF2 preset headers", presets.len());
                sf2_presets.set(presets);
                worklet_assets.set(Some(Rc::new(assets)));
                sf2_load_status.set(SoundFontLoadStatus::Ready);
            }
            Err(e) => {
                log::warn!("Failed to fetch worklet assets (oscillator fallback): {e}");
                sf2_load_status.set(SoundFontLoadStatus::Failed(e));
            }
        }
    });

    view! {
        <Router>

            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:p-3 focus:bg-white focus:text-black dark:focus:bg-gray-900 dark:focus:text-white"
            >
                "Skip to main content"
            </a>
            <main id="main-content" class="min-h-screen bg-white dark:bg-gray-900">
                <div class="mx-auto max-w-lg px-4">
                    <Routes fallback=|| {
                        view! {
                            <div class="py-12 text-center">
                                <h1 class="text-2xl font-bold dark:text-white">"Page not found"</h1>
                                <A
                                    href="/"
                                    attr:class="mt-4 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                                >
                                    "Back to Start"
                                </A>
                            </div>
                        }
                    }>
                        <Route path=path!("/") view=StartPage />
                        <Route path=path!("/training/comparison") view=PitchComparisonView />
                        <Route path=path!("/training/pitch-matching") view=PitchMatchingView />
                        <Route path=path!("/profile") view=ProfileView />
                        <Route path=path!("/settings") view=SettingsView />
                        <Route path=path!("/info") view=InfoView />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

/// Pre-fetched worklet assets (compiled WASM module + SF2 data) stored for Phase 2 connection.
pub struct WorkletAssets {
    pub wasm_module: JsValue,
    pub sf2_buffer: JsValue,
}

/// Phase 1: Fetch and compile worklet assets without creating an AudioContext.
///
/// Compiles WASM on the main thread so the compiled Module can be sent to the
/// AudioWorklet via postMessage. This is more compatible than compiling inside
/// the worklet scope (recommended pattern for iOS Safari 16.4+).
async fn fetch_worklet_assets() -> Result<WorkletAssets, String> {
    // Fetch and compile the synth WASM module on the main thread
    let wasm_response = JsFuture::from(
        web_sys::window()
            .ok_or("no window")?
            .fetch_with_str("/soundfont/synth_worklet.wasm"),
    )
    .await
    .map_err(|e| format!("fetch synth WASM failed: {e:?}"))?;
    let wasm_response: web_sys::Response = wasm_response
        .dyn_into()
        .map_err(|_| "fetch didn't return Response")?;
    let wasm_buffer = JsFuture::from(
        wasm_response
            .array_buffer()
            .map_err(|e| format!("arrayBuffer failed: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("arrayBuffer promise failed: {e:?}"))?;
    let wasm_module = JsFuture::from(js_sys::WebAssembly::compile(&wasm_buffer))
        .await
        .map_err(|e| format!("WASM compile failed: {e:?}"))?;

    // Fetch SF2 file
    let sf2_response = JsFuture::from(
        web_sys::window()
            .ok_or("no window")?
            .fetch_with_str("/GeneralUser-GS.sf2"),
    )
    .await
    .map_err(|e| format!("fetch SF2 failed: {e:?}"))?;
    let sf2_response: web_sys::Response = sf2_response
        .dyn_into()
        .map_err(|_| "fetch SF2 didn't return Response")?;
    let sf2_buffer = JsFuture::from(
        sf2_response
            .array_buffer()
            .map_err(|e| format!("SF2 arrayBuffer failed: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("SF2 arrayBuffer promise failed: {e:?}"))?;

    Ok(WorkletAssets {
        wasm_module,
        sf2_buffer,
    })
}

/// Phase 2: Connect worklet using a running AudioContext and pre-fetched assets.
///
/// Called from training views after `ensure_running()` succeeds.
pub async fn connect_worklet(
    ctx_rc: &Rc<RefCell<web_sys::AudioContext>>,
    assets: &WorkletAssets,
) -> Result<(WorkletBridge, Vec<SF2Preset>), String> {
    // Register processor JS via addModule (cache-busting query param for unhashed asset)
    let add_module_promise = {
        let ctx = ctx_rc.borrow();
        let audio_worklet = ctx
            .audio_worklet()
            .map_err(|e| format!("audioWorklet unavailable: {e:?}"))?;
        audio_worklet
            .add_module("/soundfont/synth-processor.js?v=2")
            .map_err(|e| format!("addModule failed: {e:?}"))?
    };
    JsFuture::from(add_module_promise)
        .await
        .map_err(|e| format!("addModule promise failed: {e:?}"))?;

    // Create AudioWorkletNode with WASM module in processorOptions
    let (node, port) = {
        let ctx = ctx_rc.borrow();
        let options = web_sys::AudioWorkletNodeOptions::new();
        let processor_options = js_sys::Object::new();
        js_sys::Reflect::set(&processor_options, &"wasmModule".into(), &assets.wasm_module)
            .map_err(|e| format!("set processorOptions failed: {e:?}"))?;
        options.set_processor_options(Some(&processor_options));
        let output_channels = js_sys::Array::new();
        output_channels.push(&JsValue::from(2));
        options.set_output_channel_count(&output_channels);

        let node =
            web_sys::AudioWorkletNode::new_with_options(&ctx, "synth-processor", &options)
                .map_err(|e| format!("AudioWorkletNode creation failed: {e:?}"))?;
        node.connect_with_audio_node(&ctx.destination())
            .map_err(|e| format!("connect to destination failed: {e:?}"))?;
        let port = node.port().map_err(|e| format!("no port: {e:?}"))?;
        (node, port)
    };

    // Wait for 'ready' message from worklet (after async WASM instantiation)
    let _ = wait_for_worklet_message(&port, "ready").await?;

    // Send SF2 data to worklet
    let load_msg = js_sys::Object::new();
    js_sys::Reflect::set(&load_msg, &"type".into(), &"loadSoundFont".into())
        .map_err(|e| format!("{e:?}"))?;
    js_sys::Reflect::set(&load_msg, &"data".into(), &assets.sf2_buffer)
        .map_err(|e| format!("{e:?}"))?;
    port.post_message(&load_msg)
        .map_err(|e| format!("postMessage loadSoundFont failed: {e:?}"))?;

    // Wait for 'soundFontLoaded' and extract preset list
    let sf2_msg_data = wait_for_worklet_message(&port, "soundFontLoaded").await?;
    let presets = parse_sf2_presets(&sf2_msg_data);

    Ok((WorkletBridge::new(node), presets))
}

/// Ensures the worklet bridge is connected and available.
///
/// If no bridge exists yet and assets are available, connects the worklet.
/// If another caller is already connecting, waits up to 5 seconds.
/// After this returns, callers should read `worklet_bridge.get_untracked()`
/// to obtain the bridge (may be `None` if connection failed or timed out).
pub async fn ensure_worklet_connected(
    ctx_rc: &Rc<RefCell<web_sys::AudioContext>>,
    worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>, leptos::reactive::owner::LocalStorage>,
    worklet_assets: RwSignal<Option<Rc<WorkletAssets>>, leptos::reactive::owner::LocalStorage>,
    worklet_connecting: RwSignal<bool>,
    sf2_presets: RwSignal<Vec<SF2Preset>, leptos::reactive::owner::LocalStorage>,
) {
    if worklet_bridge.get_untracked().is_some() {
        return;
    }

    if !worklet_connecting.get_untracked()
        && let Some(assets) = worklet_assets.get_untracked()
    {
        worklet_connecting.set(true);
        match connect_worklet(ctx_rc, &assets).await {
            Ok((bridge, presets)) => {
                let bridge_rc = Rc::new(RefCell::new(bridge));
                worklet_bridge.set(Some(bridge_rc));
                sf2_presets.set(presets);
            }
            Err(e) => {
                log::warn!("SoundFont worklet connect failed (oscillator fallback): {e}");
                // Temporary: surface error on mobile where console is inaccessible
                let _ = web_sys::window()
                    .and_then(|w| w.alert_with_message(&format!("Worklet error: {e}")).ok());
            }
        }
        worklet_connecting.set(false);
        return;
    }

    // Another caller is connecting — wait with timeout
    let mut wait_iters = 0u32;
    while worklet_connecting.get_untracked() {
        wait_iters += 1;
        if wait_iters > 100 {
            log::warn!("Worklet connection wait timed out after 5s");
            return;
        }
        gloo_timers::future::TimeoutFuture::new(50).await;
    }
}

/// Wait for a specific message type from the worklet port.
/// Returns the message data JsValue on success.
async fn wait_for_worklet_message(
    port: &MessagePort,
    expected_type: &str,
) -> Result<JsValue, String> {
    let expected = expected_type.to_string();
    let (sender, receiver) = futures_channel::oneshot::channel::<Result<JsValue, String>>();
    let sender = Rc::new(RefCell::new(Some(sender)));

    let callback = {
        let sender = Rc::clone(&sender);
        let expected = expected.clone();
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            let data = event.data();
            let msg_type = js_sys::Reflect::get(&data, &"type".into())
                .ok()
                .and_then(|v| v.as_string());
            if let Some(ref t) = msg_type {
                if t == &expected {
                    if let Some(s) = sender.borrow_mut().take() {
                        let _ = s.send(Ok(data));
                    }
                } else if t == "error" {
                    let err_msg = js_sys::Reflect::get(&data, &"message".into())
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_else(|| "unknown worklet error".to_string());
                    if let Some(s) = sender.borrow_mut().take() {
                        let _ = s.send(Err(err_msg));
                    }
                }
            }
        })
    };

    port.set_onmessage(Some(callback.as_ref().unchecked_ref()));

    let result = receiver
        .await
        .map_err(|_| format!("channel closed waiting for '{expected}'"))?;

    // Clear the handler (will be replaced or re-set as needed)
    port.set_onmessage(None);

    // Keep closure alive until we're done
    drop(callback);

    result
}

/// Parse SF2 preset headers directly from raw SF2 bytes (RIFF/sfbk format).
///
/// Navigates the RIFF chunk structure to find the "pdta" LIST → "phdr" sub-chunk,
/// then reads preset headers (38 bytes each: 20-byte name, u16 preset, u16 bank, …).
/// The last entry is always "EOP" (sentinel) and is excluded.
fn parse_sf2_preset_headers(buffer: &JsValue) -> Vec<SF2Preset> {
    let array = js_sys::Uint8Array::new(buffer);
    let bytes = array.to_vec();

    if bytes.len() < 12 {
        return Vec::new();
    }

    // Verify RIFF/sfbk header
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"sfbk" {
        log::warn!("SF2 parse: not a valid RIFF/sfbk file");
        return Vec::new();
    }

    // Find pdta LIST chunk, then phdr sub-chunk within it
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let chunk_id = &bytes[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;

        if chunk_id == b"LIST" && pos + 12 <= bytes.len() {
            let list_type = &bytes[pos + 8..pos + 12];
            if list_type == b"pdta" {
                // Search inside pdta for phdr
                let pdta_start = pos + 12;
                let pdta_end = (pos + 8 + chunk_size).min(bytes.len());
                let mut inner = pdta_start;
                while inner + 8 <= pdta_end {
                    let sub_id = &bytes[inner..inner + 4];
                    let sub_size = u32::from_le_bytes([
                        bytes[inner + 4],
                        bytes[inner + 5],
                        bytes[inner + 6],
                        bytes[inner + 7],
                    ]) as usize;

                    if sub_id == b"phdr" {
                        return parse_phdr_entries(&bytes[inner + 8..], sub_size);
                    }

                    // Advance to next sub-chunk (pad to even boundary)
                    inner += 8 + sub_size + (sub_size % 2);
                }
                break;
            }
        }

        // Advance to next chunk (pad to even boundary)
        pos += 8 + chunk_size + (chunk_size % 2);
    }

    Vec::new()
}

/// Parse individual preset header records from the phdr chunk data.
fn parse_phdr_entries(data: &[u8], size: usize) -> Vec<SF2Preset> {
    const PHDR_SIZE: usize = 38;
    let count = size / PHDR_SIZE;
    if count <= 1 {
        return Vec::new(); // Only the EOP sentinel
    }

    let mut presets = Vec::with_capacity(count - 1);
    for i in 0..count - 1 {
        // Last entry is EOP sentinel — skip it
        let offset = i * PHDR_SIZE;
        if offset + PHDR_SIZE > data.len() {
            break;
        }

        let name_bytes = &data[offset..offset + 20];
        let name = String::from_utf8_lossy(
            &name_bytes[..name_bytes.iter().position(|&b| b == 0).unwrap_or(20)],
        )
        .to_string();

        let program = u16::from_le_bytes([data[offset + 20], data[offset + 21]]);
        let bank = u16::from_le_bytes([data[offset + 22], data[offset + 23]]);

        presets.push(SF2Preset {
            name,
            bank,
            program,
        });
    }
    presets
}

/// Parse SF2 preset list from the `soundFontLoaded` message data.
fn parse_sf2_presets(data: &JsValue) -> Vec<SF2Preset> {
    let presets_array = match js_sys::Reflect::get(data, &"presets".into())
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
    {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut presets = Vec::new();
    for i in 0..presets_array.length() {
        let item = presets_array.get(i);
        let bank = js_sys::Reflect::get(&item, &"bank".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u16;
        let program = js_sys::Reflect::get(&item, &"program".into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u16;
        let name = js_sys::Reflect::get(&item, &"name".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        presets.push(SF2Preset {
            name,
            bank,
            program,
        });
    }
    presets
}
