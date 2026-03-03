---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
inputDocuments:
  - docs/planning-artifacts/prd.md
  - docs/planning-artifacts/architecture.md
  - docs/planning-artifacts/ux-design-specification.md
  - docs/ios-reference/domain-blueprint.md (reference)
---

# peach-web - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for peach-web, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

**Comparison Training**
- FR1: User can start comparison training in unison mode from the start page
- FR2: User can start comparison training in interval mode from the start page
- FR3: User can hear two sequential notes played at the configured duration and loudness variation
- FR4: User can answer "higher" or "lower" as soon as the second note begins playing (early answer)
- FR5: User can see brief visual feedback (correct/incorrect) after each answer
- FR6: User can stop comparison training at any time by navigating away
- FR7: System discards incomplete comparisons silently when training stops
- FR8: System selects the next comparison using the adaptive algorithm based on user's perceptual profile and last answer

**Pitch Matching Training**
- FR9: User can start pitch matching training in unison mode from the start page
- FR10: User can start pitch matching training in interval mode from the start page
- FR11: User can hear a reference note followed by a tunable note at a random pitch offset
- FR12: User can adjust the tunable note's pitch in real time by dragging a vertical slider
- FR13: User can commit their pitch answer by releasing the slider
- FR14: User can see directional feedback (sharp/flat/center) with signed cent offset after each attempt
- FR15: User can stop pitch matching training at any time by navigating away

**Perceptual Profile**
- FR16: User can view a perceptual profile visualization showing pitch discrimination ability across the training range
- FR17: User can view summary statistics: overall mean detection threshold, standard deviation, and trend indicator
- FR18: User can view pitch matching statistics: mean absolute error, standard deviation, and sample count
- FR19: User can see a compact profile preview on the start page
- FR20: User can click the profile preview to navigate to the full profile view
- FR21: System rebuilds the perceptual profile from stored training records on every app launch

**Settings**
- FR22: User can configure the training note range (lower and upper MIDI note bounds)
- FR23: User can configure note duration
- FR24: User can configure reference pitch
- FR25: User can select a sound source
- FR26: User can configure loudness variation amount
- FR27: User can select which directed intervals to train
- FR28: User can select the tuning system (equal temperament or just intonation)
- FR29: User can reset all training data with a confirmation step
- FR30: System auto-saves all settings changes to browser storage

**Audio Engine**
- FR31: System can play notes at arbitrary frequencies with sub-semitone precision
- FR32: System can play timed notes (fixed duration) and indefinite notes (until explicitly stopped)
- FR33: System can adjust the frequency of a playing note in real time
- FR34: System can vary playback amplitude in decibels
- FR35: System activates the audio context on the user's first training interaction

**Data Persistence**
- FR36: System persists all comparison training records in browser storage
- FR37: System persists all pitch matching training records in browser storage
- FR38: System persists user settings across page refreshes and browser restarts
- FR39: User can export training records and settings to a file
- FR40: User can import training records and settings from a file

**Input & Accessibility**
- FR41: User can answer comparisons via keyboard shortcuts (Arrow Up/H for higher, Arrow Down/L for lower)
- FR42: User can start training via keyboard (Enter/Space)
- FR43: User can stop training via keyboard (Escape)
- FR44: User can fine-adjust the pitch slider via keyboard (Arrow Up/Down)
- FR45: User can commit pitch via keyboard (Enter/Space)
- FR46: System provides screen reader announcements for training feedback events

**Navigation**
- FR47: User can navigate between start page, training views, profile, settings, and info
- FR48: System returns to start page after any training interruption (tab hidden, AudioContext suspended)
- FR49: User can access settings and profile from within training views (which stops training)

### NonFunctional Requirements

**Performance**
- NFR1: Audio playback onset within 50ms of trigger. Degradation beyond 100ms noticeably harms training quality.
- NFR2: Frequency generation accurate to within 0.1 cent of target frequency (mathematical precision, not hardware output).
- NFR3: State machine transitions, observer notifications, and profile updates complete within a single frame (16ms).
- NFR4: Profile hydration (replaying all stored records) completes in under 1 second for up to 10,000 records.
- NFR5: Real-time pitch adjustment on the tunable note has no perceptible lag between slider input and audible frequency change.

**Data Integrity**
- NFR6: Training records survive page refresh, browser crash, and device restart.
- NFR7: Storage writes are as atomic as the browser platform allows — no partial writes that corrupt data.
- NFR8: If a storage write fails, the user is informed. No silent data loss.
- NFR9: Profile rebuilt from stored records produces identical results on every launch given the same record set.

**Offline Capability**
- NFR10: After initial page load, the app functions with zero network requests.
- NFR11: All assets (WASM, JS, CSS, audio data) cached via Service Worker for offline access.
- NFR12: WASM binary plus all assets under 2 MB gzipped (soft target — learning takes priority over optimization).

**Browser Compatibility**
- NFR13: Full functionality in current versions of Chrome, Firefox, Safari, and Edge.
- NFR14: Graceful handling of browser-specific AudioContext policies (autoplay restrictions, tab suspension).
- NFR15: Functional at 200% browser zoom without layout breakage.

### Additional Requirements

**From Architecture — Starter Template & Project Structure**
- Leptos CSR + Trunk starter template (cargo generate from leptos-rs/start-trunk)
- Cargo workspace with two crates: `domain` (pure Rust, no browser deps) and `web` (Leptos + web-sys + adapters)
- Tailwind CSS integrated with Trunk build pipeline
- Compiler-enforced separation: domain crate must have zero browser/WASM dependencies

**From Architecture — Audio**
- OscillatorNotePlayer as Phase 1 audio (Web Audio OscillatorNode + GainNode)
- SoundFontNotePlayer via OxiSynth crate for Phase 2 audio quality upgrade
- Hybrid audio: oscillator fallback while SoundFont loads and as resilient default
- AudioContext lifecycle: created on first user gesture, shared via Rc<RefCell<AudioContext>>
- Tab visibility change detection via Page Visibility API — stop training on tab hide

**From Architecture — Data & Async**
- Split storage: IndexedDB for training records (comparison_records, pitch_matching_records object stores), localStorage for settings (peach.* prefix keys)
- Async model: wasm-bindgen-futures for spawning Rust futures, gloo-timers for delays
- Session cancellation via Rc<Cell<bool>> flag checked between await points
- Serialization: serde + serde_json; timestamps as ISO 8601 strings; enum variants as camelCase

**From Architecture — UI & Integration**
- UIObserver bridge pattern: domain observers fire-and-forget, UIObserver writes Leptos RwSignals
- Client-side routing via leptos_router (/, /training/comparison, /training/pitch-matching, /profile, /settings, /info)
- Component architecture: one view component per route, custom components for profile visualization, pitch slider, feedback indicators
- Signal naming convention: session_state, show_feedback, is_last_correct
- Domain type names must match the domain blueprint exactly (MIDINote, DetunedMIDINote, CompletedComparison, etc.)

**From Architecture — Error Handling & Patterns**
- Idiomatic Rust error handling: Result<T,E>, custom error enums with thiserror, no silent failures
- Observers must not panic — errors caught internally and logged via web_sys::console::warn_1()
- unwrap()/expect() only for genuine programming errors (invariant violations)
- Storage write failures surfaced to user (NFR8 compliance)

**From Architecture — Deployment**
- Static files served from Apache on personal website
- trunk build --release produces dist/ folder for upload
- CI/CD deferred — local testing (cargo test for domain, manual browser testing for web) initially

**From UX Design — Responsive & Layout**
- Single-column layout for training views, no breakpoints needed
- Mobile-first with Tailwind responsive utilities (md:, lg: prefixes)
- Minimum 44x44px touch targets on all interactive elements
- Content centered at comfortable maximum width on desktop
- Viewport meta tag: width=device-width, initial-scale=1

**From UX Design — Accessibility (WCAG 2.1 AA)**
- Semantic HTML first (<button>, <a>, <input>, <select>, <dialog>), ARIA only where insufficient
- Full keyboard navigation for all interactive elements
- aria-live="polite" region for feedback announcements ("Correct", "Incorrect", "4 cents sharp")
- Skip link at top of page: "Skip to main content"
- Focus management: when navigating to training view, focus first interactive element; on stop, focus returns to start page
- Visible focus rings on all interactive elements
- 4.5:1 color contrast for text, 3:1 for UI components
- prefers-reduced-motion respected (eliminate fade transitions if set)
- prefers-color-scheme dark mode support via Tailwind dark: variants

**From UX Design — Interaction Patterns**
- Comparison feedback: thumbs up/down, green/red, ~400ms, equal visual weight for correct/incorrect
- Pitch matching feedback: directional arrow + signed cent offset, color coded by proximity band (green/yellow/red)
- Early answer: buttons enabled the moment second note begins playing
- No onboarding, no tutorials, no session framing, no gamification
- Auto-save all settings changes — no save/cancel buttons
- Reset all training data requires explicit confirmation dialog
- All interruptions follow same rule: stop audio, discard incomplete, return to start page

### FR Coverage Map

- FR1: Epic 1 — Start comparison training in unison mode
- FR2: Epic 5 — Start comparison training in interval mode
- FR3: Epic 1 — Hear two sequential notes at configured duration/loudness
- FR4: Epic 1 — Answer higher/lower as soon as second note begins (early answer)
- FR5: Epic 1 — See brief visual feedback after each answer
- FR6: Epic 1 — Stop comparison training by navigating away
- FR7: Epic 1 — Incomplete comparisons discarded silently
- FR8: Epic 1 — Adaptive algorithm selects next comparison
- FR9: Epic 4 — Start pitch matching in unison mode
- FR10: Epic 5 — Start pitch matching in interval mode
- FR11: Epic 4 — Hear reference note followed by tunable note at random offset
- FR12: Epic 4 — Adjust tunable note pitch via vertical slider
- FR13: Epic 4 — Commit pitch answer by releasing slider
- FR14: Epic 4 — See directional feedback with signed cent offset
- FR15: Epic 4 — Stop pitch matching by navigating away
- FR16: Epic 3 — View perceptual profile visualization
- FR17: Epic 3 — View summary statistics (mean, std dev, trend)
- FR18: Epic 3 — View pitch matching statistics
- FR19: Epic 3 — See compact profile preview on start page
- FR20: Epic 3 — Click profile preview to navigate to full profile
- FR21: Epic 1 — System rebuilds profile from stored records on launch
- FR22: Epic 2 — Configure training note range
- FR23: Epic 2 — Configure note duration
- FR24: Epic 2 — Configure reference pitch
- FR25: Epic 2 — Select sound source
- FR26: Epic 2 — Configure loudness variation
- FR27: Epic 2 — Select directed intervals to train
- FR28: Epic 2 — Select tuning system
- FR29: Epic 2 — Reset all training data with confirmation
- FR30: Epic 1 — Auto-save settings to browser storage
- FR31: Epic 1 — Play notes at arbitrary frequencies with sub-semitone precision
- FR32: Epic 1 — Play timed and indefinite notes
- FR33: Epic 4 — Adjust frequency of playing note in real time
- FR34: Epic 1 — Vary playback amplitude in decibels
- FR35: Epic 1 — Activate audio context on first training interaction
- FR36: Epic 1 — Persist comparison training records
- FR37: Epic 4 — Persist pitch matching training records
- FR38: Epic 1 — Persist user settings across refreshes/restarts
- FR39: Epic 6 — Export training records and settings to file
- FR40: Epic 6 — Import training records and settings from file
- FR41: Epic 1 — Answer comparisons via keyboard shortcuts
- FR42: Epic 1 — Start training via keyboard
- FR43: Epic 1 — Stop training via keyboard (Escape)
- FR44: Epic 4 — Fine-adjust pitch slider via keyboard
- FR45: Epic 4 — Commit pitch via keyboard
- FR46: Epic 6 — Screen reader announcements for feedback events
- FR47: Epic 6 — Navigate between all views (complete with Info view)
- FR48: Epic 1 — Return to start page after training interruption
- FR49: Epic 2 — Access settings/profile from within training views

## Epic List

### Epic 1: Core Comparison Training
User can open Peach, click "Comparison," hear two notes via oscillator, answer higher/lower via click or keyboard, see feedback, and keep training. Data persists across sessions. The adaptive algorithm targets weak spots from the start.
**FRs covered:** FR1, FR3, FR4, FR5, FR6, FR7, FR8, FR21, FR30, FR31, FR32, FR34, FR35, FR36, FR38, FR41, FR42, FR43, FR48

### Epic 2: Training Customization
User can configure their training experience — note range, note duration, reference pitch, sound source, loudness variation, tuning system — and reset all training data if desired. Settings accessible from within training views.
**FRs covered:** FR22, FR23, FR24, FR25, FR26, FR27, FR28, FR29, FR49

### Epic 3: Perceptual Profile & Visualization
User can see their pitch discrimination ability visualized on a piano keyboard with confidence band, view summary statistics (mean threshold, standard deviation, trend), and see a compact profile preview on the start page.
**FRs covered:** FR16, FR17, FR18, FR19, FR20

### Epic 4: Pitch Matching Training
User can do pitch matching training — hear a reference note, tune a note by ear using a vertical slider with real-time pitch adjustment, see directional feedback with cent accuracy. Matching data persists and feeds into the profile.
**FRs covered:** FR9, FR11, FR12, FR13, FR14, FR15, FR33, FR37, FR44, FR45

### Epic 5: Interval Training & Sound Quality
User can train comparison and pitch matching with musical intervals (minor second through octave). Richer instrument sounds available via SoundFont. Interval selection setting and sound source selection become fully meaningful.
**FRs covered:** FR2, FR10

### Epic 6: Offline Support, Accessibility Polish & Data Portability
App works fully offline after initial load via Service Worker. Complete screen reader support for all feedback events. Info view. User can export and import training data in JSON format.
**FRs covered:** FR39, FR40, FR46, FR47
