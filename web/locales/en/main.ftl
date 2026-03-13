## General
app-name = Peach
skip-to-content = Skip to main content
page-not-found = Page not found
back-to-start = Back to Start
back = Back
done = Done
cancel = Cancel
loading = Loading…
loading-sounds = Loading sounds…

## Navigation
nav-info = Info
nav-profile = Profile
nav-settings = Settings
nav-help = Help
page-navigation = Page navigation

## Start Page
single-notes = Single Notes
intervals = Intervals
hear-and-compare = Hear & Compare
tune-and-match = Tune & Match
hear-compare-single-aria = Hear and Compare, Single Notes
tune-match-single-aria = Tune and Match, Single Notes
hear-compare-intervals-aria = Hear and Compare, Intervals
tune-match-intervals-aria = Tune and Match, Intervals

## Training
training-started = Training started
training-stopped = Training stopped
higher = Higher
lower = Lower
correct = Correct
incorrect = Incorrect
tap-to-start = Tap to Start Training
pitch-adjustment = Pitch adjustment

## Training - Pitch Matching
dead-center = Dead center
cents-sharp = { $value } cents sharp
cents-flat = { $value } cents flat
cents-signed = { $value } cents
sharp-label = sharp
flat-label = flat

## Training Stats
latest = Latest:
best = Best:
value-cents = { $value } cents
trend-improving = Improving
trend-stable = Stable
trend-declining = Declining

## Settings
settings-title = Settings
settings-help-title = Settings Help
pitch-range = Pitch Range
lowest-note = Lowest Note: { $note }
highest-note = Highest Note: { $note }
intervals-section = Intervals
intervals-hint = Select the intervals you want to practice. At least one must remain active.
ascending = ascending
descending = descending
sound-section = Sound
sound-label = Sound
sine-oscillator = Sine Oscillator
stop-preview = Stop preview
preview-sound = Preview sound
duration-label = Duration: { $value }s
concert-pitch-label = Concert Pitch: { $value } Hz
tuning-label = Tuning
equal-temperament = Equal Temperament
just-intonation = Just Intonation
tuning-hint = Select the tuning for intervals. Equal temperament divides the octave into 12 equal steps. Just intonation uses pure frequency ratios.
difficulty-section = Difficulty
loudness-variation = Loudness Variation
loudness-variation-aria = Loudness variation
off = Off
max = Max
data-section = Data
export-training-data = Export Training Data
exporting = Exporting…
exported = Exported!
import-training-data = Import Training Data
importing = Importing…
delete-all-training-data = Delete All Training Data
resetting = Resetting…
data-reset = Data Reset
reset-failed = Reset Failed
select-csv = Select CSV file to import
file-too-large = File too large (max 10 MB)
records-imported = { $count } records imported
records-merged = { $imported } records imported, { $skipped } duplicates skipped
data-exported = Data exported
export-failed = Export failed: { $error }
import-failed = Import failed: { $error }
import-dialog-title = Import Training Data
import-dialog-found = Found { $comparisons } comparison records and { $matchings } pitch matching records.{ $warnings } How would you like to import?
import-dialog-warnings = { " " }({ $count } rows skipped with warnings)
replace-all-data = Replace All Data
merge-with-existing = Merge with Existing
reset-dialog-title = Reset Training Data?
reset-dialog-message = This will permanently delete all training data, including your perceptual profile and comparison history. This cannot be undone.
delete-all-data = Delete All Data
database-not-available = Database not available
language-label = Language

## Comparison Training
comparison-title = Hear & Compare
comparison-help-title = Comparison Training

## Pitch Matching
matching-title = Tune & Match
matching-help-title = Pitch Matching Training

## Profile
profile-title = Profile
no-training-data = No training data yet. Start a training session to see your progress.
profile-no-data-aria = Profile. No training data available.
profile-showing-progress = Profile showing progress for: { $modes }
progress-for = Progress for { $name }: { $ewma } cents, { $trend }
progress-chart-for = Progress chart for { $name }
chart-today = Today
current-trend = Current: { $ewma } cents, trend: { $trend }
value-trend = { $ewma } cents, trend { $trend }

## Info
version-label = Version { $number }
developer = Developer
project = Project
github-label = GitHub:
license-label = License:
copyright-label = Copyright:
copyright-text = © 2026 Michael Schürig
acknowledgments = Acknowledgments
acknowledgments-body = Sounds provided by <a href="https://schristiancollins.com/generaluser.php" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">GeneralUser GS by S. Christian Collins</a>.

## Audio Errors
audio-engine-failed = Audio engine failed to start
audio-playback-failed = Audio playback failed
sound-load-failed = Selected sound could not be loaded. Using default sound.
storage-write-failed = Training data may not have been saved. Training continues.

## Help - Settings
help-training-range-title = Training Range
help-training-range-body = Set the **lowest** and **highest note** for your training. A wider range is more challenging. If you're just starting out, try a smaller range and expand it as your ear improves.
help-intervals-title = Intervals
help-intervals-body = Intervals are the distance between two notes. Choose which intervals you want to practice. Start with a few and add more as you gain confidence.
help-sound-title = Sound
help-sound-body = Pick the **sound** you want to train with — each instrument has a different character.{"\u000A\u000A"}**Duration** controls how long each note plays.{"\u000A\u000A"}**Concert Pitch** sets the reference tuning. Most musicians use 440 Hz. Some orchestras tune to 442 Hz.{"\u000A\u000A"}**Tuning System** determines how intervals are calculated. Equal Temperament divides the octave into 12 equal steps and is standard for most Western music. Just Intonation uses pure frequency ratios and sounds smoother for some intervals.
help-difficulty-title = Difficulty
help-difficulty-body = **Vary Loudness** changes the volume of notes randomly. This makes training harder but more realistic — in real music, notes are rarely played at the same volume.
help-data-title = Data
help-data-body = **Export** saves your training data as a file you can keep as a backup or transfer to another device.{"\u000A\u000A"}**Import** loads training data from a file. You can replace your current data or merge it with existing records.{"\u000A\u000A"}**Reset** permanently deletes all training data and resets your profile. This cannot be undone.

## Help - Comparison
help-comparison-goal-title = Goal
help-comparison-goal-body = Two notes play one after the other. Your job is to decide: was the **second note higher or lower** than the first? The closer the notes are, the harder it gets — and the sharper your ear becomes.
help-comparison-controls-title = Controls
help-comparison-controls-body = After both notes have played, the **Higher** and **Lower** buttons become active. Tap the one that matches what you heard. You can also use keyboard shortcuts: **Arrow Up** or **H** for higher, **Arrow Down** or **L** for lower.
help-comparison-feedback-title = Feedback
help-comparison-feedback-body = After each answer you'll see a brief **checkmark** (correct) or **X** (incorrect). Use this to calibrate your listening — over time, you'll notice patterns in what you get right.
help-comparison-difficulty-title = Difficulty
help-comparison-difficulty-body = The difference between the two notes is measured in **cents** (1/100 of a semitone). A smaller number means a harder comparison. The app adapts difficulty to your skill level automatically.
help-comparison-intervals-title = Intervals
help-comparison-intervals-body = In interval mode, the two notes are separated by a specific **musical interval** (like a fifth or an octave) instead of a small pitch difference. You still decide which note is higher — but now you're training your sense of musical distance.

## Help - Pitch Matching
help-matching-goal-title = Goal
help-matching-goal-body = You'll hear a **reference note**. Your goal is to match its pitch by sliding to the exact same frequency. The closer you get, the better your ear is becoming.
help-matching-controls-title = Controls
help-matching-controls-body = **Touch** the slider to hear your note, then **drag** up or down to adjust the pitch. When you think you've matched the reference, **release** the slider to lock in your answer. You can also press **Enter** or **Space** to commit.
help-matching-feedback-title = Feedback
help-matching-feedback-body = After each attempt, you'll see how many **cents** off you were. A smaller number means a closer match — zero would be perfect. Use the feedback to fine-tune your listening.
help-matching-intervals-title = Intervals
help-matching-intervals-body = In interval mode, your target pitch is a specific **musical interval** away from the reference note. Instead of matching the same note, you're matching a note that's a certain distance above or below it.

## Help - Info
help-info-what-title = What is Peach?
help-info-what-body = Peach helps you train your ear for music. Practice hearing the difference between notes and learn to match pitches accurately.
help-info-modes-title = Training Modes
help-info-modes-body = **Hear & Compare — Single Notes** — Listen to two notes and decide which one is higher.{"\u000A\u000A"}**Hear & Compare — Intervals** — The same idea, but with musical intervals between notes.{"\u000A\u000A"}**Tune & Match — Single Notes** — Hear a note and slide to match its pitch.{"\u000A\u000A"}**Tune & Match — Intervals** — Match pitches using musical intervals.
help-info-start-title = Getting Started
help-info-start-body = Just pick any training mode on the home screen and start practicing. Peach adapts to your skill level automatically.

## Domain - Training Modes
training-mode-hear-compare-single = Hear & Compare — Single Notes
training-mode-hear-compare-intervals = Hear & Compare — Intervals
training-mode-tune-match-single = Tune & Match — Single Notes
training-mode-tune-match-intervals = Tune & Match — Intervals

## Domain - Intervals
interval-prime = Prime
interval-minor-second = Minor Second
interval-major-second = Major Second
interval-minor-third = Minor Third
interval-major-third = Major Third
interval-perfect-fourth = Perfect Fourth
interval-tritone = Tritone
interval-perfect-fifth = Perfect Fifth
interval-minor-sixth = Minor Sixth
interval-major-sixth = Major Sixth
interval-minor-seventh = Minor Seventh
interval-major-seventh = Major Seventh
interval-octave = Octave

## Domain - Direction
direction-up = Up
direction-down = Down
