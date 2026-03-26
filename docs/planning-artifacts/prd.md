---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
inputDocuments:
  - docs/domain-blueprint.md
  - docs/peach-web-prd.md
  - docs/peach-web-ux-spec.md
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 3
workflowType: 'prd'
classification:
  projectType: 'Web App (SPA/PWA, Rust/WASM)'
  domain: 'Music Education / Perceptual Training'
  complexity: 'medium'
  projectContext: 'brownfield'
---

# Product Requirements Document - peach-web

**Author:** Michael
**Date:** 2026-03-02

## Executive Summary

Peach is a browser-based ear training application that builds a perceptual profile of the user's pitch discrimination ability through adaptive pitch discrimination training. Built with Rust compiled to WebAssembly, it reimplements the existing Peach iOS app for the web platform, making the same training approach accessible to anyone with a modern browser — desktop or mobile, any operating system.

The application operates entirely offline after initial page load. There is no server backend, no user accounts, no network dependency. All training data is stored locally in the browser. A static file server delivers the app; the browser does everything else.

Two training disciplines: **pitch discrimination** (two notes play sequentially, user judges higher or lower) and **pitch matching** (user tunes a note to match a reference by ear). Both disciplines use an adaptive algorithm that narrows difficulty after correct answers and widens it after incorrect ones, converging on the user's detection threshold. A perceptual profile tracks ability across the full MIDI range using Welford's online statistics. MIDI controller input is supported as progressive enhancement — note-on events for rhythm tapping, pitch bend for pitch matching — falling back silently to pointer/keyboard when unavailable.

The developer's primary motivation is learning Rust, WASM, and browser-native APIs through a real, non-trivial project with a well-understood domain. The secondary goal is a genuinely usable tool.

### What Makes This Special

Peach trains — it does not test. Where conventional ear training apps escalate difficulty until failure to produce a score, Peach builds a statistical model of what the user can hear and targets weak spots. No scores, no gamification, no session framing. Every answer is data that improves the model. Wrong answers are information, not failure.

The interaction is radically simple: hear two notes, indicate higher or lower. The intelligence is entirely in the adaptive algorithm, invisible to the user. Training fits into the cracks of a day — 30 seconds is as valid as 30 minutes. It should feel like time well spent.

The web version's differentiator is accessibility, not novelty. Same philosophy, same algorithms, new platform. Remove the iOS gate.

## Project Classification

- **Project Type:** Web application (SPA/PWA) built with Rust/WebAssembly
- **Domain:** Music education / perceptual training
- **Complexity:** Medium — non-trivial domain algorithms (adaptive difficulty, online statistics, real-time audio synthesis, session state machines) but exceptionally well-specified; no regulatory or compliance concerns
- **Project Context:** Brownfield — reimplementation of an existing iOS app with complete domain blueprint, web PRD, and UX specification already documented

## Success Criteria

### User Success

- A musician can open Peach in any modern browser (Chrome, Firefox, Safari, Edge) and train their pitch discrimination without installing anything
- Pitch discrimination training feels reflexive — the loop of hear/answer/feedback runs without noticeable delay or UI friction
- Pitch matching slider responds in real time with no perceptible lag between drag and pitch change
- Training data persists across page refreshes, browser restarts, and device reboots — no silent data loss
- The perceptual profile accurately reflects the user's training history, rebuilt identically from stored records on every launch

### Developer Success

- Hands-on proficiency with Rust, WebAssembly, and browser-native APIs (Web Audio, IndexedDB/localStorage) through building a real, non-trivial application
- Ability to articulate architectural trade-offs made during implementation — framework choice, audio approach, storage strategy, async model
- A working application the developer actually uses

### Technical Success

- Audio playback begins within 50ms of being triggered
- Frequency generation accurate to within 0.1 cent of the target
- UI remains responsive during training — state transitions, observer notifications, and profile updates complete within 16ms
- Profile hydration from stored records completes in under 1 second for up to 10,000 records
- App functions fully offline after initial page load (Service Worker caching)
- WASM binary under 2 MB gzipped (soft target — learning takes priority)

### Measurable Outcomes

- All domain algorithms produce identical results to the iOS implementation when given the same inputs (Kazez formulas, Welford's statistics, tuning system conversions)
- All training disciplines from the iOS app are functional in the browser: pitch discrimination, pitch matching, interval pitch discrimination, interval pitch matching
- Keyboard shortcuts provide a complete hands-free training experience for pitch discrimination

## Product Scope

### Phase 1: Foundation

Project setup (Rust/WASM toolchain, UI framework selection, build pipeline). Domain value types in Rust. Basic oscillator-based NotePlayer (sine wave via Web Audio API). Simple pitch discrimination training loop with fixed difficulty. Minimal UI: start button, higher/lower buttons, feedback indicator.

### Phase 2: Core Training

Full Kazez adaptive algorithm. Perceptual profile with Welford's algorithm. Persistent storage for pitch discrimination records. Profile hydration on startup. Settings screen. TrendAnalyzer and ThresholdTimeline.

### Phase 3: Pitch Matching

PitchMatchingSession state machine. Vertical slider with real-time frequency adjustment. Pitch matching records and profile tracking.

### Phase 4: Visualization and Polish

Profile screen with piano keyboard confidence band. Timeline chart. Interval training disciplines. Keyboard shortcuts. Offline support via Service Worker.

### Phase 5: Interoperability

Data export/import (JSON format). iOS app matching feature. Cross-platform data exchange verification.

## User Journeys

### The Musician — First Encounter

A string player opens Peach in their browser. No account, no onboarding, no tutorial. They see training buttons and a profile preview. They click "Compare." Two notes play. They click "Higher." A brief thumbs-up flashes. The next pair begins. Within seconds they're in the loop — reacting to sounds, not thinking about an app. They close the tab when their rehearsal starts. No session summary, no guilt. Their data is already saved.

### The Musician — Daily Practice

Same musician, two weeks later. Opens Peach during a coffee break. The profile preview on the start page shows their confidence band filling in. They train for three minutes, close the tab, done. The algorithm already knows their weak spots and targets them. Over weeks, the band tightens. They check the full profile — their average threshold has dropped from 40 cents to 18. No celebration screen. The data speaks for itself.

### The Musician — Pitch Matching

A singer switches to pitch matching mode. A reference note plays, then a tunable note starts at a random offset. They drag the vertical slider by ear — no visual guide, no markings. They release when it sounds right. Feedback shows "+4 cents" with a short green arrow. Next reference plays. The loop is slower and more deliberate than pitch discrimination, but equally reflexive once familiar.

### Edge Cases — Interruption and Recovery

Mid-training, the user switches tabs. The browser suspends the AudioContext. Training stops, incomplete trial discarded. When they return, they're back at the start page. One click to resume training — the algorithm picks up where their profile left off. After a browser crash, all training records are intact in local storage. After three months away, the app is identical — no "welcome back," no streak reset, just the start page and a profile that remembers everything.

### Journey Requirements Summary

All journeys require the same core capabilities:
- **Instant start/stop** — one action to begin training, one action (or tab close) to end it
- **Persistent local storage** — training records survive any interruption
- **Adaptive algorithm** — targets user's actual ability level without manual configuration
- **Profile visualization** — glanceable progress without scores or gamification
- **AudioContext lifecycle management** — graceful handling of browser audio policies and tab suspension
- **Multi-input support** — mouse, keyboard, touch, and MIDI controller for all interactions (MIDI via progressive enhancement)

## Web Application Requirements

### Project-Type Overview

Single-page application built with Rust/WebAssembly. Static deployment — HTML + WASM + JS + assets served from any CDN or static host. No server-side rendering, no backend logic, no API calls after initial load.

### Browser Support

| Browser | Support Level |
|---|---|
| Chrome (current) | Full support |
| Firefox (current) | Full support |
| Safari (current) | Full support |
| Edge (current) | Full support |
| Internet Explorer | Not supported |

All browsers must support WebAssembly, Web Audio API, and IndexedDB/localStorage.

### Responsive Design

- Single-column layout for training views
- Desktop: keyboard shortcuts as primary power-user input
- Mobile: touch-friendly targets (minimum 44x44px), vertical slider works with touch drag
- No breakpoints for training functionality — core interaction works at any reasonable viewport size
- Viewport meta tag for proper mobile rendering

### Accessibility

WCAG 2.1 AA compliance:
- Semantic HTML with ARIA attributes for custom components (profile visualization, pitch slider)
- Full keyboard navigation for all interactive elements
- Minimum 4.5:1 color contrast for text, 3:1 for UI components
- Visible focus indicators throughout
- `prefers-reduced-motion` and `prefers-color-scheme` respected
- Screen reader announcements for feedback events ("Correct", "Incorrect", "4 cents sharp")
- Audio dependency acknowledged as fundamental domain constraint

### Implementation Considerations

- **Audio context activation:** Web Audio API requires user gesture — training start button serves as activation
- **Audio interruption:** Tab visibility changes and AudioContext suspension handled via Page Visibility API; training stops, incomplete exercise discarded
- **Storage strategy:** localStorage for settings (key-value), IndexedDB for training records (structured data with timestamps) — final decision deferred to Phase 2
- **Offline support:** Service Worker caching of all static assets — deferred to Phase 4
- **SEO:** Not applicable — this is a web application, not a content site

## Risk Mitigation

**Technical Risks:**

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Web Audio API latency exceeds 50ms on some browsers | Medium | High — degrades training quality | Start with oscillators (simplest audio path); measure latency early in Phase 1; if problematic, investigate AudioWorklet |
| Rust/WASM framework (Leptos/Dioxus) doesn't integrate well with Web Audio API | Medium | High — blocks core functionality | Evaluate both frameworks with a minimal audio prototype before committing; fallback to Rust/WASM domain + thin JS audio layer |
| IndexedDB performance degrades with large record sets | Low | Medium — slow startup | Profile hydration is already designed for 10k records in < 1 second; monitor and optimize if needed in Phase 2 |
| SoundFont playback in WASM proves too complex | Medium | Low — oscillators are acceptable | Oscillators are the Phase 1 approach by design; SoundFont is an optional enhancement |

**Resource Risks:**

| Risk | Mitigation |
|---|---|
| Learning curve slows progress | Domain is fully specified — no ambiguity in what to build. The unknown is *how* to build it in Rust/WASM, which is the point |
| Motivation stalls mid-project | Each phase produces a usable increment. Phase 1 alone yields a working (if basic) training app |

## Functional Requirements

### Pitch Discrimination Training

- FR1: User can start pitch discrimination training in unison mode from the start page
- FR2: User can start pitch discrimination training in interval mode from the start page
- FR3: User can hear two sequential notes played at the configured duration and loudness variation
- FR4: User can answer "higher" or "lower" as soon as the second note begins playing (early answer)
- FR5: User can see brief visual feedback (correct/incorrect) after each answer
- FR6: User can stop pitch discrimination training at any time by navigating away
- FR7: System discards incomplete trials silently when training stops
- FR8: System selects the next trial using the adaptive algorithm based on user's perceptual profile and last answer

### Pitch Matching Training

- FR9: User can start pitch matching training in unison mode from the start page
- FR10: User can start pitch matching training in interval mode from the start page
- FR11: User can hear a reference note followed by a tunable note at a random pitch offset
- FR12: User can adjust the tunable note's pitch in real time by dragging a vertical slider
- FR13: User can commit their pitch answer by releasing the slider
- FR14: User can see directional feedback (sharp/flat/center) with signed cent offset after each attempt
- FR15: User can stop pitch matching training at any time by navigating away

### Perceptual Profile

- FR16: User can view a perceptual profile visualization showing pitch discrimination ability across the training range
- FR17: User can view summary statistics: overall mean detection threshold, standard deviation, and trend indicator
- FR18: User can view pitch matching statistics: mean absolute error, standard deviation, and sample count
- FR19: User can see a compact profile preview on the start page
- FR20: User can click the profile preview to navigate to the full profile view
- FR21: System rebuilds the perceptual profile from stored training records on every app launch

### Settings

- FR22: User can configure the training note range (lower and upper MIDI note bounds)
- FR23: User can configure note duration
- FR24: User can configure reference pitch
- FR25: User can select a sound source
- FR26: User can configure loudness variation amount
- FR27: User can select which directed intervals to train
- FR28: User can select the tuning system (equal temperament or just intonation)
- FR29: User can reset all training data with a confirmation step
- FR30: System auto-saves all settings changes to browser storage

### Audio Engine

- FR31: System can play notes at arbitrary frequencies with sub-semitone precision
- FR32: System can play timed notes (fixed duration) and indefinite notes (until explicitly stopped)
- FR33: System can adjust the frequency of a playing note in real time
- FR34: System can vary playback amplitude in decibels
- FR35: System activates the audio context on the user's first training interaction

### Data Persistence

- FR36: System persists all pitch discrimination training records in browser storage
- FR37: System persists all pitch matching training records in browser storage
- FR38: System persists user settings across page refreshes and browser restarts
- FR39: User can export training records and settings to a file
- FR40: User can import training records and settings from a file

### Input & Accessibility

- FR41: User can answer pitch discrimination trials via keyboard shortcuts (Arrow Up/H for higher, Arrow Down/L for lower)
- FR42: User can start training via keyboard (Enter/Space)
- FR43: User can stop training via keyboard (Escape)
- FR44: User can fine-adjust the pitch slider via keyboard (Arrow Up/Down)
- FR45: User can commit pitch via keyboard (Enter/Space)
- FR46: System provides screen reader announcements for training feedback events
- FR50: User can tap rhythm training beats via MIDI note-on events from any connected MIDI controller (progressive enhancement — pointer/keyboard input always available)
- FR51: User can control the pitch matching slider via MIDI pitch bend wheel, with auto-start on first deflection and commit on return-to-center (progressive enhancement)
- FR52: System detects Web MIDI API availability and silently falls back to pointer/keyboard when MIDI is unavailable or permission is denied

### Navigation

- FR47: User can navigate between start page, training views, profile, settings, and info
- FR48: System returns to start page after any training interruption (tab hidden, AudioContext suspended)
- FR49: User can access settings and profile from within training views (which stops training)

## Non-Functional Requirements

### Performance

- NFR1: Audio playback onset within 50ms of trigger. Degradation beyond 100ms noticeably harms training quality.
- NFR2: Frequency generation accurate to within 0.1 cent of target frequency (mathematical precision, not hardware output).
- NFR3: State machine transitions, observer notifications, and profile updates complete within a single frame (16ms).
- NFR4: Profile hydration (replaying all stored records) completes in under 1 second for up to 10,000 records.
- NFR5: Real-time pitch adjustment on the tunable note has no perceptible lag between slider input and audible frequency change.

### Data Integrity

- NFR6: Training records survive page refresh, browser crash, and device restart.
- NFR7: Storage writes are as atomic as the browser platform allows — no partial writes that corrupt data.
- NFR8: If a storage write fails, the user is informed. No silent data loss.
- NFR9: Profile rebuilt from stored records produces identical results on every launch given the same record set.

### Offline Capability

- NFR10: After initial page load, the app functions with zero network requests.
- NFR11: All assets (WASM, JS, CSS, audio data) cached via Service Worker for offline access.
- NFR12: WASM binary plus all assets under 2 MB gzipped (soft target — learning takes priority over optimization).

### Browser Compatibility

- NFR13: Full functionality in current versions of Chrome, Firefox, Safari, and Edge.
- NFR14: Graceful handling of browser-specific AudioContext policies (autoplay restrictions, tab suspension).
- NFR15: Functional at 200% browser zoom without layout breakage.
