# Story 5.2: SoundFont Audio

Status: done

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

- [x] Task 1: Add OxiSynth dependency and SoundFont asset (AC: #4)
  - [x] 1.1 Add `oxisynth = "0.1.0"` to separate `synth-worklet/Cargo.toml` crate (Option A from Dev Notes)
  - [x] 1.2 Use existing GeneralUser GS SF2 from `bin/download-sf2.sh` (cached at `.cache/GeneralUser-GS.sf2`, 32 MB)
  - [x] 1.3 Configure Trunk to copy SF2 and worklet assets to `dist/` via `<link data-trunk>` in `index.html` + pre-build hook in `Trunk.toml`
  - [x] 1.4 Build spike passed: OxiSynth compiles cleanly to wasm32-unknown-unknown, 222KB output

- [x] Task 2: Create synth WASM module for AudioWorklet (AC: #4, #5)
  - [x] 2.1 Created `synth-worklet/src/lib.rs` as separate workspace crate with `#[no_mangle]` C-style exports
  - [x] 2.2 Export `synth_new(sample_rate: f32) -> *mut Synth`
  - [x] 2.3 Export `synth_load_soundfont(synth, data, len) -> i32`
  - [x] 2.4 Export `synth_select_program(synth, bank, preset)`
  - [x] 2.5 Export `synth_note_on(synth, key, vel)` and `synth_note_off(synth, key)`
  - [x] 2.6 Export `synth_pitch_bend(synth, bend_value)`
  - [x] 2.7 Export `synth_render(synth, left, right, len)` using `write_f32()`
  - [x] 2.8 Export `synth_all_notes_off(synth)`
  - [x] Also exports `alloc`/`dealloc` for WASM memory management
  - [x] Used Option A (separate crate) — Trunk pre-build hook handles compilation and asset copy

- [x] Task 3: Create AudioWorklet processor JS file (AC: #4, #5)
  - [x] 3.1 Created `web/assets/soundfont/synth-processor.js` (~80 lines)
  - [x] 3.2 Constructor receives `WebAssembly.Module` via `processorOptions`, instantiates synchronously
  - [x] 3.3 Handles all message types: loadSoundFont, noteOn, noteOff, pitchBend, selectProgram, allNotesOff
  - [x] 3.4 `process()` calls `synth_render()` to fill 128-sample output buffers per frame
  - [x] 3.5 Posts `{ type: 'ready' }` after WASM instantiation
  - [x] 3.6 Posts `{ type: 'soundFontLoaded' }` after successful SF2 load

- [x] Task 4: Implement SoundFontNotePlayer adapter (AC: #2, #4, #5)
  - [x] 4.1 Created `web/src/adapters/audio_soundfont.rs`
  - [x] 4.2 Implemented `WorkletBridge` with all message-passing methods
  - [x] 4.3 Implemented `SoundFontNotePlayer` holding `Rc<RefCell<WorkletBridge>>`
  - [x] 4.4 Implemented `NotePlayer` trait with frequency-to-MIDI conversion and initial pitch bend
  - [x] 4.5 Implemented `SoundFontPlaybackHandle` with `stop()` and `adjust_frequency()` via pitch bend
  - [x] 4.6 Frequency-to-MIDI helpers: `frequency_to_midi()`, `frequency_to_cents_from_midi()`, `cents_to_pitch_bend()`
  - [x] 4.7 Velocity parameter passed to noteOn message
  - [x] 4.8 Registered module in `web/src/adapters/mod.rs`

- [x] Task 5: Implement unified NotePlayer enum wrapper (AC: #2, #3)
  - [x] 5.1 Created `web/src/adapters/note_player.rs` with `UnifiedNotePlayer` enum
  - [x] 5.2 Created `UnifiedPlaybackHandle` enum
  - [x] 5.3 Implemented `NotePlayer` trait delegating to inner variant
  - [x] 5.4 Implemented `PlaybackHandle` trait delegating to inner variant
  - [x] 5.5 Added `create_note_player()` factory with sound source parsing and program selection

- [x] Task 6: Async SoundFont + worklet initialization at app startup (AC: #1, #6)
  - [x] 6.1 Added `worklet_bridge: RwSignal<Option<Rc<RefCell<WorkletBridge>>>>` in `app.rs`
  - [x] 6.2 Implemented full async init sequence in separate `spawn_local` (parallel with hydration)
  - [x] 6.3 On success: wraps in `WorkletBridge`, sets signal
  - [x] 6.4 On failure: logs warning, leaves `None` (oscillator fallback)
  - [x] 6.5 Bridge signal provided as Leptos context
  - [x] 6.6 Browser caching via standard `fetch()` — no special logic
  - [x] 6.7 Used Option A: AudioContext created eagerly, may start suspended, training views resume on gesture

- [x] Task 7: Update training views to use UnifiedNotePlayer (AC: #2, #3)
  - [x] 7.1 `comparison_view.rs`: uses `create_note_player()` with worklet bridge signal and sound source setting
  - [x] 7.2 `pitch_matching_view.rs`: same, with `UnifiedPlaybackHandle` for tunable handle
  - [x] 7.3 Both views read `peach.sound_source` from localStorage
  - [x] 7.4 Falls back to oscillator when bridge is `None`

- [x] Task 8: Update Settings view sound source dropdown (AC: #7)
  - [x] 8.1 Added 8 SoundFont preset options (Piano, Electric Piano, Nylon Guitar, Flute, Oboe, Clarinet, Violin, Synth Square)
  - [x] 8.2 Uses `sf2:<bank>:<preset>` format for SoundFont, `oscillator:sine` for oscillator
  - [x] 8.3 Existing auto-save pattern preserved
  - [x] 8.4 Change takes effect on next training session start

- [x] Task 9: Verify and test (AC: all)
  - [x] 9.1 `cargo test -p domain` — 293 tests pass, zero regressions
  - [x] 9.2 `trunk build` — succeeds with synth-worklet pre-build hook
  - [x] 9.3 `cargo clippy` — zero warnings across all three crates
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

Claude Opus 4.6

### Debug Log References

- OxiSynth 0.1.0 compiles cleanly to wasm32-unknown-unknown with no getrandom issues
- synth_worklet.wasm output: 222KB (release, opt-level z, LTO)
- GeneralUser GS SF2: 32MB, downloaded via `bin/download-sf2.sh` to `.cache/`
- Trunk pre-build hook handles synth-worklet build + asset copy automatically
- Clippy clean across all 3 crates (domain, web, synth-worklet)
- No domain crate changes — 293 tests pass unchanged

### Completion Notes List

- **Task 1:** Created `synth-worklet` workspace crate (Option A from Dev Notes). SF2 sourced from existing iOS project download script. Trunk pre-build hook automates synth WASM build. Added `.cache/` and `web/assets/soundfont/synth_worklet.wasm` to .gitignore.
- **Task 2:** All 8 C-style exports implemented plus `alloc`/`dealloc` for WASM memory management. Used `write_f32()` for sample rendering. `SoundFont::load()` uses `Cursor` to wrap raw bytes.
- **Task 3:** AudioWorkletProcessor JS (~80 lines) handles WASM instantiation, message routing, and real-time audio rendering in `process()`.
- **Task 4:** `WorkletBridge` wraps MessagePort communication. `SoundFontNotePlayer` implements `NotePlayer` trait with frequency-to-MIDI conversion and pitch bend for fractional cent offsets. `play_for_duration` uses `gloo_timers::callback::Timeout` for noteOff scheduling.
- **Task 5:** `UnifiedNotePlayer`/`UnifiedPlaybackHandle` enums delegate to Oscillator or SoundFont variants. `create_note_player()` factory parses `sf2:bank:preset` format and sends selectProgram.
- **Task 6:** Async init runs in parallel with hydration. Structured to avoid holding RefCell borrows across await points (clippy `await_holding_refcell_ref`). Uses `futures_channel::oneshot` for worklet message waiting.
- **Task 7:** Both training views now use `create_note_player()` with worklet bridge signal. PitchMatchingView's tunable_handle changed from `OscillatorPlaybackHandle` to `UnifiedPlaybackHandle`. Fallback to oscillator is automatic when bridge is `None`.
- **Task 8:** Added 8 SoundFont presets from GeneralUser GS (Piano, E.Piano, Guitar, Flute, Oboe, Clarinet, Violin, Synth) to Settings dropdown.
- **Task 9:** Automated tests pass (293 domain, trunk build, clippy). Manual browser tests pending user verification.

### File List

New files:
- synth-worklet/Cargo.toml
- synth-worklet/src/lib.rs
- web/assets/soundfont/synth-processor.js
- web/src/adapters/audio_soundfont.rs
- web/src/adapters/note_player.rs

Modified files:
- .gitignore
- Cargo.toml (workspace members)
- Trunk.toml (pre-build hook)
- index.html (copy-dir and copy-file directives)
- web/Cargo.toml (futures-channel, web-sys features)
- web/src/adapters/mod.rs (register new modules)
- web/src/app.rs (worklet bridge signal, async init)
- web/src/components/comparison_view.rs (UnifiedNotePlayer)
- web/src/components/pitch_matching_view.rs (UnifiedNotePlayer + UnifiedPlaybackHandle)
- web/src/components/settings_view.rs (SoundFont preset options)
- docs/implementation-artifacts/sprint-status.yaml (status update)
- docs/implementation-artifacts/5-2-soundfont-audio.md (this file)

## Change Log

- 2026-03-04: Implemented story 5.2 SoundFont Audio — AudioWorklet real-time synthesis with OxiSynth, separate synth-worklet WASM crate, UnifiedNotePlayer abstraction, async SoundFont loading with oscillator fallback, 8 instrument presets in Settings
