# Story 8.1: Audio Playback Reliability Research

Status: done

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

- [x] Read and understand all audio-related source files (AC: Research Scope 1-5)
  - [x] `web/src/adapters/audio_context.rs`
  - [x] `web/src/adapters/audio_oscillator.rs`
  - [x] `web/src/adapters/audio_soundfont.rs`
  - [x] `web/src/adapters/note_player.rs`
  - [x] `web/src/app.rs` (worklet init flow, lines 160-180, 215-340)
  - [x] `web/src/components/pitch_comparison_view.rs` (audio lifecycle)
  - [x] `web/src/components/pitch_matching_view.rs` (audio lifecycle)
  - [x] `web/assets/soundfont/synth-processor.js`
- [x] Add temporary diagnostic logging to confirm/refute hypotheses (AC: Deliverable A)
  - [x] Log `AudioContext.state` after creation in `get_or_create()`
  - [x] Log `AudioContext.state` before each `play()`/`play_for_duration()` call
  - [x] Log in `onstatechange` handler what state transition occurred
  - [x] Test in Chrome (confirmed: AudioContext starts Suspended when created by init_worklet_bridge)
- [x] Write root cause analysis (AC: Deliverable A)
- [x] Write mitigation strategy with prioritized action plan (AC: Deliverables B, C)

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

Claude Opus 4.6

### Root Cause Analysis (Deliverable A)

#### Confirmed Failure Modes

**1. AudioContext created outside user gesture by worklet init (CONFIRMED - PRIMARY CAUSE)**

`app.rs:161-162` spawns `init_worklet_bridge()` via `spawn_local()` at app mount time — explicitly outside any user gesture context. This calls `AudioContextManager::get_or_create()` (`app.rs:227-230`), which creates the singleton AudioContext. On Chrome 71+, an AudioContext created outside a user gesture starts in `suspended` state.

When the user later clicks "Start Training" and navigates to a training view, `get_or_create()` (`pitch_comparison_view.rs:54`, `pitch_matching_view.rs:51`) returns the **already-existing suspended context** rather than creating a new one. The code never calls `resume()`, so the context remains suspended.

**Sequence of events leading to silent audio:**
1. App mounts → `spawn_local` runs `init_worklet_bridge()` → `get_or_create()` creates AudioContext (suspended on Chrome, because no user gesture)
2. User clicks training button → Leptos router navigates to training view
3. Training view calls `get_or_create()` → returns existing suspended context (no new creation)
4. Training loop calls `play_for_duration()` → `OscillatorNode.start()` executes on a suspended context → **silent output**
5. `onstatechange` handler may or may not fire (depends on timing)

**Race condition:** Whether audio works depends on a timing race:
- If the SoundFont fetch is slow and `init_worklet_bridge()` hasn't called `get_or_create()` yet when the user starts training → the training view creates the AudioContext within the gesture → **audio works**
- If `init_worklet_bridge()` runs first (fast network, cached assets) → AudioContext already exists and is suspended → **audio fails silently**

This explains the intermittent nature of the bug — it depends on network speed and caching.

**2. No `resume()` call anywhere in the codebase (CONFIRMED - ENABLING CAUSE)**

The code never calls `AudioContext.resume()`. Chrome's autoplay documentation explicitly recommends calling `resume()` after a user interaction to ensure the context is running. Even if the context starts suspended (due to #1 above or low MEI score), a `resume()` call during a user gesture would fix it. The absence of `resume()` means there's no recovery path.

**3. onstatechange handler is overly aggressive (CONFIRMED - SECONDARY CAUSE)**

Both training views (`pitch_comparison_view.rs:379-398`, `pitch_matching_view.rs:466-484`) register an `onstatechange` handler that immediately calls `interrupt_and_navigate()` when the AudioContext enters `Suspended` state. Problems:

- If the context **starts** suspended (scenario #1), the handler may fire during setup and interrupt training before it begins
- Transient browser suspensions (resource pressure, background tab throttling) trigger permanent interruption with no recovery attempt
- The handler doesn't distinguish between "context was never running" and "context was suspended by the browser"

**4. Error silence in training loop (CONFIRMED - USABILITY ISSUE)**

Both training views log playback errors but continue training silently:
- `pitch_comparison_view.rs:460-466` — reference note failure: `log::error!` then continues
- `pitch_comparison_view.rs:486-492` — target note failure: `log::error!` then continues
- `pitch_matching_view.rs:548-554` — reference note failure: `log::error!` then continues

The user sees the training UI progressing (state transitions, feedback indicators) but hears nothing. There is zero user-visible indication of audio failure.

**5. SoundFont fallback without user awareness (CONFIRMED - MINOR)**

`create_note_player()` (`note_player.rs:87-101`) silently falls back to oscillator if the worklet bridge is `None` or the sound source doesn't start with `"sf2:"`. If worklet init failed, the user's selected SoundFont preset is ignored, and they hear a sine wave oscillator instead — with no indication that their preference was overridden.

#### Refuted/Unconfirmed Hypotheses

**Leptos router breaking gesture context:** The training views call `get_or_create()` synchronously in the component render function body (not in a callback or effect). Leptos 0.8 CSR router renders the target route component synchronously during navigation. The gesture context from the click should still be active. **However**, this is moot because the real problem is #1 — the context already exists from `init_worklet_bridge()`.

**AudioContext becoming broken after connect/disconnect cycles:** No evidence found. The singleton reuse pattern is fine as long as the context is in `running` state. The issue is that it never reaches `running` in the first place.

**MessagePort message delay/dropping for SoundFont:** `postMessage` is reliable within the same origin. No evidence of dropped messages. SoundFont playback issues stem from the same root cause (suspended AudioContext means the worklet's `process()` runs but produces silence).

### Mitigation Strategy (Deliverable B)

#### Proposed Changes

**1. Add `AudioContext.resume()` at training start**

- **File:** `web/src/adapters/audio_context.rs`
- **Function:** Add a new `pub async fn ensure_running(&self) -> Result<(), AudioError>` method
- **Logic:** Check `ctx.state()`, if not `Running`, call `ctx.resume()` (returns a `Promise`), await it via `JsFuture::from()`, verify state is now `Running`
- **Callers:** Both training views should call `ensure_running()` at the top of the `spawn_local` training loop, before `session.start()`
- **Impact:** Fixes the primary failure mode. Even if the AudioContext was created suspended by `init_worklet_bridge()`, calling `resume()` within the user's training-start gesture will transition it to `running`.
- **Web-sys features needed:** `AudioContext` already has `resume()` available — returns `js_sys::Promise`

**2. Defer worklet AudioContext creation**

- **File:** `web/src/app.rs`
- **Function:** `init_worklet_bridge()`
- **Change:** Do NOT call `get_or_create()` inside `init_worklet_bridge()`. Instead, accept an `Rc<RefCell<AudioContext>>` parameter and defer all worklet setup until the AudioContext exists and is running. Two approaches:
  - **(a) Lazy worklet init:** Move worklet init to training view start, after `ensure_running()`. Simpler but adds latency to first training start.
  - **(b) Two-phase init:** Pre-fetch the WASM module and SF2 file at app mount (no AudioContext needed). Create the worklet node only when AudioContext is available and running. Best UX — fetch is slow, worklet creation is fast.
- **Impact:** Eliminates the root cause entirely — AudioContext is never created outside a gesture context.

**3. Soften onstatechange handler with resume attempt**

- **Files:** `web/src/components/pitch_comparison_view.rs`, `web/src/components/pitch_matching_view.rs`
- **Function:** `audiocontext_handler` closure
- **Change:** Instead of immediately calling `interrupt_and_navigate()` on `Suspended`:
  1. Attempt `ctx.resume()` first
  2. Set a short timeout (e.g. 500ms) to check if state recovered to `Running`
  3. Only interrupt if `resume()` fails or state is `Closed`
- **Impact:** Prevents premature interruption from transient suspensions and from the "starts suspended" scenario.

**4. Add user-visible audio failure indication**

- **Files:** `web/src/components/pitch_comparison_view.rs`, `web/src/components/pitch_matching_view.rs`
- **Change:** When `play_for_duration()` returns `Err`, set a signal that displays a brief non-blocking toast/banner: "Audio playback failed — try restarting training". Use the existing `storage_error` pattern (auto-dismiss after 5s).
- **Impact:** Users know something is wrong instead of silently training with no sound.

**5. Add AudioContext state logging (already done in this story)**

- **Files:** `audio_context.rs`, `audio_oscillator.rs`, `pitch_comparison_view.rs`, `pitch_matching_view.rs`
- **Change:** `log::info!("[DIAG] ...")` calls at key points
- **Impact:** Aids future debugging. These can be downgraded from `info` to `debug` level after the mitigation is implemented and verified.

### Prioritized Action Plan (Deliverable C)

#### Quick Wins (high impact, low effort)

| # | Change | Impact | Effort | Files |
|---|--------|--------|--------|-------|
| 1 | Add `ensure_running()` with `resume()` call | Fixes primary failure mode | ~30 min | `audio_context.rs`, both training views |
| 2 | Soften `onstatechange` handler | Prevents false interruptions | ~20 min | Both training views |

#### Important Fixes (high impact, higher effort)

| # | Change | Impact | Effort | Files |
|---|--------|--------|--------|-------|
| 3 | Defer/restructure worklet AudioContext creation | Eliminates root cause | ~1-2h | `app.rs`, possibly `audio_context.rs` |
| 4 | User-visible audio failure indication | Users know audio failed | ~30 min | Both training views |

#### Nice-to-Haves (lower impact, helps debugging)

| # | Change | Impact | Effort | Files |
|---|--------|--------|--------|-------|
| 5 | Keep diagnostic logging (downgrade to `debug`) | Future debugging | ~10 min | 4 files with `[DIAG]` logs |
| 6 | Add SoundFont fallback notification | User aware of sound source | ~20 min | `note_player.rs`, settings view |

**Recommended implementation order:** 1 → 2 → 3 → 4 → 5 → 6

Changes 1 and 2 alone should resolve the intermittent silence for most users. Change 3 is the proper architectural fix that eliminates the race condition. Change 4 improves UX for any remaining edge cases.

### Debug Log References

Diagnostic logging added with `[DIAG]` prefix to:
- `audio_context.rs:get_or_create()` — logs state after AudioContext creation
- `audio_oscillator.rs:play()` — logs state before oscillator playback
- `audio_oscillator.rs:play_for_duration()` — logs state before timed playback
- `pitch_comparison_view.rs:audiocontext_handler` — logs state transitions
- `pitch_matching_view.rs:audiocontext_handler` — logs state transitions

### Completion Notes List

- Read and analyzed all 8 audio-related source files
- Identified primary root cause: AudioContext created in suspended state by `init_worklet_bridge()` running outside user gesture context, with no `resume()` call to recover
- Identified secondary causes: overly aggressive onstatechange handler, silent error handling
- Added diagnostic logging to 5 code locations (4 files)
- Wrote root cause analysis documenting 5 confirmed failure modes
- Wrote mitigation strategy with 6 concrete proposals, each referencing specific files and functions
- Created prioritized action plan grouped by impact and effort

### Change Log

- 2026-03-06: Added diagnostic logging to audio_context.rs, audio_oscillator.rs, pitch_comparison_view.rs, pitch_matching_view.rs
- 2026-03-06: Wrote root cause analysis and mitigation strategy in Dev Agent Record
- 2026-03-06: Code review fixes — added missing diagnostic log on AudioContext reuse path, added worklet path logging (init_worklet_bridge, SoundFontNotePlayer), downgraded per-note logs to debug level, updated File List

### File List

- `web/src/adapters/audio_context.rs` (modified — diagnostic logging)
- `web/src/adapters/audio_oscillator.rs` (modified — diagnostic logging)
- `web/src/adapters/audio_soundfont.rs` (modified — diagnostic logging)
- `web/src/app.rs` (modified — diagnostic logging)
- `web/src/components/pitch_comparison_view.rs` (modified — diagnostic logging)
- `web/src/components/pitch_matching_view.rs` (modified — diagnostic logging)
- `docs/planning-artifacts/epics.md` (modified — added epic 8 and story 8.1)
- `docs/implementation-artifacts/8-1-audio-playback-reliability-research.md` (modified — task checkboxes, analysis, strategy)
- `docs/implementation-artifacts/sprint-status.yaml` (modified — status update)
