# Peach — Platform-Agnostic Domain Blueprint

**Purpose:** This document specifies the complete domain logic of the Peach ear-training application in a language-agnostic way. An implementing agent should be able to build a functionally equivalent application on any platform (web, desktop, CLI) using only this document as the domain reference — without reading the iOS source code.

**Scope:** Domain model, algorithms, state machines, port interfaces, persistence schemas, and composition rules. Excludes: UI layout, platform-specific APIs, visual design.

**Companion documents:** `docs/planning-artifacts/glossary.md` (full term definitions), `docs/planning-artifacts/prd.md` (product requirements, success metrics, user personas).

---

## 1. Architectural Overview

### 1.1 Two-World Model

The domain enforces a strict boundary between two conceptual worlds:

- **Logical World:** Musical abstractions — MIDI notes (integer grid 0-127), intervals (semitone distances), cents (microtonal offsets), directions (up/down). No frequency or tuning knowledge.
- **Physical World:** Acoustic reality — frequencies in Hz. This is what the audio engine receives.

**The bridge** between worlds is the `TuningSystem`, which converts logical pitch identities to physical frequencies using a reference pitch (e.g. A4 = 440 Hz). This bridge is the *only* place where the two worlds touch. No other code performs this conversion.

### 1.2 Ports and Adapters

All external capabilities are abstracted behind port interfaces:

| Port | Direction | Purpose |
|---|---|---|
| `NotePlayer` | Outbound | Play audio at a given frequency |
| `PlaybackHandle` | Outbound | Control a playing note (stop, adjust frequency) |
| `TrainingDataStore` | Outbound | Persist and retrieve training records |
| `UserSettings` | Inbound | Read user configuration (live, not cached) |
| `SoundSourceProvider` | Inbound | Discover available instrument sounds |

Domain logic never imports or references platform frameworks. Adapters implement the port interfaces using platform-specific APIs.

### 1.3 Observer Pattern

Sessions broadcast results to a list of observers injected at construction time. This decouples the training loop from persistence, profile updates, analytics, and feedback. Observers are fire-and-forget — they must not throw errors back into the session.

### 1.4 Composition Root

A single entry point wires all dependencies:
1. Creates adapters (audio engine, persistence store, settings store)
2. Creates domain services (profile, strategy, trend analyzer, timeline)
3. Creates sessions with injected dependencies and observer lists
4. Hands sessions and services to the UI layer

No domain object creates its own dependencies.

---

## 2. Domain Value Types

All types in this section are **value types** (immutable after creation, equality by value). Field names are normative — use them as-is or map them explicitly.

### 2.1 MIDINote

A discrete position on the 128-note MIDI grid.

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | integer | 0 to 127 inclusive |

**Invariant:** Construction with a value outside 0-127 is a programming error (panic/assert).

**Derived properties:**
- `name` — Human-readable note name. Computed as: `noteNames[rawValue % 12]` + `(rawValue / 12 - 1)` where `noteNames = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]`. Example: rawValue 60 = "C4", rawValue 69 = "A4".

**Operations:**
- `random(in: ClosedRange<MIDINote>) -> MIDINote` — Uniform random within range.
- `transposed(by: DirectedInterval) -> MIDINote` — Add (direction=up) or subtract (direction=down) the interval's semitone count. Panic if result falls outside 0-127.
- Comparable, Hashable, Codable.

### 2.2 Cents

A microtonal offset measured in cents (1/100 of a semitone, 1/1200 of an octave).

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | float64 | Unrestricted (can be negative) |

**Derived properties:**
- `magnitude` — `abs(rawValue)`

**Traits:** Comparable, Hashable.

### 2.3 DetunedMIDINote

A MIDI note with a microtonal cent offset — a logical pitch identity.

| Field | Type |
|---|---|
| `note` | MIDINote |
| `offset` | Cents |

**Construction shorthand:** From a bare MIDINote, offset defaults to 0.0 cents.

**Traits:** Hashable.

### 2.4 Frequency

A physical frequency in Hz.

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | float64 | Must be > 0 (panic otherwise) |

**Named constant:** `concert440 = Frequency(440.0)`

**Traits:** Comparable, Hashable.

### 2.5 Interval

Semitone distance between two notes, from prime (unison) through octave.

| Case | Raw Value (semitones) | Abbreviation |
|---|---|---|
| `prime` | 0 | P1 |
| `minorSecond` | 1 | m2 |
| `majorSecond` | 2 | M2 |
| `minorThird` | 3 | m3 |
| `majorThird` | 4 | M3 |
| `perfectFourth` | 5 | P4 |
| `tritone` | 6 | d5 |
| `perfectFifth` | 7 | P5 |
| `minorSixth` | 8 | m6 |
| `majorSixth` | 9 | M6 |
| `minorSeventh` | 10 | m7 |
| `majorSeventh` | 11 | M7 |
| `octave` | 12 | P8 |

**Operations:**
- `between(reference: MIDINote, target: MIDINote) -> Result<Interval, Error>` — Computes `abs(reference.rawValue - target.rawValue)`. Returns error if distance > 12.

**Traits:** Comparable (by semitone count), Hashable, Codable, Enumerable.

### 2.6 Direction

| Case | Raw Value |
|---|---|
| `up` | 0 |
| `down` | 1 |

**Traits:** Comparable (by raw value), Hashable, Codable, Enumerable.

### 2.7 DirectedInterval

An interval with a direction — used for asymmetric interval training.

| Field | Type |
|---|---|
| `interval` | Interval |
| `direction` | Direction |

**Special case:** Prime is always direction=up (there is no "down prime").

**Operations:**
- `between(reference: MIDINote, target: MIDINote) -> Result<DirectedInterval, Error>` — Computes interval via `Interval.between()`, direction is `up` if target >= reference, else `down`.

**Traits:** Comparable (by interval first, then direction), Hashable, Codable.

### 2.8 NoteDuration

Duration of a played note in seconds.

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | float64 | Clamped to 0.3 ... 3.0 |

### 2.9 MIDIVelocity

MIDI velocity for note-on messages.

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | uint8 | 1 to 127 inclusive (panic outside range) |

### 2.10 AmplitudeDB

Amplitude offset in decibels for loudness variation.

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | float32 | Clamped to -90.0 ... 12.0 |

### 2.11 UnitInterval

A floating-point value clamped to [0.0, 1.0]. Used for the "vary loudness" slider.

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | float64 | Clamped to 0.0 ... 1.0 |

### 2.12 SoundSourceID

Opaque identifier for an instrument sound (e.g. a SoundFont preset).

| Field | Type | Constraint |
|---|---|---|
| `rawValue` | string | Defaults to `"sf2:8:80"` if empty |

---

## 3. Tuning System (The Bridge)

An enum with two variants:

| Variant | Description |
|---|---|
| `equalTemperament` | Standard 12-TET — each semitone is exactly 100 cents |
| `justIntonation` | Pure interval ratios — cent offsets differ from 12-TET |

### 3.1 Cent Offset Table

`centOffset(interval) -> float64`:

| Interval | Equal Temperament | Just Intonation |
|---|---|---|
| prime | 0.0 | 0.0 |
| minorSecond | 100.0 | 111.731 |
| majorSecond | 200.0 | 203.910 |
| minorThird | 300.0 | 315.641 |
| majorThird | 400.0 | 386.314 |
| perfectFourth | 500.0 | 498.045 |
| tritone | 600.0 | 590.224 |
| perfectFifth | 700.0 | 701.955 |
| minorSixth | 800.0 | 813.686 |
| majorSixth | 900.0 | 884.359 |
| minorSeventh | 1000.0 | 1017.596 |
| majorSeventh | 1100.0 | 1088.269 |
| octave | 1200.0 | 1200.0 |

### 3.2 Frequency Conversion Formula

```
frequency(note: DetunedMIDINote, referencePitch: Frequency) -> Frequency:

    REFERENCE_MIDI_NOTE = 69          // A4
    SEMITONES_PER_OCTAVE = 12.0
    CENTS_PER_SEMITONE = 100.0
    OCTAVE_RATIO = 2.0

    semitones = (note.note.rawValue - REFERENCE_MIDI_NOTE)
                + note.offset.rawValue / CENTS_PER_SEMITONE

    return referencePitch.rawValue * pow(OCTAVE_RATIO, semitones / SEMITONES_PER_OCTAVE)
```

**Convenience:** `frequency(note: MIDINote, referencePitch)` treats the MIDINote as a DetunedMIDINote with offset = 0.

### 3.3 Storage Identifiers

For serialization: `equalTemperament` maps to string `"equalTemperament"`, `justIntonation` maps to string `"justIntonation"`.

---

## 4. Training Domain Entities

### 4.1 Comparison

A single comparison training interaction — two sequential notes where the user judges "higher" or "lower".

| Field | Type | Description |
|---|---|---|
| `referenceNote` | MIDINote | The first note played (exact MIDI pitch) |
| `targetNote` | DetunedMIDINote | The second note played (MIDI note + cent offset) |

**Derived properties:**
- `isTargetHigher` — `targetNote.offset.rawValue > 0`
- `referenceFrequency(tuningSystem, referencePitch)` — converts referenceNote to Hz
- `targetFrequency(tuningSystem, referencePitch)` — converts targetNote to Hz

**Correctness check:**
- `isCorrect(userAnswerHigher: bool) -> bool` — `userAnswerHigher == isTargetHigher`

### 4.2 CompletedComparison

A comparison bundled with the user's answer.

| Field | Type |
|---|---|
| `comparison` | Comparison |
| `userAnsweredHigher` | bool |
| `tuningSystem` | TuningSystem |
| `timestamp` | datetime |

**Derived:** `isCorrect` — delegates to `comparison.isCorrect(userAnswerHigher: userAnsweredHigher)`

### 4.3 PitchMatchingChallenge

Parameters for a single pitch matching attempt.

| Field | Type | Description |
|---|---|---|
| `referenceNote` | MIDINote | The reference note played first |
| `targetNote` | MIDINote | The target note the user tunes to (may differ from reference for interval mode) |
| `initialCentOffset` | float64 | Starting offset of the tunable note in cents (random in -20.0 ... +20.0) |

### 4.4 CompletedPitchMatching

A pitch matching attempt bundled with results.

| Field | Type | Description |
|---|---|---|
| `referenceNote` | MIDINote | |
| `targetNote` | MIDINote | |
| `initialCentOffset` | float64 | |
| `userCentError` | float64 | Signed difference: positive = sharp, negative = flat |
| `tuningSystem` | TuningSystem | |
| `timestamp` | datetime | |

**User cent error formula:**
```
userCentError = 1200.0 * log2(userFrequency / referenceFrequency)
```

---

## 5. Perceptual Profile (User Model)

The central model of a user's pitch perception abilities. **In-memory only** — never directly persisted. Rebuilt from stored records on each app launch.

### 5.1 PerceptualNote (per-note statistics)

A 128-element array, one entry per MIDI note.

| Field | Type | Default | Description |
|---|---|---|---|
| `mean` | float64 | 0.0 | Mean detection threshold (Welford's running mean) |
| `stdDev` | float64 | 0.0 | Standard deviation of detection threshold |
| `m2` | float64 | 0.0 | Welford's M2 accumulator |
| `sampleCount` | int | 0 | Number of comparisons for this note |
| `currentDifficulty` | float64 | 100.0 | Active difficulty during regional training (cents) |

**Derived:** `isTrained` — `sampleCount > 0`

### 5.2 Pitch Discrimination (per-note tracking)

**Incremental update (Welford's online algorithm):**
```
update(note: MIDINote, centOffset: float64, isCorrect: bool):
    stats = noteStats[note.rawValue]

    stats.sampleCount += 1
    delta = centOffset - stats.mean
    stats.mean += delta / stats.sampleCount
    delta2 = centOffset - stats.mean
    stats.m2 += delta * delta2

    variance = if sampleCount < 2 then 0.0
               else stats.m2 / (stats.sampleCount - 1)
    stats.stdDev = sqrt(variance)
```

**When called from ComparisonObserver:** `centOffset` is `comparison.targetNote.offset.magnitude` (always positive — the absolute cent difference presented).

**Weak spot identification:**
```
weakSpots(count: int = 10) -> [MIDINote]:
    For each MIDI note 0-127:
        if sampleCount == 0: score = +infinity   // untrained = weakest
        else: score = mean                        // higher threshold = weaker
    Sort descending by score
    Return top `count` as MIDINote values
```

**Summary statistics:**
- `overallMean` — Average of `mean` across all trained notes. `nil` if no trained notes.
- `overallStdDev` — Sample standard deviation of `mean` values across trained notes. `nil` if fewer than 2 trained notes.
- `averageThreshold(midiRange)` — Average `mean` across trained notes in the given MIDI range. `nil` if none trained.

### 5.3 Pitch Matching (aggregate tracking)

Separate accumulators — not per-note, just overall:

| Field | Type | Default |
|---|---|---|
| `matchingCount` | int | 0 |
| `matchingMeanAbs` | float64 | 0.0 |
| `matchingM2` | float64 | 0.0 |

**Incremental update (Welford's on absolute error):**
```
updateMatching(note: MIDINote, centError: float64):
    absError = abs(centError)
    matchingCount += 1
    delta = absError - matchingMeanAbs
    matchingMeanAbs += delta / matchingCount
    delta2 = absError - matchingMeanAbs
    matchingM2 += delta * delta2
```

**Derived:**
- `matchingMean` — `matchingMeanAbs` if count > 0, else `nil`
- `matchingStdDev` — `sqrt(matchingM2 / (matchingCount - 1))` if count >= 2, else `nil`

### 5.4 Reset

`reset()` — Replaces all 128 note stats with fresh defaults (cold start). `resetMatching()` — Zeroes matching accumulators.

---

## 6. Adaptive Algorithm (Kazez Note Strategy)

Stateless function: all inputs via parameters, output depends only on inputs.

### 6.1 Interface

```
nextComparison(
    profile: PitchDiscriminationProfile,
    settings: TrainingSettings,
    lastComparison: CompletedComparison | nil,
    interval: DirectedInterval
) -> Comparison
```

### 6.2 TrainingSettings

| Field | Type | Default |
|---|---|---|
| `noteRangeMin` | MIDINote | 36 (C2) |
| `noteRangeMax` | MIDINote | 84 (C6) |
| `referencePitch` | Frequency | 440.0 |
| `minCentDifference` | Cents | 0.1 |
| `maxCentDifference` | Cents | 100.0 |

### 6.3 Algorithm

```
difficultyRange = [settings.minCentDifference.rawValue, settings.maxCentDifference.rawValue]

IF lastComparison exists:
    p = lastComparison.comparison.targetNote.offset.magnitude
    IF lastComparison.isCorrect:
        magnitude = kazezNarrow(p), clamped to difficultyRange
    ELSE:
        magnitude = kazezWiden(p), clamped to difficultyRange

ELSE IF profile.overallMean exists:
    magnitude = profile.overallMean, clamped to difficultyRange

ELSE:
    magnitude = settings.maxCentDifference.rawValue    // cold start: 100 cents

// Random sign — equally likely higher or lower
signed = random_bool() ? magnitude : -magnitude

// Note selection — ensure target stays in MIDI 0-127 after transposition
IF interval.direction == up:
    minNote = settings.noteRangeMin
    maxNote = min(settings.noteRangeMax, 127 - interval.semitones)
ELSE:
    minNote = max(settings.noteRangeMin, interval.semitones)
    maxNote = settings.noteRangeMax

referenceNote = MIDINote.random(in: minNote...maxNote)
targetBaseNote = referenceNote.transposed(by: interval)

RETURN Comparison(
    referenceNote: referenceNote,
    targetNote: DetunedMIDINote(note: targetBaseNote, offset: Cents(signed))
)
```

### 6.4 Kazez Formulas

```
kazezNarrow(p: float64) -> float64:
    return p * (1.0 - 0.05 * sqrt(p))

kazezWiden(p: float64) -> float64:
    return p * (1.0 + 0.09 * sqrt(p))
```

**Behavior:** After a correct answer, the difficulty narrows (gets harder). After an incorrect answer, it widens (gets easier). The square-root factor makes the step size proportional to the current difficulty — large steps at high cent values, small steps near threshold.

**Example convergence at 50 cents:**
- Correct: `50 * (1.0 - 0.05 * 7.07)` = `50 * 0.646` = `32.3 cents`
- Wrong: `50 * (1.0 + 0.09 * 7.07)` = `50 * 1.636` = `81.8 cents`

---

## 7. Session State Machines

Sessions are the central orchestrators. They coordinate comparison generation, note playback, answer handling, observer notification, and feedback display. Sessions are **reference types** (classes) with mutable state.

### 7.1 ComparisonSession

#### States

```
idle -> playingNote1 -> playingNote2 -> awaitingAnswer -> showingFeedback -> playingNote1 (loop)
                                    \                  /
                                     +-> (user answers during playingNote2) -+
```

| State | Description | UI Behavior |
|---|---|---|
| `idle` | Not training | Start button visible |
| `playingNote1` | Reference note playing | Answer buttons disabled |
| `playingNote2` | Target note playing | Answer buttons **enabled** (early answer allowed) |
| `awaitingAnswer` | Both notes finished, waiting | Answer buttons enabled |
| `showingFeedback` | Showing correct/incorrect for 400ms | Buttons disabled, feedback indicator visible |

#### Dependencies (injected at construction)

| Dependency | Type | Purpose |
|---|---|---|
| `notePlayer` | NotePlayer | Audio playback |
| `strategy` | NextComparisonStrategy | Comparison selection |
| `profile` | PitchDiscriminationProfile | User's perceptual model |
| `userSettings` | UserSettings | Live configuration |
| `observers` | [ComparisonObserver] | Result broadcasting |
| `resettables` | [Resettable] | Components to reset on data clear |

#### Configuration Constants

| Constant | Value | Description |
|---|---|---|
| `velocity` | 63 | MIDI velocity for all notes |
| `feedbackDuration` | 0.4 seconds | Duration of feedback indicator display |
| `maxLoudnessOffsetDB` | 5.0 | Maximum amplitude variation when varyLoudness > 0 |

#### Observable State

| Property | Type | Description |
|---|---|---|
| `state` | ComparisonSessionState | Current state machine state |
| `showFeedback` | bool | Whether feedback indicator is visible |
| `isLastAnswerCorrect` | bool? | Result of most recent answer (nil initially) |
| `sessionBestCentDifference` | float64? | Smallest correctly answered cent difference this session |
| `currentInterval` | DirectedInterval? | Interval being trained in current comparison |

#### start(intervals: Set<DirectedInterval>)

**Guard:** Must be in `idle` state. Must have at least one interval.

1. Store `intervals` as session intervals
2. Snapshot `tuningSystem` from userSettings (locked for entire session)
3. Begin async training loop

#### Training Loop (async)

```
LOOP:
    interval = random element from sessionIntervals
    comparison = strategy.nextComparison(profile, currentSettings, lastCompleted, interval)

    amplitudeDB = calculateTargetAmplitude(userSettings.varyLoudness)

    state = playingNote1
    play referenceFrequency for noteDuration at velocity=63, amplitudeDB=0.0
    IF cancelled or stopped: RETURN

    state = playingNote2
    play targetFrequency for noteDuration at velocity=63, amplitudeDB=amplitudeDB
    IF cancelled or stopped: RETURN

    IF still in playingNote2:
        state = awaitingAnswer

    // WAIT for handleAnswer() to be called (async coordination)
```

**Amplitude calculation:**
```
calculateTargetAmplitude(varyLoudness: float64) -> AmplitudeDB:
    IF varyLoudness <= 0.0: return AmplitudeDB(0.0)
    range = varyLoudness * 5.0     // maxLoudnessOffsetDB
    offset = random float in [-range, +range]
    return AmplitudeDB(offset)
```

#### handleAnswer(isHigher: bool)

**Guard:** State must be `playingNote2` or `awaitingAnswer`.

1. If target note is still playing, stop it immediately
2. Create `CompletedComparison` with current timestamp and session's tuning system
3. Track session best: if correct and cent difference < current best, update best
4. Broadcast to all observers: `observer.comparisonCompleted(completed)`
5. Transition to feedback state:
   - Set `isLastAnswerCorrect` and `showFeedback = true`
   - `state = showingFeedback`
   - After 400ms: `showFeedback = false`, begin next comparison

#### stop()

**Guard:** Not already idle.

1. Stop all audio playback via `notePlayer.stopAll()`
2. Cancel async tasks (training loop, feedback timer)
3. Reset all session state to defaults
4. `state = idle`

#### resetTrainingData()

1. Stop session if running
2. Clear `lastCompletedComparison` and `sessionBestCentDifference`
3. Call `profile.reset()`
4. Call `reset()` on each resettable

### 7.2 PitchMatchingSession

#### States

```
idle -> playingReference -> awaitingSliderTouch -> playingTunable -> showingFeedback -> playingReference (loop)
```

| State | Description | UI Behavior |
|---|---|---|
| `idle` | Not training | Start button visible |
| `playingReference` | Reference note playing for fixed duration | Slider visible but disabled |
| `awaitingSliderTouch` | Reference finished, waiting for user to touch slider | Slider visible, waiting for first touch |
| `playingTunable` | Tunable note playing indefinitely, user adjusting | Slider active, pitch changes in real time |
| `showingFeedback` | Result displayed for 400ms | Slider inactive, feedback indicator visible |

#### Dependencies (injected at construction)

| Dependency | Type | Purpose |
|---|---|---|
| `notePlayer` | NotePlayer | Audio playback |
| `profile` | PitchMatchingProfile | Matching statistics |
| `observers` | [PitchMatchingObserver] | Result broadcasting |
| `userSettings` | UserSettings | Live configuration |

#### Configuration Constants

| Constant | Value |
|---|---|
| `velocity` | 63 |
| `feedbackDuration` | 0.4 seconds |
| `initialCentOffsetRange` | -20.0 ... +20.0 cents |

#### Training Loop (async)

```
LOOP:
    interval = random element from sessionIntervals
    challenge = generateChallenge(settings, interval)

    refFreq = tuningSystem.frequency(challenge.referenceNote, referencePitch)
    targetFreq = tuningSystem.frequency(challenge.targetNote, referencePitch)
    store targetFreq as referenceFrequency (the "correct answer" frequency)

    state = playingReference
    play refFreq for noteDuration
    IF cancelled: RETURN

    tunableFreq = tuningSystem.frequency(
        DetunedMIDINote(challenge.targetNote, Cents(challenge.initialCentOffset)),
        referencePitch)

    state = awaitingSliderTouch
    WAIT for user to touch slider (async suspension point)
    IF cancelled: RETURN

    state = playingTunable
    handle = play tunableFreq indefinitely (returns PlaybackHandle)
    store handle for frequency adjustment
```

#### generateChallenge(settings, interval)

```
// Ensure transposed note stays in range
IF interval.direction == up:
    minNote = settings.noteRangeMin
    maxNote = min(settings.noteRangeMax, 127 - interval.semitones)
ELSE:
    minNote = max(settings.noteRangeMin, interval.semitones)
    maxNote = settings.noteRangeMax

referenceNote = MIDINote.random(in: minNote...maxNote)
targetNote = referenceNote.transposed(by: interval)
offset = random float64 in [-20.0, +20.0]

RETURN PitchMatchingChallenge(referenceNote, targetNote, offset)
```

#### adjustPitch(value: float64)

Called continuously as the user drags the slider. `value` is in range [-1.0, +1.0].

```
IF state == awaitingSliderTouch:
    state = playingTunable
    resume slider-touch suspension
    RETURN

IF state == playingTunable AND referenceFrequency exists:
    centOffset = value * 20.0     // initialCentOffsetRange.upperBound
    frequency = referenceFrequency * pow(2.0, centOffset / 1200.0)
    handle.adjustFrequency(frequency)
```

#### commitPitch(value: float64)

Called when the user releases the slider.

```
IF state == playingTunable AND referenceFrequency exists:
    centOffset = value * 20.0
    userFrequency = referenceFrequency * pow(2.0, centOffset / 1200.0)

    stop the playing note (via handle)

    userCentError = 1200.0 * log2(userFrequency / referenceFrequency)

    result = CompletedPitchMatching(
        referenceNote, targetNote, initialCentOffset,
        userCentError, tuningSystem, now())

    broadcast to all observers
    state = showingFeedback
    after 400ms: play next challenge
```

#### stop()

Same pattern as ComparisonSession — cancel tasks, stop audio, reset state, return to idle.

---

## 8. Trend Analysis

### 8.1 TrendAnalyzer

Computes whether the user's detection ability is improving, stable, or declining.

**Input:** Chronological sequence of absolute cent offsets from completed comparisons.

**Constants:**
- `minimumRecordCount = 20` — No trend shown below this
- `changeThreshold = 0.05` — 5% change required for non-stable classification

**Algorithm:**
```
IF absOffsets.count < 20: trend = nil (insufficient data)
ELSE:
    midpoint = count / 2
    earlierMean = mean(absOffsets[0..<midpoint])
    laterMean = mean(absOffsets[midpoint..])

    IF earlierMean <= 0: trend = stable
    ELSE:
        changeRatio = (laterMean - earlierMean) / earlierMean
        IF changeRatio < -0.05: trend = improving      // threshold going down = better
        ELSE IF changeRatio > 0.05: trend = declining   // threshold going up = worse
        ELSE: trend = stable
```

**Trend values:** `improving`, `stable`, `declining`, or `nil` (insufficient data).

**Observer integration:** Implements ComparisonObserver — appends `abs(centOffset)` on each comparison, recomputes immediately.

### 8.2 ThresholdTimeline

Records all comparison data points for timeline visualization and computes rolling aggregates.

**Per-datapoint:**

| Field | Type |
|---|---|
| `timestamp` | datetime |
| `centOffset` | float64 (absolute) |
| `isCorrect` | bool |
| `referenceNote` | int |

**Aggregation:** Groups data points by calendar period (default: day). Per period:

| Field | Type |
|---|---|
| `periodStart` | datetime |
| `meanThreshold` | float64 (mean of centOffsets in period) |
| `comparisonCount` | int |
| `correctCount` | int |

**Rolling mean/stdDev:** Computed over a sliding window (default: 20 periods) of aggregated points.

**Observer integration:** Implements ComparisonObserver.

---

## 9. Port Interfaces

### 9.1 NotePlayer

```
interface NotePlayer:
    // Play indefinitely, return handle for control
    play(frequency: Frequency, velocity: MIDIVelocity, amplitudeDB: AmplitudeDB)
        -> async throws PlaybackHandle

    // Play for fixed duration (convenience — implemented via play + sleep + stop)
    play(frequency: Frequency, duration: TimeInterval, velocity: MIDIVelocity, amplitudeDB: AmplitudeDB)
        -> async throws void

    // Stop all currently playing notes
    stopAll() -> async throws void
```

**Default implementation of timed play:**
```
play(frequency, duration, velocity, amplitudeDB):
    IF duration <= 0: throw invalidDuration
    handle = play(frequency, velocity, amplitudeDB)
    sleep(duration)
    handle.stop()
    // If sleep is cancelled, still stop the handle
```

### 9.2 PlaybackHandle

```
interface PlaybackHandle:
    stop() -> async throws void               // Idempotent — subsequent calls are no-ops
    adjustFrequency(Frequency) -> async throws void   // Real-time pitch change
```

### 9.3 AudioError

```
enum AudioError:
    engineStartFailed(string)
    invalidFrequency(string)
    invalidDuration(string)
    invalidPreset(string)
    contextUnavailable
    invalidInterval(string)
```

### 9.4 UserSettings

```
interface UserSettings:
    noteRangeMin: MIDINote          // Default: 36 (C2)
    noteRangeMax: MIDINote          // Default: 84 (C6)
    noteDuration: NoteDuration      // Default: 1.0 seconds
    referencePitch: Frequency       // Default: 440.0 Hz
    soundSource: SoundSourceID      // Default: "sf2:8:80"
    varyLoudness: UnitInterval      // Default: 0.0 (no variation)
    intervals: Set<DirectedInterval> // Default: {prime_up} (unison only)
    tuningSystem: TuningSystem      // Default: equalTemperament
```

**Important:** Settings are read live on every comparison. They are never cached by sessions. This allows changes to take effect immediately.

### 9.5 SoundSourceProvider

```
interface SoundSourceProvider:
    availableSources: [SoundSourceID]
    displayName(for: SoundSourceID) -> string
```

### 9.6 TrainingDataStore

```
interface TrainingDataStore:
    // Comparison records
    save(ComparisonRecord) -> throws void
    fetchAllComparisons() -> throws [ComparisonRecord]   // sorted by timestamp ascending
    delete(ComparisonRecord) -> throws void

    // Pitch matching records
    save(PitchMatchingRecord) -> throws void
    fetchAllPitchMatchings() -> throws [PitchMatchingRecord]  // sorted by timestamp ascending

    // Bulk delete
    deleteAll() -> throws void    // transactional — all or nothing
```

Also implements `ComparisonObserver` and `PitchMatchingObserver` to auto-persist results.

### 9.7 Resettable

```
interface Resettable:
    reset() -> throws void
```

### 9.8 Observer Protocols

```
interface ComparisonObserver:
    comparisonCompleted(CompletedComparison) -> void

interface PitchMatchingObserver:
    pitchMatchingCompleted(CompletedPitchMatching) -> void
```

**Observers must not throw.** Errors in observers must be caught internally and logged, never propagated to the session.

---

## 10. Persistence Record Schemas

These are the flat records stored in the database. All domain types are decomposed to primitives for storage.

### 10.1 ComparisonRecord

| Field | Type | Description |
|---|---|---|
| `referenceNote` | int | MIDI note 0-127 |
| `targetNote` | int | MIDI note 0-127 |
| `centOffset` | float64 | Signed cent offset applied to target |
| `isCorrect` | bool | Whether user answered correctly |
| `interval` | int | Semitone distance between reference and target (0-12) |
| `tuningSystem` | string | Storage identifier (`"equalTemperament"` or `"justIntonation"`) |
| `timestamp` | datetime | When the comparison was answered |

### 10.2 PitchMatchingRecord

| Field | Type | Description |
|---|---|---|
| `referenceNote` | int | MIDI note 0-127 |
| `targetNote` | int | MIDI note 0-127 |
| `initialCentOffset` | float64 | Starting offset of tunable note |
| `userCentError` | float64 | Signed error (positive = sharp, negative = flat) |
| `interval` | int | Semitone distance (0-12) |
| `tuningSystem` | string | Storage identifier |
| `timestamp` | datetime | When the attempt was completed |

### 10.3 Record Construction from Domain Types

**ComparisonRecord from CompletedComparison:**
```
record.referenceNote = completed.comparison.referenceNote.rawValue
record.targetNote = completed.comparison.targetNote.note.rawValue
record.centOffset = completed.comparison.targetNote.offset.rawValue
record.isCorrect = completed.isCorrect
record.interval = Interval.between(referenceNote, targetNote.note).semitones  // 0 if error
record.tuningSystem = completed.tuningSystem.storageIdentifier
record.timestamp = completed.timestamp
```

**PitchMatchingRecord from CompletedPitchMatching:**
```
record.referenceNote = result.referenceNote.rawValue
record.targetNote = result.targetNote.rawValue
record.initialCentOffset = result.initialCentOffset
record.userCentError = result.userCentError
record.interval = Interval.between(referenceNote, targetNote).semitones  // 0 if error
record.tuningSystem = result.tuningSystem.storageIdentifier
record.timestamp = result.timestamp
```

---

## 11. Composition Rules

The following wiring must occur at application startup:

### 11.1 Object Creation Order

1. **Persistence adapter** — Platform-specific store implementing TrainingDataStore
2. **Profile** — `PerceptualProfile()` (cold start)
3. **Profile hydration** — Fetch all ComparisonRecords, replay each through `profile.comparisonCompleted()`. Fetch all PitchMatchingRecords, replay each through `profile.pitchMatchingCompleted()`.
4. **Strategy** — `KazezNoteStrategy()`
5. **Trend analyzer** — `TrendAnalyzer(records: allComparisonRecords)`
6. **Threshold timeline** — `ThresholdTimeline(records: allComparisonRecords)`
7. **Settings adapter** — Platform-specific implementation of UserSettings
8. **Audio adapter** — Platform-specific implementation of NotePlayer
9. **ComparisonSession** with:
   - notePlayer = audio adapter
   - strategy = strategy
   - profile = profile
   - userSettings = settings adapter
   - observers = [dataStore, profile, trendAnalyzer, thresholdTimeline, hapticManager*]
   - resettables = [dataStore, trendAnalyzer, thresholdTimeline]
10. **PitchMatchingSession** with:
    - notePlayer = audio adapter
    - profile = profile
    - observers = [dataStore, profile]
    - userSettings = settings adapter

*hapticManager is iOS-specific; web may substitute a no-op or visual feedback observer.

### 11.2 Profile Hydration Invariant

The profile is **never persisted directly**. On every launch:
1. Load all comparison records (sorted by timestamp)
2. For each record, call `profile.update(note: MIDINote(referenceNote), centOffset: abs(centOffset), isCorrect: isCorrect)`
3. Load all pitch matching records (sorted by timestamp)
4. For each record, call `profile.updateMatching(note: MIDINote(referenceNote), centError: userCentError)`

This guarantees the profile is always consistent with the stored records. No migration, no cache invalidation, no versioning — the records are the single source of truth.

### 11.3 Reset All Training Data

When the user resets:
1. Stop active session
2. `profile.reset()` + `profile.resetMatching()`
3. `trendAnalyzer.reset()`
4. `thresholdTimeline.reset()`
5. `dataStore.deleteAll()`

---

## 12. Web-Specific Decision Points

These are the decisions that must be made for the web platform. They are **not** covered by this blueprint because they have no iOS equivalent or require fundamentally different solutions.

| Decision | Context |
|---|---|
| **Audio engine** | Web Audio API via Rust/WASM. SoundFont playback requires a SoundFont parser and synthesizer in Rust (or JS). Consider: `rustysynth`, `oxisynth`, or JS-based FluidSynth. |
| **Pitch bend / detuning** | Web Audio API uses `AudioBufferSourceNode.detune` (in cents) or `playbackRate`. If using SoundFont in Rust, pitch bend must be handled in the synthesizer. |
| **Storage** | IndexedDB (via `web-sys` or `idb` crate), localStorage (limited), or even in-memory with file export. Must support the same CRUD operations as TrainingDataStore. |
| **UI framework** | Leptos, Dioxus, Yew, or Sycamore (Rust/WASM frameworks). Or: Rust/WASM for domain + plain HTML/JS for UI. |
| **Reactivity** | The iOS app uses `@Observable` for automatic UI updates. Web frameworks have their own reactivity systems (signals, hooks, etc.). The domain types don't need to participate — only session state and profile need to trigger re-renders. |
| **Async model** | The iOS app uses Swift async/await and Task. Rust/WASM uses `wasm-bindgen-futures` or `gloo-timers`. The session state machine pattern remains the same. |
| **Haptic feedback** | No direct equivalent on web. Could substitute with a brief visual flash, screen shake, or Gamepad API vibration (limited support). |
| **Audio interruption handling** | Browser tab visibility changes, audio context suspension. Use Page Visibility API and AudioContext state changes. |
| **SoundFont loading** | Bundle as static asset or let user upload. Parse in Rust or use JS SoundFont library. |
| **Deployment** | Static files (Wasm + HTML + CSS + assets) — can be served from any CDN. No server needed. |
