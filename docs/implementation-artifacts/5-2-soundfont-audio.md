# Story 5.2: SoundFont Audio

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to hear richer instrument sounds during training,
so that training feels more musical and the sound source setting becomes meaningful.

## Acceptance Criteria

1. **Given** the app loads **When** SoundFont loading begins **Then** the SoundFont file and synth WASM module are fetched asynchronously **And** the Start Page is interactive immediately with oscillator fallback **And** no loading indicator is shown — loading is invisible to the user.

2. **Given** the SoundFont has loaded successfully **When** training starts **Then** notes are rendered in real-time by OxiSynth running inside an AudioWorklet **And** the swap from oscillator to SoundFont is silent — no interruption.

3. **Given** the SoundFont fails to load **When** training starts **Then** the OscillatorNotePlayer continues as the fallback **And** no error message is shown (oscillators are a valid sound source).

4. **Given** SoundFontNotePlayer is active **When** a note is played **Then** OxiSynth renders audio samples in real-time on the audio thread via AudioWorkletProcessor **And** the synth holds the note until explicitly told to stop (NoteOff).

5. **Given** SoundFontNotePlayer is active during pitch matching **When** adjustFrequency is called **Then** pitch is adjusted in real-time via MIDI pitch bend events sent to the worklet (within +/-20 cents range).

6. **Given** the SoundFont has been loaded once **When** the app is visited again **Then** the browser cache serves the SoundFont — load is near-instant.

7. **Given** the sound source setting (FR25, from Epic 2) **When** the user selects a different sound source **Then** the SoundFont preset changes accordingly **And** the change takes effect on the next note played.

## Tasks / Subtasks

- [ ] Task 1: Add OxiSynth dependency and SoundFont asset (AC: #4)
  - [ ] 1.1 Add `oxisynth = "0.1.0"` to `web/Cargo.toml`
  - [ ] 1.2 Source a small SF2 file (piano or similar, ideally < 5 MB) and place in `web/assets/soundfont/`
  - [ ] 1.3 Configure Trunk to copy the SF2 asset and worklet JS file to `dist/` — add `<link data-trunk rel="copy-dir" href="web/assets/soundfont" />` or equivalent in `index.html`
  - [ ] 1.4 **Build spike:** Add the dependency and run `trunk build` to confirm OxiSynth compiles to WASM. If `getrandom` feature issues arise, add `[patch]` or feature unification in workspace `Cargo.toml`. Resolve before proceeding.

- [ ] Task 2: Create synth WASM module for AudioWorklet (AC: #4, #5)
  - [ ] 2.1 Create `web/src/synth_worklet/mod.rs` — a separate compilation unit with `#[no_mangle]` C-style exports for the AudioWorklet to call
  - [ ] 2.2 Export `synth_new(sample_rate: f32) -> *mut Synth` — creates a Synth with `SynthDescriptor { sample_rate, reverb_active: false, chorus_active: false, .. }`
  - [ ] 2.3 Export `synth_load_soundfont(synth: *mut Synth, data: *const u8, len: usize) -> i32` — loads SF2 bytes, calls `add_font()`, returns sfont_id or -1 on failure
  - [ ] 2.4 Export `synth_select_program(synth: *mut Synth, sfont_id: i32, bank: u32, preset: u8)`
  - [ ] 2.5 Export `synth_note_on(synth: *mut Synth, key: u8, vel: u8)` and `synth_note_off(synth: *mut Synth, key: u8)`
  - [ ] 2.6 Export `synth_pitch_bend(synth: *mut Synth, bend_value: u16)` — for real-time pitch adjustment (MIDI pitch bend, center = 8192)
  - [ ] 2.7 Export `synth_render(synth: *mut Synth, left: *mut f32, right: *mut f32, len: usize)` — renders `len` samples into the provided buffers by calling `synth.write_f32()`
  - [ ] 2.8 Export `synth_all_notes_off(synth: *mut Synth)` — sends all-notes-off CC to stop all voices
  - [ ] **ALTERNATIVE approach (simpler):** Instead of a separate WASM build, compile the synth exports as part of the main `web` crate WASM binary. The AudioWorklet JS loads the *same* `.wasm` file that Trunk produces and calls the `#[no_mangle]` exports directly. This avoids a second Cargo build target. Investigate feasibility — if Trunk/wasm-bindgen interfere with raw exports, fall back to a separate `synth-worklet` crate in the workspace.

- [ ] Task 3: Create AudioWorklet processor JS file (AC: #4, #5)
  - [ ] 3.1 Create `web/assets/soundfont/synth-processor.js` — the AudioWorkletProcessor subclass (~50 lines of JS)
  - [ ] 3.2 In `constructor()`: receive the compiled `WebAssembly.Module` via `processorOptions`, instantiate it, call `synth_new(sampleRate)` to create the synth instance
  - [ ] 3.3 Handle `port.onmessage` for commands from the main thread:
    - `{ type: 'loadSoundFont', data: ArrayBuffer }` — copy bytes into WASM memory, call `synth_load_soundfont()`, then `synth_select_program()` with default preset
    - `{ type: 'noteOn', key: u8, vel: u8 }` — call `synth_note_on()`
    - `{ type: 'noteOff', key: u8 }` — call `synth_note_off()`
    - `{ type: 'pitchBend', value: u16 }` — call `synth_pitch_bend()`
    - `{ type: 'selectProgram', bank: u32, preset: u8 }` — call `synth_select_program()`
    - `{ type: 'allNotesOff' }` — call `synth_all_notes_off()`
  - [ ] 3.4 In `process(inputs, outputs, parameters)`: call `synth_render(left_ptr, right_ptr, 128)` to fill the 128-sample output buffer each frame. Return `true` to keep the processor alive.
  - [ ] 3.5 Post `{ type: 'ready' }` back to main thread after WASM instantiation succeeds
  - [ ] 3.6 Post `{ type: 'soundFontLoaded' }` after successful SoundFont load

- [ ] Task 4: Implement SoundFontNotePlayer adapter (AC: #2, #4, #5)
  - [ ] 4.1 Create `web/src/adapters/audio_soundfont.rs`
  - [ ] 4.2 Implement `WorkletBridge` struct — manages the AudioWorkletNode and message passing:
    - Holds `web_sys::AudioWorkletNode` and its `MessagePort`
    - Methods: `send_note_on(key, vel)`, `send_note_off(key)`, `send_pitch_bend(value)`, `send_all_notes_off()`, `send_select_program(bank, preset)`
    - Uses `port.post_message()` for all communication
  - [ ] 4.3 Implement `SoundFontNotePlayer` struct holding `Rc<RefCell<WorkletBridge>>`
  - [ ] 4.4 Implement `NotePlayer` trait on `SoundFontNotePlayer`:
    - `play()`: convert `Frequency` to nearest MIDI note + fractional cents, send `noteOn` message, set initial pitch bend for cent offset, return `SoundFontPlaybackHandle`
    - `play_for_duration()`: same as `play()`, but schedule a `noteOff` after `duration` using `gloo_timers::callback::Timeout` (the synth will render the release tail naturally)
    - `stop_all()`: send `allNotesOff` message
  - [ ] 4.5 Implement `SoundFontPlaybackHandle` struct:
    - Holds `Rc<RefCell<WorkletBridge>>`, the MIDI `key`, and the original MIDI note for cent offset calculation
    - `stop()`: sends `noteOff` for this key
    - `adjust_frequency()`: compute cent difference between new frequency and original MIDI note frequency, convert to MIDI pitch bend value (center 8192, range ±200 cents = ±8192), send `pitchBend` message
  - [ ] 4.6 Frequency-to-MIDI conversion helpers (same as before — these are correct):
    ```rust
    fn frequency_to_midi(freq: f64) -> u8
    fn frequency_to_cents_from_midi(freq: f64, midi_note: u8) -> f64
    ```
  - [ ] 4.7 Use velocity parameter — pass to `noteOn` message (unlike oscillator which ignores it)
  - [ ] 4.8 Register module in `web/src/adapters/mod.rs`

- [ ] Task 5: Implement unified NotePlayer enum wrapper (AC: #2, #3)
  - [ ] 5.1 Create `web/src/adapters/note_player.rs` with `UnifiedNotePlayer` enum wrapping `OscillatorNotePlayer` and `SoundFontNotePlayer`
  - [ ] 5.2 Create `UnifiedPlaybackHandle` enum wrapping `OscillatorPlaybackHandle` and `SoundFontPlaybackHandle`
  - [ ] 5.3 Implement `NotePlayer` trait on `UnifiedNotePlayer` delegating to inner variant
  - [ ] 5.4 Implement `PlaybackHandle` trait on `UnifiedPlaybackHandle` delegating to inner variant
  - [ ] 5.5 Add factory function: `create_note_player(sound_source: &str, audio_ctx, worklet_bridge_option) -> UnifiedNotePlayer`

- [ ] Task 6: Async SoundFont + worklet initialization at app startup (AC: #1, #6)
  - [ ] 6.1 In `web/src/app.rs`, add `worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>>` signal (local, starts as `None`)
  - [ ] 6.2 In `spawn_local` block, perform the initialization sequence:
    1. `fetch()` the synth WASM module bytes
    2. Compile the WASM module via `WebAssembly.compile()`
    3. Register the AudioWorklet processor: `audio_ctx.audioWorklet.addModule('synth-processor.js')`
    4. Create `AudioWorkletNode`, passing the compiled WASM module in `processorOptions`
    5. Connect the AudioWorkletNode to `AudioContext.destination`
    6. Wait for `{ type: 'ready' }` message from the worklet
    7. `fetch()` the SF2 file, send bytes to worklet via `{ type: 'loadSoundFont', data }`
    8. Wait for `{ type: 'soundFontLoaded' }` confirmation
  - [ ] 6.3 On success: wrap the AudioWorkletNode/port in `WorkletBridge`, set signal to `Some(bridge)`
  - [ ] 6.4 On failure at any step: log warning, leave signal as `None` (oscillator fallback)
  - [ ] 6.5 Provide the bridge signal as Leptos context for views to consume
  - [ ] 6.6 Browser caching handled automatically by `fetch()` — no special cache logic needed
  - [ ] 6.7 **AudioContext timing:** The AudioWorklet registration must happen after the AudioContext is created. But AudioContext creation requires a user gesture. Two options:
    - Option A: Create AudioContext eagerly in `spawn_local` (may be suspended until user gesture), register worklet, then resume on first training click
    - Option B: Defer worklet registration until first training view mount (when AudioContext is created via user gesture). This delays SoundFont availability but is safer for Safari/iOS.
    - Prefer Option A with a note that the AudioContext may start suspended — the existing resume-on-gesture pattern in training views handles this.

- [ ] Task 7: Update training views to use UnifiedNotePlayer (AC: #2, #3)
  - [ ] 7.1 In `comparison_view.rs`: replace `OscillatorNotePlayer::new()` with `create_note_player()` that checks worklet bridge signal and sound source setting
  - [ ] 7.2 In `pitch_matching_view.rs`: same replacement. Change `tunable_handle` type from `Option<OscillatorPlaybackHandle>` to `Option<UnifiedPlaybackHandle>`
  - [ ] 7.3 Both views: read `peach.sound_source` from `LocalStorageSettings` to decide which variant to create
  - [ ] 7.4 If worklet bridge signal is `None` (still loading or failed), create `OscillatorNotePlayer` variant regardless of setting

- [ ] Task 8: Update Settings view sound source dropdown (AC: #7)
  - [ ] 8.1 Add SoundFont preset options to the sound source `<select>` element (currently only "Sine Oscillator")
  - [ ] 8.2 Parse `SoundSourceID` format: `"oscillator:sine"` for oscillator, `"sf2:<bank>:<preset>"` for SoundFont presets
  - [ ] 8.3 Sound source change auto-saves to `peach.sound_source` in localStorage (existing pattern)
  - [ ] 8.4 Change takes effect on next note played (next training session start or next comparison)

- [ ] Task 9: Verify and test (AC: all)
  - [ ] 9.1 `cargo test -p domain` — confirm no regressions
  - [ ] 9.2 `trunk build` — confirm WASM compilation with new OxiSynth dependency
  - [ ] 9.3 `cargo clippy` — zero warnings
  - [ ] 9.4 Manual browser test: app loads, start page interactive immediately, training works with oscillator while SF2 loads
  - [ ] 9.5 Manual browser test: after worklet is ready, start training — hear SoundFont audio with real-time synthesis (no pre-rendering stutter)
  - [ ] 9.6 Manual browser test: pitch matching — slider adjusts pitch smoothly in real-time via pitch bend
  - [ ] 9.7 Manual browser test: hold a note in pitch matching — note sustains indefinitely until commit (no buffer cutoff)
  - [ ] 9.8 Manual browser test: change sound source in Settings, start training — new sound plays
  - [ ] 9.9 Manual browser test: disable network, reload — SF2 served from browser cache
  - [ ] 9.10 Manual browser test: block SF2 URL — training works with oscillator fallback, no error shown

## Dev Notes

### Architecture — AudioWorklet Real-Time Synthesis

**Why not pre-render to AudioBuffer?** The original approach (render OxiSynth samples into an AudioBuffer, play via AudioBufferSourceNode) has fundamental problems:
1. **Main thread blocking:** Rendering 44,100 samples/second synchronously blocks the UI. A 1-second note = 44K iterations of `read_next()` on the main thread.
2. **Indefinite duration impossible:** `NotePlayer::play()` must sustain a note indefinitely (pitch matching). AudioBufferSourceNode plays a finite buffer — you'd need to guess a duration or loop, creating audible artifacts.
3. **Not how a synth works:** OxiSynth is a real-time streaming synthesizer. Its `read_next()`/`write_f32()` API is pull-based — send NoteOn, pull samples continuously, send NoteOff, synth renders the release tail naturally. Pre-rendering throws away this capability.

**The correct architecture: AudioWorklet**

```
Main Thread (Leptos/WASM)                Audio Thread (AudioWorklet)
┌──────────────────────────┐             ┌─────────────────────────────┐
│ SoundFontNotePlayer      │  postMsg    │ synth-processor.js          │
│   - sends NoteOn/Off    ─┼────────────>│   - OxiSynth WASM instance │
│   - sends PitchBend     ─┼────────────>│   - calls synth_render()   │
│   - sends AllNotesOff   ─┼────────────>│     in process() callback  │
│                          │             │   - fills 128-sample output │
│ WorkletBridge            │  postMsg    │     buffers per frame       │
│   - wraps MessagePort   ─┼────────────>│   - ~375 calls/sec at 48kHz│
└──────────────────────────┘             └─────────────────────────────┘
```

The OxiSynth `Synth` instance lives **inside the AudioWorklet** thread. The main thread never touches audio samples. Communication is via `postMessage`:
- Main → Worklet: NoteOn, NoteOff, PitchBend, LoadSoundFont, SelectProgram, AllNotesOff
- Worklet → Main: Ready, SoundFontLoaded

This gives:
- **Zero main thread blocking** — synthesis runs on the audio thread
- **Indefinite note duration** — synth sustains until NoteOff, naturally
- **Real-time pitch adjustment** — pitch bend messages, processed at audio rate
- **Natural release tails** — handled by the synth automatically
- **<50ms audio onset** — AudioWorklet latency is ~2.7ms per 128-sample chunk

### AudioWorklet + WASM Integration Pattern

The browser's AudioWorklet scope cannot use `fetch()` or `import()`. The established pattern (used by every Rust+AudioWorklet project) is:

1. **Main thread:** Fetch and compile the WASM module
2. **Pass via processorOptions:** Send the compiled `WebAssembly.Module` to the worklet constructor
3. **Worklet:** Instantiate the module synchronously, call exported functions from `process()`

The AudioWorkletProcessor is written in **JavaScript** (~50 lines). This is required because `wasm-bindgen` cannot subclass JS classes (`AudioWorkletProcessor.process()` must be overridden). This is the standard approach — not a limitation.

```javascript
// synth-processor.js (sketch)
class SynthProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    const wasmModule = options.processorOptions.wasmModule;
    // Instantiate WASM synchronously
    this.wasm = new WebAssembly.Instance(wasmModule, imports);
    this.synth = this.wasm.exports.synth_new(sampleRate);
    this.port.onmessage = (e) => this.handleMessage(e.data);
    this.port.postMessage({ type: 'ready' });
  }

  handleMessage(msg) {
    switch (msg.type) {
      case 'loadSoundFont':
        // Copy bytes into WASM memory, call synth_load_soundfont
        break;
      case 'noteOn':
        this.wasm.exports.synth_note_on(this.synth, msg.key, msg.vel);
        break;
      case 'noteOff':
        this.wasm.exports.synth_note_off(this.synth, msg.key);
        break;
      case 'pitchBend':
        this.wasm.exports.synth_pitch_bend(this.synth, msg.value);
        break;
      // ... other commands
    }
  }

  process(inputs, outputs, parameters) {
    const output = outputs[0];
    const left = output[0];
    const right = output[1];
    // Write f32 samples into WASM memory, copy out
    this.wasm.exports.synth_render(this.synth, this.leftPtr, this.rightPtr, left.length);
    left.set(new Float32Array(this.wasm.exports.memory.buffer, this.leftPtr, left.length));
    right.set(new Float32Array(this.wasm.exports.memory.buffer, this.rightPtr, right.length));
    return true;
  }
}

registerProcessor('synth-processor', SynthProcessor);
```

### WASM Module for the AudioWorklet

**Key decision: separate build vs shared binary.**

The AudioWorklet needs raw `#[no_mangle]` C-style exports (no wasm-bindgen). Two approaches:

**Option A — Separate `synth-worklet` crate (recommended):**
Add a `synth-worklet/` crate to the workspace. It depends on `oxisynth` and exports C-style functions. Build with `cargo build --target wasm32-unknown-unknown -p synth-worklet --release`. Copy the `.wasm` output to `web/assets/soundfont/`. This keeps the worklet WASM small and avoids interfering with Trunk's build.

**Option B — Exports in the main `web` crate:**
Add `#[no_mangle]` exports alongside wasm-bindgen code. Risk: Trunk/wasm-bindgen may strip or mangle these exports. Would need testing.

**Option A is safer** and produces a small, focused WASM binary (~200KB + oxisynth code) that loads quickly in the worklet.

### Synth Worklet Crate Structure

```
synth-worklet/
├── Cargo.toml          # depends on oxisynth only
└── src/
    └── lib.rs          # #[no_mangle] exports
```

```rust
// synth-worklet/src/lib.rs (sketch)
use oxisynth::{MidiEvent, SoundFont, Synth, SynthDescriptor};
use std::io::Cursor;

#[no_mangle]
pub extern "C" fn synth_new(sample_rate: f32) -> *mut Synth {
    let desc = SynthDescriptor {
        sample_rate,
        reverb_active: false,
        chorus_active: false,
        ..Default::default()
    };
    match Synth::new(desc) {
        Ok(synth) => Box::into_raw(Box::new(synth)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn synth_note_on(synth: *mut Synth, key: u8, vel: u8) {
    if let Some(synth) = synth.as_mut() {
        let _ = synth.send_event(MidiEvent::NoteOn { channel: 0, key, vel });
    }
}

#[no_mangle]
pub unsafe extern "C" fn synth_render(
    synth: *mut Synth, left: *mut f32, right: *mut f32, len: usize,
) {
    if let Some(synth) = synth.as_mut() {
        let left_buf = std::slice::from_raw_parts_mut(left, len);
        let right_buf = std::slice::from_raw_parts_mut(right, len);
        // Use write_f32 or read_next to fill buffers
        for i in 0..len {
            let (l, r) = synth.read_next();
            left_buf[i] = l;
            right_buf[i] = r;
        }
    }
}
// ... other exports
```

### Associated Type Problem and Solution

Unchanged from original — the `NotePlayer` trait uses an associated type `Handle`:
```rust
pub trait NotePlayer {
    type Handle: PlaybackHandle;
    fn play(&self, ...) -> Result<Self::Handle, AudioError>;
    // ...
}
```

This makes `dyn NotePlayer` impossible. The solution is the **wrapper enum** pattern:
```rust
pub enum UnifiedNotePlayer {
    Oscillator(OscillatorNotePlayer),
    SoundFont(SoundFontNotePlayer),
}
pub enum UnifiedPlaybackHandle {
    Oscillator(OscillatorPlaybackHandle),
    SoundFont(SoundFontPlaybackHandle),
}
```
Implement `NotePlayer` on `UnifiedNotePlayer` with `type Handle = UnifiedPlaybackHandle`, delegating each method to the inner variant.

### Frequency-to-MIDI and Pitch Bend Conversion

The domain uses `Frequency` (Hz) but OxiSynth needs MIDI note numbers and pitch bend:

```rust
fn frequency_to_midi(freq: f64) -> u8 {
    (69.0 + 12.0 * (freq / 440.0).log2()).round() as u8
}

fn frequency_to_cents_from_midi(freq: f64, midi_note: u8) -> f64 {
    let midi_freq = 440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
    1200.0 * (freq / midi_freq).log2()
}

/// Convert cent offset to MIDI pitch bend value.
/// MIDI pitch bend range: 0-16383, center = 8192.
/// Default pitch bend range is ±200 cents (2 semitones).
fn cents_to_pitch_bend(cents: f64) -> u16 {
    let bend = 8192.0 + (cents / 200.0) * 8192.0;
    (bend.clamp(0.0, 16383.0)) as u16
}
```

For `play()`: convert frequency to nearest MIDI note, send NoteOn, then send pitch bend for any fractional cent offset.

For `adjust_frequency()` in pitch matching: compute cent difference between new frequency and the originally-played MIDI note's frequency, convert to pitch bend, send to worklet. The ±20 cents range for pitch matching is well within the ±200 cents default pitch bend range.

### SoundFont Loading — Async Init Sequence

```
1. Main thread: fetch('/assets/soundfont/synth-worklet.wasm')  → ArrayBuffer
2. Main thread: WebAssembly.compile(bytes)                      → WebAssembly.Module
3. Main thread: audioCtx.audioWorklet.addModule('synth-processor.js')
4. Main thread: new AudioWorkletNode(audioCtx, 'synth-processor', { processorOptions: { wasmModule } })
5. Wait for: worklet posts { type: 'ready' }
6. Main thread: fetch('/assets/soundfont/piano.sf2')            → ArrayBuffer
7. Main thread: node.port.postMessage({ type: 'loadSoundFont', data: sf2ArrayBuffer })
8. Wait for: worklet posts { type: 'soundFontLoaded' }
9. Main thread: set worklet_bridge signal to Some(bridge)
```

If any step fails, log a warning and leave the signal as `None` — oscillator fallback activates automatically.

### AudioContext Lifecycle Consideration

AudioWorklet registration (`audioWorklet.addModule()`) requires an existing AudioContext. The current app creates AudioContext lazily on first training button click (user gesture requirement for Safari/iOS).

**Approach:** In the `spawn_local` init block, create an AudioContext for worklet registration. This context may start in "suspended" state (no user gesture yet). That's fine — the worklet can be registered and the WASM+SoundFont loaded while suspended. When the user clicks to start training, the existing `AudioContextManager.get_or_create()` should return or resume this same context.

**Important:** The AudioContextManager currently creates a new AudioContext. It must be updated to accept a pre-created context OR the worklet init must use the same AudioContextManager. The simplest approach: create the AudioContext in `app.rs` during init, pass it to AudioContextManager (add a `set_context()` method or construct with an existing context), then register the worklet on it.

### Concrete Type Coupling in PitchMatchingView

**Critical:** `pitch_matching_view.rs:95` stores the tunable handle with a concrete type:
```rust
let tunable_handle: Rc<RefCell<Option<OscillatorPlaybackHandle>>> = ...;
```
This MUST change to:
```rust
let tunable_handle: Rc<RefCell<Option<UnifiedPlaybackHandle>>> = ...;
```

### Sound Source Setting Wiring

Currently disconnected: `peach.sound_source` exists in localStorage (set by Settings view) but training views never read it. They always create `OscillatorNotePlayer`.

The factory function bridges this gap:
```rust
pub fn create_note_player(
    sound_source: &str,
    audio_ctx: Rc<RefCell<AudioContextManager>>,
    worklet_bridge: Option<Rc<RefCell<WorkletBridge>>>,
) -> UnifiedNotePlayer {
    match (sound_source, worklet_bridge) {
        (s, Some(bridge)) if s.starts_with("sf2:") => {
            // Parse bank:preset from sound_source, send selectProgram to worklet
            UnifiedNotePlayer::SoundFont(SoundFontNotePlayer::new(bridge))
        }
        _ => UnifiedNotePlayer::Oscillator(OscillatorNotePlayer::new(audio_ctx)),
    }
}
```

### Performance Considerations

- **Audio onset:** AudioWorklet processes 128-sample chunks at ~48kHz = ~2.7ms per chunk. After NoteOn, the first audible output arrives within one chunk. Well under the 50ms NFR1 requirement.
- **Message latency:** `postMessage` between main thread and AudioWorklet has negligible latency (sub-millisecond, same-process transfer). For pitch bend during slider adjustment, this is imperceptible.
- **SF2 file size:** Keep the SoundFont small. A piano-only SF2 can be 2-5 MB. The architecture doc mentions 31 MB but that's excessive for web — find a smaller alternative.
- **Worklet WASM size:** The synth-worklet crate produces a small binary (oxisynth + thin export layer). Estimate ~200-400KB gzipped.
- **Memory:** The Synth instance and SoundFont data live in the worklet's WASM memory. The main thread only holds the MessagePort reference — negligible memory.

### What Already Exists (DO NOT Rebuild)

| Component | File | Status |
|-----------|------|--------|
| `NotePlayer` trait + `PlaybackHandle` trait | `domain/src/ports.rs` | Complete |
| `AudioError` enum | `domain/src/ports.rs` | Complete with 5 variants |
| `OscillatorNotePlayer` | `web/src/adapters/audio_oscillator.rs` | Complete — reference implementation |
| `AudioContextManager` | `web/src/adapters/audio_context.rs` | Complete — may need minor update for pre-created context |
| `SoundSourceID` type (default `"sf2:8:80"`) | `domain/src/types/sound_source.rs` | Complete |
| Sound source dropdown in Settings | `web/src/components/settings_view.rs:93-94` | Has single "Sine Oscillator" option |
| `peach.sound_source` localStorage key | `web/src/adapters/localstorage_settings.rs` | Read via `get_string()` |
| Async init pattern (IndexedDB) | `web/src/app.rs` | Template for worklet init |
| `SendWrapper` for non-Send context | `web/src/app.rs` | Pattern to follow for worklet bridge context |

### What Must NOT Change

- `domain/src/ports.rs` — the `NotePlayer` and `PlaybackHandle` traits must remain unchanged
- `domain/` crate — zero changes. No browser dependencies.
- `web/src/adapters/audio_oscillator.rs` — keep working as-is for fallback
- Existing training behavior — oscillator-based training must continue working identically

### Project Structure Notes

**New files:**
- `synth-worklet/Cargo.toml` — new workspace crate for AudioWorklet WASM module
- `synth-worklet/src/lib.rs` — `#[no_mangle]` C-style exports wrapping OxiSynth
- `web/assets/soundfont/synth-processor.js` — AudioWorkletProcessor JS file (~50 lines)
- `web/assets/soundfont/<name>.sf2` — SoundFont file asset
- `web/src/adapters/audio_soundfont.rs` — SoundFontNotePlayer + WorkletBridge
- `web/src/adapters/note_player.rs` — UnifiedNotePlayer/UnifiedPlaybackHandle enum wrapper

**Modified files:**
- `Cargo.toml` (workspace) — add `synth-worklet` member
- `web/Cargo.toml` — add `oxisynth` (for types only, if needed), new `web-sys` features (`AudioWorklet`, `AudioWorkletNode`, `AudioWorkletNodeOptions`, `MessagePort`)
- `web/src/adapters/mod.rs` — register new modules
- `web/src/adapters/audio_context.rs` — possibly add `set_context()` or constructor accepting existing AudioContext
- `web/src/app.rs` — async worklet init + SoundFont loading + context provider
- `web/src/components/comparison_view.rs` — use UnifiedNotePlayer instead of OscillatorNotePlayer
- `web/src/components/pitch_matching_view.rs` — use UnifiedNotePlayer, update handle types
- `web/src/components/settings_view.rs` — add SoundFont options to sound source dropdown
- `index.html` — add `<link data-trunk rel="copy-dir" ...>` for assets

**No changes to:**
- Any `domain/` crate files
- `web/src/adapters/audio_oscillator.rs` (keep as-is)
- `web/src/bridge.rs` (no observer changes)
- `web/src/interval_codes.rs` (unrelated)

### Build Notes

The synth-worklet WASM must be built separately from Trunk:
```bash
cargo build --target wasm32-unknown-unknown -p synth-worklet --release
# Copy to assets:
cp target/wasm32-unknown-unknown/release/synth_worklet.wasm web/assets/soundfont/
```

Consider adding a Trunk `[[hooks]]` pre-build step to automate this, or document it as a manual step. The worklet WASM should be committed to the repo (it's a build artifact but small enough) or built in CI.

### Known Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| OxiSynth `getrandom` dependency fails in WASM | Task 1.4 build spike catches this early. Use feature unification or `[patch]`. |
| AudioWorklet not available (very old browsers) | Oscillator fallback. AudioWorklet is supported in all modern browsers since 2018. |
| WASM module too large for worklet instantiation | synth-worklet crate is minimal. Compile with `wasm-opt -Oz`. |
| Safari AudioContext restrictions | Create AudioContext in init, resume on user gesture. Existing pattern handles this. |
| Pitch bend range mismatch | OxiSynth default pitch bend range is ±2 semitones (±200 cents). ±20 cents for pitch matching is well within range. |

### References

- [Source: docs/planning-artifacts/epics.md#Story 5.2 — Acceptance criteria and BDD]
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture — hybrid oscillator+OxiSynth approach]
- [Source: docs/planning-artifacts/architecture.md#Implementation Sequence — step 8, SoundFontNotePlayer]
- [Source: docs/planning-artifacts/architecture.md#Performance Requirements — NFR1 audio onset < 50ms]
- [Source: docs/planning-artifacts/ux-design-specification.md#Loading States — non-blocking SoundFont loading]
- [Source: docs/planning-artifacts/ux-design-specification.md#Error States — silent SoundFont failure]
- [Source: docs/planning-artifacts/ux-design-specification.md#Experience Principles — sound first, pixels second]
- [Source: docs/project-context.md#Technology Stack — OxiSynth, web-sys]
- [Source: docs/project-context.md#Web Audio Edge Cases — SoundFont loading non-blocking]
- [Source: domain/src/ports.rs — NotePlayer trait, PlaybackHandle trait, AudioError]
- [Source: domain/src/types/sound_source.rs — SoundSourceID default "sf2:8:80"]
- [Source: web/src/adapters/audio_oscillator.rs — OscillatorNotePlayer reference pattern]
- [Source: web/src/adapters/audio_context.rs — AudioContextManager shared singleton]
- [Source: web/src/app.rs — async init pattern with IndexedDB, context providers]
- [Source: web/src/components/comparison_view.rs:62 — note player creation point]
- [Source: web/src/components/pitch_matching_view.rs:59 — note player creation point]
- [Source: web/src/components/settings_view.rs:93-94 — sound source dropdown]
- [Source: docs/ios-reference/domain-blueprint.md#9.5 — SoundSourceProvider port]
- [Source: https://crates.io/crates/oxisynth — v0.1.0, pure Rust, WASM compatible]
- [Source: https://docs.rs/oxisynth — Synth, SoundFont, MidiEvent API]
- [Source: https://github.com/PolyMeilex/OxiSynth — "built with WASM in mind", live demo at oxisynth.netlify.app]
- [Source: https://whoisryosuke.com/blog/2025/processing-web-audio-with-rust-and-wasm — Rust+WASM AudioWorklet pattern]
- [Source: https://github.com/PaulBatchelor/rust-wasm-audioworklet — minimal Rust AudioWorklet example]

## Previous Story Intelligence

### From Story 5.1 (Interval Training Mode)

- Story 5.1 was entirely web-layer work — zero domain changes. Story 5.2 is also web-layer only (plus new synth-worklet crate).
- `LocalStorageSettings::get_selected_intervals()` is a static method called with `::` not `.` — same applies to `get_string("peach.sound_source")`.
- Code review caught duplicated code between ComparisonView and PitchMatchingView — extracted shared `parse_intervals_param()`. Apply same DRY principle: the `create_note_player()` factory should be shared, not duplicated.
- Code review added screen reader announcements for accessibility — consider if sound source changes need any announcement.
- All 293+ domain tests pass, trunk build and clippy clean as of story 5.1 completion.

### From Story 4.4 (Pitch Matching Persistence)

- "Mirror with targeted changes" pattern: when making parallel changes to ComparisonView and PitchMatchingView, copy the pattern from one to the other with minimal targeted modifications.
- Two-borrow pattern in pitch_matching_view.rs: pre-check state then mutably borrow.
- Code review caught redundant `.abs()` and ambiguous log messages — be precise.
- Never silently discard Results: `let _ = fallible()` is forbidden.

## Git Intelligence

Recent commits (newest first):
```
6438c72 Apply code review fixes for story 5.1 and mark as done
e595890 Implement story 5.1 Interval Training Mode
0ff658f Add story 5.1 Interval Training Mode and mark as ready-for-dev
6d29983 Apply code review fixes for story 4.4 and mark epic 4 as done
895d216 Implement story 4.4 Pitch Matching Persistence and fix pitch matching bugs
```

Convention: story creation commit -> implementation commit -> code review fixes commit. Commit messages use imperative mood, reference the story number.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
