# Story 1.7: Comparison Training UI

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to click "Comparison," hear two notes, answer higher/lower via click or keyboard, see brief feedback, and keep training,
so that I can train my pitch discrimination through the core reflexive loop.

## Acceptance Criteria

1. **AC1 — Start from Start Page:** Given I am on the Start Page, when I click the "Comparison" button, then I navigate to the Comparison Training view, the AudioContext activates (FR35), and the first comparison begins immediately — no countdown, no delay (FR1).

2. **AC2 — Keyboard start:** Given I am on the Start Page, when I press Enter or Space with the Comparison button focused, then training starts identically to clicking (FR42).

3. **AC3 — Note1 buttons disabled:** Given state is `playingNote1`, when the reference note is playing, then Higher and Lower buttons are visually disabled and not clickable/pressable.

4. **AC4 — Note2 early answer:** Given state is `playingNote2`, when the target note begins playing, then Higher and Lower buttons become enabled immediately (FR4 — early answer).

5. **AC5 — Answer higher:** Given buttons are enabled, when I click "Higher" or press Arrow Up / H, then my answer is registered as "higher" and both buttons disable immediately (FR41).

6. **AC6 — Answer lower:** Given buttons are enabled, when I click "Lower" or press Arrow Down / L, then my answer is registered as "lower" and both buttons disable immediately (FR41).

7. **AC7 — Correct feedback:** Given a correct answer, when feedback displays, then a green thumbs-up indicator appears for ~400ms (FR5).

8. **AC8 — Incorrect feedback:** Given an incorrect answer, when feedback displays, then a red thumbs-down indicator appears for ~400ms with same visual weight, position, and duration as correct feedback (FR5).

9. **AC9 — Feedback to next:** Given feedback is showing, when ~400ms elapses, then the indicator disappears and the next comparison begins automatically.

10. **AC10 — Escape stops:** Given I am in Comparison Training, when I press Escape, then training stops and I return to the Start Page (FR43).

11. **AC11 — Nav stops training:** Given I am in Comparison Training, when I click Settings or Profile links, then training stops and I navigate to the selected view (FR6).

12. **AC12 — UIObserver bridge:** Given the UIObserver bridge is implemented, when domain session state changes, then corresponding Leptos signals update and UI re-renders affected elements.

13. **AC13 — Composition wiring:** Given the composition root, when the app starts, then shared dependencies are wired: profile, AudioContext manager, and adapters are available via Leptos context (blueprint §11).

14. **AC14 — Screen reader:** Given the Comparison Training view, when I inspect the HTML, then an `aria-live="polite"` region exists, and "Correct" or "Incorrect" is announced to screen readers after each answer.

## Tasks / Subtasks

- [x] Task 1: Add web crate dependencies (AC: 5,6,12)
  - [x] 1.1 Add `js-sys = "0.3"` to web/Cargo.toml for ISO 8601 timestamps
  - [x] 1.2 Add `web-sys` features: `KeyboardEvent`, `Document`, `EventTarget`, `Window`, `HtmlElement`
  - [x] 1.3 Verify `cargo check -p web` passes with new dependencies

- [x] Task 2: Create DefaultSettings adapter (AC: 1,13)
  - [x] 2.1 Create `web/src/adapters/default_settings.rs` implementing `UserSettings` trait with hardcoded defaults: note_range_min=36 (C2), note_range_max=84 (C6), note_duration=1.0s, reference_pitch=440Hz, tuning_system=EqualTemperament, vary_loudness=0.0
  - [x] 2.2 Add `pub mod default_settings;` to `web/src/adapters/mod.rs`

- [x] Task 3: Create ProfileObserver adapter (AC: 12,13)
  - [x] 3.1 Create `web/src/bridge.rs` with `ProfileObserver` struct wrapping `Rc<RefCell<PerceptualProfile>>`
  - [x] 3.2 Implement `ComparisonObserver` for `ProfileObserver`: on comparison_completed, call `profile.update(reference_note, cent_offset_abs, is_correct)`
  - [x] 3.3 Add `mod bridge;` to `web/src/main.rs`

- [x] Task 4: Set up shared context in App component (AC: 13)
  - [x] 4.1 In `web/src/app.rs`, create `Rc<RefCell<PerceptualProfile>>` (cold start, empty)
  - [x] 4.2 In `web/src/app.rs`, create `Rc<RefCell<AudioContextManager>>`
  - [x] 4.3 Provide both via `provide_context()` for child components
  - [x] 4.4 Verify all routes still render correctly

- [x] Task 5: Implement ComparisonView training loop and UI (AC: 1,3,4,5,6,7,8,9,12)
  - [x] 5.1 Rewrite `web/src/components/comparison_view.rs` — remove test button, implement full comparison training view
  - [x] 5.2 Retrieve shared profile and AudioContextManager from Leptos context
  - [x] 5.3 Create `OscillatorNotePlayer` from shared AudioContextManager
  - [x] 5.4 Create `DefaultSettings` adapter
  - [x] 5.5 Create `ProfileObserver` wrapping shared profile
  - [x] 5.6 Create `ComparisonSession` with profile, observers=[ProfileObserver], resettables=[], intervals=[Prime/Up for unison mode]
  - [x] 5.7 Create `RwSignal`s for UI state: `show_feedback`, `is_last_correct`, `buttons_enabled`, `sr_announcement`
  - [x] 5.8 Implement `sync_signals()` closure that reads session state and updates all signals
  - [x] 5.9 Implement async training loop via `wasm_bindgen_futures::spawn_local`: play note1 → wait → on_note1_finished → play note2 → wait → on_note2_finished (if still PlayingNote2) → wait for answer → wait feedback → on_feedback_finished → loop
  - [x] 5.10 Implement answer handler closure: guard on PlayingNote2/AwaitingAnswer states, call `handle_answer(is_higher, timestamp)`, sync signals, schedule `on_feedback_finished` via `gloo_timers::callback::Timeout` after 400ms
  - [x] 5.11 Implement Higher/Lower buttons: enabled/disabled based on `buttons_enabled` signal, call answer handler on click
  - [x] 5.12 Implement feedback indicator: conditional rendering based on `show_feedback` signal, green thumbs-up or red thumbs-down based on `is_last_correct` signal
  - [x] 5.13 Implement `on_cleanup` to set cancelled flag, stop session, stop all notes

- [x] Task 6: Implement keyboard shortcuts (AC: 5,6,10)
  - [x] 6.1 Add document-level `keydown` event listener in ComparisonView
  - [x] 6.2 Arrow Up / H → answer "higher" (same as button click)
  - [x] 6.3 Arrow Down / L → answer "lower" (same as button click)
  - [x] 6.4 Escape → set cancelled flag, stop session, navigate to "/"
  - [x] 6.5 Remove event listener in `on_cleanup`

- [x] Task 7: Implement navigation links that stop training (AC: 10,11)
  - [x] 7.1 Add Settings and Profile navigation links to ComparisonView
  - [x] 7.2 On click, set cancelled flag and stop session before navigation
  - [x] 7.3 Use `leptos_router` `A` component or programmatic navigation

- [x] Task 8: Implement screen reader accessibility (AC: 14)
  - [x] 8.1 Add `aria-live="polite"` region to ComparisonView
  - [x] 8.2 After each answer, update the live region text to "Correct" or "Incorrect"
  - [x] 8.3 Ensure buttons have accessible labels ("Higher" / "Lower")

- [x] Task 9: Verify and test (AC: all)
  - [x] 9.1 `cargo test -p domain` — all existing tests pass (no regressions)
  - [x] 9.2 `cargo clippy -p web` — zero warnings
  - [x] 9.3 `cargo clippy -p domain` — zero warnings
  - [x] 9.4 `trunk serve` — manual browser test: click Comparison → hear notes → answer → see feedback → loop continues
  - [x] 9.5 Test keyboard shortcuts: Arrow Up/H, Arrow Down/L, Escape
  - [x] 9.6 Test early answer during note2 playback
  - [x] 9.7 Test navigation to Settings/Profile stops training
  - [x] 9.8 Test returning to Start Page and starting again (profile accumulates in-memory)

## Dev Notes

### Core Architecture: Web-Layer Training Loop Driver

Story 1.6 implemented the domain-pure `ComparisonSession` state machine. This story (1.7) implements the **web-layer driver** that orchestrates the training loop: audio playback, timing, user input, and Leptos UI rendering.

The session is synchronous — it exposes event-driven transition methods. The web layer drives the async loop:

```
ComparisonSession (domain, sync)     Web Layer (async loop + UI)
─────────────────────────────────    ──────────────────────────────
start(intervals, &settings)       ← User clicks "Comparison" button
  → state = PlayingNote1             Play reference note via NotePlayer
                                     Sleep for note duration
on_note1_finished()               ← Sleep completes
  → state = PlayingNote2             Play target note via NotePlayer
                                     Enable Higher/Lower buttons
                                     Sleep for note duration
on_note2_finished()               ← Sleep completes (if no early answer)
  → state = AwaitingAnswer           Wait for user input
handle_answer(is_higher, ts)      ← User clicks/presses button
  → state = ShowingFeedback          Show feedback indicator
  → notifies observers               Schedule feedback timer (400ms)
on_feedback_finished()            ← Timer fires
  → generates next comparison        Loop back to PlayingNote1
  → state = PlayingNote1
stop()                            ← User presses Escape or navigates away
  → state = Idle                     Stop audio, clean up
```

### Async Training Loop Implementation

The training loop uses `wasm_bindgen_futures::spawn_local` with `gloo_timers::future::TimeoutFuture` for timing. The loop structure handles both normal and early-answer flows:

```rust
// Cancellation flag — shared between loop and event handlers
let cancelled = Rc::new(Cell::new(false));

spawn_local(async move {
    session.borrow_mut().start(intervals, &settings);
    sync_signals(&session.borrow());

    'training: loop {
        if cancelled.get() { break; }

        let data = session.borrow().current_playback_data().unwrap();

        // === PlayingNote1 phase (buttons disabled) ===
        if let Err(e) = note_player.borrow().play_for_duration(
            data.reference_frequency, data.duration,
            MIDIVelocity::new(63), AmplitudeDB::new(0.0),
        ) {
            log::error!("Note1 playback failed: {e}");
        }
        TimeoutFuture::new((data.duration.raw_value() * 1000.0) as u32).await;
        if cancelled.get() { break; }

        // Transition: PlayingNote1 → PlayingNote2
        session.borrow_mut().on_note1_finished();
        sync_signals(&session.borrow());

        // === PlayingNote2 phase (buttons enabled — early answer possible) ===
        if let Err(e) = note_player.borrow().play_for_duration(
            data.target_frequency, data.duration,
            MIDIVelocity::new(63), data.target_amplitude_db,
        ) {
            log::error!("Note2 playback failed: {e}");
        }
        TimeoutFuture::new((data.duration.raw_value() * 1000.0) as u32).await;
        if cancelled.get() { break; }

        // Only transition to AwaitingAnswer if no early answer was given
        if session.borrow().state() == ComparisonSessionState::PlayingNote2 {
            session.borrow_mut().on_note2_finished();
            sync_signals(&session.borrow());
        }

        // Wait for answer + feedback to complete
        // (Button handler calls handle_answer + schedules on_feedback_finished)
        while session.borrow().state() != ComparisonSessionState::PlayingNote1 {
            if cancelled.get() { break 'training; }
            TimeoutFuture::new(50).await; // Poll at ~20Hz
        }
        sync_signals(&session.borrow());
    }

    // Cleanup
    session.borrow_mut().stop();
    note_player.borrow().stop_all();
    sync_signals(&session.borrow());
});
```

### Answer Handler + Feedback Timer

The button click handler and keyboard handler both delegate to the same answer function. The handler is responsible for calling `handle_answer` AND scheduling the feedback timer:

```rust
let on_answer = {
    let session = Rc::clone(&session);
    let sync = sync_fn.clone();
    let cancelled = Rc::clone(&cancelled);
    move |is_higher: bool| {
        if cancelled.get() { return; }
        let state = session.borrow().state();
        if state != ComparisonSessionState::PlayingNote2
            && state != ComparisonSessionState::AwaitingAnswer
        {
            return; // Guard: only accept answers in these states
        }

        let timestamp = js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_default();
        session.borrow_mut().handle_answer(is_higher, timestamp);
        sync();

        // Schedule feedback end after 400ms
        let session_clone = Rc::clone(&session);
        let sync_clone = sync.clone();
        gloo_timers::callback::Timeout::new(400, move || {
            if session_clone.borrow().state() == ComparisonSessionState::ShowingFeedback {
                session_clone.borrow_mut().on_feedback_finished();
                sync_clone();
            }
        })
        .forget(); // Prevent cancellation on drop
    }
};
```

**Why `Timeout::new().forget()`:** The `Timeout` is single-use and self-cleaning. `.forget()` prevents the timeout from being cancelled when the closure drops. The timeout callback checks the session state before acting, so it's safe even if the session was stopped.

**Early Answer Timing:** When the user answers during note2 playback, the 400ms feedback timer starts immediately from the answer moment. The main loop's note2 sleep may still be running, but when it wakes up it will find the state has progressed (either ShowingFeedback or already PlayingNote1), and the polling loop handles it correctly.

### Signal Structure and sync_signals Pattern

Leptos `RwSignal`s bridge domain state to reactive UI:

```rust
let session_state = RwSignal::new(ComparisonSessionState::Idle);
let show_feedback = RwSignal::new(false);
let is_last_correct = RwSignal::new(false);
let buttons_enabled = RwSignal::new(false);
let sr_announcement = RwSignal::new(String::new()); // Screen reader
```

The `sync_signals` closure reads session state and pushes to all signals:

```rust
let sync_signals = {
    let session = Rc::clone(&session);
    move || {
        let s = session.borrow();
        session_state.set(s.state());
        show_feedback.set(s.show_feedback());
        is_last_correct.set(s.is_last_answer_correct());
        let state = s.state();
        buttons_enabled.set(
            state == ComparisonSessionState::PlayingNote2
                || state == ComparisonSessionState::AwaitingAnswer
        );
        if s.show_feedback() {
            sr_announcement.set(
                if s.is_last_answer_correct() { "Correct".into() }
                else { "Incorrect".into() }
            );
        }
    }
};
```

**Signal setters happen ONLY in sync_signals and the answer handler** — components are read-only consumers. This follows the architecture's UIObserver bridge pattern.

### Keyboard Event Handling

Register a document-level `keydown` listener using `wasm_bindgen::closure::Closure`:

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

let keydown_handler = {
    let on_answer = on_answer.clone();
    let cancelled = Rc::clone(&cancelled);
    let session = Rc::clone(&session);
    let note_player = Rc::clone(&note_player);
    let sync = sync_fn.clone();
    Closure::<dyn Fn(KeyboardEvent)>::new(move |ev: KeyboardEvent| {
        match ev.key().as_str() {
            "ArrowUp" | "h" | "H" => {
                ev.prevent_default();
                on_answer(true);
            }
            "ArrowDown" | "l" | "L" => {
                ev.prevent_default();
                on_answer(false);
            }
            "Escape" => {
                ev.prevent_default();
                cancelled.set(true);
                session.borrow_mut().stop();
                note_player.borrow().stop_all();
                sync();
                // Navigate to start page
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/", Default::default());
            }
            _ => {}
        }
    })
};

// Register on document
let document = web_sys::window().unwrap().document().unwrap();
document
    .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())
    .unwrap();

// Store for cleanup (do NOT call .forget() — we need to remove it)
// Store in a local variable that persists for the component's lifetime
```

**Cleanup:** Use `on_cleanup` to remove the event listener when the component unmounts:

```rust
on_cleanup(move || {
    let document = web_sys::window().unwrap().document().unwrap();
    let _ = document.remove_event_listener_with_callback(
        "keydown",
        keydown_handler.as_ref().unchecked_ref(),
    );
    // Also ensure training stops
    cancelled.set(true);
    session.borrow_mut().stop();
    note_player.borrow().stop_all();
});
```

**Important:** The `keydown_handler` `Closure` must remain alive for the component's entire lifetime. Store it in a way that prevents premature drop. Use `StoredValue` or `store_value` from Leptos to hold the Closure, or use `std::mem::forget` paired with manual removal in cleanup. The preferred pattern is to use `leptos::StoredValue` to keep the Closure alive without leaking:

```rust
let _keydown_closure = StoredValue::new(keydown_handler);
```

**Note on `use_navigate()`:** In Leptos 0.8, `use_navigate()` must be called within the component's reactive scope (not inside the Closure). Call it once at the top of the component and clone it into the closure.

### Navigation Links That Stop Training

Settings and Profile links must stop training before navigating. Use click handlers that set the cancelled flag:

```rust
let on_nav_away = {
    let cancelled = Rc::clone(&cancelled);
    let session = Rc::clone(&session);
    let note_player = Rc::clone(&note_player);
    let sync = sync_fn.clone();
    move || {
        cancelled.set(true);
        session.borrow_mut().stop();
        note_player.borrow().stop_all();
        sync();
    }
};
```

Then in the view:
```rust
<a href="/settings"
    on:click=move |_| on_nav_away()
    class="...">
    "Settings"
</a>
<a href="/profile"
    on:click=move |_| on_nav_away()
    class="...">
    "Profile"
</a>
```

**Note:** Using `<a href="...">` with Leptos Router's client-side routing: the `A` component from `leptos_router::components::A` handles client-side navigation. Adding an `on:click` handler to a regular `<a>` tag may not work as expected with Leptos Router. Instead, use programmatic navigation via `use_navigate()` inside the click handler, and prevent default link behavior.

### Composition Root Wiring (Simplified for Story 1.7)

For story 1.7, the composition root is simplified — no persistence (story 1.8), no SoundFont (later), no trend analyzer/timeline (later). The wiring is:

**App component (web/src/app.rs):**
```rust
// Shared across all routes
let profile = Rc::new(RefCell::new(PerceptualProfile::new()));
let audio_ctx_manager = Rc::new(RefCell::new(AudioContextManager::new()));
provide_context(profile);
provide_context(audio_ctx_manager);
```

**ComparisonView component:**
```rust
// Retrieve from context
let profile: Rc<RefCell<PerceptualProfile>> = use_context().expect("PerceptualProfile not provided");
let audio_ctx: Rc<RefCell<AudioContextManager>> = use_context().expect("AudioContextManager not provided");

// Create session-local components
let settings = DefaultSettings;
let note_player = Rc::new(RefCell::new(OscillatorNotePlayer::new(audio_ctx)));
let profile_observer = ProfileObserver(Rc::clone(&profile));

let session = Rc::new(RefCell::new(ComparisonSession::new(
    Rc::clone(&profile),
    vec![Box::new(profile_observer)],
    vec![], // No resettables yet — story 1.8 adds persistence
)));
```

**Why profile is in App, not ComparisonView:** The profile must persist across navigation within a single page load. If the user trains, navigates to Settings, returns and trains again, the adaptive algorithm should use the accumulated profile data. Creating the profile in App ensures it survives component unmount/remount.

**Why AudioContextManager is in App:** AudioContext should be a singleton per browser tab. Sharing it via context prevents creating multiple AudioContexts.

### DefaultSettings Adapter

```rust
// web/src/adapters/default_settings.rs
use domain::ports::UserSettings;
use domain::types::{Frequency, MIDINote, NoteDuration};
use domain::TuningSystem;

pub struct DefaultSettings;

impl UserSettings for DefaultSettings {
    fn note_range_min(&self) -> MIDINote { MIDINote::new(36) }  // C2
    fn note_range_max(&self) -> MIDINote { MIDINote::new(84) }  // C6
    fn note_duration(&self) -> NoteDuration { NoteDuration::new(1.0) }
    fn reference_pitch(&self) -> Frequency { Frequency::CONCERT_440 }
    fn tuning_system(&self) -> TuningSystem { TuningSystem::EqualTemperament }
    fn vary_loudness(&self) -> f64 { 0.0 }
}
```

This will be replaced by `LocalStorageSettings` in story 1.8. The default values match the PRD defaults: C2-C6 range, 1 second notes, A440, equal temperament, no loudness variation.

### ProfileObserver Bridge

```rust
// web/src/bridge.rs
use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::ComparisonObserver;
use domain::training::CompletedComparison;
use domain::PerceptualProfile;

pub struct ProfileObserver(pub Rc<RefCell<PerceptualProfile>>);

impl ComparisonObserver for ProfileObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let mut profile = self.0.borrow_mut();
        let cent_offset = completed.comparison().target_note().offset.raw_value.abs();
        profile.update(
            completed.comparison().reference_note(),
            cent_offset,
            completed.is_correct(),
        );
    }
}
```

**This is the minimal bridge for story 1.7.** Future stories will add:
- `DataStoreObserver` (story 1.8) — persists ComparisonRecord to IndexedDB
- `TrendAnalyzerObserver` — updates TrendAnalyzer on each comparison
- `ThresholdTimelineObserver` — updates ThresholdTimeline

### Unison Mode Intervals

For story 1.7, comparison training runs in **unison mode** only (interval mode is story 2.2). Unison mode uses a single interval:

```rust
use domain::types::{Direction, DirectedInterval, Interval};

let intervals = vec![DirectedInterval::new(Interval::Prime, Direction::Up)];
session.borrow_mut().start(intervals, &settings);
```

With `Interval::Prime` (0 semitones), the adaptive algorithm selects a single MIDI note as both reference and target, applying a cent offset to the target. The user is comparing two renditions of essentially the same note with a small pitch difference.

### Feedback Indicator Component

The feedback indicator is a simple show/hide element with conditional styling:

```rust
// In the view
<div
    class="flex items-center justify-center h-16"
    aria-hidden="true"
>
    {move || {
        if show_feedback.get() {
            if is_last_correct.get() {
                view! { <span class="text-4xl text-green-600 dark:text-green-400">"👍"</span> }.into_any()
            } else {
                view! { <span class="text-4xl text-red-600 dark:text-red-400">"👎"</span> }.into_any()
            }
        } else {
            view! { <span></span> }.into_any()
        }
    }}
</div>

// Screen reader live region (visually hidden)
<div
    aria-live="polite"
    class="sr-only"
>
    {move || sr_announcement.get()}
</div>
```

**UX requirements for feedback:**
- Same visual weight, position, and duration for correct and incorrect
- ~400ms display duration (handled by the Timeout in the answer handler)
- Green thumbs-up / red thumbs-down (Unicode emoji or SVG — emoji is simplest for MVP)
- No animation required (simple show/hide). Optional subtle fade acceptable per UX spec.
- `aria-hidden="true"` on the visual indicator; screen reader gets the text announcement separately

### Button States and Styling

Higher/Lower buttons follow the UX spec button behavior pattern:

```rust
<div class="flex gap-4 justify-center mt-8">
    <button
        on:click=move |_| on_answer_higher()
        disabled=move || !buttons_enabled.get()
        class=move || if buttons_enabled.get() {
            "min-h-11 min-w-[120px] rounded-lg bg-indigo-600 px-6 py-4 text-lg font-semibold text-white shadow-md hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-400"
        } else {
            "min-h-11 min-w-[120px] rounded-lg bg-gray-300 px-6 py-4 text-lg font-semibold text-gray-500 cursor-not-allowed dark:bg-gray-700 dark:text-gray-500"
        }
        aria-label="Higher"
    >
        "Higher"
    </button>
    <button
        on:click=move |_| on_answer_lower()
        disabled=move || !buttons_enabled.get()
        class=move || /* same pattern as above */
        aria-label="Lower"
    >
        "Lower"
    </button>
</div>
```

**Touch targets:** Both buttons have `min-h-11` (44px) and `min-w-[120px]` for comfortable click/tap areas per WCAG guidelines.

**Disabled state:** Visually dimmed + `cursor-not-allowed`. The `disabled` HTML attribute prevents click events. The answer handler also guards on session state as a defense-in-depth measure.

### Dependencies to Add to web/Cargo.toml

```toml
# Add to [dependencies]
js-sys = "0.3"

# Add to web-sys features list:
"Document"
"EventTarget"
"KeyboardEvent"
"Window"
"HtmlElement"
```

### What This Story Does NOT Implement

Explicitly scoped out (later stories):

- **Persistence** (story 1.8) — No IndexedDB, no localStorage. Profile is in-memory only, resets on page refresh.
- **Interval mode** (story 2.2) — Unison mode only. No interval selection, no query parameter parsing.
- **Tab visibility / interruption handling** (story 1.9) — No `visibilitychange` listener, no AudioContext suspension detection.
- **SoundFont audio** (story 5.2) — Oscillator only.
- **Profile display** (stories 3.x) — Profile accumulates in-memory but isn't displayed.
- **Settings UI** (story 2.1) — Hardcoded defaults via `DefaultSettings`.

### Existing Code Dependencies (Verify Before Implementing)

**Domain crate types used directly in ComparisonView:**
- `ComparisonSession::new(profile, observers, resettables)` — check constructor signature
- `ComparisonSession::start(intervals, &dyn UserSettings)` — takes Vec<DirectedInterval> and &dyn UserSettings
- `ComparisonSession::state() -> ComparisonSessionState` — read current state
- `ComparisonSession::show_feedback() -> bool` — feedback visibility
- `ComparisonSession::is_last_answer_correct() -> bool` — correctness
- `ComparisonSession::current_playback_data() -> Option<ComparisonPlaybackData>` — audio data
- `ComparisonSession::on_note1_finished()`, `on_note2_finished()`, `on_feedback_finished()` — state transitions
- `ComparisonSession::handle_answer(is_higher: bool, timestamp: String)` — process answer
- `ComparisonSession::stop()` — stop training
- `ComparisonSessionState` enum — Idle, PlayingNote1, PlayingNote2, AwaitingAnswer, ShowingFeedback
- `ComparisonPlaybackData` — reference_frequency, target_frequency, duration, target_amplitude_db
- `FEEDBACK_DURATION_SECS: f64` = 0.4 — use for the feedback timer (400ms)
- `MIDIVelocity::new(63)` — reference and target velocity
- `AmplitudeDB::new(0.0)` — reference amplitude (unity)
- `DirectedInterval::new(Interval::Prime, Direction::Up)` — unison mode interval
- `PerceptualProfile::new()` — cold start constructor (verify this exists)

**Web crate types used:**
- `AudioContextManager::new()` → `get_or_create() -> Result<Rc<RefCell<AudioContext>>, AudioError>`
- `OscillatorNotePlayer::new(Rc<RefCell<AudioContextManager>>)` → implements `NotePlayer`
- `NotePlayer::play_for_duration(frequency, duration, velocity, amplitude_db) -> Result<(), AudioError>`
- `NotePlayer::stop_all()`

**Verify these exist and have the expected signatures before implementing.**

### Project Structure Notes

```
web/src/
├── main.rs                        (MODIFIED — add mod bridge)
├── app.rs                         (MODIFIED — add shared context providers)
├── bridge.rs                      (NEW — ProfileObserver)
├── adapters/
│   ├── mod.rs                     (MODIFIED — add pub mod default_settings)
│   ├── audio_context.rs           (unchanged)
│   ├── audio_oscillator.rs        (unchanged)
│   └── default_settings.rs        (NEW — DefaultSettings implementing UserSettings)
├── components/
│   ├── mod.rs                     (unchanged)
│   ├── start_page.rs              (unchanged)
│   ├── comparison_view.rs         (REWRITE — full comparison training UI)
│   └── ... (other views unchanged)
web/Cargo.toml                     (MODIFIED — add js-sys, web-sys features)
```

- Alignment with unified project structure: all new files follow the architecture's adapter and bridge patterns
- `bridge.rs` at `web/src/` level (not inside adapters/) — it bridges domain observers to the web layer, which is cross-cutting
- `default_settings.rs` in `adapters/` — it's a port implementation (adapter pattern)

### Previous Story Intelligence (Story 1.6)

**Patterns established:**
- `Rc<RefCell<>>` for shared ownership — used for AudioContextManager and PerceptualProfile. Same pattern applies for ComparisonSession in this story.
- `OscillatorNotePlayer.play_for_duration()` — schedules stop via Web Audio's internal clock. Uses `stop_with_when()` for precise timing. The note self-terminates; no need for manual stop on timed notes.
- Module structure: `mod.rs` + individual files. Apply for new files.
- `cargo clippy -p domain` and `cargo clippy -p web` — run early, fix warnings.
- Domain test count at end of story 1.6: **223 tests passing**. Do not regress.

**Code review feedback from 1.6:**
- `stop()` must reset ALL session-level state, not just the state enum — the session already handles this
- `current_interval()` logs errors instead of silent fallback — error handling over silent failure
- `FEEDBACK_DURATION_SECS` is re-exported from the session module — use `domain::FEEDBACK_DURATION_SECS` in web crate

**Relevant from 1.6 Dev Notes:**
- ComparisonSession provides `ComparisonPlaybackData` (reference_frequency, target_frequency, duration, target_amplitude_db) — the web layer reads this to call NotePlayer
- Reference velocity: `MIDIVelocity::new(63)`, reference amplitude: `AmplitudeDB::new(0.0)` (constants, not from settings)
- Target velocity: `MIDIVelocity::new(63)`, target amplitude: from `ComparisonPlaybackData.target_amplitude_db`

### Git History Context

Recent commits follow the pattern: create story → implement → code review → done:
```
0719c2d Apply code review fixes for story 1.6 and mark as done
0b69ded Implement story 1.6 Comparison Session State Machine
e70f3b2 Add story 1.6 Comparison Session State Machine and mark as ready-for-dev
83af585 Apply code review fixes for story 1.5 and mark as done
2b4395e Implement story 1.5 Audio Engine (Oscillator)
```

### References

- [Source: docs/planning-artifacts/epics.md#Story 1.7: Comparison Training UI]
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: docs/planning-artifacts/architecture.md#Async Model]
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture]
- [Source: docs/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: docs/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/planning-artifacts/ux-design-specification.md#Comparison Loop Mechanics]
- [Source: docs/planning-artifacts/ux-design-specification.md#Keyboard Shortcuts]
- [Source: docs/planning-artifacts/ux-design-specification.md#Interruption Handling]
- [Source: docs/planning-artifacts/ux-design-specification.md#Comparison Training View]
- [Source: docs/planning-artifacts/ux-design-specification.md#Comparison Feedback Indicator]
- [Source: docs/planning-artifacts/ux-design-specification.md#Accessibility]
- [Source: docs/planning-artifacts/ux-design-specification.md#Button Behavior]
- [Source: docs/ios-reference/domain-blueprint.md#§11 Composition Rules]
- [Source: docs/project-context.md#Critical Implementation Rules]
- [Source: docs/project-context.md#Leptos Framework Rules]
- [Source: docs/project-context.md#Rust Language Rules]
- [Source: docs/implementation-artifacts/1-6-comparison-session-state-machine.md#Dev Notes]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- Leptos 0.8 `provide_context` requires `Send + Sync` — resolved using `send_wrapper` crate to wrap `Rc<RefCell<>>` types for WASM single-threaded context
- Leptos 0.8 `on_cleanup` also requires `Send + Sync` closure — resolved using `SendWrapper` around captured `Rc` values
- `ComparisonSession::start()` takes `HashSet<DirectedInterval>`, not `Vec` as Dev Notes suggested — corrected in implementation
- `StoredValue::new_local()` confirmed available in Leptos 0.8 reactive_graph for keeping non-Send closures alive

### Completion Notes List

- ✅ Task 1: Added `js-sys`, `send_wrapper`, and `web-sys` features (Document, EventTarget, KeyboardEvent, Window, HtmlElement) to web/Cargo.toml
- ✅ Task 2: Created `DefaultSettings` adapter implementing `UserSettings` trait with hardcoded defaults matching PRD
- ✅ Task 3: Created `ProfileObserver` bridge implementing `ComparisonObserver` — updates `PerceptualProfile` on each comparison
- ✅ Task 4: Set up shared `PerceptualProfile` and `AudioContextManager` in App component via `provide_context` with `SendWrapper`
- ✅ Task 5: Complete rewrite of `ComparisonView` — async training loop with `spawn_local`/`TimeoutFuture`, sync_signals bridge pattern, answer handler with 400ms feedback timer, Higher/Lower buttons with enabled/disabled states, feedback indicator (thumbs up/down)
- ✅ Task 6: Document-level keydown listener — ArrowUp/H for higher, ArrowDown/L for lower, Escape to stop and navigate home. Closure stored via `StoredValue::new_local` to prevent premature drop
- ✅ Task 7: Settings and Profile nav links with programmatic navigation via `use_navigate()` — stop training before navigating
- ✅ Task 8: `aria-live="polite"` region announces "Correct"/"Incorrect", buttons have `aria-label` attributes
- ✅ Task 9 (automated): 223 domain tests pass, zero clippy warnings on both crates
- ✅ Task 9 (manual): All browser tests verified — training loop, keyboard shortcuts, early answer, navigation, re-entry

### Change Log

- 2026-03-03: Implemented story 1.7 Comparison Training UI — full training loop, keyboard shortcuts, navigation, accessibility, composition root wiring
- 2026-03-03: Fixed feedback timing (main loop controls feedback duration instead of separate timer), early answer detection (responsive polling during note2), note overlap (stop_all before each note1, stop note2 on early answer), keydown listener cleanup on unmount, play_for_duration now keeps handles for stop_all support
- 2026-03-03: Code review fixes — Start Page Comparison button Space key support (AC2), eager AudioContext creation for Safari compatibility, corrected task 5.7 description, updated File List

### Code Review Fixes Applied

- **H1 (AC2):** Changed Start Page Comparison link from `<A>` to `<button>` with `use_navigate()` — Space key now activates it
- **H2 (Safari audio):** Added eager `AudioContext::get_or_create()` in ComparisonView synchronous render path before `spawn_local` — satisfies Safari/iOS user gesture requirement
- **M1 (Task accuracy):** Fixed task 5.7 description to reflect actual signals created (`sr_announcement` instead of `session_state`)
- **M2 (File List):** Added `Cargo.lock` and `start_page.rs` to Modified files list

### Known Issues

- **Audio clicks at note start/end:** Oscillator playback produces audible clicks caused by abrupt waveform cut-off (no amplitude envelope/fade). Same behavior observed on iOS. **No action needed now** — revisit only if clicks persist after the SoundFont-based NotePlayer implementation (story 5.2). If they do persist, apply a short attack/release ramp (~5-10ms) to the GainNode.

### File List

New files:
- web/src/adapters/default_settings.rs
- web/src/bridge.rs

Modified files:
- Cargo.lock (updated for new dependencies)
- web/Cargo.toml (added js-sys, send_wrapper, web-sys features)
- web/src/adapters/mod.rs (added default_settings module)
- web/src/adapters/audio_oscillator.rs (play_for_duration keeps handles in active_handles for stop_all support)
- web/src/main.rs (added bridge module)
- web/src/app.rs (added shared context providers for PerceptualProfile and AudioContextManager)
- web/src/components/start_page.rs (Comparison link changed to button for Space key support — AC2)
- web/src/components/comparison_view.rs (complete rewrite: training loop, UI, keyboard, nav, accessibility)
- docs/implementation-artifacts/sprint-status.yaml (status: in-progress → review)
- docs/implementation-artifacts/1-7-comparison-training-ui.md (task checkboxes, dev record, status)
