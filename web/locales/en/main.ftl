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
rhythm = Rhythm
compare = Compare
match = Match
compare-timing = Compare Timing
fill-the-gap = Fill the Gap
compare-pitch-aria = Compare Pitch
match-pitch-aria = Match Pitch
compare-intervals-aria = Compare Intervals
match-intervals-aria = Match Intervals
compare-timing-aria = Compare Timing
fill-the-gap-aria = Fill the Gap
rhythm-offset-description = Listen to four clicks. One of them is slightly off — decide whether it came early or late.
continuous-rhythm-description = A steady beat plays with a gap in it. Tap at the right moment to fill the missing beat.
coming-soon = Coming soon

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
pitch-range = Training Range
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
tuning-label = Tuning System
equal-temperament = Equal Temperament
just-intonation = Just Intonation
tuning-hint = Select how intervals are tuned. Equal Temperament divides the octave into 12 equal steps. Just Intonation uses pure frequency ratios.
difficulty-section = Difficulty
loudness-variation = Vary Loudness
loudness-variation-aria = Loudness variation
note-gap-label = Note Gap (Compare): { $value }s
note-gap-aria = Note gap for pitch discrimination
off = Off
max = Max
rhythm-section = Rhythm
tempo-label = Tempo: { $value } BPM
tempo-aria = Tempo in beats per minute
gap-positions-label = Gap Positions
gap-positions-hint = Select which subdivisions are used in Fill the Gap training.
gap-position-beat = Beat
gap-position-e = E
gap-position-and = And
gap-position-a = A
data-section = Data
export-training-data = Export Training Data
exporting = Exporting…
exported = Exported!
import-training-data = Import Training Data
importing = Importing…
delete-all-training-data = Reset Training Data
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
import-dialog-found = Found { $discriminations } discrimination records and { $matchings } pitch matching records.{ $warnings } How would you like to import?
import-dialog-warnings = { " " }({ $count } rows skipped with warnings)
replace-all-data = Replace All Data
merge-with-existing = Merge with Existing
reset-dialog-title = Reset All Training Data
reset-dialog-message = This will permanently delete all training data and reset your profile. This cannot be undone.
delete-all-data = Reset
database-not-available = Database not available
language-label = Language

## Pitch Discrimination Training
discrimination-title = Compare
discrimination-help-title = Pitch Discrimination Training

## Pitch Matching
matching-title = Match
matching-help-title = Pitch Matching Training

## Profile
profile-title = Profile
no-training-data = No training data yet. Start training to build your profile.
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
acknowledgments-body = Piano sounds from <a href="https://member.keymusician.com/Member/FluidR3_GM/index.html" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">FluidR3_GM by Frank Wen</a> (MIT License). All other sounds from <a href="https://schristiancollins.com/generaluser.php" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">GeneralUser GS by S. Christian Collins</a>.

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
help-difficulty-body = **Vary Loudness** changes the volume of notes randomly. This makes training harder but more realistic — in real music, notes are rarely played at the same volume.{"\u000A\u000A"}**Note Gap** adds a pause between the two notes in Compare training. At zero, notes play back-to-back.
help-rhythm-title = Rhythm
help-rhythm-body = **Tempo** sets the speed for rhythm training in beats per minute (BPM). A lower tempo is easier.{"\u000A\u000A"}**Gap Positions** control which subdivisions of the beat can have a gap in Fill the Gap training. "Beat" is the downbeat, "E", "And", and "A" are the subdivisions in between.
help-data-title = Data
help-data-body = **Export** saves your training data as a file you can keep as a backup or transfer to another device.{"\u000A\u000A"}**Import** loads training data from a file. You can replace your current data or merge it with existing records.{"\u000A\u000A"}**Reset** permanently deletes all training data and resets your profile. This cannot be undone.

## Help - Pitch Discrimination
help-discrimination-goal-title = Goal
help-discrimination-goal-body = Two notes play one after the other. Your job is to decide: was the **second note higher or lower** than the first? The closer the notes are, the harder it gets — and the sharper your ear becomes.
help-discrimination-controls-title = Controls
help-discrimination-controls-body = Once the second note starts playing, the **Higher** and **Lower** buttons become active. Tap the one that matches what you heard. You can also use keyboard shortcuts: **Arrow Up** or **H** for higher, **Arrow Down** or **L** for lower.
help-discrimination-feedback-title = Feedback
help-discrimination-feedback-body = After each answer you'll see a brief **checkmark** (correct) or **X** (incorrect). Use this to calibrate your listening — over time, you'll notice patterns in what you get right.
help-discrimination-difficulty-title = Difficulty
help-discrimination-difficulty-body = The number at the top shows the **cent difference** between the two notes – a smaller number means a harder comparison. Your **session best** tracks the smallest difference you answered correctly.
help-discrimination-intervals-title = Intervals
help-discrimination-intervals-body = In interval mode, the two notes are separated by a specific **musical interval** (like a fifth or an octave) instead of a small pitch difference. You still decide which note is higher — but now you're training your sense of musical distance.

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
help-info-modes-body = **Compare – Single Notes** – Listen to two notes and decide which one is higher.{"\u000A\u000A"}**Compare – Intervals** – The same idea, but with musical intervals between notes.{"\u000A\u000A"}**Match – Single Notes** – Hear a note and slide to match its pitch.{"\u000A\u000A"}**Match – Intervals** – Match pitches using musical intervals.
help-info-start-title = Getting Started
help-info-start-body = Just pick any training mode on the home screen and start practicing. Peach adapts to your skill level automatically.

## Help - Profile
profile-help-title = Chart Help
help-profile-chart-title = Your Progress Chart
help-profile-chart-body = This chart shows how your pitch perception is developing over time.
help-profile-trend-title = Trend Line
help-profile-trend-body = The blue line shows your smoothed average — it filters out random ups and downs to reveal your real progress.
help-profile-band-title = Variability Band
help-profile-band-body = The shaded area around the line shows how consistent you are — a narrower band means more reliable results.
help-profile-baseline-title = Target Baseline
help-profile-baseline-body = The green dashed line is your goal — as the trend line approaches it, your ear is getting sharper.
help-profile-zones-title = Time Zones
help-profile-zones-body = The chart groups your data by time: months on the left, recent days in the middle, and today's sessions on the right.

## Chart Annotations
chart-annotation-records = { $count } records
chart-annotation-summary = { $date } — { $mean } { $unit }, ±{ $stddev }, { $count } records

## Domain - Training Modes
training-discipline-hear-discriminate-single = Compare – Single Notes
training-discipline-hear-discriminate-intervals = Compare – Intervals
training-discipline-tune-match-single = Match – Single Notes
training-discipline-tune-match-intervals = Match – Intervals
training-mode-compare-timing = Compare Timing
training-mode-fill-the-gap = Fill the Gap

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
