# Story 8.1: Audio Playback Reliability Research

Status: ready-for-dev

## Story

As a developer,
I want to research why audio playback intermittently fails and propose a mitigation strategy,
so that training sessions reliably produce sound across browsers and usage patterns.

## Problem Statement

Audio playback is unreliable — sometimes it works, sometimes it doesn't. The root cause is unknown. This story is a **research and mitigation design** task, not an implementation task. The deliverable is a written analysis with a concrete action plan.

## Research Scope

### 1. Audit Current AudioContext Lifecycle

The singleton `AudioContextManager` (`web/src/adapters/audio_context.rs`) creates an `AudioContext` lazily via `get_or_create()`. Key questions:

- **User gesture timing:** Both training views call `get_or_create()` synchronously at component render time (pitch_comparison_view.rs:54, pitch_matching_view.rs:51). The assumption is that this runs within the click event call stack from the Start Page. **Verify:** Does Leptos router navigation preserve the user gesture context? If the router uses `requestAnimationFrame`, `setTimeout`, or `Promise` microtasks internally, the gesture context may be lost by the time the component renders.
- **AudioContext.state after creation:** The code never checks `AudioContext.state` after `new()`. Per Chrome autoplay policy, it may start `suspended` even when created during a gesture if the Media Engagement Index (MEI) is low. **Verify:** Log `ctx.state()` immediately after creation.
- **No `resume()` call:** The code never calls `AudioContext.resume()`. Chrome's autoplay documentation explicitly recommends calling `resume()` after user interaction. If the context starts suspended, all `OscillatorNode.start()` calls silently produce no output.
- **Singleton reuse across sessions:** The `AudioContextManager` lives in app-level context. If a user starts training, navigates away (triggering `stop_all()` and `interrupt_and_navigate()`), then starts again, the same `AudioContext` is reused. **Verify:** Can an `AudioContext` enter a state where it's technically `running` but its audio graph is broken after extensive connect/disconnect cycles?

### 2. Audit State Change Handler Behavior

Both training views register an `onstatechange` handler (pitch_comparison_view.rs:378-398, pitch_matching_view.rs:465-484) that interrupts training on `Suspended` or `Closed` state. Key questions:

- **Race condition:** If `AudioContext` starts suspended (not yet resumed), does the `onstatechange` handler fire immediately and interrupt training before audio even has a chance to play?
- **Transient suspensions:** Browsers may briefly suspend/resume AudioContext during resource pressure. The current handler treats ANY suspension as permanent and navigates away. Is this too aggressive?

### 3. Audit SoundFont Worklet Path

The worklet bridge is initialized in `app.rs` via `spawn_local()` — explicitly NOT in a user gesture context. Key questions:

- **AudioContext created in async context:** `init_worklet_bridge()` calls `get_or_create()` (app.rs:162). If the training view hasn't been visited yet, this creates the AudioContext outside a user gesture. When the user later navigates to training, `get_or_create()` returns the existing (potentially suspended) context.
- **Worklet ↔ main thread timing:** Does the `MessagePort.postMessage()` for `noteOn` guarantee synchronous audio output, or can messages be delayed/dropped?
- **SoundFont fallback:** If worklet init fails, the oscillator fallback kicks in silently. **Verify:** Is the user's `sound_source` setting respected after fallback? If they selected an SF2 preset but worklet failed, `create_note_player()` falls through to oscillator (note_player.rs:100). The user sees no indication.

### 4. Cross-Browser Autoplay Policies

Reference: [Chrome Autoplay Policy](https://developer.chrome.com/blog/autoplay#web_audio)

- **Chrome (71+):** AudioContext created before user gesture starts `suspended`. Must call `resume()` after gesture. Alternatively, calling `start()` on a connected node after a gesture auto-resumes. MEI score affects behavior.
- **Safari/iOS:** Stricter — AudioContext must be created AND first `start()` must happen within the same synchronous gesture handler call stack. Async gaps (even microtasks) can break this.
- **Firefox:** Generally more permissive but still enforces autoplay policy on some configurations.

### 5. Error Silence Analysis

Current error handling in the training loop logs errors but continues training silently:
- pitch_comparison_view.rs:460-466 — reference note failure logged, training continues
- pitch_comparison_view.rs:486-492 — target note failure logged, training continues
- pitch_matching_view.rs:548-554 — reference note failure logged, training continues

The user sees the training UI progressing (state transitions, feedback) but hears nothing. There's no user-visible indication of audio failure.

## Deliverables

### A. Root Cause Analysis Document

Add a section to this story file (under Dev Agent Record) documenting:
1. Which of the above hypotheses are confirmed/refuted
2. Console logs and browser-specific observations
3. The exact sequence of events that leads to silent audio

### B. Mitigation Strategy Proposal

Propose concrete code changes. Likely candidates based on the audit:

1. **Add `AudioContext.resume()` call** — In `get_or_create()` or at training start, check `ctx.state()` and call `resume()` if suspended. This is the #1 recommendation from Chrome's autoplay docs.
2. **Defer AudioContext creation** — Don't create in `init_worklet_bridge()`. Let the first training view create it on user gesture, then init worklet afterward.
3. **Soften state change handler** — Instead of immediately interrupting on `Suspended`, attempt `resume()` first. Only interrupt if resume fails or state is `Closed`.
4. **Add user-visible audio failure indication** — If `play_for_duration()` fails, show a brief toast/banner instead of silently continuing.
5. **Add AudioContext state logging** — Instrument `get_or_create()`, `play()`, and state changes with `log::info!()` to aid future debugging.

### C. Prioritized Action Plan

Rank the proposed changes by impact and effort. Group into:
- **Quick wins** (high impact, low effort)
- **Important fixes** (high impact, higher effort)
- **Nice-to-haves** (lower impact, helps debugging)

## Tasks / Subtasks

- [ ] Read and understand all audio-related source files (AC: Research Scope 1-5)
  - [ ] `web/src/adapters/audio_context.rs`
  - [ ] `web/src/adapters/audio_oscillator.rs`
  - [ ] `web/src/adapters/audio_soundfont.rs`
  - [ ] `web/src/adapters/note_player.rs`
  - [ ] `web/src/app.rs` (worklet init flow, lines 160-180, 215-340)
  - [ ] `web/src/components/pitch_comparison_view.rs` (audio lifecycle)
  - [ ] `web/src/components/pitch_matching_view.rs` (audio lifecycle)
  - [ ] `web/assets/soundfont/synth-processor.js`
- [ ] Add temporary diagnostic logging to confirm/refute hypotheses (AC: Deliverable A)
  - [ ] Log `AudioContext.state` after creation in `get_or_create()`
  - [ ] Log `AudioContext.state` before each `play()`/`play_for_duration()` call
  - [ ] Log in `onstatechange` handler what state transition occurred
  - [ ] Test in Chrome, Safari, Firefox
- [ ] Write root cause analysis (AC: Deliverable A)
- [ ] Write mitigation strategy with prioritized action plan (AC: Deliverables B, C)

## Acceptance Criteria

1. Root cause analysis identifies the specific failure mode(s) causing intermittent silence
2. Mitigation strategy proposes concrete, actionable code changes
3. Each proposed change references the specific file and function to modify
4. Changes are prioritized by impact and effort
5. Strategy accounts for Chrome, Safari, and Firefox autoplay policies
6. No code changes beyond diagnostic logging are made in this story

## Dev Notes

### Key Files

| File | Role |
|---|---|
| `web/src/adapters/audio_context.rs` | AudioContext singleton manager |
| `web/src/adapters/audio_oscillator.rs` | OscillatorNode playback |
| `web/src/adapters/audio_soundfont.rs` | SoundFont worklet playback |
| `web/src/adapters/note_player.rs` | Unified player factory |
| `web/src/app.rs` | Worklet init (async, outside gesture) |
| `web/src/components/pitch_comparison_view.rs` | Comparison training audio lifecycle |
| `web/src/components/pitch_matching_view.rs` | Pitch matching audio lifecycle |
| `web/assets/soundfont/synth-processor.js` | AudioWorklet processor |

### Architecture Constraints

- No `tokio` — async via `wasm-bindgen-futures::spawn_local()` only
- AudioContext shared via `Rc<RefCell<AudioContext>>`
- `web-sys` features are opt-in — `AudioContext`, `AudioContextState`, `BaseAudioContext` must be enabled
- Error handling: `Result<T, AudioError>` for all fallible ops, no silent failures (NFR8)
- `resume()` returns a `Promise` — will need `JsFuture::from()` to await in Rust

### Browser Autoplay Policy Reference

**Chrome:** AudioContext may start suspended. Call `resume()` after user gesture. [Source](https://developer.chrome.com/blog/autoplay#web_audio)

**Pattern A (recommended by Chrome docs):**
```javascript
// Create context anytime
var context = new AudioContext();
// Resume on user gesture
button.addEventListener('click', () => {
  context.resume().then(() => { /* ready */ });
});
```

**Pattern B:**
```javascript
// Create on gesture
button.addEventListener('click', () => {
  var context = new AudioContext();
  // Immediately running
});
```

Current code uses Pattern B (create on gesture), but the Leptos router may break the synchronous gesture chain. Pattern A with explicit `resume()` is more robust.

### Project Structure Notes

- Domain crate has no audio code — all audio is in web crate adapters
- `AudioError` enum defined in `domain/src/ports.rs` (variants: `EngineStartFailed`, `PlaybackFailed`)
- Training views handle audio errors by logging and continuing — no user feedback

### References

- [Source: web/src/adapters/audio_context.rs] — AudioContextManager singleton
- [Source: web/src/adapters/audio_oscillator.rs] — OscillatorNotePlayer with Web Audio scheduling
- [Source: web/src/app.rs#init_worklet_bridge] — Async worklet init outside user gesture
- [Source: web/src/components/pitch_comparison_view.rs#L51-56] — Eager AudioContext creation
- [Source: web/src/components/pitch_matching_view.rs#L48-53] — Same pattern
- [Source: Chrome Autoplay Policy](https://developer.chrome.com/blog/autoplay#web_audio) — Recommends `resume()` after gesture

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
