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

### Epic 7: iOS Parity — Profile & UI Refresh
Redesigned start screen, profile progress charts with EWMA adaptive bucketing, training stats with trend arrows, sparklines on start page, and help content for all views.

### Epic 8: Fixes & Improvements
Audio playback reliability, SoundFont loading UX, iOS UI alignment across settings and training pages, feedback icons, settings view logic extraction, keyboard interaction fixes, and sound source preview.

### Epic 9: Mobile Compatibility
Fix SoundFont playback on mobile Safari and other mobile browsers where AudioWorklet behavior differs.

### Epic 10: CI/CD Pipeline & GitHub Pages Deployment
Automated quality checks (fmt, clippy, tests) and deployment to GitHub Pages on every push to `main`.

### Epic 11: UI Polish & Sound Level
Visual polish to match iOS app: audio volume alignment, consistent headline layouts with true title centering, modal behavior standardization, and icon consistency across all pages.

### Epic 12: Profile Progress Charts
Users can see their training progress over time, with a chart card for each active training mode showing trend lines, variability bands, session dots, baselines, scrollable history, and a headline with EWMA value and trend indicator.

### Epic 13: Chart Exploration & Help
Tap/click annotations on chart data points with date, mean, stddev, and record count. Help overlay explaining chart elements (trend line, variability band, target baseline, time zones).

### Epic 14: iOS Terminology Alignment
Rename domain types and web crate terminology to match iOS/psychoacoustic standards: `TrainingMode` → `TrainingDiscipline`, `PitchComparison` → `PitchDiscriminationTrial`, settings drop "Training" suffix. Cascade through living documentation.

### Epic 15: Rhythm Discipline Foundation
Extend `TrainingDiscipline` with rhythm cases, add rhythm sections to start screen, placeholder rhythm training screens, and rhythm settings (tempo, gap positions).

### Epic 16: Statistics Generalization
Generalize statistics from cents to f64, introduce `StatisticsKey` for multi-key expansion (tempo range x direction), decouple observers with generic port traits, and generalize IndexedDB store and hydration.

### Epic 17: Rhythm Offset Detection
Rhythm domain types and records, click synthesis with lookahead scheduler, rhythm offset detection session state machine, and full training screen UI with visual metronome.

### Epic 18: Continuous Rhythm Matching
Session state machine and step sequencer for continuous rhythm matching ("Fill the Gap"), plus full training screen UI with looping beat and tap interaction.

### Epic 19: CSV V3 & Data Portability
CSV export/import using V3 format compatible with the iOS app (19 columns, all 4 training types). Rhythm profile visualization on the profile screen.

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

## Epic 10: CI/CD Pipeline & GitHub Pages Deployment

Automated quality checks and deployment to GitHub Pages on every push to `main`. The app is deployed at `https://<username>.github.io/peach-web/`.

### Story 10.1: CI Quality Gate

**As a** developer,
**I want** automated quality checks to run on every push to `main`,
**So that** broken code is never deployed and I get fast feedback on regressions.

**Acceptance Criteria:**
1. A GitHub Actions workflow triggers on push to `main`
2. `cargo fmt --check` runs and fails the pipeline if formatting is wrong
3. `cargo clippy --workspace` runs and fails the pipeline if there are warnings
4. `cargo test -p domain` runs and fails the pipeline if any test fails
5. The workflow uses caching for the Cargo registry and build artifacts to speed up subsequent runs
6. If any check fails, the pipeline stops — no build or deploy step runs

### Story 10.2: Build & Deploy to GitHub Pages

**As a** developer,
**I want** the app to be automatically built and deployed to GitHub Pages after all checks pass,
**So that** the latest version of the app is always live without manual intervention.

**Acceptance Criteria:**
1. This job only runs after Story 10.1's quality gate passes
2. Rust toolchain with `wasm32-unknown-unknown` target and Trunk are installed in CI
3. The SoundFont file is downloaded via `bin/download-sf2.sh` with GitHub Actions caching so it is fetched only once (or when the cache expires)
4. `trunk build --public-url /peach-web/` produces the deployable WASM artifacts
5. The build output is deployed to GitHub Pages using `actions/deploy-pages`
6. The GitHub repository is configured with GitHub Pages source set to GitHub Actions
7. The app is accessible and functional at `https://<username>.github.io/peach-web/`
8. Cargo registry, build artifacts, and SoundFont caches are shared between the quality gate and build jobs where possible

## Epic 11: UI Polish & Sound Level

Visual polish to match iOS app: audio volume alignment, consistent headline layouts with true title centering, modal behavior standardization, and icon consistency across all pages.

### Story 11.1: UI Consistency and Sound Level Fixes

**As a** user,
**I want** the web app to match the iOS app's visual layout and sound level,
**So that** the experience is consistent across platforms.

**Acceptance Criteria:**
1. Audio playback is approximately 12 dB louder than current levels, matching the perceived volume of the iOS app
2. Training mode names on the start page cards match the names used on the training pages and in help sections
3. The help icon on training pages uses a circled question mark character (matching the circled info icon on the start page)
4. Start page header layout matches the iOS reference screenshot: info icon (left) has a visible circle background, chart and settings icons (right) are grouped in a pill-shaped container
5. Section titles ("Single Notes", "Intervals") on the start page are centered
6. Training cards on the start page have a fixed height regardless of whether a sparkline is present
7. Training page header layout matches the iOS reference screenshot: title is left-aligned (after the back button), right side groups Help, Settings, and Chart icons in a pill-shaped container
8. In the info/acknowledgements section, "GeneralUser GS by S. Christian Collins" is a hyperlink to the author's page

### Story 11.2: Headline Layout and Modal Consistency

**As a** user,
**I want** consistent headline layouts and modal behavior across all pages,
**So that** the app feels polished and predictable.

**Acceptance Criteria:**
1. On all pages (start page, training pages, settings, profile, info), the headline layout follows this structure: left pill flush left, title centered in the remaining space, right pill flush right
2. The help modal is closed by a "Done" button (no decoration) rendered inside a pill in the top left corner of the modal, replacing the current top-right "Done" button
3. The info view is closed by a "Done" button (no decoration) rendered inside a pill in the top left corner, replacing the current back-arrow pill
4. Help modals display without a grey backdrop overlay — matching the info view's no-overlay appearance

## Epic 12: Profile Progress Charts

Users can see their training progress over time, with a chart card for each active training mode showing trend lines, variability bands, session dots, baselines, scrollable history, and a headline with EWMA value and trend indicator. Replaces the earlier piano keyboard visualization (Epic 7 stories 7.2/7.4) with a design proven across three iOS iterations.

**Source reference:** `docs/ios-reference/profile-screen-specification.md` (authoritative)

**Profile Screen FRs covered:** PFR1-PFR14, PFR16
**Profile Screen NFRs covered:** PNFR1-PNFR5

### Profile Screen Requirements

**Functional Requirements:**

- PFR1: Display a scrollable profile screen with progress chart cards for each training mode that has at least one record, ordered by TrainingMode enum order
- PFR2: Show a headline row per card with mode display name, current EWMA value (1 decimal place, locale-aware formatting), per-bucket stddev of the most recent bucket (±format), and a colored trend arrow (improving/stable/declining)
- PFR3: Extract metric points from raw training records per mode — absolute cent offset for comparison modes, absolute user cent error for matching modes — as (timestamp, value) pairs
- PFR4: Bucket metric points into three granularity zones with boundaries snapped to calendar-day boundaries (local time): session zone starts at midnight today, daily zone spans the 7 calendar days before today, monthly zone covers everything older. Sessions within the session zone are defined by a 30-minute gap rule. Monthly buckets truncate at the day zone boundary to prevent overlap.
- PFR5: Compute per-bucket mean and population standard deviation (not sample stddev); single-record buckets have stddev = 0
- PFR6: Compute EWMA over session-level buckets (separate from display buckets) using exponential decay with a 7-day half-life (604800 seconds)
- PFR7: Compute trend direction requiring at least 2 records: improving (latest value < EWMA), stable (latest value >= EWMA AND <= running mean + running stddev), declining (latest value > running mean + running stddev) — using running population statistics across all individual records
- PFR8: Render chart with index-based X-axis (equal visual width per bucket), Y domain from 0 to max(1, max bucket mean + stddev)
- PFR9: Enable horizontal scrolling when more than 8 buckets exist, with 8 buckets visible at a time, initial scroll position showing the most recent data at the right edge
- PFR10: Render zone background tints and vertical divider lines at zone transitions and year boundaries (only when more than one zone exists); suppress year boundaries within 1 index of a zone transition
- PFR11: Render a standard deviation band connecting non-session buckets, with a weighted-average session bridge point at the zone boundary (firstSessionIndex - 0.5); no band through session zone
- PFR12: Render a mean trend line through non-session bucket means plus the session bridge point; session buckets excluded from the line
- PFR13: Render disconnected session dots (circle symbol, size 20) for each session bucket
- PFR14: Render a horizontal dashed baseline at each mode's optimal baseline value (green, dashed pattern [5,3])
- PFR15: Show an annotation popover on tap/click with bucket date (formatted per zone), mean value, stddev, and record count; tapping again or scrolling dismisses the annotation
- PFR16: Display X-axis labels: short month names for monthly zone, short weekday names for daily zone, "Today" for first session bucket (empty for subsequent); strip trailing dots from abbreviated names in languages that add them. Year labels below the monthly zone, centered per calendar year span.
- PFR17: Provide a help overlay (triggered by ? toolbar button) with five sections: Your Progress Chart, Trend Line, Variability Band, Target Baseline, Time Zones

**Non-Functional Requirements:**

- PNFR1: Card-level accessibility: each card is an accessibility container with label "Progress chart for {mode display name}" and value "Current: {EWMA} {unit}, trend: {trend label}". Screen-level: scroll view aria-label summarizing which modes have data. Zone-level VoiceOver summaries deferred to follow-up story.
- PNFR2: All fonts use semantic text styles with relative units (rem/em) — no hardcoded pixel sizes for text
- PNFR3: Increased contrast mode: detect via @media (prefers-contrast: more) and double opacity values for stddev band (0.15→0.30), baseline (0.60→0.90), zone backgrounds (0.06→0.12), selection indicator (0.50→0.80), zone dividers switch from secondary to primary color
- PNFR4: Locale-aware number formatting with 1 decimal place (minimumFractionDigits: 1, maximumFractionDigits: 1) — e.g. "25.3" in English, "25,3" in German
- PNFR5: Responsive chart height: 180px on compact/mobile, 240px on regular/tablet/desktop

**Additional Requirements:**

- Two distinct bucketing pipelines: (1) multi-granularity display buckets with calendar-day-snapped zone boundaries for chart rendering, and (2) session-level buckets for EWMA computation. These are independent — the display pipeline must not reuse the EWMA pipeline's bucketing logic.
- Chart data pipeline (bucketing, aggregation, EWMA, trend) lives in the domain crate (pure Rust, no browser deps)
- Chart rendering lives in the web crate
- Card background: frosted glass via CSS backdrop-filter: blur() with semi-transparent backgrounds, 12px corner radius
- Charting approach: Canvas/SVG or JS charting library (to be decided during implementation)
- Per-mode chart parameters: Unison Comparison (baseline 8¢, half-life 7d, gap 30min), Interval Comparison (baseline 12¢, half-life 7d, gap 30min), Unison Matching (baseline 5¢, half-life 7d, gap 30min), Interval Matching (baseline 8¢, half-life 7d, gap 30min)
- Empty state: modes with zero records show no card; if no mode has data, empty scroll view with nav bar and help button still visible

**Profile Screen FR Coverage Map:**

- PFR1: Epic 12 — Profile screen with chart cards per mode
- PFR2: Epic 12 — Headline row with EWMA, stddev, trend arrow
- PFR3: Epic 12 — Metric extraction from training records
- PFR4: Epic 12 — Multi-granularity bucketing with calendar-day boundaries
- PFR5: Epic 12 — Per-bucket mean and population stddev
- PFR6: Epic 12 — EWMA computation over session-level buckets
- PFR7: Epic 12 — Trend direction computation
- PFR8: Epic 12 — Index-based X-axis chart rendering
- PFR9: Epic 12 — Horizontal scrolling for >8 buckets
- PFR10: Epic 12 — Zone backgrounds and divider lines
- PFR11: Epic 12 — StdDev band with session bridge
- PFR12: Epic 12 — Mean trend line
- PFR13: Epic 12 — Session dots
- PFR14: Epic 12 — Baseline dashed line
- PFR15: Epic 13 — Annotation popover on tap/click
- PFR16: Epic 12 — X-axis labels and year labels
- PFR17: Epic 13 — Help overlay with five sections

### Story 12.1: Progress Data Pipeline

**As a** developer,
**I want** the domain crate to extract metric points from training records, bucket them into multi-granularity display zones, compute EWMA over session-level buckets, and determine trend direction,
**So that** the chart rendering layer receives fully computed, ready-to-render data structures.

**Acceptance Criteria:**

**Given** a list of PitchComparisonRecord entries with interval == 0
**When** metric extraction runs for the Unison Comparison mode
**Then** each record produces a metric point of (timestamp, abs(centOffset))
**And** records with interval != 0 are excluded from this mode

**Given** a list of PitchComparisonRecord entries with interval != 0
**When** metric extraction runs for the Interval Comparison mode
**Then** each record produces a metric point of (timestamp, abs(centOffset))

**Given** a list of PitchMatchingRecord entries with interval == 0
**When** metric extraction runs for the Unison Matching mode
**Then** each record produces a metric point of (timestamp, abs(userCentError))

**Given** a list of PitchMatchingRecord entries with interval != 0
**When** metric extraction runs for the Interval Matching mode
**Then** each record produces a metric point of (timestamp, abs(userCentError))

**Given** metric points spanning months, recent days, and today
**When** multi-granularity display bucketing runs with current time T
**Then** points with timestamp >= startOfDay(T) are bucketed into session zone, grouped by 30-minute session gap
**And** points with timestamp >= startOfDay(T) - 7 days AND < startOfDay(T) are bucketed by calendar day
**And** points with timestamp < startOfDay(T) - 7 days are bucketed by calendar month
**And** the last monthly bucket's end date is truncated to the day zone boundary
**And** all buckets are concatenated chronologically: months, days, sessions

**Given** a bucket with multiple metric values
**When** aggregation runs
**Then** mean = sum(values) / count
**And** stddev = sqrt(sum((value - mean)^2) / count) (population stddev)

**Given** a bucket with exactly one metric value
**When** aggregation runs
**Then** mean = that value and stddev = 0

**Given** session-level buckets (the EWMA pipeline, separate from display buckets)
**When** EWMA computation runs
**Then** ewma[0] = bucket[0].mean
**And** for subsequent buckets: alpha = 1.0 - exp(-ln(2) * dt / 604800), ewma[i] = alpha * bucket[i].mean + (1 - alpha) * ewma[i-1]
**And** the current EWMA is ewma[last]

**Given** at least 2 individual records and a computed current EWMA
**When** trend direction is computed
**Then** running mean and running population stddev are calculated across all individual metric values (not buckets)
**And** if latestValue > runningMean + runningStddev then trend is Declining
**And** if latestValue >= currentEWMA then trend is Stable
**And** otherwise trend is Improving

**Given** fewer than 2 records for a mode
**When** trend is queried
**Then** trend returns None

**Given** the TrainingMode enum
**When** chart parameters are queried
**Then** Unison Comparison returns baseline=8, halfLife=604800, sessionGap=1800
**And** Interval Comparison returns baseline=12, halfLife=604800, sessionGap=1800
**And** Unison Matching returns baseline=5, halfLife=604800, sessionGap=1800
**And** Interval Matching returns baseline=8, halfLife=604800, sessionGap=1800

**Given** all pipeline code
**When** I run `cargo test -p domain`
**Then** all tests pass with zero browser dependencies

### Story 12.2: Profile Screen Layout & Chart Cards

**As a** musician,
**I want** to see a profile screen with a card for each training mode I've used, showing my current EWMA and trend at a glance,
**So that** I can quickly see how I'm doing in each mode.

**Acceptance Criteria:**

**Given** I navigate to `/profile`
**When** training data exists for at least one mode
**Then** I see a scrollable list of chart cards, one per mode with data
**And** cards appear in TrainingMode enum order: unison comparison, interval comparison, unison matching, interval matching
**And** modes with zero records show no card
**And** 16px gap between cards

**Given** the profile screen
**When** no mode has any training data
**Then** I see the navigation bar with "Profile" title and help button
**And** the scroll area is empty (no cards)

**Given** a chart card for a mode
**When** it renders
**Then** the card has a frosted glass background (backdrop-filter: blur() with semi-transparent background) and 12px corner radius
**And** internal padding follows the existing app's system default pattern

**Given** a chart card headline row
**When** it renders with sufficient data
**Then** the left side shows the mode display name in headline font
**And** the right side shows the EWMA value in title 2 bold (e.g. "25.3")
**And** next to the EWMA, the stddev of the most recent display bucket in caption secondary color (e.g. "±4.2")
**And** a trend arrow icon colored by direction: green down-right for improving, gray right for stable, orange up-right for declining

**Given** a mode with fewer than 2 records
**When** the headline renders
**Then** the EWMA value is shown (computed from the single record)
**And** no trend arrow is displayed

**Given** the EWMA value "25.3" in an English locale
**When** formatted for display
**Then** it shows "25.3" with a decimal point

**Given** the EWMA value "25.3" in a German locale
**When** formatted for display
**Then** it shows "25,3" with a comma

**Given** a chart card
**When** the chart area renders (before Story 12.3)
**Then** a placeholder area is shown at the correct height: 180px on mobile, 240px on tablet/desktop

**Given** the profile screen scroll view
**When** inspected for accessibility
**Then** it has an aria-label summarizing which modes have data (e.g. "Profile showing progress for: Hear & Compare – Single Notes, Tune & Match – Single Notes")

**Given** a chart card
**When** inspected for accessibility
**Then** it has role and aria-label "Progress chart for {mode display name}"
**And** aria-valuenow or equivalent conveys "Current: {EWMA} cents, trend: {trend label}"

**Given** the profile screen on any viewport
**When** rendered
**Then** all text uses semantic sizes (rem/em) with no hardcoded pixel font sizes

### Story 12.3: Chart Rendering

**As a** musician,
**I want** to see my progress visualized as a chart with trend lines, variability bands, session dots, and a target baseline,
**So that** I can understand how my pitch perception is developing over time.

**Acceptance Criteria:**

**Given** computed display buckets for a mode
**When** the chart renders
**Then** the X-axis is index-based: each bucket gets equal visual width regardless of time span
**And** X domain is -0.5 to bucketCount - 0.5
**And** Y domain is 0 to max(1, max(bucket.mean + bucket.stddev))

**Given** buckets spanning multiple granularity zones
**When** zone backgrounds render (Layer 1)
**Then** colored rectangles span each zone from startIndex - 0.5 to endIndex + 0.5
**And** monthly zone uses system background color at 6% opacity
**And** daily zone uses secondary system background at 6% opacity
**And** session zone uses system background color at 6% opacity

**Given** all data falls in a single zone
**When** the chart renders
**Then** no zone backgrounds or divider lines are drawn

**Given** zone transitions exist
**When** dividers render (Layer 2)
**Then** vertical lines appear at each index where granularity changes, drawn at index - 0.5
**And** year boundary dividers appear within the monthly zone where calendar year changes
**And** year boundaries within 1 index of a zone transition are suppressed
**And** lines are solid, 1px, secondary color

**Given** non-session buckets exist
**When** the stddev band renders (Layer 3)
**Then** a shaded area spans from max(0, mean - stddev) to mean + stddev for each non-session bucket
**And** the band connects via a continuous line through non-session buckets only
**And** color is blue at 15% opacity

**Given** both non-session and session buckets exist
**When** the session bridge renders
**Then** a bridge point is computed at X = firstSessionIndex - 0.5
**And** bridgeMean = weighted average of session bucket means by record count
**And** bridgeStddev = sqrt(weighted average of session bucket variances by record count)
**And** the band and line extend to this bridge point

**Given** only session buckets exist (all data is from today)
**When** the chart renders
**Then** no line or band is drawn — only disconnected session dots appear

**Given** non-session buckets
**When** the mean trend line renders (Layer 4)
**Then** a blue line connects the mean values of non-session buckets plus the bridge point
**And** session buckets are not connected by the line

**Given** session buckets
**When** session dots render (Layer 5)
**Then** each session bucket is shown as a disconnected blue circle (point mark, size 20 area units)

**Given** a mode's optimal baseline value
**When** the baseline renders (Layer 6)
**Then** a horizontal dashed line appears at that Y value
**And** dash pattern is [5, 3] (5px dash, 3px gap), 1px width
**And** color is green at 60% opacity

**Given** X-axis labels
**When** they render
**Then** monthly zone buckets show short month name (e.g. "Jan", "Feb")
**And** daily zone buckets show short weekday name (e.g. "Mon", "Tue")
**And** the first session bucket shows "Today"; subsequent session buckets show nothing
**And** trailing dots are stripped from abbreviated names (e.g. "Dez." → "Dez" in German)

**Given** monthly zone spans multiple calendar years
**When** year labels render
**Then** year text (e.g. "2025", "2026") is centered below the monthly zone between the first and last bucket index of each year span
**And** font is caption2 size, secondary color
**And** extra bottom padding (16px baseline, scales with text size) is added to accommodate year labels

**Given** no monthly zone exists
**When** the chart renders
**Then** no year labels appear and no extra bottom padding is added

**Given** @media (prefers-contrast: more) is active
**When** the chart renders
**Then** stddev band opacity is 0.30 (instead of 0.15)
**And** baseline opacity is 0.90 (instead of 0.60)
**And** zone background opacity is 0.12 (instead of 0.06)
**And** zone dividers use primary color (instead of secondary)

**Given** the chart area
**When** rendered on mobile
**Then** chart height is 180px

**Given** the chart area
**When** rendered on tablet or desktop
**Then** chart height is 240px

### Story 12.4: Chart Scrolling

**As a** musician,
**I want** to scroll through my chart history when I have more than 8 data points,
**So that** I can review my full training timeline while keeping the chart readable.

**Acceptance Criteria:**

**Given** a mode with 8 or fewer display buckets
**When** the chart renders
**Then** all buckets are visible in a static layout with no scrolling

**Given** a mode with more than 8 display buckets
**When** the chart renders
**Then** the chart is horizontally scrollable
**And** 8 buckets are visible at a time
**And** the initial scroll position shows the rightmost (most recent) data at the right edge
**And** scroll position = max(0, bucketCount - 8)

**Given** a scrollable chart
**When** I scroll horizontally
**Then** the chart pans smoothly to reveal earlier or later buckets
**And** any active selection annotation is dismissed

**Given** a scrollable chart with @media (prefers-reduced-motion: reduce) active
**When** the initial scroll position is set
**Then** it is applied without animation

---

## Epic 13: Chart Exploration & Help

Users can tap chart data points to see detailed annotations (date, mean, stddev, record count) and access a help overlay explaining what each chart element means. Standalone — the charts from Epic 12 are fully functional without this.

**Source reference:** `docs/ios-reference/profile-screen-specification.md` (authoritative)

**Profile Screen FRs covered:** PFR15, PFR17

### Story 13.1: Chart Tap Annotation

**As a** musician,
**I want** to tap a data point on the chart and see its details in a popover,
**So that** I can explore specific time periods and see exactly how I performed.

**Acceptance Criteria:**

**Given** a rendered chart
**When** I tap/click on the chart area
**Then** the tap resolves to the nearest bucket index by rounding the X coordinate
**And** a vertical dashed selection line appears at that bucket's X position
**And** an annotation popover appears at the top of the chart

**Given** the selection line
**When** rendered
**Then** it is dashed with pattern [5, 3], 1px width
**And** gray at 50% opacity (80% in increased contrast mode)

**Given** the annotation popover for a monthly zone bucket
**When** displayed
**Then** it shows the date as "MMM yyyy" (e.g. "Jan 2026")
**And** the bucket mean value in caption bold (e.g. "25.3")
**And** the bucket stddev in caption2 secondary (e.g. "±4.2")
**And** the record count in caption2 secondary (e.g. "47 records")

**Given** the annotation popover for a daily zone bucket
**When** displayed
**Then** it shows the date as "E MMM d" (e.g. "Mon, Mar 5")

**Given** the annotation popover for a session zone bucket
**When** displayed
**Then** it shows the time as "HH:mm" (e.g. "14:30")

**Given** the popover
**When** rendered
**Then** it has a frosted glass background with 6px corner radius
**And** 6px padding on all sides, 2px VStack spacing
**And** overflow resolution keeps the popover within chart bounds on both axes

**Given** a bucket is selected
**When** I tap the same area again
**Then** the annotation and selection line are dismissed

**Given** a bucket is selected on a scrollable chart
**When** I scroll the chart
**Then** the annotation and selection line are dismissed

**Given** number values in the popover
**When** formatted
**Then** they use locale-aware formatting with 1 decimal place (matching the headline row)

### Story 13.2: Profile Help Overlay

**As a** musician,
**I want** a help overlay on the profile screen that explains what each chart element means,
**So that** I can learn to read my progress charts.

**Acceptance Criteria:**

**Given** the profile screen navigation bar
**When** rendered
**Then** a help button (? icon) is visible in the toolbar trailing position

**Given** I tap the help button
**When** the help overlay opens
**Then** it uses the same overlay mechanism as all other screens in the app

**Given** the help overlay content
**When** displayed
**Then** it shows five sections in this order:
**And** "Your Progress Chart" — "This chart shows how your pitch perception is developing over time"
**And** "Trend Line" — "The blue line shows your smoothed average — it filters out random ups and downs to reveal your real progress"
**And** "Variability Band" — "The shaded area around the line shows how consistent you are — a narrower band means more reliable results"
**And** "Target Baseline" — "The green dashed line is your goal — as the trend line approaches it, your ear is getting sharper"
**And** "Time Zones" — "The chart groups your data by time: months on the left, recent days in the middle, and today's sessions on the right"

**Given** the help overlay
**When** I dismiss it (Done button or equivalent)
**Then** I return to the profile screen with chart state preserved

---

## Epic 14: iOS Terminology Alignment

Rename domain types and web crate terminology to match iOS/psychoacoustic standards. The iOS sibling app renamed types for music pedagogy and psychoacoustic accuracy: `TrainingMode` → `TrainingDiscipline`, `PitchComparison` → `PitchDiscriminationTrial`, `PitchMatchingChallenge` → `PitchMatchingTrial`, settings drop "Training" suffix. This epic cascades those changes through the web codebase and living documentation.

**Source reference:** `docs/ios-reference/ios-changes-since-f70e3f.md`, section 1

### Story 14.1: Domain Crate Terminology Rename

**As a** developer,
**I want** all domain crate types renamed to match the iOS terminology alignment,
**So that** both apps use identical domain language and cross-platform CSV compatibility is maintained.

Key renames: `TrainingMode` → `TrainingDiscipline`, `PitchComparison` → `PitchDiscriminationTrial`, `CompletedPitchComparison` → `CompletedPitchDiscriminationTrial`, `PitchMatchingChallenge` → `PitchMatchingTrial`, `CompletedPitchMatching` → `CompletedPitchMatchingTrial`, `PitchComparisonSession` → `PitchDiscriminationSession`, observer and port trait renames, file renames, slug updates.

### Story 14.2: Web Crate Terminology Rename

**As a** developer,
**I want** the web crate updated to match the domain terminology from story 14.1,
**So that** components, routes, bridge adapters, and UI labels use consistent iOS-aligned naming.

Key changes: component file/function renames (`PitchComparisonView` → `PitchDiscriminationView`), route path update (`/training/comparison` → `/training/pitch-discrimination`), bridge adapter renames, localization key updates (`comparison-*` → `discrimination-*`), button labels simplified ("Hear & Compare" → "Compare", "Tune & Match" → "Match").

### Story 14.3: Update Living Documentation

**As a** developer,
**I want** living documentation updated to reflect the new terminology,
**So that** planning and architecture docs remain accurate and useful for AI agents and contributors.

Updates architecture.md, prd.md, ux-design-specification.md, arc42-architecture.md, project-context.md, epics.md, and sprint-status.yaml. Historical docs (completed story files, domain blueprint) are NOT updated.

---

## Epic 15: Rhythm Discipline Foundation

Extend `TrainingDiscipline` with two rhythm cases (Rhythm Offset Detection, Continuous Rhythm Matching), add a three-section layout to the start screen, placeholder rhythm training screens, and rhythm settings (tempo BPM, gap step positions).

**Prerequisite:** Epic 14 (terminology rename complete)

### Story 15.1: Rhythm Discipline Enum Cases and Domain Types

**As a** developer,
**I want** the `TrainingDiscipline` enum extended with rhythm cases and basic rhythm domain types added,
**So that** the start screen and routing infrastructure can reference rhythm disciplines.

Adds `RhythmOffsetDetection` and `ContinuousRhythmMatching` variants to `TrainingDiscipline`, plus `TempoBPM` and `StepPosition` domain types.

### Story 15.2: Start Screen Three-Section Layout with Rhythm Routes

**As a** user,
**I want** to see rhythm training options on the start screen alongside pitch training,
**So that** I can navigate to rhythm training as soon as it becomes available.

Three sections of 2 buttons each matching the iOS layout, with routes for all 6 training disciplines.

### Story 15.3: Placeholder Rhythm Training Screens

**As a** user,
**I want** to see informative placeholder screens when navigating to rhythm training,
**So that** I know the features are coming and can test the navigation flow.

Two placeholder screens with NavBar, title, and description — later replaced by full training UIs.

### Story 15.4: Rhythm Settings

**As a** user,
**I want** to configure rhythm training parameters in the settings screen,
**So that** I can adjust tempo and gap positions before rhythm training is fully implemented.

Adds a "Rhythm" section to settings with tempo BPM slider and gap step position selection.

---

## Epic 16: Statistics Generalization

Refactor the statistics and storage architecture so that adding new training disciplines (rhythm) doesn't require new observer traits, store methods, or hydration loops. Generalizes statistics from cents to f64, introduces `StatisticsKey` for multi-key expansion, decouples observers with generic port traits, and generalizes the IndexedDB store.

**Prerequisite:** Epic 15 (rhythm types exist)

### Story 16.1: Generalize TrainingDisciplineStatistics from Cents to f64

**As a** developer,
**I want** statistics tracking to work with any f64 metric value (not just Cents),
**So that** rhythm disciplines can use percentage-of-sixteenth-note as their metric unit.

Replaces `Cents` with `f64` in the statistics engine. `WelfordAccumulator` is already generic; needs `MetricPoint<f64>` instead of `MetricPoint<Cents>`.

### Story 16.2: StatisticsKey and Profile Redesign

**As a** developer,
**I want** the perceptual profile to use typed `StatisticsKey`s instead of `TrainingDiscipline` as map keys,
**So that** rhythm disciplines can expand to multiple keys (per tempo range x direction) while pitch disciplines keep 1:1 key mapping.

### Story 16.3: Decouple Observers with Port Traits

**As a** developer,
**I want** discipline-specific observer traits replaced with generic port traits,
**So that** adding a new discipline doesn't require new observer traits, bridge classes, or profile/store changes.

Two generic port traits that all disciplines use, replacing per-discipline observers.

### Story 16.4: Generalize IndexedDB Store and Hydration

**As a** developer,
**I want** the IndexedDB store to use a generic record interface and the hydration pipeline to be discipline-driven,
**So that** adding a rhythm discipline doesn't require new store methods or hydration loops.

---

## Epic 17: Rhythm Offset Detection

Full implementation of the rhythm offset detection training mode: domain types and records, click synthesis with a Web Audio lookahead scheduler, session state machine, and training screen UI with visual metronome.

**Prerequisite:** Epic 16 (architecture refactoring complete)

### Story 17.1: Rhythm Domain Types, Records, and Observer Ports

**As a** developer,
**I want** rhythm-specific domain types, record structs, and trial types defined,
**So that** the rhythm offset detection session can be implemented.

Adds rhythm-specific domain types mirroring the iOS domain model.

### Story 17.2: Click Synthesis and Rhythm Scheduler

**As a** developer,
**I want** a Web Audio lookahead scheduler that plays percussion clicks with sample-accurate timing,
**So that** rhythm training sessions can present precisely timed audio patterns.

Uses the "two clocks" pattern: main-thread timer looks ahead ~100ms and schedules `AudioBufferSourceNode.start(when)` calls on the audio clock.

### Story 17.3: Rhythm Offset Detection Session State Machine

**As a** developer,
**I want** a session state machine for rhythm offset detection training,
**So that** the exercise flow (play pattern → await answer → show feedback → repeat) is managed cleanly.

### Story 17.4: Rhythm Offset Detection Screen UI

**As a** user,
**I want** to practice rhythm offset detection with a visual metronome, Early/Late buttons, and feedback display,
**So that** I can train my ability to detect timing deviations.

Replaces the placeholder screen from story 15.3 with the full training UI.

---

## Epic 18: Continuous Rhythm Matching

Full implementation of the continuous rhythm matching ("Fill the Gap") training mode: session state machine with step sequencer, and training screen UI with looping beat and tap interaction.

**Prerequisite:** Story 17.2 (rhythm scheduler with loop mode)

### Story 18.1: Continuous Rhythm Matching Session and Step Sequencer

**As a** developer,
**I want** a session state machine and step sequencer for continuous rhythm matching,
**So that** the exercise loop (play pattern with gap → user taps to fill → aggregate → repeat) is managed cleanly.

### Story 18.2: Continuous Rhythm Matching Screen UI

**As a** user,
**I want** to practice continuous rhythm matching with a looping beat, a tap button, and timing feedback,
**So that** I can train my ability to maintain steady rhythm and fill gaps accurately.

Replaces the placeholder screen from story 15.3 with the full "Fill the Gap" training UI.

---

## Epic 19: CSV V3 & Data Portability

CSV export/import using the V3 format compatible with the iOS app (19 columns covering all 4 training types), plus rhythm profile visualization on the profile screen.

**Prerequisite:** Epics 17-18 (all rhythm record types exist)

### Story 19.1: CSV V3 Format Alignment

**As a** user,
**I want** CSV export/import to use the V3 format compatible with the iOS app,
**So that** I can transfer all training data (pitch and rhythm) between platforms.

V3 format: 19 columns covering all 4 training types. Replaces V1 (12 columns, pitch only).

### Story 19.2: Rhythm Profile Visualization

**As a** user,
**I want** to see my rhythm training progress on the profile screen,
**So that** I can track improvement across tempos and directions.

Adds rhythm chart cards to the profile screen with discipline-aware unit labels.

## Epic 21: Rhythm Tap Latency & Audio Doubling

Improve the "Fill the Gap" training experience by reducing tap-to-sound latency and eliminating audio doubling when user taps coincide with scheduled beats. Based on technical research in `docs/planning-artifacts/research/domain-web-audio-tap-latency-research-2026-03-25.md`.

**Prerequisite:** Epic 18 (continuous rhythm matching exists)

**Scope constraint:** Bluetooth keyboards/trackpads are out of scope — their 40-200ms input latency makes rhythm training unviable. Optimizations target mobile touchscreens and wired/built-in input devices.

### Story 21.1: AudioContext latencyHint and Tap Click Overlap Suppression

**As a** user,
**I want** the audio engine to use minimum-latency settings and not double-play clicks when my tap coincides with a scheduled beat,
**So that** rhythm training feels responsive and sounds clean.

Two quick wins bundled in one story:
1. Set `latencyHint: 0` (float) on `AudioContext` construction via `AudioContextOptions` — reduces audio pipeline latency by 5-20ms.
2. Suppress the tap click when it falls within ±15ms of a non-gap scheduled beat — eliminates the "jittery doubling" artifact at slow tempos.

Requires sharing `beat_times` and `gap_index` with the tap handler (via `Rc<Cell<>>`, following existing `shared_click_buffer` pattern).

### Story 21.2: Tap Timestamp Bridging via getOutputTimestamp

**As a** developer,
**I want** the tap handler to convert `PointerEvent.timeStamp` to audio clock time using `getOutputTimestamp()`,
**So that** tap offset measurement is accurate regardless of main-thread delivery delay.

Currently `on_tap` reads `ctx.current_time()` at handler execution time, which is 10-30ms after the physical touch. Using `getOutputTimestamp()` bridges the DOM performance clock to the audio clock, eliminating systematic late bias.

Requires manual `#[wasm_bindgen]` FFI for `getOutputTimestamp()` (not in `web_sys`). Falls back to `current_time()` when unsupported (Safari). The `on_tap` closure must be updated to accept the event timestamp.

### Story 21.3: Output Latency Compensation in Tap Evaluation

**As a** user,
**I want** my tap accuracy to be evaluated relative to when I *heard* the beat, not when it was scheduled,
**So that** my timing scores are accurate even when using external speakers or headphones.

Adds `output_latency_secs: f64` parameter to `evaluate_tap()` in the domain crate. The web layer reads `AudioContext.outputLatency` (manual FFI, not in `web_sys`) and passes it in. Compensates for the delay between audio scheduling and physical sound emission — critical for users with USB audio interfaces or wired headphones with non-trivial DAC latency.

Falls back to 0.0 when `outputLatency` is unsupported (Safari) or returns NaN. Domain crate remains pure (no `web_sys` dependency).

### Story 21.4: Immediate Tap Click Playback with start(0)

**As a** user,
**I want** the tap click to play at the earliest possible moment,
**So that** I get the most immediate audible feedback when I tap.

Splits `play_click_at` into two variants: `play_click_immediate` (calls `source.start()` with no argument for reactive tap clicks) and `schedule_click_at` (existing `start_with_when(when)` for lookahead-scheduled beats). Avoids edge case where `currentTime` advances between the read and the `start()` call, deferring playback to the next render quantum.

---

## Epic 20: Training View Refactoring

Extract shared infrastructure from the four training view components to eliminate ~600 lines of duplicated code. Modeled after the iOS app's `SessionLifecycle` + observer/adapter architecture (see `peach-ios/docs/arc42.md` sections 5-8).

**Motivation:** Adversarial codebase review (2025-03-25) identified ARV-001/ARV-002 as highest-impact findings — each training view is a 700-854 line monolith that copy-pastes AudioContext state handlers, visibility-change handlers, error auto-dismiss Effects, help-modal coordination, and cancellation/termination flag logic. A bug fix in one view must be manually replicated in three others.

**Prerequisite:** Epics 17-18 (all four training views exist)

### Story 20.1: Extract Shared Training Session Lifecycle

**As a** developer,
**I want** shared training lifecycle infrastructure extracted from the four training views,
**So that** cross-cutting behavior (audio interruption, visibility changes, error handling, cleanup) is defined once and reused.

Extracts a `training_common` module with: `SessionLifecycle` (cancellation/termination flags, AudioContext state handler, visibility-change handler, navigation cleanup), error auto-dismiss Effect factory, and help-modal pause coordination. Each training view delegates to shared utilities instead of reimplementing them.

### Story 20.2: Harden Numerical Guards in Statistics Pipeline

**As a** user,
**I want** the statistics engine to gracefully reject invalid float values at runtime,
**So that** a single poisoned measurement (NaN, infinity, negative) cannot silently corrupt my training profile.

Promotes `debug_assert!` in `WelfordAccumulator::update()` to a silent early-return guard; changes `MetricPoint::new()` to return `Option<Self>`. Updates all callers.

---

## Epic 22: MIDI Input for Rhythm Training

Users with a connected MIDI controller can tap rhythm training beats using any MIDI note-on event, automatically and alongside existing pointer/keyboard input. Progressive enhancement — the app works identically when MIDI is unavailable. Based on technical research in `docs/planning-artifacts/research/technical-web-midi-input-research-2026-03-26.md`.

**Prerequisites:** Epic 18 (continuous rhythm matching exists), Epic 21 (tap latency pipeline with `bridge_event_to_audio_time`)

### Story 22.1: MIDI Adapter Module with Note-On Detection

**As a** developer,
**I want** a `midi_input.rs` adapter that handles Web MIDI API access, feature detection, and note-on event listening,
**So that** MIDI input is encapsulated in a single module following the existing adapter pattern.

**Acceptance Criteria:**

**Given** the `web/Cargo.toml` dependency on `web-sys`
**When** the MIDI feature flags are added
**Then** `MidiAccess`, `MidiInput`, `MidiInputMap`, `MidiMessageEvent`, `MidiOptions`, `MidiPort`, `MidiConnectionEvent`, and `Navigator` features are enabled

**Given** a browser that supports Web MIDI API
**When** `is_midi_available()` is called
**Then** it returns `true`

**Given** a browser that does not support Web MIDI API (e.g., Safari)
**When** `is_midi_available()` is called
**Then** it returns `false`

**Given** a 3-byte MIDI message with status byte `0x90`–`0x9F` and velocity > 0
**When** `is_note_on(data)` is called
**Then** it returns `true`

**Given** a MIDI message with velocity 0, a note-off status (`0x80`–`0x8F`), a control change, or fewer than 3 bytes
**When** `is_note_on(data)` is called
**Then** it returns `false`

**Given** MIDI access is granted and one or more MIDI inputs are connected
**When** `setup_midi_listeners(on_note_on)` is called
**Then** a `midimessage` event listener is attached to each input that calls `on_note_on(timestamp_ms)` for note-on events

**Given** a `MidiCleanupHandle` returned from `setup_midi_listeners`
**When** `cleanup()` is called
**Then** all `midimessage` listeners are removed from their respective `MidiInput` targets

**Given** the `is_note_on` function
**When** `cargo test -p web` is run
**Then** unit tests pass covering: note-on channel 1, note-on channel 16, velocity-zero note-off, explicit note-off, control change ignored, truncated message ignored

**Given** the new module
**When** `cargo clippy --workspace` is run
**Then** no warnings are produced

### Story 22.2: Wire MIDI Input into Continuous Rhythm Matching View

**As a** user with a MIDI controller,
**I want** my MIDI note-on events to trigger tap evaluation in rhythm training,
**So that** I can practice rhythm with a pad, keyboard, or any MIDI device instead of (or alongside) tapping the screen.

**Acceptance Criteria:**

**Given** the training view is mounting and the browser supports Web MIDI
**When** AudioContext is resumed (user gesture)
**Then** `setup_midi_listeners` is called with a clone of the existing `on_tap` closure

**Given** MIDI setup succeeds
**When** a MIDI note-on event fires during training
**Then** the tap evaluation pipeline processes it identically to a pointer or keyboard tap (same `bridge_event_to_audio_time` → `evaluate_tap` → `RhythmOffset` path)

**Given** MIDI setup fails (permission denied, API error)
**When** the failure occurs
**Then** a warning is logged via `log::warn!` and training continues with pointer/keyboard input only — no error dialog or UI disruption

**Given** MIDI is not available (unsupported browser)
**When** the training view mounts
**Then** MIDI setup is skipped entirely and training works as before

**Given** the training view is unmounting (navigation away)
**When** `on_cleanup` runs
**Then** the `MidiCleanupHandle` is cleaned up, removing all MIDI event listeners

**Given** the complete implementation
**When** `cargo test -p domain` is run
**Then** all existing domain tests pass unchanged (domain crate is not modified)

**Given** the complete implementation
**When** `cargo clippy --workspace` is run
**Then** no warnings are produced

### Story 22.3: MIDI Pitch Bend for Pitch Matching

**As a** user with a MIDI controller,
**I want** my pitch bend wheel to control the pitch matching slider,
**So that** I can practice pitch matching with a physical controller for a more tactile experience.

**Acceptance Criteria:**

**Given** a 3-byte MIDI message with status byte `0xE0`–`0xEF`
**When** `is_pitch_bend(data)` is called
**Then** it returns `true`

**Given** a valid pitch bend message
**When** `parse_pitch_bend(data)` is called
**Then** it returns a 14-bit value normalized to `[-1.0, +1.0]`

**Given** the pitch matching view is active and MIDI is available
**When** `setup_midi_pitch_bend_listeners(on_pitch_bend)` is called
**Then** a `MidiCleanupHandle` is returned and pitch bend events drive the slider pipeline

**Given** the user deflects the pitch bend wheel from center
**When** the first deflection is detected
**Then** the tunable note auto-starts (same as first slider interaction)

**Given** the user returns the pitch bend wheel to center (within ±3.125% dead-zone)
**When** the return-to-center is detected
**Then** the current pitch is committed as the answer

**Given** `VerticalPitchSlider` component
**When** an `external_value: Option<Signal<f64>>` prop is provided
**Then** the slider position tracks the external signal value

**Given** MIDI pitch bend setup fails or is unavailable
**When** the pitch matching view mounts
**Then** the slider works normally with pointer/keyboard input — no error UI

**Given** the complete implementation
**When** `cargo test -p domain` is run
**Then** all existing domain tests pass unchanged (domain crate is not modified)

**Given** the complete implementation
**When** `cargo clippy --workspace` is run
**Then** no warnings are produced

### Story 22.4: Update Planning Docs for MIDI Features

**As a** developer,
**I want** living documentation updated to reflect the MIDI input features added in Epic 22,
**So that** planning and architecture docs remain accurate for AI agents and contributors.

**Acceptance Criteria:**

**Given** the MIDI features implemented in Stories 22.1–22.3
**When** the PRD is reviewed
**Then** it includes functional requirements for MIDI note-on tap input and pitch bend slider control

**Given** the MIDI adapter module at `web/src/adapters/midi_input.rs`
**When** the architecture doc is reviewed
**Then** it documents the adapter in the file tree, Web MIDI API as a dependency, MIDI event flow, web-sys feature flags, and `MidiCleanupHandle` cleanup pattern

**Given** MIDI is a progressive enhancement input method
**When** the UX design spec is reviewed
**Then** it mentions MIDI as an additional input for rhythm training and pitch matching with no new screens

**Given** the MIDI adapter implementation
**When** the arc42 doc is reviewed
**Then** it includes the MIDI adapter in the building block view and a runtime sequence diagram for MIDI event flow

**Given** the project context doc
**When** it is reviewed
**Then** it lists Web MIDI API in the technology stack and documents MIDI-specific patterns

**Given** Story 22.3 was implemented but not documented in the epics file
**When** the epics doc is reviewed
**Then** Story 22.3 appears with full BDD acceptance criteria
