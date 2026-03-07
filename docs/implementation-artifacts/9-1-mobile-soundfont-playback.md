# Story 9.1: Mobile SoundFont Playback

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want SoundFont playback to work on mobile browsers (iOS Safari, mobile Chromium),
so that I hear realistic instrument sounds during training on my phone, not just oscillator tones.

## Acceptance Criteria

1. SoundFont playback produces audible sound on iOS Safari (17+) when the user has selected a SoundFont preset as sound source
2. SoundFont playback produces audible sound on mobile Chromium-based browsers (Chrome, Edge, Brave on Android/iOS) when the user has selected a SoundFont preset
3. The SoundFont file loads, presets appear in settings, and the selected preset plays during training — the full pipeline works end-to-end on mobile
4. Desktop browser playback (Chrome, Firefox, Safari, Edge) continues to work without regression
5. The oscillator fallback still activates when SoundFont loading fails (network error, unsupported browser)
6. Settings sound source preview (story 8.10) works on mobile with SoundFont presets

## Tasks / Subtasks

- [ ] Task 1: Send raw WASM bytes to worklet via postMessage instead of compiled module via processorOptions (AC: 1, 2, 3)
  - [ ] 1.1 In `app.rs`, change `WorkletAssets.wasm_module: JsValue` to `wasm_bytes: JsValue` — store the raw `ArrayBuffer` from fetch, remove the `WebAssembly::compile()` call
  - [ ] 1.2 In `app.rs` `connect_worklet()`, remove `wasmModule` from `processorOptions`; after node creation and `port` acquisition, send `{ type: 'initWasm', wasmBytes }` via `port.post_message()`
  - [ ] 1.3 Update `connect_worklet()` to wait for `'ready'` message (which now comes after async WASM instantiation inside the worklet)
- [ ] Task 2: Async WASM instantiation in AudioWorklet processor (AC: 1, 2, 3)
  - [ ] 2.1 In `synth-processor.js`, remove WASM instantiation from constructor — constructor only sets up `port.onmessage` handler and initial state
  - [ ] 2.2 Add `initWasm` message handler: receives raw bytes, calls `await WebAssembly.instantiate(wasmBytes, {})`, sets up synth/buffers, posts `{ type: 'ready' }` on success or `{ type: 'error' }` on failure
  - [ ] 2.3 Guard `loadSoundFont` and other message handlers against `this.wasm === null` (messages arriving before WASM init completes)
- [ ] Task 3: Verify no regressions on desktop (AC: 4, 5)
  - [ ] 3.1 Manual test: SoundFont playback in desktop Chrome, Firefox, Safari
  - [ ] 3.2 Manual test: oscillator fallback when SF2 fetch fails (e.g., block network request in DevTools)
  - [ ] 3.3 Manual test: settings sound source preview with SoundFont preset on desktop
- [ ] Task 4: Manual mobile verification (AC: 1, 2, 6)
  - [ ] 4.1 Test on iOS Safari: load app, select SoundFont preset in settings, run pitch comparison training — confirm audible SoundFont playback
  - [ ] 4.2 Test on mobile Chrome (Android or iOS): same flow as 4.1
  - [ ] 4.3 Test settings sound source preview on mobile with SoundFont preset
  - [ ] 4.4 Test oscillator fallback on mobile when SoundFont is unavailable
- [ ] Task 5: Code quality (AC: all)
  - [ ] 5.1 `cargo clippy --workspace` clean
  - [ ] 5.2 `cargo test -p domain` passes

## Dev Notes

### Root Cause

The SoundFont AudioWorklet pipeline fails on mobile browsers due to two compounding issues:

1. **Synchronous `new WebAssembly.Instance()` size restriction**: iOS Safari (and some mobile Chromium builds) restrict synchronous WASM instantiation to modules under ~4KB. The `synth_worklet.wasm` is ~221KB. In `synth-processor.js:84`, the constructor calls `new WebAssembly.Instance(wasmModule, {})` which silently fails on these platforms.

2. **Structured cloning of `WebAssembly.Module` via `processorOptions`**: The main thread compiles WASM with `WebAssembly.compile()` and passes the resulting `WebAssembly.Module` to the AudioWorklet constructor via `processorOptions`. Not all mobile browsers reliably support structured cloning of `WebAssembly.Module` across thread boundaries. When this fails, the module arrives as `undefined` in the worklet.

The result: the worklet never initializes, `worklet_bridge` stays `None`, and the note player factory (`note_player.rs:82-102`) silently falls back to `OscillatorNotePlayer`. The SoundFont file itself loads fine (parsed on the main thread in `app.rs:194`), which is why presets appear in settings.

### Fix Architecture

Change the WASM delivery from:

```
Main thread: fetch bytes -> compile to Module -> structured clone via processorOptions -> Worklet: sync instantiate
```

To:

```
Main thread: fetch bytes (keep as ArrayBuffer) -> postMessage to Worklet -> Worklet: async WebAssembly.instantiate()
```

This avoids both the synchronous instantiation limit and the structured cloning issue. `WebAssembly.instantiate(bytes, {})` performs compilation + instantiation asynchronously and has no size limit on any platform.

### Key Architectural Constraint

The `synth-processor.js` constructor **cannot be async**. The solution is:
- Constructor: only set up `port.onmessage`, initialize state to not-ready
- `initWasm` message handler: async method that instantiates WASM, creates synth, allocates buffers, then posts `{ type: 'ready' }`
- `process()` already guards on `this.ready` (line 141), so it safely outputs silence until WASM is initialized

### Message Protocol Change

Current protocol:
```
1. Main: create node with processorOptions.wasmModule
2. Worklet constructor: sync instantiate, post 'ready'
3. Main: post 'loadSoundFont'
4. Worklet: post 'soundFontLoaded'
```

New protocol:
```
1. Main: create node (no processorOptions needed)
2. Main: post 'initWasm' with raw ArrayBuffer bytes
3. Worklet: async instantiate, post 'ready'
4. Main: post 'loadSoundFont'
5. Worklet: post 'soundFontLoaded'
```

The rest of the message protocol (noteOn, noteOff, pitchBend, selectProgram, allNotesOff) is unchanged.

### What NOT to Change

- `synth-worklet/src/lib.rs` — the WASM FFI exports are not involved in this bug
- `web/src/adapters/audio_soundfont.rs` — WorkletBridge and SoundFontNotePlayer are fine; the issue is upstream (worklet never initializes)
- `web/src/adapters/note_player.rs` — the factory logic is correct; it properly falls back when bridge is `None`
- `web/src/adapters/audio_context.rs` — AudioContext lifecycle and gesture handling are not related
- SF2 preset parsing on the main thread (`app.rs:465-556`) — this works correctly and is why presets show in settings
- The `process()` method in `synth-processor.js` — rendering logic is correct

### Guard Against Early Messages

After the change, `loadSoundFont` and other MIDI messages could theoretically arrive before `initWasm` completes. Add a null check on `this.wasm` in `handleMessage` for `loadSoundFont`, `noteOn`, `noteOff`, `pitchBend`, `selectProgram`, and `allNotesOff` cases. In practice this won't happen because `connect_worklet` awaits `'ready'` before sending `loadSoundFont`, but defensive coding prevents silent failures.

### Files to Modify

| File | Change |
|------|--------|
| `web/src/app.rs` | `WorkletAssets.wasm_module` -> `wasm_bytes`; remove `WebAssembly::compile()` call; send `initWasm` message in `connect_worklet()` instead of passing via `processorOptions` |
| `web/assets/soundfont/synth-processor.js` | Remove sync WASM instantiation from constructor; add `initWasm` async message handler; guard other handlers against null `this.wasm` |

### web-sys Features

No new web-sys features required. Existing features cover `AudioWorkletNode`, `AudioWorkletNodeOptions`, `MessagePort`, and `AudioContext`.

### Project Structure Notes

- Both files are already in the codebase; no new files needed
- The `synth-processor.js` in `web/assets/soundfont/` is copied to `dist/soundfont/` by Trunk via `<link data-trunk rel="copy-dir">` in `index.html`
- The `synth_worklet.wasm` is built by Trunk pre-build hook (`Trunk.toml` lines 8-12) and also copied to `web/assets/soundfont/`

### References

- [Source: web/src/app.rs] `WorkletAssets` struct (line 245-248), `fetch_worklet_assets()` (lines 253-299), `connect_worklet()` (lines 304-360)
- [Source: web/assets/soundfont/synth-processor.js] `SynthProcessor` class (lines 72-167), constructor with sync instantiation (lines 73-101), `handleMessage` (lines 103-138), `process()` (lines 140-164)
- [Source: web/src/adapters/note_player.rs] `create_note_player()` factory (lines 82-102) — SoundFont only if bridge is `Some`
- [Source: web/src/adapters/audio_soundfont.rs] `WorkletBridge` (lines 19-86), `SoundFontNotePlayer` (lines 158-238)
- [Source: synth-worklet/src/lib.rs] WASM FFI exports (lines 1-133) — no changes needed
- [Source: Trunk.toml] Pre-build hook for synth_worklet.wasm (lines 8-12)
- [Source: docs/planning-artifacts/architecture.md] Section 7: Audio Architecture

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
