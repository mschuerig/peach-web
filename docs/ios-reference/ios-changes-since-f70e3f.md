# iOS Changes Since f70e3f — Impact Analysis for peach-web

## Overview

196 commits across Epics 44–60. Three major themes:

1. **Terminology renames** — align with musical/psychoacoustic usage
2. **Architecture refactoring** — decouple disciplines, generalize profile
3. **Rhythm training** — three new training disciplines

---

## 1. Terminology Renames

### Naming Matrix

| Perceptual Domain | Threshold Task (passive) | Reproduction Task (active) |
|---|---|---|
| **Pitch** | Pitch Discrimination | Pitch Matching |
| **Rhythm** | Rhythm Offset Detection | Continuous Rhythm Matching |

UI groups: **Compare** (threshold/passive tasks) and **Match** (reproduction/active tasks). The Rhythm section uses "Compare Timing" and "Fill the Gap" as button labels.

> **Note:** An intermediate `RhythmMatching` discipline (tap where the 4th beat should be) was implemented and then removed in favor of `ContinuousRhythmMatching`. The removal touched ~39 files — which motivated the Epic 55 discipline decoupling.

### Complete Rename Map

| Old (current in peach-web) | New (iOS) |
|---|---|
| `TrainingMode` | `TrainingDiscipline` |
| `TrainingMode::UnisonPitchComparison` | `TrainingDiscipline::UnisonPitchDiscrimination` |
| `TrainingMode::IntervalPitchComparison` | `TrainingDiscipline::IntervalPitchDiscrimination` |
| `TrainingMode::UnisonMatching` | `TrainingDiscipline::UnisonPitchMatching` |
| `TrainingMode::IntervalMatching` | `TrainingDiscipline::IntervalPitchMatching` |
| `PitchComparison` | `PitchDiscriminationTrial` |
| `CompletedPitchComparison` | `CompletedPitchDiscriminationTrial` |
| `PitchComparisonSession` | `PitchDiscriminationSession` |
| `PitchComparisonSessionState` | `PitchDiscriminationSessionState` |
| `PitchComparisonObserver` | `PitchDiscriminationObserver` |
| `PitchComparisonRecord` | `PitchDiscriminationRecord` |
| `PitchComparisonSettings` (was `*TrainingSettings`) | `PitchDiscriminationSettings` |
| `PitchMatchingChallenge` | `PitchMatchingTrial` |
| `CompletedPitchMatching` | `CompletedPitchMatchingTrial` |
| `PitchMatchingSettings` (was `*TrainingSettings`) | `PitchMatchingSettings` |
| `TrainingModeStatistics` | `TrainingDisciplineStatistics` |
| `TrainingModeState` | `TrainingDisciplineState` |

### Slug Values (persistence/URLs)

| Old | New |
|---|---|
| `"pitch-comparison"` | `"pitch-discrimination"` |
| `"interval-comparison"` | `"interval-discrimination"` |
| `"pitch-matching"` | `"pitch-matching"` (unchanged) |
| `"interval-matching"` | `"interval-matching"` (unchanged) |

### UI Button Labels

Start screen buttons simplified: "Hear & Compare" / "Tune & Match" → **"Compare"** / **"Match"**, with section headers ("Single Notes", "Intervals", "Rhythm") providing context.

### Key Glossary Concept

**Trial** — one atomic presentation-response cycle. Replaces "Comparison" and "Challenge" as the unit-of-work noun.

---

## 2. Architecture Changes

### 2a. Module Split: Core/Audio → Core/Music + Core/Audio

**Rationale:** Separate musical domain value types from audio infrastructure to prepare for rhythm training.

- **Core/Music**: Pure domain value types — `MIDINote`, `DetunedMIDINote`, `Frequency`, `Cents`, `Interval`, `DirectedInterval`, `Direction`, `TuningSystem`, `MIDIVelocity`, `AmplitudeDB`, `NoteDuration`, `NoteRange`, `SoundSourceID`, plus new rhythm types (`TempoBPM`, `RhythmOffset`, `RhythmDirection`, `TempoRange`)
- **Core/Audio**: Engine, players, sequencer — `SoundFontEngine`, `NotePlayer`, `SoundFontNotePlayer`, `RhythmPlayer`, `StepSequencer`

**Web app impact:** Mostly organizational. The web app's `domain/` crate already separates types from audio. New rhythm types need to be added.

### 2b. PerceptualProfile Redesign

**Problem:** Profile accumulated discipline-specific convenience methods, violating open-closed principle.

**Solution — three design moves:**

1. **`StatisticalSummary` sum type** — wraps `TrainingDisciplineStatistics` with an enum discriminator (`.continuous(stats)`, future: `.ordinal(...)`, `.spatial(...)`). Common computed properties (`recordCount`, `trend`, `ewma`, `metrics`) available without pattern matching.

2. **`StatisticsKey`** — typed key for the profile store:
   - `.pitch(TrainingDisciplineID)` — 1:1 mapping (one key per pitch discipline)
   - `.rhythm(TrainingDisciplineID, TempoRange, RhythmDirection)` — expands to `TempoRange × RhythmDirection` keys (3×2 = 6 per discipline). Expansion logic lives on the discipline, not the profile.

3. **Generic merge** — `mergedStatistics(for: [StatisticsKey])` collects metrics from multiple keys, merges chronologically, rebuilds summary. One code path for all disciplines.

**Result:** `PerceptualProfile` became a domain-agnostic `HashMap<StatisticsKey, TrainingDisciplineStatistics>`. ~130 lines of legacy convenience methods deleted.

**Web app impact:** Major refactor of `profile.rs`. The current `HashMap<TrainingMode, TrainingModeStatistics>` becomes `HashMap<StatisticsKey, TrainingDisciplineStatistics>`. Backward-compat APIs (`comparison_mean`, `matching_mean`) can be removed.

### 2c. Discipline Decoupling (Epic 55)

**Problem:** Adding/removing a discipline touched ~39 files across 8 coupling categories.

**Solution — three stories:**

1. **Port protocols** — `ProfileUpdating` and `TrainingRecordPersisting` traits. Profile and data store lose all discipline-specific observer conformances. Adapter structs in each feature directory map trial results to generic port operations.

2. **`TrainingDiscipline` protocol** — each discipline provides: `id`, `config`, `statisticsKeys`, `recordType`, CSV ownership (`csvColumns`, `csvKeyValuePairs`, `parseCSVRow`), and profile feeding (`feedRecords`).

3. **`TrainingDisciplineRegistry`** — compile-time registration. Single point that knows active disciplines.

**Result:** Adding a discipline = create feature directory + register (1 line) + nav case (1 file) + localization (1 file). Removing = reverse.

**Web app impact:** Consider whether to adopt this pattern. The web app currently has a simpler structure. With rhythm adding 3 new disciplines (total 7), the decoupling pattern becomes more valuable.

---

## 3. New Rhythm Training Disciplines

### 3a. Rhythm Offset Detection

**User experience:** Four percussion clicks at sixteenth-note intervals. The 3rd note is offset early or late. User taps "Early" or "Late."

- Dots light up as visual metronome (non-positional)
- First note accented (visual + audio)
- Difficulty adapts independently for early vs. late (asymmetric tracking)
- Feedback: checkmark/cross + difficulty as percentage

### 3b. Continuous Rhythm Matching ("Fill the Gap")

**User experience:** Step-sequencer loop of 4 sixteenth notes plays indefinitely. One step per cycle is a silent gap. User taps to fill it.

- Gap position configurable (positions 1-4: "Beat", "E", "And", "A")
- Multiple positions can be enabled; each cycle randomly selects one
- Tap evaluation window: ±50% of one sixteenth-note duration
- 16 cycles aggregate into one trial; incomplete trials discarded on exit
- After Epic 57: tap triggers click sound (user audibly fills the gap), directional timing indicator

### New Domain Types

| Type | Purpose | Web app equivalent needed |
|---|---|---|
| `TempoBPM` | Int BPM, computes `sixteenth_note_duration` | Yes |
| `RhythmOffset` | Signed duration; negative=early, positive=late | Yes |
| `RhythmDirection` | Enum: `.early` / `.late` | Yes |
| `TempoRange` | Tempo bucketing for multi-key statistics | Yes |
| `StepPosition` | Enum: positions 1-4 in step sequencer | Yes |
| `CompletedRhythmOffsetDetectionTrial` | Result: tempo, offset, isCorrect | Yes |
| `CompletedRhythmMatchingTrial` | Result: tempo, userOffset | Yes |
| `CompletedContinuousRhythmMatchingTrial` | Result: tempo, meanOffset, hitRate, breakdown | Yes |
| `RhythmOffsetDetectionRecord` | Persistence record | Yes |
| `RhythmMatchingRecord` | Persistence record | Yes |
| `ContinuousRhythmMatchingRecord` | Persistence record | Yes |

### New Training Disciplines (enum cases)

| Discipline | Slug |
|---|---|
| `TrainingDiscipline::RhythmOffsetDetection` | `"rhythm-offset-detection"` |
| `TrainingDiscipline::ContinuousRhythmMatching` | `"continuous-rhythm-matching"` |

> `RhythmMatching` (`"rhythm-matching"`) was implemented and later removed. It does not exist in the current iOS codebase.

### Difficulty Model

- **Axis 1 (adaptive):** Time offset magnitude — tracked independently for early vs. late
- **Axis 2 (user-selected):** Tempo (40-200 BPM, default 80)
- **User-facing unit:** Percentage of sixteenth note (analogous to cents for pitch)

### Audio Requirements

- Sample-accurate scheduling (Web Audio `AudioWorklet` or `ScriptProcessorNode`)
- Percussion click synthesis or SoundFont percussion channel
- Step sequencer loop for Continuous Rhythm Matching
- Sub-millisecond timing measurement

### Start Screen Changes

From 4 buttons to 6, organized in 3 sections:
- **Single Notes**: Compare (discrimination), Match (matching)
- **Intervals**: Compare (discrimination), Match (matching)
- **Rhythm**: Compare Timing (offset detection), Fill the Gap (continuous rhythm matching)

---

## 4. Current Web App State (Delta Summary)

| Aspect | Web App Now | iOS Now | Gap |
|---|---|---|---|
| Terminology | `TrainingMode`, `PitchComparison`, `Challenge` | `TrainingDiscipline`, `PitchDiscrimination`, `Trial` | Full rename needed |
| Training modes | 4 (pitch only) | 6 (4 pitch + 2 rhythm) | 2 disciplines to add |
| Profile model | `HashMap<TrainingMode, Stats>` | `HashMap<StatisticsKey, Stats>` | Redesign needed |
| Discipline coupling | Moderate (enum-based) | Fully decoupled (protocol registry) | Consider adopting |
| Rhythm support | None | Full (2 disciplines, audio engine, profiles) | Major feature add |
| Start screen | 4 buttons, 2 sections | 6 buttons, 3 sections | Layout + routing |
| CSV format | V1 (pitch only) | V3 (discipline-owned, rhythm included) | Version bump needed |
| Button labels | "Hear & Compare" / "Tune & Match" | "Compare" / "Match" | Simplify labels |

---

## 5. Implementation Roadmap

### Epic 14: Terminology Alignment (3 stories)
- 14.1: Domain crate type/enum/session/observer/record renames
- 14.2: Web crate component/route/bridge/localization renames
- 14.3: Update living docs (architecture, PRD, arc42)

### Epic 15: Start Screen + Rhythm Scaffolding (4 stories)
- 15.1: Extend `TrainingDiscipline` with rhythm cases, add `TempoBPM` and `StepPosition` types
- 15.2: Start screen 3-section layout (6 buttons) with rhythm routes
- 15.3: Placeholder rhythm training screens
- 15.4: Rhythm settings (tempo stepper, gap position toggles)

### Epic 16: Architecture Refactoring (4 stories)
- 16.1: Generalize statistics from `Cents` to `f64` metric
- 16.2: `StatisticsKey` enum and `PerceptualProfile` redesign (multi-key rhythm support)
- 16.3: Decouple observers with generic port traits (`ProfileUpdating`, `TrainingRecordPersisting`)
- 16.4: Generalize IndexedDB store and hydration pipeline

### Epic 17: Rhythm Offset Detection (4 stories)
- 17.1: Rhythm domain types (`RhythmOffset`, records, trial types)
- 17.2: Click synthesis and Web Audio lookahead scheduler
- 17.3: Rhythm offset detection session state machine + adaptive strategy
- 17.4: Rhythm offset detection screen UI (replaces placeholder)

### Epic 18: Continuous Rhythm Matching (2 stories)
- 18.1: Session state machine, step sequencer loop mode, tap evaluation, 16-cycle aggregation
- 18.2: "Fill the Gap" screen UI (replaces placeholder)

### Epic 19: CSV V3 + Rhythm Profiles (2 stories)
- 19.1: CSV V3 format alignment (19-column header, cross-platform compatibility)
- 19.2: Rhythm profile visualization (progress cards, sparklines, charts)
