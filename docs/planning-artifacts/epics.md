---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
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

## Epic 1: Core Comparison Training

User can open Peach, click "Comparison," hear two notes via oscillator, answer higher/lower via click or keyboard, see feedback, and keep training. Data persists across sessions. The adaptive algorithm targets weak spots from the start.

### Story 1.1: Project Scaffold

**As a** developer,
**I want** a working Cargo workspace with Leptos CSR, Trunk, and Tailwind CSS configured,
**So that** I have a solid foundation to build the application on.

**Acceptance Criteria:**

**Given** a fresh checkout of the repository
**When** I run `trunk serve`
**Then** the application compiles and serves a page at localhost
**And** the page displays "Peach" as a heading

**Given** the workspace is set up
**When** I inspect the project structure
**Then** there is a `domain` crate with `src/lib.rs` and zero browser dependencies in Cargo.toml
**And** there is a `web` crate with `src/main.rs` that depends on the domain crate
**And** there is a workspace-level `Cargo.toml`

**Given** Tailwind CSS is configured
**When** the app is built with `trunk build`
**Then** Tailwind utility classes are processed and included in the output CSS

**Given** the project builds
**When** I run `cargo test -p domain`
**Then** the domain crate compiles and tests pass (even if empty)
**And** the domain crate has no `web-sys`, `wasm-bindgen`, or `leptos` dependencies

**Given** the `index.html` entry point
**When** I inspect it
**Then** it includes `<meta name="viewport" content="width=device-width, initial-scale=1">`

### Story 1.2: Domain Value Types & Tuning System

**As a** developer,
**I want** all domain value types and the tuning system implemented with unit tests,
**So that** the foundational types are correct, precise, and ready for use by sessions and profiles.

**Acceptance Criteria:**

**Given** the domain crate
**When** I construct a MIDINote with rawValue 60
**Then** it succeeds and `name` returns "C4"
**And** rawValue 69 returns "A4"

**Given** a MIDINote
**When** I construct with a value outside 0-127
**Then** it panics (programming error invariant)

**Given** a MIDINote
**When** I call `random(in: 36..=84)`
**Then** the result is within that range

**Given** a MIDINote with rawValue 60
**When** I call `transposed(by: DirectedInterval(perfectFifth, up))`
**Then** the result has rawValue 67

**Given** Cents, Frequency, DetunedMIDINote, NoteDuration, MIDIVelocity, AmplitudeDB, UnitInterval, SoundSourceID
**When** constructed
**Then** each enforces its constraints: Frequency panics on <= 0, NoteDuration clamps to 0.3-3.0, UnitInterval clamps to 0.0-1.0, MIDIVelocity panics outside 1-127

**Given** Interval enum
**When** I call `between(MIDINote(60), MIDINote(67))`
**Then** it returns `perfectFifth`
**And** distance > 12 returns an error

**Given** DirectedInterval
**When** I call `between(MIDINote(60), MIDINote(67))`
**Then** it returns `DirectedInterval(perfectFifth, up)`
**And** `between(MIDINote(67), MIDINote(60))` returns `DirectedInterval(perfectFifth, down)`

**Given** TuningSystem::EqualTemperament with referencePitch 440Hz
**When** I convert `DetunedMIDINote(MIDINote(69), Cents(0.0))` to frequency
**Then** the result is exactly 440.0 Hz

**Given** TuningSystem with equal temperament
**When** I convert various MIDI notes to frequencies
**Then** results are accurate to within 0.1 cent of mathematically correct values (NFR2)

**Given** TuningSystem::JustIntonation
**When** I query cent offsets for all intervals
**Then** values match the blueprint table (e.g. perfectFifth = 701.955 cents)

**Given** all types are implemented
**When** I run `cargo test -p domain`
**Then** all tests pass with zero browser dependencies

### Story 1.3: Perceptual Profile & Adaptive Algorithm

**As a** developer,
**I want** the perceptual profile, adaptive algorithm, trend analyzer, and threshold timeline implemented with unit tests,
**So that** the core intelligence of the app is correct and ready for integration.

**Acceptance Criteria:**

**Given** a fresh PerceptualProfile
**When** I call `update(MIDINote(60), centOffset: 50.0, isCorrect: true)`
**Then** note 60's mean is 50.0, sampleCount is 1, stdDev is 0.0

**Given** multiple updates for the same note
**When** I check statistics
**Then** mean and stdDev match Welford's online algorithm
**And** results are identical to manual computation of mean and sample standard deviation

**Given** trained and untrained notes in the profile
**When** I call `weakSpots(count: 10)`
**Then** untrained notes rank weakest (infinite score)
**And** among trained notes, higher mean = weaker

**Given** trained notes in the profile
**When** I query `overallMean` and `overallStdDev`
**Then** overallMean is the average of per-note means across trained notes
**And** overallStdDev is sample std dev of per-note means
**And** both return None when no notes are trained

**Given** pitch matching accumulators
**When** I call `updateMatching` with multiple samples
**Then** matchingMeanAbs tracks running mean of absolute errors via Welford's
**And** matchingStdDev is correct for 2+ samples
**And** returns None when count is 0

**Given** `profile.reset()` is called
**When** I check statistics
**Then** all 128 notes return to defaults and matching accumulators are zeroed

**Given** KazezNoteStrategy with a correct last answer at 50 cents
**When** nextComparison is called
**Then** new magnitude = kazezNarrow(50.0) ≈ 32.3 cents, clamped to [min, max]

**Given** KazezNoteStrategy with an incorrect last answer at 50 cents
**When** nextComparison is called
**Then** new magnitude = kazezWiden(50.0) ≈ 81.8 cents, clamped to [min, max]

**Given** no previous comparison and no profile data
**When** nextComparison is called
**Then** magnitude defaults to maxCentDifference (100 cents — cold start)

**Given** no previous comparison but profile has overallMean
**When** nextComparison is called
**Then** magnitude starts at overallMean, clamped to difficulty range

**Given** nextComparison generates a comparison
**When** the reference note is selected
**Then** it falls within the configured note range
**And** the target note stays within MIDI 0-127 after interval transposition

**Given** TrendAnalyzer with fewer than 20 data points
**When** trend is queried
**Then** it returns None (insufficient data)

**Given** TrendAnalyzer with 20+ points where later half is >5% lower
**When** trend is queried
**Then** it returns "improving"

**Given** ThresholdTimeline receives comparison data
**When** data points are added
**Then** they are recorded with timestamp, centOffset, isCorrect, referenceNote
**And** daily aggregation produces correct meanThreshold and counts

### Story 1.4: App Shell & Routing

**As a** musician,
**I want** to see a start page with a Comparison training button and navigate between views,
**So that** I can access the app's features from a clean, simple hub.

**Acceptance Criteria:**

**Given** the app is loaded in a browser
**When** I navigate to `/`
**Then** I see the Start Page with a "Comparison" button as the primary action
**And** I see navigation links for Settings, Profile, and Info
**And** no onboarding, tutorial, or welcome message is shown

**Given** I am on the Start Page
**When** I click the Settings link
**Then** I navigate to `/settings` and see a placeholder Settings view
**And** I can navigate back to the Start Page

**Given** I am on the Start Page
**When** I click the Profile link
**Then** I navigate to `/profile` and see a placeholder Profile view

**Given** I am on any secondary view
**When** I navigate back
**Then** I return to the Start Page (hub-and-spoke navigation)

**Given** the routes are configured
**When** I access `/training/comparison`
**Then** a placeholder Comparison Training view loads

**Given** the app is rendered
**When** I inspect the HTML
**Then** a "Skip to main content" link is present at the top of the page
**And** semantic HTML elements are used for navigation and main content areas

**Given** the layout
**When** viewed on any viewport width
**Then** content is centered at a comfortable maximum width on desktop
**And** single-column layout works on mobile without horizontal scrolling

### Story 1.5: Audio Engine (Oscillator)

**As a** musician,
**I want** to hear notes played through my browser,
**So that** I can begin ear training with accurate audio.

**Acceptance Criteria:**

**Given** OscillatorNotePlayer is implemented
**When** I call play with a frequency, duration, velocity, and amplitudeDB
**Then** a sine wave plays at the specified frequency via Web Audio OscillatorNode + GainNode
**And** the note stops after the specified duration (FR32)

**Given** OscillatorNotePlayer is implemented
**When** I call play with no duration (indefinite mode)
**Then** the note plays continuously until explicitly stopped via PlaybackHandle.stop() (FR32)

**Given** the audio engine
**When** a note is triggered
**Then** audio onset occurs within 50ms (NFR1)

**Given** a note is playing
**When** it was created with an AmplitudeDB offset
**Then** the GainNode adjusts volume by the specified decibel offset relative to baseline (FR34)

**Given** no user gesture has occurred
**When** the app loads
**Then** no AudioContext is created (FR35)

**Given** the user clicks a training button (first gesture)
**When** the AudioContext is created
**Then** it is stored in a shared reference (Rc<RefCell<AudioContext>>) for reuse
**And** subsequent audio operations use this same context

**Given** an active AudioContext
**When** stopAll() is called
**Then** all currently playing notes stop immediately

**Given** the audio engine plays a specific frequency
**When** the frequency is measured
**Then** it is accurate to the Web Audio API's precision for the requested value (FR31)

### Story 1.6: Comparison Session State Machine

**As a** developer,
**I want** the ComparisonSession state machine implemented with its full training loop and observer pattern,
**So that** the domain logic drives the comparison training experience correctly.

**Acceptance Criteria:**

**Given** ComparisonSession is constructed with injected dependencies
**When** it is in `idle` state
**Then** no audio is playing and no training loop is running

**Given** ComparisonSession is idle
**When** start(intervals) is called with at least one interval
**Then** state transitions to `playingReferenceNote`
**And** the tuning system is snapshot from userSettings for the session

**Given** the training loop is running
**When** a comparison is generated
**Then** the reference note plays at velocity 63, amplitudeDB 0.0
**And** then the target note plays at velocity 63 with calculated amplitude variation
**And** both play for the configured noteDuration (FR3)

**Given** state is `playingReferenceNote`
**When** the reference note finishes
**Then** state transitions to `playingTargetNote`

**Given** state is `playingTargetNote`
**When** the target note finishes and no answer was given
**Then** state transitions to `awaitingAnswer`

**Given** state is `playingTargetNote` or `awaitingAnswer`
**When** handleAnswer(isHigher) is called
**Then** the target note stops if still playing
**And** a CompletedComparison is created with timestamp and session tuning system
**And** all observers receive comparisonCompleted(completed)
**And** state transitions to `showingFeedback` for 400ms (FR4)

**Given** state is `showingFeedback`
**When** 400ms elapses
**Then** showFeedback becomes false and the next comparison begins

**Given** varyLoudness is 0.0
**When** target amplitude is calculated
**Then** amplitudeDB is 0.0

**Given** varyLoudness is > 0.0
**When** target amplitude is calculated
**Then** amplitudeDB is random in [-varyLoudness * 5.0, +varyLoudness * 5.0]

**Given** session is running
**When** stop() is called
**Then** all audio stops, async tasks cancel, state returns to idle, incomplete comparison is discarded (FR7)

**Given** the observer pattern
**When** an observer throws an error
**Then** the error is caught internally and logged — never propagated to the session

### Story 1.7: Comparison Training UI

**As a** musician,
**I want** to click "Comparison," hear two notes, answer higher/lower via click or keyboard, see brief feedback, and keep training,
**So that** I can train my pitch discrimination through the core reflexive loop.

**Acceptance Criteria:**

**Given** I am on the Start Page
**When** I click the "Comparison" button
**Then** I navigate to the Comparison Training view
**And** the AudioContext activates (FR35)
**And** the first comparison begins immediately — no countdown, no delay (FR1)

**Given** I am on the Start Page
**When** I press Enter or Space with the Comparison button focused
**Then** training starts identically to clicking (FR42)

**Given** state is `playingReferenceNote`
**When** the reference note is playing
**Then** Higher and Lower buttons are visually disabled and not clickable/pressable

**Given** state is `playingTargetNote`
**When** the target note begins playing
**Then** Higher and Lower buttons become enabled immediately (FR4 — early answer)

**Given** buttons are enabled
**When** I click "Higher" or press Arrow Up / H
**Then** my answer is registered as "higher" and both buttons disable immediately (FR41)

**Given** buttons are enabled
**When** I click "Lower" or press Arrow Down / L
**Then** my answer is registered as "lower" and both buttons disable immediately (FR41)

**Given** a correct answer
**When** feedback displays
**Then** a green thumbs-up indicator appears for ~400ms (FR5)

**Given** an incorrect answer
**When** feedback displays
**Then** a red thumbs-down indicator appears for ~400ms with same visual weight, position, and duration as correct feedback (FR5)

**Given** feedback is showing
**When** ~400ms elapses
**Then** the indicator disappears and the next comparison begins automatically

**Given** I am in Comparison Training
**When** I press Escape
**Then** training stops and I return to the Start Page (FR43)

**Given** I am in Comparison Training
**When** I click Settings or Profile links
**Then** training stops and I navigate to the selected view (FR6)

**Given** the UIObserver bridge is implemented
**When** domain session state changes
**Then** corresponding Leptos signals update and UI re-renders affected elements

**Given** the composition root
**When** the app starts
**Then** all dependencies are wired: adapters → domain services → sessions → UI (blueprint §11)

**Given** the Comparison Training view
**When** I inspect the HTML
**Then** an `aria-live="polite"` region exists
**And** "Correct" or "Incorrect" is announced to screen readers after each answer

### Story 1.8: Persistence & Profile Hydration

**As a** musician,
**I want** my training data and settings to persist across page refreshes and browser restarts,
**So that** my perceptual profile accumulates over time and my preferences are remembered.

**Acceptance Criteria:**

**Given** the IndexedDB adapter is implemented
**When** a comparison is completed during training
**Then** a ComparisonRecord is saved to `comparison_records` in the `peach` database (FR36)
**And** the record contains: referenceNote, targetNote, centOffset, isCorrect, interval, tuningSystem, timestamp

**Given** records exist in IndexedDB
**When** fetchAllComparisons() is called
**Then** all records are returned sorted by timestamp ascending

**Given** the localStorage adapter is implemented
**When** a setting changes
**Then** it is immediately saved with `peach.` prefix keys (FR30)

**Given** settings exist in localStorage
**When** the app loads
**Then** settings are read and applied (FR38)

**Given** no settings exist in localStorage
**When** the app loads
**Then** sensible defaults are used: noteRangeMin=36, noteRangeMax=84, noteDuration=1.0, referencePitch=440, tuningSystem=equalTemperament, varyLoudness=0.0

**Given** comparison records exist in IndexedDB
**When** the app launches
**Then** all records are replayed through profile.update() to rebuild the PerceptualProfile (FR21)
**And** TrendAnalyzer and ThresholdTimeline are hydrated from the same records
**And** hydration completes in under 1 second for up to 10,000 records (NFR4)

**Given** the profile has been hydrated
**When** the user starts training
**Then** the adaptive algorithm uses hydrated profile data to select the first comparison

**Given** a storage write operation
**When** the write fails
**Then** the user is informed that data may not have been saved (NFR8)
**And** training continues — non-blocking

**Given** training records have been saved
**When** the page is refreshed, browser crashes, or device restarts
**Then** all previously saved records are intact (NFR6)

### Story 1.9: Interruption Handling

**As a** musician,
**I want** training to stop cleanly when I switch tabs or encounter an audio interruption,
**So that** no data is lost and I can resume training with one click.

**Acceptance Criteria:**

**Given** training is in progress
**When** I switch to another browser tab
**Then** training stops immediately, audio stops, incomplete comparison is discarded silently (FR7, FR48)

**Given** training was interrupted by tab switch
**When** I return to the Peach tab
**Then** I see the Start Page (FR48)
**And** I can start training again with one click

**Given** training is in progress
**When** the browser suspends the AudioContext
**Then** training stops with the same behavior as tab hidden (FR48)

**Given** training is in progress
**When** I navigate to Settings or Profile
**Then** training stops and incomplete comparison is discarded (FR6)

**Given** any interruption occurs
**When** training stops
**Then** no error dialogs are shown
**And** no "resume session" prompts appear
**And** no session summary is displayed

**Given** training was interrupted
**When** I start training again
**Then** the algorithm uses my existing profile — no data was lost
**And** training begins from the adaptive algorithm's current state

## Epic 2: Training Customization

User can configure their training experience — note range, note duration, reference pitch, sound source, loudness variation, tuning system — and reset all training data if desired. Settings accessible from within training views.

### Story 2.1: Settings View — Core Training Parameters

**As a** musician,
**I want** to configure my training note range, note duration, reference pitch, sound source, loudness variation, and tuning system,
**So that** I can personalize my training experience to match my instrument and goals.

**Acceptance Criteria:**

**Given** I navigate to the Settings view
**When** the view loads
**Then** all settings display their current values from localStorage
**And** if no values exist, sensible defaults are shown (noteRangeMin=36/C2, noteRangeMax=84/C6, noteDuration=1.0s, referencePitch=440Hz, varyLoudness=0%, tuningSystem=Equal Temperament)

**Given** the note range controls (FR22)
**When** I adjust the lower bound
**Then** it is constrained to valid MIDI note values and cannot exceed the upper bound
**And** the change is immediately saved to localStorage

**Given** the note range controls (FR22)
**When** I adjust the upper bound
**Then** it is constrained to valid MIDI note values and cannot go below the lower bound
**And** the change is immediately saved to localStorage

**Given** the note duration control (FR23)
**When** I adjust the duration
**Then** it is constrained to 0.3-3.0 seconds
**And** the change is immediately saved to localStorage

**Given** the reference pitch control (FR24)
**When** I select a reference pitch (440Hz, 442Hz, 432Hz, 415Hz, or custom)
**Then** the selection is immediately saved to localStorage

**Given** the sound source control (FR25)
**When** I select a sound source from the dropdown
**Then** the selection is immediately saved to localStorage

**Given** the loudness variation control (FR26)
**When** I adjust the slider
**Then** it is constrained to 0-100%
**And** the change is immediately saved to localStorage

**Given** the tuning system control (FR28)
**When** I select Equal Temperament or Just Intonation
**Then** the selection is immediately saved to localStorage

**Given** any setting has been changed
**When** I return to training
**Then** the new setting takes effect on the next comparison

**Given** I access Settings from within a training view (FR49)
**When** the Settings view loads
**Then** training has already stopped (handled by Epic 1 navigation)
**And** I see the fully functional Settings view with all controls

**Given** the Settings view
**When** I navigate back
**Then** I return to the Start Page

**Given** the Settings view on any device
**When** I interact with controls
**Then** all controls have minimum 44x44px touch targets
**And** all controls are keyboard-accessible

### Story 2.2: Interval Selection

**As a** musician,
**I want** to select which directed intervals I want to train,
**So that** I can focus on specific intervals that are relevant to my musical practice.

**Acceptance Criteria:**

**Given** the interval selection control in Settings (FR27)
**When** the control loads
**Then** all directed intervals are displayed (13 intervals × 2 directions, with prime always up-only)
**And** currently selected intervals are checked
**And** default selection is prime/up only (unison)

**Given** the interval multi-select
**When** I check or uncheck an interval
**Then** the selection is immediately saved to localStorage

**Given** the interval selection
**When** I deselect all intervals
**Then** at least one interval must remain selected (prime/up cannot be deselected if it's the last one)

**Given** intervals are displayed
**When** I read the labels
**Then** each shows the interval name and direction (e.g. "Minor Second Up", "Perfect Fifth Down")

**Given** interval selection has been changed
**When** I start training
**Then** the session uses only the selected intervals for comparison generation

### Story 2.3: Reset Training Data

**As a** musician,
**I want** to reset all my training data with a confirmation step,
**So that** I can start fresh if I choose to, without accidentally losing my data.

**Acceptance Criteria:**

**Given** the Settings view
**When** I see the "Reset all training data" button
**Then** it is visually distinct as a destructive action (FR29)

**Given** I click the reset button
**When** the confirmation dialog appears
**Then** it clearly states that all training data will be permanently deleted
**And** it requires explicit confirmation before proceeding

**Given** I confirm the reset
**When** the reset executes
**Then** all comparison records are deleted from IndexedDB
**And** the PerceptualProfile is reset (all 128 notes to defaults, matching accumulators zeroed)
**And** the TrendAnalyzer is reset
**And** the ThresholdTimeline is reset
**And** if a training session is active, it is stopped first

**Given** I cancel the confirmation dialog
**When** the dialog closes
**Then** no data is deleted and the Settings view remains unchanged

**Given** a reset has completed
**When** I start training
**Then** the adaptive algorithm begins at cold-start difficulty (100 cents)
**And** the profile preview shows the empty state

## Epic 3: Perceptual Profile & Visualization

User can see their pitch discrimination ability visualized on a piano keyboard with confidence band, view summary statistics (mean threshold, standard deviation, trend), and see a compact profile preview on the start page.

### Story 3.1: Profile View with Summary Statistics

**As a** musician,
**I want** to view my detection threshold statistics and pitch matching accuracy,
**So that** I can understand my pitch discrimination ability in concrete numbers.

**Acceptance Criteria:**

**Given** I navigate to `/profile`
**When** the Profile view loads
**Then** it replaces the placeholder from Epic 1 with the full profile view
**And** I see a back navigation link to the Start Page

**Given** comparison training data exists
**When** I view summary statistics (FR17)
**Then** I see the overall mean detection threshold in cents
**And** I see the standard deviation
**And** I see a trend indicator (improving/stable/declining)

**Given** fewer than 20 comparison records exist
**When** I view the trend indicator
**Then** the trend indicator is hidden (insufficient data)

**Given** pitch matching data exists
**When** I view pitch matching statistics (FR18)
**Then** I see the mean absolute error in cents
**And** I see the standard deviation
**And** I see the sample count

**Given** no pitch matching data exists
**When** I view the Profile view
**Then** the pitch matching statistics section is hidden or shows dashes

**Given** no training data exists at all (cold start)
**When** I view the Profile view
**Then** statistics show dashes ("—") instead of numbers
**And** the trend indicator is hidden
**And** text reads "Start training to build your profile."

**Given** the Profile view
**When** I inspect the HTML
**Then** statistics use semantic HTML with appropriate headings
**And** the view is keyboard-navigable

### Story 3.2: Perceptual Profile Visualization

**As a** musician,
**I want** to see my pitch discrimination ability visualized across the piano keyboard,
**So that** I can see where my hearing is strong and where it needs work.

**Acceptance Criteria:**

**Given** the Profile view with training data (FR16)
**When** the visualization renders
**Then** I see a horizontal piano keyboard strip with stylized key rectangles
**And** a confidence band area chart is overlaid above the keyboard
**And** the Y-axis is inverted (lower = better discrimination)
**And** note names appear at octave boundaries (C2, C3, C4, etc.)

**Given** training data exists for some notes
**When** the visualization renders (sparse state)
**Then** the band renders where data exists
**And** the band fades or is absent where no data exists
**And** no interpolation across large gaps

**Given** extensive training data (populated state)
**When** the visualization renders
**Then** a continuous confidence band is visible across the trained range

**Given** no training data (empty state)
**When** the visualization renders
**Then** the piano keyboard renders fully
**And** the band is absent or shown as a faint uniform placeholder at 100 cents

**Given** the visualization is implemented in Canvas or SVG
**When** the view is resized
**Then** the visualization uses the full available width

**Given** the visualization
**When** I inspect the HTML
**Then** it has an ARIA role and label on the container
**And** a text alternative: "Perceptual profile: average detection threshold X cents across Y trained notes"

**Given** `prefers-color-scheme: dark` is set
**When** the visualization renders
**Then** colors adapt appropriately for dark mode

### Story 3.3: Profile Preview on Start Page

**As a** musician,
**I want** to see a compact profile preview on the start page that I can click to view details,
**So that** I get a glanceable snapshot of my progress every time I open Peach.

**Acceptance Criteria:**

**Given** the Start Page
**When** it loads with training data (FR19)
**Then** a compact profile preview is visible showing the same band shape as the full visualization
**And** no axis labels or numerical values are shown in the preview

**Given** the Start Page with no training data
**When** it loads (FR19)
**Then** the profile preview shows a subtle placeholder shape

**Given** the profile preview
**When** I click it (FR20)
**Then** I navigate to the full Profile view at `/profile`

**Given** the profile preview
**When** I inspect the HTML
**Then** it has an accessible label: "Your pitch profile. Click to view details."
**And** if data exists: "Your pitch profile. Average threshold: X cents. Click to view details."

**Given** the profile preview
**When** I use keyboard navigation
**Then** it is focusable and activatable via Enter/Space

## Epic 4: Pitch Matching Training

User can do pitch matching training — hear a reference note, tune a note by ear using a vertical slider with real-time pitch adjustment, see directional feedback with cent accuracy. Matching data persists and feeds into the profile.

### Story 4.1: Pitch Matching Session State Machine

**As a** developer,
**I want** the PitchMatchingSession state machine implemented with challenge generation, pitch adjustment, and commit logic,
**So that** the domain logic correctly drives the pitch matching training experience.

**Acceptance Criteria:**

**Given** PitchMatchingSession is constructed with injected dependencies (notePlayer, profile, observers, userSettings)
**When** it is in `idle` state
**Then** no audio is playing and no training loop is running

**Given** PitchMatchingSession is idle
**When** start(intervals) is called
**Then** state transitions to `playingReference`
**And** a challenge is generated with a random reference note in the configured range
**And** the reference note plays for the configured duration at velocity 63

**Given** generateChallenge is called
**When** generating the challenge
**Then** the reference note is random within the note range (adjusted for interval transposition)
**And** the target note is the reference transposed by the selected interval
**And** initialCentOffset is random in [-20.0, +20.0] cents

**Given** state is `playingReference`
**When** the reference note finishes playing
**Then** the tunable note starts at the initial cent offset from the target frequency
**And** state transitions to `awaitingSliderTouch`

**Given** state is `awaitingSliderTouch`
**When** adjustPitch(value) is called (first slider interaction)
**Then** state transitions to `playingTunable`

**Given** state is `playingTunable`
**When** adjustPitch(value) is called with value in [-1.0, +1.0]
**Then** the tunable note's frequency is adjusted in real time (FR33)
**And** the frequency calculation uses: `referenceFrequency * pow(2.0, (value * 20.0) / 1200.0)`

**Given** state is `playingTunable`
**When** commitPitch(value) is called (slider released, FR13)
**Then** the tunable note stops immediately
**And** userCentError is calculated as `1200.0 * log2(userFrequency / referenceFrequency)`
**And** a CompletedPitchMatching is created with all fields and current timestamp
**And** all observers receive pitchMatchingCompleted(result)
**And** state transitions to `showingFeedback` for 400ms

**Given** state is `showingFeedback`
**When** 400ms elapses
**Then** the next reference note plays (loop continues)

**Given** session is running
**When** stop() is called
**Then** all audio stops, async tasks cancel, state returns to idle

### Story 4.2: Vertical Pitch Slider

**As a** musician,
**I want** a vertical slider that I can drag to tune a note by ear,
**So that** I can practice pitch matching using only my hearing as a guide.

**Acceptance Criteria:**

**Given** the Vertical Pitch Slider component
**When** rendered
**Then** it is vertically oriented, occupying most of the training view height
**And** it has a large thumb/handle for comfortable dragging
**And** there are no markings, tick marks, or center indicators

**Given** the slider is in active state
**When** I drag the thumb with a mouse
**Then** the slider value changes continuously as I drag
**And** up = sharper (positive), down = flatter (negative)

**Given** the slider is in active state
**When** I drag with touch
**Then** the slider responds to touch drag identically to mouse drag

**Given** the slider is in active state
**When** I press Arrow Up (FR44)
**Then** the pitch adjusts by a fine increment (upward/sharper)

**Given** the slider is in active state
**When** I press Arrow Down (FR44)
**Then** the pitch adjusts by a fine increment (downward/flatter)

**Given** the slider is being dragged
**When** I release (mouse up / touch end)
**Then** the commit event fires with the current slider value

**Given** the slider
**When** it starts a new challenge
**Then** it always begins at the same physical center position regardless of the pitch offset

**Given** the slider in inactive state (during reference playback)
**When** displayed
**Then** it appears dimmed/disabled and does not respond to input

**Given** the slider
**When** I inspect the HTML
**Then** it has ARIA role "slider" and label "Pitch adjustment"
**And** it is keyboard-operable

### Story 4.3: Pitch Matching Training UI

**As a** musician,
**I want** to start pitch matching from the start page, tune notes by ear with real-time audio feedback, and see how close I was after each attempt,
**So that** I can develop my pitch matching ability through deliberate practice.

**Acceptance Criteria:**

**Given** I am on the Start Page
**When** I click the "Pitch Matching" button (FR9)
**Then** I navigate to the Pitch Matching Training view
**And** the AudioContext activates
**And** the first reference note plays immediately

**Given** state is `playingReference`
**When** the reference note is playing
**Then** the slider is visible but dimmed/disabled (FR11)

**Given** state is `awaitingSliderTouch` or `playingTunable`
**When** the tunable note is playing
**Then** the slider is active and I can drag it
**And** the pitch changes in real time as I drag — no visual proximity feedback, ear only (FR12, FR33)

**Given** I release the slider
**When** feedback displays (FR14)
**Then** I see a directional arrow (up for sharp, down for flat) or dot (dead center)
**And** I see the signed cent offset (e.g. "+4 cents", "-22 cents")
**And** the color indicates proximity: green (<10 cents), yellow (10-30 cents), red (>30 cents)
**And** feedback persists for ~400ms

**Given** I am in Pitch Matching Training
**When** I press Enter or Space (FR45)
**Then** the pitch is committed (same as releasing the slider)

**Given** I am in Pitch Matching Training
**When** I press Escape or click Settings/Profile
**Then** training stops and I navigate away (FR15)
**And** incomplete attempt is discarded

**Given** the UIObserver bridge
**When** PitchMatchingSession state changes
**Then** corresponding Leptos signals update and UI re-renders

**Given** the Pitch Matching Training view
**When** I inspect the HTML
**Then** an `aria-live="polite"` region announces feedback (e.g. "4 cents sharp", "Dead center")

### Story 4.4: Pitch Matching Persistence

**As a** musician,
**I want** my pitch matching results to persist and feed into my perceptual profile,
**So that** my matching accuracy is tracked over time.

**Acceptance Criteria:**

**Given** the IndexedDB adapter
**When** a pitch matching attempt is completed
**Then** a PitchMatchingRecord is saved to `pitch_matching_records` in the `peach` database (FR37)
**And** the record contains: referenceNote, targetNote, initialCentOffset, userCentError, interval, tuningSystem, timestamp

**Given** pitch matching records exist in IndexedDB
**When** fetchAllPitchMatchings() is called
**Then** all records are returned sorted by timestamp ascending

**Given** a pitch matching attempt is completed
**When** the PitchMatchingObserver fires
**Then** the PerceptualProfile's matching accumulators are updated via `updateMatching()`

**Given** pitch matching records exist in IndexedDB
**When** the app launches
**Then** all pitch matching records are replayed through `profile.updateMatching()` during hydration
**And** the Profile view shows pitch matching statistics (FR18, from Epic 3)

**Given** a storage write fails during pitch matching
**When** the error occurs
**Then** the user is informed (NFR8)
**And** training continues

## Epic 5: Interval Training & Sound Quality

User can train comparison and pitch matching with musical intervals (minor second through octave). Richer instrument sounds available via SoundFont. Interval selection setting and sound source selection become fully meaningful.

### Story 5.1: Interval Training Mode

**As a** musician,
**I want** to train comparison and pitch matching with musical intervals,
**So that** I can develop my ability to hear pitch differences within specific interval contexts.

**Acceptance Criteria:**

**Given** the Start Page
**When** it loads
**Then** I see "Interval Comparison" and "Interval Pitch Matching" buttons below a visual separator (FR2, FR10)
**And** these buttons are secondary prominence (below the primary Comparison and Pitch Matching buttons)

**Given** I click "Interval Comparison" (FR2)
**When** the training view loads
**Then** the ComparisonSession starts with the user's selected intervals from Settings (not just prime/up)
**And** the route is `/training/comparison?intervals=<comma-separated interval codes>` (e.g., `?intervals=M3u,M3d,m6u,M6d`)

**Given** I click "Interval Pitch Matching" (FR10)
**When** the training view loads
**Then** the PitchMatchingSession starts with the user's selected intervals
**And** the route is `/training/pitch-matching?intervals=<comma-separated interval codes>`

**Given** training is in interval mode
**When** a comparison or challenge is generated
**Then** a random interval is selected from the user's configured interval set
**And** the target note is transposed from the reference by that interval

**Given** training is in interval mode
**When** the training view renders
**Then** a target interval label is visible showing the current interval name and direction (e.g. "Perfect Fifth Up")
**And** this label is hidden in unison mode

**Given** interval mode training
**When** I answer a comparison or commit a pitch
**Then** the record includes the correct interval semitone distance
**And** the next challenge may use a different interval from the selected set

**Given** the user has selected no non-prime intervals in Settings
**When** they click an interval training button
**Then** training starts with only prime/up (effectively unison mode — same as regular mode)

### Story 5.2: SoundFont Audio

**As a** musician,
**I want** to hear richer instrument sounds during training,
**So that** training feels more musical and the sound source setting becomes meaningful.

**Acceptance Criteria:**

**Given** the app loads
**When** SoundFont loading begins
**Then** the SoundFont file is fetched asynchronously via `fetch()`
**And** the Start Page is interactive immediately with oscillator fallback
**And** no loading indicator is shown — loading is invisible to the user

**Given** the SoundFont has loaded successfully
**When** training starts
**Then** the SoundFontNotePlayer renders notes using OxiSynth with the selected preset
**And** the swap from oscillator to SoundFont is silent — no interruption

**Given** the SoundFont fails to load
**When** training starts
**Then** the OscillatorNotePlayer continues as the fallback
**And** no error message is shown (oscillators are a valid sound source)

**Given** SoundFontNotePlayer is active
**When** a note is played
**Then** the AudioBuffer is rendered from the SoundFont preset via OxiSynth
**And** playback uses AudioBufferSourceNode

**Given** SoundFontNotePlayer is active during pitch matching
**When** adjustFrequency is called
**Then** the pitch is adjusted via `.detune` on the AudioBufferSourceNode (within ±20 cents range)

**Given** the SoundFont has been loaded once
**When** the app is visited again
**Then** the browser cache serves the SoundFont — load is near-instant

**Given** the sound source setting (FR25, from Epic 2)
**When** the user selects a different sound source
**Then** the SoundFont preset changes accordingly
**And** the change takes effect on the next note played

## Epic 6: Offline Support, Accessibility Polish & Data Portability

App works fully offline after initial load via Service Worker. Complete screen reader support for all feedback events. Info view. User can export and import training data in JSON format.

### Story 6.1: Info View & Complete Navigation

**As a** musician,
**I want** to see basic app information and navigate smoothly between all views,
**So that** I know what I'm using and can access every part of the app.

**Acceptance Criteria:**

**Given** I navigate to `/info`
**When** the Info view loads (FR47)
**Then** I see the app name (Peach), developer name, copyright notice, and version number
**And** the content is minimal and static

**Given** the Info view
**When** I navigate back
**Then** I return to the Start Page

**Given** the Start Page
**When** I look at the navigation
**Then** I see links/buttons for all views: Comparison, Pitch Matching, Interval Comparison, Interval Pitch Matching, Settings, Profile, Info (FR47)

**Given** all routes are implemented
**When** I navigate between any views
**Then** client-side routing handles all transitions without full page reloads
**And** hub-and-spoke model is maintained (all views one level deep from start page)

### Story 6.2: Screen Reader Accessibility Polish

**As a** musician using a screen reader,
**I want** all training feedback and state changes announced,
**So that** I can train my pitch discrimination with full accessibility.

**Acceptance Criteria:**

**Given** comparison training (FR46)
**When** I answer a comparison
**Then** the screen reader announces "Correct" or "Incorrect"

**Given** pitch matching training (FR46)
**When** I commit a pitch
**Then** the screen reader announces the result (e.g. "4 cents sharp", "27 cents flat", "Dead center")

**Given** any training mode
**When** training starts
**Then** the screen reader announces "Training started"

**Given** any training mode
**When** training stops
**Then** the screen reader announces "Training stopped"

**Given** interval training mode
**When** the interval changes between challenges
**Then** the screen reader announces the target interval (e.g. "Target interval: Perfect Fifth Up")

**Given** all `aria-live` regions
**When** reviewed for completeness
**Then** all training feedback events across comparison, pitch matching, and interval modes have announcements
**And** announcements use `aria-live="polite"` to avoid interrupting the user

**Given** all interactive elements across all views
**When** audited for accessibility
**Then** visible focus indicators are present on every interactive element
**And** tab order follows visual order in every view
**And** all custom components (profile visualization, pitch slider, feedback indicators) have appropriate ARIA roles and labels

### Story 6.3: Data Export & Import

**As a** musician,
**I want** to export my training data and settings to a file and import them back,
**So that** I can back up my data, transfer it between browsers, or share it with my iOS app.

**Acceptance Criteria:**

**Given** the Settings view (or a dedicated section)
**When** I click "Export Data" (FR39)
**Then** a JSON file is generated containing all comparison records, all pitch matching records, and all settings
**And** the file is downloaded to my device with a descriptive filename (e.g. `peach-export-2026-03-03.json`)

**Given** the export file
**When** I inspect its contents
**Then** comparison records match the ComparisonRecord schema
**And** pitch matching records match the PitchMatchingRecord schema
**And** settings are included as key-value pairs
**And** timestamps are ISO 8601 strings

**Given** the Settings view
**When** I click "Import Data" (FR40)
**Then** a file picker opens for selecting a JSON file

**Given** I select a valid export file
**When** the import executes
**Then** all comparison records from the file are saved to IndexedDB
**And** all pitch matching records from the file are saved to IndexedDB
**And** settings from the file are applied to localStorage
**And** the PerceptualProfile is rebuilt from the combined record set

**Given** I select an invalid or corrupted file
**When** the import is attempted
**Then** an error message is shown
**And** no existing data is modified

**Given** existing data and an import
**When** the import completes
**Then** imported records are merged with existing records (no duplicates based on timestamp)

### Story 6.4: Service Worker & Offline Support

**As a** musician,
**I want** Peach to work fully offline after the first visit,
**So that** I can train anywhere without needing an internet connection.

**Acceptance Criteria:**

**Given** the first visit to Peach
**When** the page loads
**Then** a Service Worker is registered (NFR11)
**And** all static assets are cached: WASM binary, JS, CSS, HTML, and audio data (SoundFont)

**Given** the Service Worker is installed
**When** I visit Peach without an internet connection
**Then** the app loads and functions fully from the cache (NFR10)
**And** all training modes work without any network requests

**Given** the cached assets
**When** I check the total size
**Then** WASM binary plus all assets are under 2 MB gzipped (soft target, NFR12)

**Given** a new version of Peach is deployed
**When** I visit the app with an internet connection
**Then** the Service Worker detects the update and caches new assets
**And** the update is applied on the next page load (no disruptive in-app update)

**Given** the Service Worker is active
**When** I use the app normally
**Then** there is no perceptible difference in behavior compared to online usage

## Epic 7: iOS Parity — Profile & UI Refresh

Port improvements from the sibling iOS app (peach) that have accumulated since the web version reached feature completeness. The iOS app introduced four distinct training modes tracked independently, EWMA-based progress visualization with adaptive time bucketing, redesigned start screen with sparklines, and help content. This epic brings peach-web to parity with those changes.

**Source reference:** iOS repo `mschuerig/peach`, commits from `a1d7320b` onward.

### Story 7.0a: Rename Comparison to PitchComparison

**As a** developer,
**I want** all `Comparison*` types renamed to `PitchComparison*`,
**So that** naming is symmetric with the `PitchMatching*` family and matches the iOS sibling app.

### Story 7.0b: Extract Constants and Thread Domain Types

**As a** developer,
**I want** magic numbers replaced with named constants and raw `f64` parameters replaced with domain types where appropriate,
**So that** the code is self-documenting and type-safe.

### Story 7.1: TrainingMode Enum and TrainingModeConfig

**As a** developer,
**I want** a `TrainingMode` enum with four variants and per-mode configuration,
**So that** profile visualization and progress tracking can distinguish between unison comparison, interval comparison, unison matching, and interval matching.

### Story 7.2: ProgressTimeline with EWMA and Adaptive Bucketing

**As a** musician,
**I want** my training progress tracked with exponentially weighted moving averages and adaptive time bucketing,
**So that** recent training sessions have more weight and the timeline adapts its granularity to data density.

### Story 7.3: Start Screen Redesign

**As a** musician,
**I want** the start screen organized into "Single Notes" and "Intervals" sections with descriptive labels ("Hear & Compare", "Tune & Match"),
**So that** training modes are clearly named and visually grouped.

### Story 7.4: Profile Screen with Progress Charts

**As a** musician,
**I want** to see per-mode progress charts showing EWMA trends with stddev bands,
**So that** I can understand how my pitch discrimination is evolving in each training mode.

### Story 7.5: Training Stats with Trend Arrows

**As a** musician,
**I want** to see my latest result and session best with a trend indicator on the training screens,
**So that** I get immediate feedback on my current performance trajectory.

### Story 7.6: Start Page Sparklines

**As a** musician,
**I want** to see miniature sparklines on each training card on the start page,
**So that** I can quickly see my progress at a glance before starting a session.

### Story 7.7: Help Content

**As a** musician,
**I want** contextual help text available in the app,
**So that** I can understand how each training mode works and what the statistics mean.

---

## Epic 8: Fixes & Improvements

Open-ended epic for bug fixes, reliability improvements, and incremental enhancements discovered during use. Stories are added as issues are identified.

### Story 8.1: Audio Playback Reliability Research

**As a** developer,
**I want** to research why audio playback is intermittently failing and propose a mitigation strategy,
**So that** we can reliably play audio across browsers and sessions without silent failures.

### Story 8.2: Audio Playback Reliability Fix

**As a** user,
**I want** audio to play reliably every time I start a training session,
**So that** I never experience silent training where the UI progresses but no sound is heard.

**Acceptance Criteria:**
1. AudioContext is guaranteed to be in `Running` state before any note playback begins
2. `AudioContext.resume()` is called at training start if context is `Suspended`
3. Worklet init no longer creates AudioContext — deferred to training view (user gesture)
4. `onstatechange` handler attempts `resume()` before interrupting on `Suspended`
5. User sees a brief notification when note playback fails
6. Diagnostic `[DIAG]` logs downgraded to `debug` level
7. No regressions in pitch comparison or pitch matching modes

### Story 8.3: SoundFont Loading UX

**As a** user,
**I want** the app to wait for my chosen SoundFont to load before I can start training,
**So that** I always hear the sound I selected instead of an unexpected oscillator fallback.

**Acceptance Criteria:**
1. When the user's `sound_source` setting starts with `"sf2:"`, the Start Page training buttons are disabled until SoundFont assets have finished loading
2. A loading indicator is visible on the Start Page while SoundFont assets are being fetched
3. Once Phase 1 fetch completes successfully, training buttons become enabled and the loading indicator disappears
4. If Phase 1 fetch fails, the app falls back to oscillator, enables training buttons, and shows a brief non-blocking notification
5. When the user's `sound_source` is `"oscillator:sine"` (or any non-SF2 value), training buttons are enabled immediately
6. On Firefox (and all browsers), the app shell and Start Page render immediately — the SF2 fetch does not block page rendering
7. All existing training functionality continues to work — no regressions

### Story 8.4: iOS UI Alignment

**As a** user,
**I want** the web app's layout and navigation to look and feel closer to the native iOS version,
**So that** the experience is consistent across platforms and feels polished on both mobile and desktop.

**Acceptance Criteria:**
1. Navigation bar on all pages follows the iOS pattern: back arrow left, centered title, icon buttons right
2. Pitch Comparison buttons are large blue rounded cards with circle arrow icons, stacked vertically on portrait / side-by-side on landscape
3. Pitch Comparison view shows interval name and tuning label centered between nav and buttons
4. Pitch Matching view header shows stats left, signed cent deviation right, slider fills remaining height
5. All changes maintain keyboard accessibility and screen reader support
6. Dark mode preserved, no regressions

### Story 8.5: Settings Page iOS Alignment

**As a** user,
**I want** the settings page to use grouped card sections with iOS-style controls,
**So that** the settings experience matches the polished look of the rest of the app.

**Acceptance Criteria:**
1. Settings are organized in visually grouped card sections (Pitch Range, Intervals, Sound, Difficulty, Data) with section headers
2. Pitch range (min/max note) uses +/- stepper buttons instead of dropdown selects
3. Interval selection uses a compact grid layout with ascending/descending rows matching iOS
4. Sound settings (instrument, duration, concert pitch, tuning system) are grouped in a single card
5. All changes maintain keyboard accessibility and screen reader support
6. Dark mode preserved, no regressions

### Story 8.6: Pitch Comparison Feedback Icons

**As a** user,
**I want** to see a brief checkmark (correct) or X (incorrect) in a colored circle after each pitch comparison answer,
**So that** the feedback is clean, consistent with the help text, and positioned like the pitch matching screen.

**Acceptance Criteria:**
1. Correct answers show a white checkmark inside a green circle in the top-right area of the header
2. Incorrect answers show a white "X" inside a red circle in the top-right area of the header
3. Feedback position mirrors the pitch matching view layout: stats left, feedback indicator right
4. The help text already says "checkmark (correct) or X (incorrect)" and remains accurate
5. The old thumbs-up/thumbs-down emoji feedback in the center row is removed
6. Feedback timing and duration remain unchanged
7. Dark mode preserved, no regressions
8. Screen reader announcement behavior unchanged

### Story 8.7: Extract Business Logic from Settings View

**As a** developer,
**I want** business logic extracted from settings_view.rs into proper domain and adapter layers,
**So that** the codebase follows clean architecture with views as pure presentation.

**Acceptance Criteria:**
1. `settings_view.rs` contains only presentation logic -- no business logic, no constants that duplicate domain knowledge, no data transformation
2. The `INTERVALS` constant is removed; code derives the list from `Interval::all_chromatic()`
3. Short labels live on the `Interval` type as `short_label()` in the domain crate
4. `encode_one()` and `decode_one()` in `interval_codes.rs` reuse `Interval::short_label()`
5. `persist_intervals()` is moved to `LocalStorageSettings`
6. `ResetStatus` and `ImportExportStatus` enums are moved out of the view
7. `project-context.md` includes a rule that views must not contain business logic
8. All existing functionality works identically after refactoring
9. `cargo test -p domain` passes, `cargo clippy` clean on both crates

**Note:** AC#5 from original story (export/import orchestration extraction) deferred to story 8.8 along with broader architectural cleanup.

### Story 8.8: Clean Up Export/Import Architecture

**As a** developer,
**I want** the export/import code properly structured with domain logic on domain types, adapter code named honestly, and view orchestration extracted,
**So that** naming matches intent, there are no duplicate mappings, and the view is pure presentation.

**Acceptance Criteria:**
1. `domain/src/portability.rs` is deleted -- all its logic lives on domain types or in the web adapter
2. `Interval` gains `csv_code()` (returns "A4" for tritone, iOS CSV compat) and `from_csv_code()` methods; `from_semitones()` is made public
3. `midi_note_name()` free function removed -- callers use `MIDINote::new(n).name()` (identical logic, already exists)
4. Duplicate `NOTE_NAMES` constant in portability.rs removed (already exists in midi.rs)
5. `truncate_timestamp_to_second()` moved to the web adapter (only consumer)
6. Web adapter renamed from `data_portability.rs` to `csv_export_import.rs`
7. `data_portability_service.rs` deleted -- `ResetStatus` and `ImportExportStatus` enums absorbed into the renamed adapter module
8. FileReader-to-future conversion extracted from `settings_view.rs` into the adapter
9. View retains only signal declarations, thin event handlers, and DOM rendering
10. `interval_label()` long-name mapping in `interval_codes.rs` moved to `Interval::display_name()` in the domain crate
11. All existing functionality works identically -- zero behavioral changes
12. `cargo test -p domain` passes, `cargo clippy` clean on both crates

## Epic 9: Mobile Compatibility

Bug fixes for mobile browser compatibility issues discovered during real-device testing.

### Story 9.1: Mobile SoundFont Playback

**As a** user,
**I want** SoundFont playback to work on mobile browsers (iOS Safari, mobile Chromium),
**So that** I hear realistic instrument sounds during training on my phone, not just oscillator tones.

**Acceptance Criteria:**
1. SoundFont playback produces audible sound on iOS Safari (17+) when the user has selected a SoundFont preset as sound source
2. SoundFont playback produces audible sound on mobile Chromium-based browsers (Chrome, Edge, Brave on Android/iOS) when the user has selected a SoundFont preset
3. The SoundFont file loads, presets appear in settings, and the selected preset plays during training -- the full pipeline works end-to-end on mobile
4. Desktop browser playback (Chrome, Firefox, Safari, Edge) continues to work without regression
5. The oscillator fallback still activates when SoundFont loading fails (network error, unsupported browser)
6. Settings sound source preview (story 8.10) works on mobile with SoundFont presets
