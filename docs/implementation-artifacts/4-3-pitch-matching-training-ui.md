# Story 4.3: Pitch Matching Training UI

Status: ready-for-dev

## Story

As a musician,
I want to start pitch matching from the start page, tune notes by ear with real-time audio feedback, and see how close I was after each attempt,
so that I can develop my pitch matching ability through deliberate practice.

## Acceptance Criteria

1. **Given** I am on the Start Page **When** I click the "Pitch Matching" button (FR9) **Then** I navigate to `/training/pitch-matching`, the AudioContext activates, and the first reference note plays immediately.

2. **Given** state is `PlayingReference` **When** the reference note is playing **Then** the slider is visible but dimmed/disabled (FR11).

3. **Given** state is `AwaitingSliderTouch` or `PlayingTunable` **When** the tunable note is playing **Then** the slider is active and I can drag it, and the pitch changes in real time as I drag — no visual proximity feedback, ear only (FR12, FR33).

4. **Given** I release the slider **When** feedback displays (FR14) **Then** I see a directional arrow (up for sharp, down for flat) or a dot (dead center), the signed cent offset (e.g. "+4 cents", "-22 cents"), and color indicates proximity: green (<10 cents), yellow (10-30 cents), red (>30 cents). Feedback persists for ~400ms.

5. **Given** I am in Pitch Matching Training **When** I press Enter or Space (FR45) **Then** the pitch is committed (same as releasing the slider).

6. **Given** I am in Pitch Matching Training **When** I press Escape or click Settings/Profile **Then** training stops and I navigate away (FR15), incomplete attempt is discarded.

7. **Given** the UIObserver bridge **When** PitchMatchingSession state changes **Then** corresponding Leptos signals update and UI re-renders.

8. **Given** the Pitch Matching Training view **When** I inspect the HTML **Then** an `aria-live="polite"` region announces feedback (e.g. "4 cents sharp", "Dead center").

## Tasks / Subtasks

- [ ] Task 1: Add PitchMatchingObserver implementations to bridge.rs (AC: #7)
  - [ ] 1.1 Add `PitchMatchingProfileObserver` implementing `PitchMatchingObserver` — calls `profile.update_matching()`
  - [ ] 1.2 Add `PitchMatchingDataStoreObserver` implementing `PitchMatchingObserver` — saves `PitchMatchingRecord` to IndexedDB via `spawn_local`
- [ ] Task 2: Implement PitchMatchingView component in pitch_matching_view.rs (AC: #1, #2, #3, #6, #7)
  - [ ] 2.1 Set up context retrieval (profile, audio_ctx, db_store) — same pattern as ComparisonView
  - [ ] 2.2 Eagerly create AudioContext in synchronous render path (Safari gesture requirement)
  - [ ] 2.3 Create OscillatorNotePlayer and observers vector
  - [ ] 2.4 Create PitchMatchingSession with observers
  - [ ] 2.5 Create UI signals: slider_enabled, show_feedback, feedback_text, feedback_color, feedback_arrow, sr_announcement, storage_error, reset_trigger
  - [ ] 2.6 Implement sync_session_to_signals function
  - [ ] 2.7 Wire VerticalPitchSlider on_change → session.adjust_pitch() → handle.adjust_frequency()
  - [ ] 2.8 Wire VerticalPitchSlider on_commit → session.commit_pitch()
- [ ] Task 3: Implement async training loop (AC: #1, #2, #3, #4)
  - [ ] 3.1 Start session with intervals from settings
  - [ ] 3.2 PlayingReference phase: play reference note for duration, slider disabled
  - [ ] 3.3 AwaitingSliderTouch → PlayingTunable: play indefinite tunable note, slider enabled
  - [ ] 3.4 Handle slider interaction: on_change calls adjust_pitch, gets frequency, calls handle.adjust_frequency()
  - [ ] 3.5 Handle commit: stop tunable note, show feedback for 400ms
  - [ ] 3.6 Loop back to next reference note
- [ ] Task 4: Implement feedback display (AC: #4, #8)
  - [ ] 4.1 Compute directional arrow: up arrow for positive error (sharp), down arrow for negative (flat), dot for dead center (|error| < 1 cent)
  - [ ] 4.2 Format signed cent offset text: "+4 cents", "-22 cents", "Dead center"
  - [ ] 4.3 Compute color class: green (<10 cents), yellow (10-30 cents), red (>30 cents)
  - [ ] 4.4 Screen reader announcement via aria-live region
- [ ] Task 5: Implement interruption handling (AC: #5, #6)
  - [ ] 5.1 Register document-level keydown handler: Escape → stop and navigate home
  - [ ] 5.2 Register visibilitychange handler: tab hidden → stop and navigate home
  - [ ] 5.3 Register AudioContext statechange handler: suspended/closed → stop and navigate home
  - [ ] 5.4 Navigation link handlers for Settings/Profile: stop session, then navigate
  - [ ] 5.5 on_cleanup: cancel loop, stop session, stop audio, remove event listeners
- [ ] Task 6: Update start page button (AC: #1)
  - [ ] 6.1 Change "Pitch Matching" from `<A>` link to `<button>` with `on:click` navigate (matches Comparison pattern for gesture-based AudioContext creation)

## Dev Notes

### Architecture Pattern — Follow ComparisonView Exactly

This story replaces the placeholder `pitch_matching_view.rs` with a full implementation. **Follow `comparison_view.rs` as the structural template** — the pitch matching loop is architecturally identical but with different domain types and UI elements.

### Key Differences from ComparisonView

| Aspect | ComparisonView | PitchMatchingView |
|--------|---------------|-------------------|
| Session type | `ComparisonSession` | `PitchMatchingSession` |
| Observer trait | `ComparisonObserver` | `PitchMatchingObserver` |
| User input | Higher/Lower buttons | `VerticalPitchSlider` component |
| Audio during input | Fixed-duration target note | Indefinite tunable note (use `play()` not `play_for_duration()`) |
| Real-time audio | None | `handle.adjust_frequency()` on each slider change |
| Feedback | Thumbs up/down | Arrow + signed cent offset + color |
| Answer timing | During or after target note | User-driven (release slider or Enter/Space) |

### Domain API for Pitch Matching

```rust
// Import from domain crate
use domain::{PitchMatchingSession, PitchMatchingSessionState, PitchMatchingPlaybackData, FEEDBACK_DURATION_SECS};
use domain::ports::{PitchMatchingObserver, NotePlayer, PlaybackHandle};
use domain::records::PitchMatchingRecord;

// Session lifecycle
session.start(intervals, &settings);        // → PlayingReference
session.on_reference_finished();            // → AwaitingSliderTouch
session.adjust_pitch(value) -> Option<Frequency>  // AwaitingSliderTouch → PlayingTunable (first call), returns freq
session.commit_pitch(value, timestamp);     // PlayingTunable → ShowingFeedback
session.on_feedback_finished();             // → PlayingReference (loop)
session.stop();                             // → Idle

// Feedback data
session.last_completed() -> Option<&CompletedPitchMatching>
  .user_cent_error()  // f64, signed: positive = sharp, negative = flat
```

### Async Training Loop Structure

```
loop {
    1. Get playback data: session.current_playback_data()
    2. PlayingReference: play_for_duration(reference_freq, duration)
       - Slider disabled (slider_enabled = false)
       - Poll with POLL_INTERVAL_MS, check cancelled
    3. on_reference_finished() → AwaitingSliderTouch
       - Start tunable note: note_player.play(tunable_freq) → store handle
       - Enable slider (slider_enabled = true, reset_trigger++)
    4. Wait for commit (slider release or Enter/Space)
       - on_change: session.adjust_pitch(value) → handle.adjust_frequency(freq)
       - on_commit: session.commit_pitch(value, timestamp), handle.stop()
    5. ShowingFeedback: display arrow + cents for 400ms
       - Update feedback signals from session.last_completed()
    6. on_feedback_finished() → loop back to step 1
}
```

### Slider Integration

The `VerticalPitchSlider` component (from story 4.2) is already built at `web/src/components/pitch_slider.rs`.

```rust
// Props to pass
VerticalPitchSlider(
    enabled: slider_enabled.into(),     // Signal<bool>
    on_change: Callback::new(|value| { /* adjust_pitch + adjust_frequency */ }),
    on_commit: Callback::new(|value| { /* commit_pitch */ }),
    reset_trigger: reset_trigger.into(), // Signal<u32> — increment on each new challenge
)
```

**Critical:** The slider's `on_change` fires continuously during drag. Each call must:
1. Call `session.adjust_pitch(value)` → returns `Option<Frequency>`
2. If `Some(freq)`, call `tunable_handle.adjust_frequency(freq)`

The slider's `on_commit` fires on release (or Enter/Space). It must:
1. Call `session.commit_pitch(value, timestamp)`
2. Stop the tunable note handle

### Tunable Note Handle Management

The tunable note plays indefinitely (no duration). Use `note_player.play()` (not `play_for_duration()`):

```rust
// Start tunable note — returns handle for frequency adjustment
let handle = note_player.borrow().play(
    data.tunable_frequency,
    MIDIVelocity::new(PITCH_MATCHING_VELOCITY),
    AmplitudeDB::new(0.0),
)?;
// Store handle in Rc<RefCell<Option<OscillatorPlaybackHandle>>>
tunable_handle.replace(Some(handle));

// On slider change → adjust frequency
if let Some(freq) = session.borrow_mut().adjust_pitch(value) {
    if let Some(ref mut h) = *tunable_handle.borrow_mut() {
        let _ = h.adjust_frequency(freq);
    }
}

// On commit → stop tunable note
if let Some(ref mut h) = *tunable_handle.borrow_mut() {
    h.stop();
}
```

### Feedback Display

The feedback indicator shows 3 elements for ~400ms after each commit:

1. **Arrow**: `"\u{2191}"` (↑) if error > 0 (sharp), `"\u{2193}"` (↓) if error < 0 (flat), `"\u{00B7}"` (·) if |error| < 1 cent (dead center)
2. **Text**: `"+4 cents"`, `"-22 cents"`, or `"Dead center"`
3. **Color**: `text-green-600` if |error| < 10, `text-yellow-600` if 10-30, `text-red-600` if > 30

Access via `session.last_completed().user_cent_error()` — a signed f64 in cents.

### Observer Implementations (bridge.rs)

Add two new observer structs implementing `PitchMatchingObserver`:

```rust
// PitchMatchingProfileObserver — updates perceptual profile
impl PitchMatchingObserver for PitchMatchingProfileObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching) {
        let mut profile = self.0.borrow_mut();
        profile.update_matching(
            completed.reference_note(),
            completed.user_cent_error().abs(),
        );
    }
}

// PitchMatchingDataStoreObserver — persists to IndexedDB
impl PitchMatchingObserver for PitchMatchingDataStoreObserver {
    fn pitch_matching_completed(&mut self, completed: &CompletedPitchMatching) {
        let record = PitchMatchingRecord::from_completed(completed);
        // spawn_local async save, same pattern as DataStoreObserver
    }
}
```

### Start Page Button Fix

The "Pitch Matching" link on the start page currently uses `<A href=...>` (a Leptos router link). This must be changed to a `<button on:click=...>` that calls `navigate()` — matching the Comparison button pattern. The reason: AudioContext creation requires a user gesture. When using `<A>`, the navigation happens without a click event in the call stack, so Safari/iOS may reject AudioContext creation. The `<button on:click>` → `navigate()` pattern ensures the gesture propagates.

### Project Structure Notes

Files to modify:
- `web/src/components/pitch_matching_view.rs` — **replace** placeholder with full implementation
- `web/src/bridge.rs` — **add** PitchMatchingProfileObserver and PitchMatchingDataStoreObserver
- `web/src/components/start_page.rs` — **change** Pitch Matching `<A>` to `<button>`

Files to read (not modify):
- `web/src/components/comparison_view.rs` — structural reference
- `web/src/components/pitch_slider.rs` — component API reference
- `web/src/adapters/audio_oscillator.rs` — `play()` and `adjust_frequency()` API
- `domain/src/session/pitch_matching_session.rs` — session state machine API

No new files needed. No Cargo.toml changes needed (all web-sys features already enabled).

### Testing

- Domain: 291+ tests already pass (story 4.1 covered domain logic)
- Web: Manual browser testing — verify training loop, slider interaction, feedback display, interruption handling
- Run `cargo test -p domain` to confirm no regressions
- Run `trunk build` to confirm WASM compilation succeeds
- Verify `cargo clippy` produces zero warnings (dead-code warnings from story 4.2 should now resolve since VerticalPitchSlider is used)

### References

- [Source: docs/planning-artifacts/epics.md#Epic 4, Story 4.3]
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture, Leptos Frontend]
- [Source: docs/planning-artifacts/ux-design-specification.md#Pitch Matching Loop]
- [Source: docs/project-context.md#Leptos Rules, Observer Contract, Audio Edge Cases]
- [Source: web/src/components/comparison_view.rs — structural template]
- [Source: web/src/components/pitch_slider.rs — VerticalPitchSlider API]
- [Source: web/src/adapters/audio_oscillator.rs — play() and adjust_frequency()]
- [Source: domain/src/session/pitch_matching_session.rs — PitchMatchingSession API]
- [Source: docs/implementation-artifacts/4-1-pitch-matching-session-state-machine.md — domain patterns]
- [Source: docs/implementation-artifacts/4-2-vertical-pitch-slider.md — slider component patterns]

## Previous Story Intelligence

### From Story 4.1 (Pitch Matching Session State Machine)
- Pure domain logic — web layer calls domain methods and applies frequencies
- `adjust_pitch(value) -> Option<Frequency>` returns calculated frequencies for web to apply
- Frequency formula: `target_freq * 2^((value * 20) / 1200)` maps slider [-1, +1] to ±20 cents
- Observer notification with panic isolation via `catch_unwind`
- Code review fixes: panic guard for narrow note ranges, clamp slider values to [-1, +1]
- 2 additional tests added for narrow range edge cases

### From Story 4.2 (Vertical Pitch Slider)
- Used `Callback<f64>` for idiomatic Leptos component callback props
- Pointer capture via `set_pointer_capture` for unified mouse/touch handling
- `touch-action: none` CSS required for proper touch drag behavior
- Added web-sys features: DomRect, Element, MouseEvent, PointerEvent (already in Cargo.toml)
- Code review: added `on:pointercancel` handler, enabled guards on move/up, formatted aria-valuenow to 2 decimals
- Dead-code warnings expected until this story integrates the component
- Auto-resets to center on enabled false→true transition

### From Story 4.2 Code Review
- (H1) Log errors instead of silently ignoring pointer capture failures
- (M1) Handle `pointercancel` events (browser may cancel capture)
- (M2) Guard pointermove/pointerup with enabled check
- (M3) Format aria-valuenow to 2 decimal places for screen readers

## Git Intelligence

Recent commit pattern (newest first):
```
6de4315 Apply code review fixes for story 4.2 and mark as done
7c532fd Implement story 4.2 Vertical Pitch Slider
7ac5869 Add story 4.2 Vertical Pitch Slider and mark as ready-for-dev
c0f028d Apply code review fixes for story 4.1 and mark as done
a040bf5 Implement story 4.1 Pitch Matching Session State Machine
```

Conventions: implementation commit first, then code review fixes as separate commit. Story creation is a separate commit before implementation.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
