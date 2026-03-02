# Peach Web — Product Requirements Document

**Project:** Peach Web (public name: "Peach")
**Type:** Browser-based ear training application
**Status:** Draft
**Date:** 2026-03-02

---

## 1. Purpose and Motivation

Peach Web is a browser-only reimplementation of the Peach iOS ear training app, built with Rust compiled to WebAssembly. The primary motivation is **the developer's learning**: gaining hands-on experience with Rust, WASM, and browser-native APIs through a real, non-trivial project with a well-understood domain.

The secondary goal is making Peach accessible to users without iOS devices.

### 1.1 What This PRD Covers

Only web-specific requirements — goals, constraints, technology decisions, and browser-specific UX considerations. All domain logic (training model, algorithms, state machines, data types, port interfaces) is specified in `docs/domain-blueprint.md` and is **not repeated here**.

### 1.2 What This PRD Does Not Cover

- Domain model, training algorithms, perceptual profile, session state machines — see Domain Blueprint
- Detailed UI visual design (colors, typography, spacing) — to be determined during implementation
- Marketing, analytics, monetization — not in scope

---

## 2. Goals and Success Criteria

| # | Goal | Success Metric |
|---|---|---|
| G1 | Learn Rust/WASM in a real project context | Developer can explain architectural trade-offs made and Rust patterns used |
| G2 | Feature parity with iOS app | All training modes, profile visualization, and settings from the iOS app are functional in the browser |
| G3 | Offline capability | App works without network after initial page load |
| G4 | Data portability with iOS app | Training records and settings can be exported from one app and imported into the other via a file-based interchange format |
| G5 | Acceptable audio quality | SoundFont playback with sub-semitone pitch accuracy and latency low enough for sequential note comparison |

---

## 3. Constraints

### 3.1 Technology Stack

| Layer | Technology | Notes |
|---|---|---|
| Language | Rust | Compiled to WASM |
| Target | WebAssembly | Runs in browser, no server-side logic |
| UI framework | Leptos or Dioxus | Decision deferred to early implementation task |
| Audio | Web Audio API | Accessed via `web-sys` or JS interop |
| Storage | Browser-local (IndexedDB or localStorage) | No server database |
| Hosting | Static file server | HTML + WASM + JS + assets. Any CDN or static host suffices |

### 3.2 Hard Constraints

- **No backend logic.** The web server serves static files only. Once loaded, the app must function entirely in the browser with zero network requests.
- **No shared code with iOS project.** The two codebases are independent. Domain logic is reimplemented in Rust from the Domain Blueprint specification.
- **Browser compatibility.** Must work in current versions of Chrome, Firefox, Safari, and Edge. No IE support.

### 3.3 Soft Constraints

- **Mobile-responsive.** Should be usable on phone browsers, but native-app UX quality on mobile is not required.
- **Bundle size.** WASM binary should be reasonable for initial load (target: under 2 MB gzipped including SoundFont data). Not a hard gate — learning takes priority.

---

## 4. Feature Requirements

All features below correspond to iOS app features. Domain behavior is identical as specified in the Domain Blueprint. Only web-specific differences are noted.

### 4.1 Comparison Training (Core)

Full comparison training loop as specified in Domain Blueprint Section 7.1.

**Web-specific notes:**
- Answer buttons ("Higher" / "Lower") must be keyboard-accessible (e.g. arrow keys or H/L keys) in addition to click/tap.
- Feedback indicator (400ms) — visual only. No haptic feedback on web; consider a brief visual flash or color pulse on incorrect answers as a substitute.

### 4.2 Pitch Matching Training

Full pitch matching loop as specified in Domain Blueprint Section 7.2.

**Web-specific notes:**
- The vertical slider must work with mouse drag, touch drag, and optionally keyboard (arrow keys for fine adjustment).
- Real-time frequency adjustment via Web Audio API. Latency requirements are less strict than comparison mode since the user is continuously adjusting, not making snap judgments.

### 4.3 Perceptual Profile Visualization

Display the user's pitch discrimination and matching statistics.

**Web-specific notes:**
- Piano keyboard with confidence band overlay — render via Canvas or SVG.
- Summary statistics (overall mean, standard deviation, trend indicator).
- Pitch matching statistics (mean absolute error, standard deviation, sample count).

### 4.4 Settings

All settings from Domain Blueprint Section 9.4 (UserSettings):
- Note range (min/max MIDI note)
- Note duration
- Reference pitch (440Hz, 442Hz, 432Hz, 415Hz)
- Sound source selection
- Vary loudness slider
- Interval selection (which directed intervals to train)
- Tuning system (equal temperament / just intonation)
- Reset all training data (with confirmation)

**Web-specific notes:**
- Settings are persisted in browser storage (localStorage is sufficient for key-value settings).
- Settings must survive page refresh and browser restart.

### 4.5 Data Import/Export

**New feature not in iOS v0.2 — but planned for both platforms.**

- Export training records (comparison + pitch matching) and settings to a file (JSON or CSV).
- Import a file to restore or merge training data.
- File format specification to be defined when this feature is implemented. Must be compatible between iOS and web.

### 4.6 Audio Engine

Implement the `NotePlayer` and `PlaybackHandle` port interfaces using Web Audio API.

**Requirements:**
- Play SoundFont instruments at arbitrary frequencies with sub-semitone precision.
- Support timed playback (play for N seconds, then stop) and indefinite playback (play until explicitly stopped).
- Support real-time frequency adjustment on a playing note (for pitch matching mode).
- Amplitude control in dB (for vary-loudness feature).

**Approach options (to be decided during implementation):**
1. **Rust SoundFont synthesizer** (e.g. `rustysynth`, `oxisynth`) compiled to WASM, outputting to Web Audio API via `AudioWorkletNode` or `ScriptProcessorNode`.
2. **JavaScript SoundFont library** with Rust/WASM domain logic calling into JS for audio.
3. **Web Audio API oscillators** as a simpler starting point (sine/triangle waves), upgrading to SoundFont later.

Option 3 is recommended as the initial approach — it unblocks all training logic without the complexity of SoundFont parsing. SoundFont support can be added as a later enhancement.

---

## 5. Non-Functional Requirements

### 5.1 Audio Latency

- Note playback must begin within 50ms of being triggered. (iOS target is 10ms; browser constraints make this harder to achieve, but 50ms is acceptable for sequential comparison training.)
- If latency exceeds 100ms, training quality degrades noticeably.

### 5.2 Pitch Accuracy

- Frequency generation must be accurate to within 0.1 cent of the target frequency.
- This is a mathematical requirement on the frequency calculation, not on the audio output hardware.

### 5.3 Offline Capability

- After the initial page load, the app must function without any network connection.
- All assets (WASM, JS, CSS, SoundFont data) must be cached via a Service Worker or equivalent mechanism.
- Training data and settings must persist across browser sessions using browser storage APIs.

### 5.4 Data Integrity

- Training records must survive page refresh, browser crash, and device restart.
- IndexedDB or localStorage writes should be as atomic as the platform allows.
- No training data should be silently lost. If a write fails, the user should be informed.

### 5.5 Performance

- UI must remain responsive during training. State machine transitions, observer notifications, and profile updates must complete within a single frame (16ms).
- Profile hydration on startup (replaying all records) should complete in under 1 second for up to 10,000 records.

---

## 6. User Experience Considerations

### 6.1 Differences from iOS

| Aspect | iOS | Web | Implication |
|---|---|---|---|
| Input | Touch | Mouse, keyboard, touch | Support all three input methods |
| Haptic feedback | UIKit haptics on incorrect answer | Not available | Substitute with visual feedback |
| Audio session | AVAudioSession manages interruptions | AudioContext suspension, tab visibility | Handle via Page Visibility API + AudioContext state |
| Navigation | SwiftUI NavigationStack | Client-side routing or single-page tabs | Simpler navigation model is fine |
| Sound sources | Bundled .sf2 file with preset discovery | Bundled or user-uploaded SoundFont, or built-in oscillators | Start with oscillators, add SoundFont later |

### 6.2 Keyboard Shortcuts

The web version should support keyboard-driven training for power users:

| Action | Suggested Shortcut |
|---|---|
| Answer "Higher" | Arrow Up or H |
| Answer "Lower" | Arrow Down or L |
| Start training | Enter or Space |
| Stop training | Escape |
| Pitch slider (fine adjust) | Arrow Up/Down (when slider focused) |

### 6.3 Browser Audio Context

Web Audio API requires a user gesture to create or resume an AudioContext. The app must handle this gracefully — e.g. the "Start Training" button doubles as the audio context activation gesture.

---

## 7. Implementation Phases

The web app should ramp up to full feature parity. Suggested phasing:

### Phase 1: Foundation
- Project setup (Rust/WASM toolchain, UI framework selection, build pipeline)
- Domain value types in Rust (MIDINote, Cents, Frequency, Interval, etc.)
- Basic oscillator-based NotePlayer (sine wave via Web Audio API)
- Simple comparison training loop (state machine, no adaptive algorithm yet — fixed difficulty)
- Minimal UI: start button, higher/lower buttons, feedback indicator

### Phase 2: Core Training
- Full Kazez adaptive algorithm
- Perceptual profile with Welford's algorithm
- Persistent storage (IndexedDB or localStorage) for comparison records
- Profile hydration on startup
- Settings screen with all configurable options
- TrendAnalyzer and ThresholdTimeline

### Phase 3: Pitch Matching
- PitchMatchingSession state machine
- Vertical slider component with real-time frequency adjustment
- Pitch matching records and profile tracking
- Pitch matching feedback indicator

### Phase 4: Visualization and Polish
- Profile screen with piano keyboard confidence band
- Timeline chart (rolling mean/stdDev)
- Interval training mode (directed intervals in both sessions)
- Keyboard shortcuts
- Offline support (Service Worker)

### Phase 5: Interoperability
- Data export/import (JSON format for training records + settings)
- iOS app: add matching export/import feature
- Cross-platform data exchange verification

---

## 8. Open Decisions

These must be resolved during implementation, not upfront:

| # | Decision | When to Decide |
|---|---|---|
| D1 | UI framework: Leptos vs Dioxus | Phase 1 — after evaluating both with a minimal prototype |
| D2 | Audio approach: oscillators first vs SoundFont from day one | Phase 1 — oscillators recommended as starting point |
| D3 | Storage: IndexedDB vs localStorage | Phase 2 — depends on data volume expectations and framework support |
| D4 | SoundFont strategy: Rust-native parser vs JS interop | Phase 4 or later — when upgrading from oscillators |
| D5 | Data interchange format: JSON schema definition | Phase 5 — when implementing import/export |
| D6 | PWA: Service Worker caching strategy | Phase 4 — when adding offline support |
