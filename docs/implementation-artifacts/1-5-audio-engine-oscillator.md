# Story 1.5: Audio Engine (Oscillator)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to hear notes played through my browser,
so that I can begin ear training with accurate audio.

## Acceptance Criteria

1. **AC1 — Timed play:** Given OscillatorNotePlayer is implemented, when `play_for_duration` is called with a frequency, duration, velocity, and amplitudeDB, then a sine wave plays at the specified frequency via Web Audio `OscillatorNode` + `GainNode`, and the note stops after the specified duration (FR32).

2. **AC2 — Indefinite play:** Given OscillatorNotePlayer is implemented, when `play` is called (indefinite mode), then the note plays continuously until explicitly stopped via `PlaybackHandle.stop()` (FR32).

3. **AC3 — Audio onset:** Given the audio engine, when a note is triggered, then audio onset occurs within 50ms (NFR1).

4. **AC4 — Amplitude control:** Given a note is playing, when it was created with an AmplitudeDB offset, then the GainNode adjusts volume by the specified decibel offset relative to baseline (FR34). Conversion formula: `gain_linear = 10^(dB / 20)`.

5. **AC5 — No AudioContext on load:** Given no user gesture has occurred, when the app loads, then no AudioContext is created (FR35).

6. **AC6 — AudioContext on gesture:** Given the user clicks a training button (first gesture), when the AudioContext is created, then it is stored in a shared reference (`Rc<RefCell<AudioContext>>`) for reuse, and subsequent audio operations use this same context.

7. **AC7 — Stop all:** Given an active AudioContext, when `stop_all()` is called, then all currently playing notes stop immediately.

8. **AC8 — Frequency accuracy:** Given the audio engine plays a specific frequency, when the frequency is measured, then it is accurate to the Web Audio API's precision for the requested value (FR31).

## Tasks / Subtasks

- [x] Task 1: Define port traits in domain crate (AC: 1,2,7,8)
  - [x] 1.1 Create `domain/src/ports.rs` with `NotePlayer` trait, `PlaybackHandle` trait, and `AudioError` enum
  - [x] 1.2 Add `pub mod ports;` to `domain/src/lib.rs` and re-export public types
  - [x] 1.3 Run `cargo test -p domain` — all existing 180+ tests still pass

- [x] Task 2: Add web crate dependencies (AC: all)
  - [x] 2.1 Add `web-sys` with required Audio features to `web/Cargo.toml`
  - [x] 2.2 Move `wasm-bindgen` from dev-dependencies to regular dependencies
  - [x] 2.3 Add `wasm-bindgen-futures` and `gloo-timers` (with `futures` feature) dependencies

- [x] Task 3: Create AudioContext lifecycle manager (AC: 5,6)
  - [x] 3.1 Create `web/src/adapters/mod.rs` with module declarations
  - [x] 3.2 Create `web/src/adapters/audio_context.rs` with `AudioContextManager`
  - [x] 3.3 Lazy initialization: no AudioContext until first `get_or_create()` call
  - [x] 3.4 Shared reference via `Rc<RefCell<AudioContext>>`

- [x] Task 4: Implement OscillatorNotePlayer (AC: 1,2,3,4,7,8)
  - [x] 4.1 Create `web/src/adapters/audio_oscillator.rs`
  - [x] 4.2 Implement `NotePlayer` trait for `OscillatorNotePlayer`
  - [x] 4.3 Implement `PlaybackHandle` trait for `OscillatorPlaybackHandle`
  - [x] 4.4 Implement `stop_all` with tracking of active handles
  - [x] 4.5 dB-to-linear gain conversion: `gain = 10_f32.powf(amplitude_db.raw_value() / 20.0)`

- [x] Task 5: Wire up and verify (AC: all)
  - [x] 5.1 Add `mod adapters;` to `web/src/main.rs`
  - [x] 5.2 Add a temporary "Play Test Note" button to `ComparisonView` for manual verification (plays 440 Hz for 1 second)
  - [x] 5.3 `trunk build` compiles without errors
  - [x] 5.4 `cargo clippy -p web --target wasm32-unknown-unknown` passes with zero warnings
  - [x] 5.5 `cargo test -p domain` passes (all existing + new port tests)
  - [x] 5.6 Manual browser test: click test button, hear a 440 Hz sine tone for 1 second

## Dev Notes

### Port Trait Design (domain/src/ports.rs)

The domain blueprint §9 defines `NotePlayer`, `PlaybackHandle`, and `AudioError`. This is the first time `ports.rs` is created in the domain crate.

**NotePlayer trait — key design decisions:**

The blueprint specifies two `play` overloads (Rust has no overloading — use different method names):
- `play(frequency, velocity, amplitudeDB) -> PlaybackHandle` — indefinite play
- `play_for_duration(frequency, duration, velocity, amplitudeDB) -> ()` — timed play (convenience)
- `stop_all()` — stop all active notes

The blueprint provides a default implementation for timed play: `play + sleep + stop`. However, `sleep` requires async, which complicates the trait design. Two pragmatic approaches:

**Option A (recommended): Sync trait, timed play uses Web Audio scheduling.**
All NotePlayer methods return `Result` synchronously. The `OscillatorNotePlayer.play_for_duration()` uses Web Audio's `osc.stop_with_when(current_time + duration)` to schedule the stop — no async needed. The session state machine (story 1.6) manages timing separately via `gloo-timers` sleep.

**Option B: Async trait (Rust 1.75+ native async fn in traits).**
All methods are `async fn`. Works with generics (`impl NotePlayer`) but NOT with `dyn NotePlayer`. Sessions would use `player.play(...).await`. Domain crate tests need a lightweight executor (`pollster` dev-dependency).

Use Option A unless there's a clear reason for async. The OscillatorNotePlayer operations are all synchronous web-sys calls. The session (story 1.6) handles async timing.

**PlaybackHandle trait:**
- `stop(&mut self)` — idempotent, subsequent calls are no-ops
- `adjust_frequency(&mut self, frequency: Frequency) -> Result<(), AudioError>` — for pitch matching (story 4.x)

**Associated type pattern for PlaybackHandle:**
```rust
pub trait NotePlayer {
    type Handle: PlaybackHandle;
    fn play(&self, ...) -> Result<Self::Handle, AudioError>;
    // ...
}
```
This avoids `Box<dyn PlaybackHandle>` overhead and works with generics.

**AudioError enum (from blueprint §9.3):**
```rust
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("audio engine failed to start: {0}")]
    EngineStartFailed(String),
    #[error("invalid frequency: {0}")]
    InvalidFrequency(String),
    #[error("invalid duration: {0}")]
    InvalidDuration(String),
    #[error("audio context unavailable")]
    ContextUnavailable,
}
```
Keep only the variants needed now. `InvalidPreset` and `InvalidInterval` are for later stories (SoundFont, interval training). Add them when needed, not preemptively.

### AudioContext Lifecycle (web/src/adapters/audio_context.rs)

**AudioContextManager** manages the singleton AudioContext:

```rust
pub struct AudioContextManager {
    context: Option<Rc<RefCell<web_sys::AudioContext>>>,
}
```

- `get_or_create(&mut self) -> Result<Rc<RefCell<AudioContext>>, AudioError>` — creates on first call, returns shared ref thereafter
- No AudioContext is created on app load (AC5)
- The training start button in the UI (story 1.7) calls this method — the click IS the user gesture (FR35)
- Store the manager itself in an `Rc<RefCell<AudioContextManager>>` for sharing across components

**Critical: AudioContext creation requires a user gesture.** The browser blocks `new AudioContext()` unless called during a click/keypress event handler. The composition root (story 1.7+) must ensure `get_or_create()` is called within the training button's click handler.

### OscillatorNotePlayer Implementation (web/src/adapters/audio_oscillator.rs)

**Web Audio node graph per note:**
```
OscillatorNode (sine wave, frequency) → GainNode (amplitude) → AudioContext.destination
```

**Play (indefinite):**
1. Get AudioContext from AudioContextManager
2. `ctx.create_oscillator()` — creates OscillatorNode
3. `osc.set_type(OscillatorType::Sine)` — sine waveform
4. `osc.frequency().set_value(frequency.raw_value() as f32)` — set frequency (AudioParam uses f32)
5. `ctx.create_gain()` — creates GainNode
6. Convert AmplitudeDB to linear gain: `gain_linear = 10_f32.powf(amplitude_db.raw_value() / 20.0)`
7. `gain.gain().set_value(gain_linear)` — set volume
8. `osc.connect_with_audio_node(&gain)` → `gain.connect_with_audio_node(&ctx.destination())`
9. `osc.start()` — begin playback
10. Return `OscillatorPlaybackHandle { oscillator: osc, gain: gain }`

**Play for duration:**
Same as above, but after `start()`:
- `osc.stop_with_when(ctx.current_time() + duration.raw_value())` — Web Audio schedules stop using its own high-precision clock
- This returns immediately (non-blocking)

**OscillatorPlaybackHandle:**
```rust
pub struct OscillatorPlaybackHandle {
    oscillator: web_sys::OscillatorNode,
    stopped: bool,
}
```
- `stop()`: calls `self.oscillator.stop()` if not already stopped. Set `stopped = true`. Must be idempotent.
- `adjust_frequency()`: calls `self.oscillator.frequency().set_value(freq.raw_value() as f32)` — real-time pitch change

**stop_all() implementation:**
OscillatorNotePlayer must track active handles. Use `Vec<Rc<RefCell<OscillatorPlaybackHandle>>>` or similar. On `stop_all()`, iterate and stop each. On individual `stop()`, mark as stopped so `stop_all()` skips it.

**MIDIVelocity parameter:** The oscillator doesn't use MIDI velocity (no velocity layers in a sine wave). Ignore this parameter in the oscillator implementation. It exists on the trait for the future SoundFont implementation (story 5.2) which uses velocity to select sample layers.

### Dependencies to Add (web/Cargo.toml)

```toml
[dependencies]
# ... existing deps ...
wasm-bindgen = "0.2"           # Move from [dev-dependencies] to here
wasm-bindgen-futures = "0.4"   # Async support (needed for story 1.6+, add now)
gloo-timers = { version = "0.3", features = ["futures"] }  # Timer abstractions
web-sys = { version = "0.3", features = [
    "AudioContext",
    "AudioDestinationNode",
    "AudioNode",
    "AudioParam",
    "AudioScheduledSourceNode",
    "BaseAudioContext",
    "GainNode",
    "OscillatorNode",
    "OscillatorType",
] }
```

**Critical:** `web-sys` features are opt-in. Enable exactly the features listed above. Do NOT enable broad features like `"all"`. Each feature adds compile time.

**Note:** `wasm-bindgen-futures` and `gloo-timers` are not directly used in this story's code, but the architecture requires them for story 1.6 (session state machine). Adding them now prevents a dependency refactor later.

### web-sys API Patterns

**Method naming:** Web Audio API methods translate to snake_case in web-sys:
- `createOscillator()` → `create_oscillator()`
- `createGain()` → `create_gain()`
- `connect(dest)` → `connect_with_audio_node(&dest)`
- `start()` → `start()`
- `stop()` → `stop()`
- `start(when)` → `start_with_when(when)`
- `stop(when)` → `stop_with_when(when)`
- `currentTime` → `current_time()`
- `destination` → `destination()`

**AudioParam access:** Properties that are AudioParams become getter methods:
- `osc.frequency` → `osc.frequency()` returns `AudioParam`
- `gain.gain` → `gain.gain()` returns `AudioParam`
- Set value: `param.set_value(f32_value)`

**Return types:** Most web-sys constructors return `Result<T, JsValue>`. Convert `JsValue` errors to `AudioError::EngineStartFailed(format!("{:?}", e))`.

### Gain Conversion Formula

AmplitudeDB (in dB) must be converted to linear gain for the Web Audio GainNode:

```
gain_linear = 10^(dB / 20)
```

Examples:
- 0 dB → gain 1.0 (unity — no change)
- -6 dB → gain ~0.5 (half amplitude)
- -12 dB → gain ~0.25
- -90 dB → gain ~0.00003 (effectively silent)
- +6 dB → gain ~2.0 (double amplitude)

In Rust: `let gain = 10_f32.powf(amplitude_db.raw_value() / 20.0);`

AmplitudeDB uses f32 (per existing type definition), which matches AudioParam's f32 interface.

### File Structure

```
domain/src/
├── ports.rs          (NEW — NotePlayer, PlaybackHandle traits, AudioError)
├── lib.rs            (MODIFIED — add mod ports + re-exports)
└── ... (unchanged)

web/src/
├── main.rs           (MODIFIED — add mod adapters)
├── adapters/
│   ├── mod.rs        (NEW — module declarations)
│   ├── audio_context.rs    (NEW — AudioContextManager)
│   └── audio_oscillator.rs (NEW — OscillatorNotePlayer, OscillatorPlaybackHandle)
├── components/
│   └── comparison_view.rs  (MODIFIED — temporary test button, removed in story 1.7)
└── ... (unchanged)
```

Do NOT create `adapters/audio_soundfont.rs`, `adapters/indexeddb_store.rs`, or `adapters/localstorage_settings.rs` yet — those belong to stories 5.2, 1.8, and 2.1 respectively.

### Testing Strategy

**Domain crate (cargo test):**
- `ports.rs` defines traits and error enum — limited unit test surface
- Test AudioError display messages
- Test that trait definitions compile (marker test)

**Web crate (manual browser testing):**
- Add a temporary "Play Test Note" button to `ComparisonView`
- Button calls `AudioContextManager::get_or_create()` then `OscillatorNotePlayer::play_for_duration(440 Hz, 1 sec, velocity 80, 0 dB)`
- Verify: sine tone plays for 1 second at the correct pitch
- This button is temporary — it will be removed/replaced in story 1.7 (Comparison Training UI)
- `cargo clippy -p web --target wasm32-unknown-unknown` must pass

**What NOT to test:**
- Do NOT test that AudioContext creation requires a user gesture — this is a browser policy, not our code
- Do NOT test actual frequency accuracy with an analyzer — trust the Web Audio API
- Do NOT write wasm-bindgen-test tests for this story — the audio API needs a real browser context

### Previous Story Intelligence (Story 1.4)

Key patterns established that apply here:

- **Module structure:** `mod.rs` + individual files pattern — apply same for `adapters/mod.rs`
- **Clippy compliance:** Run `cargo clippy -p web --target wasm32-unknown-unknown` early and fix warnings
- **Component naming:** `PascalCase` function names for components, `snake_case` file names
- **Leptos imports:** `use leptos::prelude::*;` is the standard import
- **Current web/Cargo.toml:** Already has `leptos 0.8 (csr)`, `leptos_router 0.8`, `console_log`, `log`, `console_error_panic_hook`, `getrandom`. `wasm-bindgen` is currently in dev-deps only.
- **Web crate target:** Always compile/clippy with `--target wasm32-unknown-unknown`

### Git History Context

Recent commits follow the pattern: create story → implement → code review → done. Last 5 commits:
```
5e2b06f Add .idea to .gitignore.
1c27a20 Apply code review fixes for story 1.4 and mark as done
af4f205 Implement story 1.4 App Shell & Routing
1b444a5 Add story 1.4 App Shell & Routing and mark as ready-for-dev
c9068d9 Apply code review fixes for story 1.3 and mark as done
```

The web crate was first modified in story 1.4 (app shell + routing). This story (1.5) is the FIRST story that adds web-sys browser API interaction and the adapters directory.

### Project Structure Notes

- This story creates the `web/src/adapters/` directory — the foundational adapter layer for all browser API integration
- The `ports.rs` file in the domain crate is the first port interface — story 1.6+ will reference these traits
- No conflicts with existing domain or web code — purely additive changes
- AudioContextManager is a prerequisite for all future audio stories (1.6, 1.7, 4.x, 5.x)
- The temporary test button in ComparisonView is explicitly temporary — mark with a `// TODO: Remove in story 1.7` comment

### References

- [Source: docs/planning-artifacts/epics.md#Story 1.5: Audio Engine (Oscillator)]
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture]
- [Source: docs/planning-artifacts/architecture.md#Async Model]
- [Source: docs/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: docs/ios-reference/domain-blueprint.md#§9 Port Interfaces]
- [Source: docs/ios-reference/domain-blueprint.md#§9.1 NotePlayer]
- [Source: docs/ios-reference/domain-blueprint.md#§9.2 PlaybackHandle]
- [Source: docs/ios-reference/domain-blueprint.md#§9.3 AudioError]
- [Source: docs/planning-artifacts/ux-design-specification.md#Web Audio Context Activation]
- [Source: docs/planning-artifacts/ux-design-specification.md#Interruption Handling]
- [Source: docs/project-context.md#Web Audio Edge Cases]
- [Source: docs/project-context.md#Leptos Framework Rules]
- [web-sys AudioContext API: https://docs.rs/web-sys/latest/web_sys/struct.AudioContext.html]
- [MDN Web Audio API: https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API]
- [wasm-bindgen Web Audio example: https://rustwasm.github.io/docs/wasm-bindgen/examples/web-audio.html]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

No debug issues encountered.

### Completion Notes List

- **Task 1:** Created `domain/src/ports.rs` with `NotePlayer` trait (associated type `Handle`), `PlaybackHandle` trait (`stop`, `adjust_frequency`), and `AudioError` enum (4 variants). Added `pub mod ports` and re-exports to `domain/src/lib.rs`. 6 new tests: error display messages, debug formatting, and mock trait compilation. All 186 domain tests pass.
- **Task 2:** Added `web-sys` (9 Audio features), `wasm-bindgen`, `wasm-bindgen-futures`, and `gloo-timers` to `web/Cargo.toml`. Moved `wasm-bindgen` from dev-deps to regular deps.
- **Task 3:** Created `web/src/adapters/` module with `AudioContextManager` — lazy initialization (no AudioContext on load, AC5), shared via `Rc<RefCell<AudioContext>>` (AC6).
- **Task 4:** Implemented `OscillatorNotePlayer` with `NotePlayer` trait. Uses `OscillatorNode` + `GainNode` graph. `OscillatorPlaybackHandle` with shared inner state for `stop_all` tracking. dB-to-linear gain conversion: `10_f32.powf(db / 20.0)` (AC4). Timed play uses `stop_with_when()` for Web Audio scheduled stop (AC1). Interior mutability (`RefCell<Vec<>>`) for handle tracking with `&self`.
- **Task 5:** Wired `mod adapters` in `web/src/main.rs`. Added temporary "Play Test Note" button to `ComparisonView` (440 Hz, 1 sec, marked with TODO for removal in story 1.7). `trunk build` compiles, `cargo clippy` passes with zero warnings, all domain tests pass.

### Implementation Plan

Used Option A (sync trait) as recommended in Dev Notes. All NotePlayer methods return Results synchronously. Timed play uses Web Audio's `stop_with_when()` for precise scheduling. `OscillatorPlaybackHandle` uses shared inner state (`Rc<RefCell<OscillatorHandleInner>>`) to support both individual stop and `stop_all` via the player's handle tracking. `AudioContextManager` defers AudioContext creation to first call within a user gesture handler.

### File List

- `domain/src/ports.rs` — NEW: NotePlayer, PlaybackHandle traits, AudioError enum
- `domain/src/lib.rs` — MODIFIED: added `pub mod ports` and re-exports
- `web/Cargo.toml` — MODIFIED: added web-sys, wasm-bindgen, wasm-bindgen-futures, gloo-timers
- `web/src/main.rs` — MODIFIED: added `mod adapters`
- `web/src/adapters/mod.rs` — NEW: module declarations
- `web/src/adapters/audio_context.rs` — NEW: AudioContextManager
- `web/src/adapters/audio_oscillator.rs` — NEW: OscillatorNotePlayer, OscillatorPlaybackHandle
- `web/src/components/comparison_view.rs` — MODIFIED: temporary test button

## Senior Developer Review (AI)

**Reviewer:** Michael (via Claude Opus 4.6) on 2026-03-03

**Findings:** 3 High, 3 Medium, 1 Low — all fixed.

**Fixes Applied:**

1. **H1 — Memory leak in active_handles for timed notes** (audio_oscillator.rs): Timed-play handles were never pruned because the Web Audio scheduled stop didn't update the Rust `stopped` flag. Fix: Remove timed handles from `active_handles` after scheduling `stop_with_when()` since they self-terminate.
2. **H2 — `let _ = oscillator.stop()` violated error handling rules** (audio_oscillator.rs:46): Replaced with `if let Err(e) = ... { log::warn!(...) }` pattern for both oscillator stop and gain node disconnect.
3. **H3 — GainNode not stored in handle** (audio_oscillator.rs): Added `gain` field to `OscillatorHandleInner`. On `stop()`, the GainNode is now disconnected from the audio graph to prevent orphaned node accumulation.
4. **M1 — `stop_all(&mut self)` inconsistent with `play(&self)`** (ports.rs, audio_oscillator.rs): Changed `NotePlayer::stop_all` from `&mut self` to `&self` since `active_handles` uses interior mutability (`RefCell`). Eliminates unnecessary `borrow_mut()` requirement for callers.
5. **M2 — Double `get_context()` borrow in `play_for_duration`** (audio_oscillator.rs): Refactored `create_and_start` to accept `&Rc<RefCell<AudioContext>>` parameter. Callers now get the context once and pass it through.
6. **M3 — Semantically wrong error variant for scheduling errors** (ports.rs, audio_oscillator.rs): Added `AudioError::PlaybackFailed(String)` variant. `stop_with_when` errors now map to `PlaybackFailed` instead of misusing `EngineStartFailed`.
7. **L1 — Missing Default impl for AudioContextManager** (audio_context.rs): Added `#[derive(Default)]`.

**Verification:** 187 domain tests pass. Clippy zero warnings. WASM build succeeds.

## Change Log

- 2026-03-03: Implemented story 1.5 — Audio Engine (Oscillator). Added port traits in domain crate, AudioContextManager for lazy AudioContext lifecycle, OscillatorNotePlayer with Web Audio implementation, and temporary test button for manual verification.
- 2026-03-03: Code review fixes — Fixed memory leak in timed-play handle tracking, replaced error swallowing with logging, stored GainNode in handle for proper cleanup, changed stop_all to &self, eliminated double context borrow, added PlaybackFailed error variant, added Default derive for AudioContextManager. 187 domain tests pass, clippy clean.
