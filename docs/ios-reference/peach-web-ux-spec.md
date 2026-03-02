# Peach Web — UX Design Specification

**Distilled from** the iOS UX Design Specification for browser implementation.
**Date:** 2026-03-02

**Companion documents:** `docs/domain-blueprint.md` (domain logic), `docs/peach-web-prd.md` (web PRD), `docs/planning-artifacts/glossary.md` (domain terminology).

---

## 1. Executive Summary

### Project Vision

Peach is a pitch ear training app built on the philosophy of "training, not testing." Where existing apps escalate difficulty until failure to produce a score, Peach builds a perceptual profile of the user's hearing and targets weak spots. No scores, no gamification, no sessions. Every comparison makes the user better; no single answer matters.

The interaction is radically simple: two notes play in sequence, the user indicates higher or lower. The intelligence lives entirely in the adaptive algorithm, not the UI.

### Target Users

Musicians (singers, string, woodwind, brass players) for whom intonation is a practical challenge. Musically sophisticated but not necessarily tech-savvy. They want a tool that fits into the cracks of their day and makes them measurably better without demanding attention or emotional investment.

### Key Design Challenges

1. **Invisible intelligence** — The adaptive algorithm is the core value, but users can't see it working. The UX must make adaptive behavior feel purposeful, not random.
2. **Sparse-data visualization** — On first use, the perceptual profile has almost no data. The profile must look meaningful and inviting even when mostly empty.
3. **No-session training model** — Start and stop must feel natural without conventional session framing.
4. **Multi-input interaction** — The browser must support mouse, keyboard, and touch equally for the core training loop.

---

## 2. Core User Experience

### Defining Experience

The core experience is the **comparison loop**: two notes play in sequence, the user indicates higher or lower, feedback flashes, the next pair begins. This loop is the entire product. Everything else exists to support or reflect it.

The loop must feel **reflexive, not deliberative**. The user should be reacting to sounds, not thinking about an app. If the user is ever thinking about the UI during training, the design has failed.

### Effortless Interactions

1. **Start training** — One click/keypress from the start page to hearing the first note. No onboarding, no account, no "welcome back."
2. **Stop training** — One click/keypress, immediate, no confirmation dialog, no session summary. The incomplete comparison is silently discarded.
3. **Answer a comparison** — Click a button or press a key (Arrow Up / H for higher, Arrow Down / L for lower). Buttons are enabled the moment the second note begins playing.
4. **Resume where you left off** — There is no resume. There are no sessions. The user opens the page and starts training; the algorithm already knows their profile.
5. **See progress** — The profile preview on the start page provides a glanceable snapshot. Clicking it opens the full profile.

### Experience Principles

1. **Disappearing UI** — Every design decision should reduce the distance between the user and the sounds. Controls, transitions, and feedback should be felt, not studied.
2. **Every answer improves you** — No answer is wasted, no answer is punished. Wrong answers are information, not failure. No scores, no streaks, no "try again" language.
3. **Respect incidental time** — Instant start, instant stop. 30 seconds of training is as valid as 30 minutes.
4. **Show, don't score** — Progress is a perceptual profile that evolves, not a number going up.
5. **Sound first, pixels second** — Audio quality, timing, and accuracy take absolute priority over visual polish.

---

## 3. Emotional Design

### Primary Emotional Goals

1. **Calm focus** — Training should feel meditative, not competitive. The rhythmic cadence creates a flow state.
2. **Quiet confidence** — Over time, a growing inner certainty that hearing is sharpening, confirmed by the data.
3. **Freedom from judgment** — No answer carries weight. The app never evaluates the user — it trains them.

### Emotional Journey

| Moment | Desired Feeling | Anti-Pattern to Avoid |
|---|---|---|
| **First launch** | Curiosity, ease — "that's it?" | Overwhelm, setup fatigue |
| **First comparisons** | Playful alertness — like a reflex game | Scoring overhead, distraction |
| **First wrong answer** | Nothing — visual indicator and the next pair | Shame, "try again" pressure |
| **Stopping mid-training** | Neutral — like closing a book | Guilt, loss aversion, "are you sure?" |
| **Checking the profile** | Honest curiosity — "what does my hearing look like?" | Score-driven framing |
| **Seeing improvement** | Quiet satisfaction — the data speaks for itself | Artificial celebration, badges |
| **Returning after a break** | Seamless continuity | Guilt trips, streak resets |

### Design Implications

| Emotional Goal | UX Approach |
|---|---|
| Calm focus | Minimal training view — no stats, counters, or progress bars during training |
| Freedom from judgment | Feedback indicator is brief and non-emphatic. Same visual weight for correct and incorrect |
| Quiet confidence | Profile visualization uses calm, factual presentation. No celebratory animations |
| Seamless continuity | No session boundaries, no timers, no "session complete" screens |

### Sensory Hierarchy (Web Adaptation)

The iOS app uses: ears > fingers (haptic) > eyes. On the web, haptic feedback is unavailable. The adapted hierarchy is:

**Ears > eyes (feedback) > keyboard/mouse**

- Audio provides the training content (primary)
- Visual feedback provides the result (secondary) — must be brief and non-emphatic
- Input provides the response (tertiary)

The absence of haptic feedback means the web version relies more on visual feedback than iOS. The feedback indicator should compensate by being visually clear but emotionally neutral.

---

## 4. Navigation and Screen Structure

### Hub-and-Spoke Model

```
                    ┌──────────────────────────┐
                    │      Start Page          │
                    │                          │
                    │ [Comparison]             │
                    │ [Pitch Matching]         │
                    │ ── ── ── ── ── ── ──    │
                    │ [Interval Comparison]    │
                    │ [Interval Pitch Matching]│
                    │                          │
                    │ [Settings] [Profile]     │
                    │ [Info]                   │
                    └┬──┬──┬──┬──┬──┬──┬──┬──┘
                     │  │  │  │  │  │  │  │
        Comparison───┘  │  │  │  │  │  │  └───Interval PM
        Pitch Match─────┘  │  │  │  │  └──────Interval Comp
                           │  │  │  │
                    ┌──────▼──▼──▼──▼──────────┐
                    │   Comparison View        │
                    │   (± interval indicator) │
                    │   [Higher] [Lower]       │
                    │   [Settings] [Profile]   │
                    └──────────────────────────┘

                    ┌──────────────────────────┐
                    │  Pitch Matching View     │
                    │  (± interval indicator)  │
                    │  [Slider]               │
                    │  [Settings] [Profile]   │
                    └──────────────────────────┘

  All training views ──► Settings/Profile ──► Start Page
```

### Navigation Rules

| Rule | Implementation |
|---|---|
| Start page is always the hub | Client-side routing root |
| All secondary views return to start page | Back navigation or explicit link |
| Training views are entered only from start page | Direct links from start page buttons |
| Training views are exited by navigating away | Settings/Profile links, or closing the tab |
| Maximum navigation depth is always 1 level | No nested views |
| No tabs | Single navigation flow |

### Navigation Paths

```
Start Page ──► Comparison View (via Comparison)
Start Page ──► Comparison View with interval (via Interval Comparison)
Start Page ──► Pitch Matching View (via Pitch Matching)
Start Page ──► Pitch Matching View with interval (via Interval Pitch Matching)
Start Page ──► Profile View ──► Start Page
Start Page ──► Settings View ──► Start Page
Start Page ──► Info View ──► Start Page
Comparison View ──► Profile View ──► Start Page
Comparison View ──► Settings View ──► Start Page
Pitch Matching View ──► Profile View ──► Start Page
Pitch Matching View ──► Settings View ──► Start Page
Tab hidden/closed during training ──► next visit returns to Start Page
```

---

## 5. Screen Specifications

### 5.1 Start Page

**Purpose:** Hub for all app actions. Profile preview, training mode selection, navigation.

**Elements:**
- **Profile Preview** — compact visualization of the perceptual profile. Clickable → navigates to full profile view. Shows cold-start empty state when no data exists.
- **Comparison button** — primary action, most prominent. Begins comparison training.
- **Pitch Matching button** — secondary action, less prominent. Begins pitch matching training.
- **Visual separator** — subtle divider between unison and interval modes.
- **Interval Comparison button** — secondary action. Begins interval comparison training.
- **Interval Pitch Matching button** — secondary action. Begins interval pitch matching training.
- **Settings link** — navigates to settings view.
- **Profile link** — navigates to full profile view.
- **Info link** — navigates to info view.

**Button hierarchy:**

| Button | Prominence | Description |
|---|---|---|
| Comparison | Primary (most prominent) | Hero action for new and returning users |
| Pitch Matching | Secondary | Below comparison |
| Interval Comparison | Secondary | Below separator |
| Interval Pitch Matching | Secondary | Below separator |
| Settings, Profile, Info | Tertiary (icon or text link) | Utility navigation |

**Behavior:**
- No onboarding, tutorial, or welcome message.
- Identical on every visit regardless of time elapsed since last use.
- No "welcome back," streak, or activity summary.

### 5.2 Comparison Training View

**Purpose:** The core comparison training loop.

**Elements:**
- **Target interval label** — shown only in interval mode. Displays the current interval name (e.g. "Perfect Fifth Up"). Hidden in unison mode.
- **Higher button** — user's judgment that the second note was higher.
- **Lower button** — user's judgment that the second note was lower.
- **Feedback indicator** — thumbs up (correct) or thumbs down (incorrect), shown briefly after each answer.
- **Settings link** — navigates to settings (stops training).
- **Profile link** — navigates to profile (stops training).

**States and Behavior:**

| State | Higher/Lower Buttons | Feedback | Description |
|---|---|---|---|
| `playingNote1` | Disabled | Hidden | Reference note playing |
| `playingNote2` | **Enabled** | Hidden | Target note playing — early answer allowed |
| `awaitingAnswer` | Enabled | Hidden | Both notes finished, waiting for input |
| `showingFeedback` | Disabled | Visible (400ms) | Brief result display |

**Feedback indicator:**
- Correct: thumbs-up icon, green color
- Incorrect: thumbs-down icon, red color
- Same visual weight, same duration, same position for both
- No haptic equivalent on web — the visual indicator is the sole feedback channel

**Web-specific: Keyboard shortcuts**

| Action | Keys |
|---|---|
| Answer "Higher" | Arrow Up or H |
| Answer "Lower" | Arrow Down or L |
| Stop / leave | Escape |

**Timing diagram:**
```
[Note 1 ~~~] [Note 2 ~~~] [Answer] [Feedback ··] [Note 1 ~~~] [Note 2 ~~~] ...
 buttons off   buttons on   click/   brief show    buttons off   buttons on
                             key!     400ms
```

### 5.3 Pitch Matching Training View

**Purpose:** Pitch matching training — tune a note to match a reference.

**Elements:**
- **Target interval label** — shown only in interval mode. Hidden in unison mode.
- **Vertical pitch slider** — the primary interaction control. Up = sharper, down = flatter. Custom component.
- **Feedback indicator** — directional arrow + signed cent offset, shown after each attempt.
- **Settings link** — navigates to settings (stops training).
- **Profile link** — navigates to profile (stops training).

**States and Behavior:**

| State | Slider | Feedback | Description |
|---|---|---|---|
| `playingReference` | Visible, disabled (dimmed) | Hidden | Reference note playing |
| `awaitingSliderTouch` | Enabled, waiting | Hidden | Reference ended, tunable note playing, waiting for first interaction |
| `playingTunable` | Active, user dragging | Hidden | Pitch changes in real time as user drags |
| `showingFeedback` | Disabled | Visible (400ms) | Result display: arrow + cent offset |

**Vertical Pitch Slider:**
- Occupies most of the available vertical space
- Up = sharper, down = flatter (matches musical intuition)
- No markings, no tick marks, no center indicator — a blank instrument
- Always starts at the same physical position regardless of pitch offset
- Must work with: mouse drag, touch drag, and keyboard (Arrow Up/Down for fine adjustment)
- The slider provides no visual proximity feedback during tuning — the ear is the only guide
- Releasing the mouse/touch commits the result (release = commit)
- No time limit — the tunable note plays indefinitely until the user commits

**Feedback indicator (pitch matching specific):**

| User's Error | Symbol | Color | Example |
|---|---|---|---|
| Dead center (~0 cents) | Dot | Green | "0 cents" |
| Slightly off (<10 cents) | Short arrow up/down | Green | "+4 cents" |
| Moderately off (10-30 cents) | Medium arrow up/down | Yellow | "-22 cents" |
| Far off (>30 cents) | Long arrow up/down | Red | "+55 cents" |

Arrow direction indicates sharp (up) or flat (down). Arrow length is categorical (short/medium/long), not proportional. The signed cent offset provides exact detail.

**Web-specific: Keyboard support**

| Action | Keys |
|---|---|
| Fine pitch adjust | Arrow Up / Arrow Down |
| Commit pitch | Enter or Space |

**Timing diagram:**
```
[Reference ~~~] [Tunable ~~~~~~~~~~~~~~~~~~~~...release] [Feedback ··] [Reference ~~~] ...
 slider visible   auto-starts, slider active              arrow+cents   slider visible
 but disabled     user tunes by ear, no visual aid        ~400ms        but disabled
```

### 5.4 Profile View

**Purpose:** Full perceptual profile visualization and statistics.

**Elements:**
- **Perceptual profile visualization** — piano keyboard with confidence band overlay (custom component).
- **Summary statistics** — overall mean detection threshold, standard deviation, trend indicator (improving/stable/declining).
- **Pitch matching statistics** — mean absolute error, standard deviation, sample count. Shown when matching data exists.
- **Back navigation** — returns to start page.

**Profile visualization:**
- **X-axis:** Piano keyboard strip spanning the training range. Stylized (not photorealistic) — simple rectangles with standard proportions. Note names at octave boundaries (C2, C3, C4, etc.).
- **Y-axis:** Detection threshold in cents (lower is better). Inverted so improvement visually moves the band downward toward the keyboard.
- **Confidence band:** Filled area chart overlaid above the keyboard. Width represents uncertainty — wider where data is sparse, narrower where many comparisons exist.
- **Render via:** Canvas element or SVG.

**Empty states:**
- **Cold start (no data):** Keyboard renders fully. Band absent or shown as a faint uniform placeholder at 100 cents. Text: "Start training to build your profile." No call-to-action button.
- **Statistics cold start:** Show dashes ("—") instead of numbers. Trend indicator hidden.
- **Partial data:** Band renders where data exists and fades where it doesn't. No interpolation across large gaps.

**Behavior:** Read-only. No interactions beyond viewing and navigating back.

### 5.5 Settings View

**Purpose:** Configuration of all training parameters.

**Settings:**

| Setting | Control Type | Notes |
|---|---|---|
| Note range (lower bound) | Dropdown or stepper | MIDI note range |
| Note range (upper bound) | Dropdown or stepper | MIDI note range |
| Note duration | Slider or stepper | 0.3 to 3.0 seconds |
| Reference pitch | Dropdown or stepper | 440Hz, 442Hz, 432Hz, 415Hz, or custom |
| Sound source | Dropdown | Available instrument sounds |
| Vary loudness | Slider | 0% to 100% |
| Interval selection | Multi-select | Which directed intervals to train |
| Tuning system | Dropdown | Equal Temperament / Just Intonation |
| Reset all training data | Button with confirmation | Destructive action — requires confirmation dialog |

**Behavior:**
- All changes auto-save to browser storage — no save/cancel buttons.
- No form validation needed — all controls are bounded.
- Changes take effect on the next comparison after returning to training.
- Settings persist across page refreshes and browser restarts via browser storage (localStorage).

### 5.6 Info View

**Purpose:** Basic app information.

**Contents:**
- App name (Peach)
- Developer name
- Copyright notice
- Version number

Minimal content. Can be a modal/overlay or a separate page.

---

## 6. Interaction Patterns

### Comparison Loop Mechanics

1. **Initiation:** User clicks a training button on the start page. Training view appears. First comparison begins immediately — no countdown, no "get ready."
2. **Note 1 plays:** Answer buttons disabled.
3. **Note 2 plays:** Answer buttons enabled the moment the second note begins. User can answer during or after the note.
4. **Answer:** User clicks a button or presses a key. Both buttons disable immediately.
5. **Feedback:** Indicator appears instantly (thumbs up/down). Persists ~400ms. No other information shown.
6. **Loop:** Indicator clears, next comparison begins. Loop continues indefinitely until the user navigates away.
7. **Leaving:** User clicks Settings/Profile link, or closes/hides the tab. Incomplete comparison silently discarded. No session summary, no confirmation.

### Pitch Matching Loop Mechanics

1. **Initiation:** User clicks Pitch Matching on start page. Training view appears. First reference note plays immediately.
2. **Reference plays:** Configured duration. Slider visible but disabled.
3. **Reference stops, tunable auto-starts:** Tunable note begins at random offset. Slider becomes active.
4. **User tunes:** Dragging the slider changes pitch in real time. No visual proximity feedback.
5. **User releases:** Note stops immediately. Result recorded. Feedback appears (arrow + cents, ~400ms).
6. **Loop:** Feedback clears, next reference note plays. Loop continues until user navigates away.

### Interruption Handling

| Interruption | Behavior |
|---|---|
| Navigate to Settings/Profile | Stop training, discard incomplete |
| Close or hide tab | Stop training, discard incomplete |
| AudioContext suspended by browser | Stop training, require user gesture to resume |
| Return after interruption | Start page — user starts fresh |

All interruptions follow the same rule: stop audio, discard incomplete exercise, return to start page.

### Web Audio Context Activation

The Web Audio API requires a user gesture to create or resume an AudioContext. The training start button serves as this gesture — clicking "Comparison" or "Pitch Matching" both activates the AudioContext and begins training in one action.

If the AudioContext becomes suspended (e.g. tab hidden, browser policy), training stops. The user must return to the start page and click a training button again to resume.

---

## 7. Visual Design Foundation

### Design System Approach

Use the browser's default styling as the foundation, enhanced with a minimal, clean CSS layer. The goal is **platform-native feel** — the app should feel like a well-built web application, not an iOS port.

**Principles:**
- System font stack (the user's OS default fonts)
- System color scheme support (light/dark mode via `prefers-color-scheme`)
- Minimal custom styling — let the content and interaction be the focus
- No CSS framework required (though one may be used for convenience)

### Color System

- **Feedback green:** Standard green for correct/close match
- **Feedback yellow:** Standard yellow/amber for moderate match
- **Feedback red:** Standard red for incorrect/far match
- **Text:** System foreground colors with semantic hierarchy (primary, secondary, muted)
- **Background:** System background, respecting dark mode
- **No brand colors.** The app's identity comes from its behavior.

### Typography

- System font stack: `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`
- Standard size scale for hierarchy (headings, body, captions)
- No custom fonts

### Spacing and Layout

- Consistent spacing scale (e.g. 4/8/16/24/32/48px)
- Single-column layout for training views
- Responsive: works on desktop and mobile browsers
- Training view buttons should be large and easy to click/tap

---

## 8. Custom Components

### 8.1 Perceptual Profile Visualization

**Purpose:** Display pitch discrimination ability across the training range.

**Implementation options:** HTML Canvas or SVG.

**Visual specification:**
- Piano keyboard strip (horizontal, stylized rectangles with standard proportions)
- Confidence band area chart overlaid above the keyboard
- Inverted Y-axis (lower = better)
- System colors for band fill, with opacity for confidence range
- Note names at octave boundaries

**States:** Empty (keyboard only), sparse (partial band with gaps), populated (continuous band).

**Accessibility:**
- Provide a text alternative summarizing the data: "Perceptual profile: average detection threshold X cents across Y trained notes"
- ARIA role and label on the canvas/SVG container

### 8.2 Profile Preview

**Purpose:** Compact, clickable miniature of the profile on the start page.

**Visual specification:**
- Same band shape as full visualization, no axis labels, no numerical values
- Clickable → navigates to full profile view
- Empty state: subtle placeholder shape

**Accessibility:**
- "Your pitch profile. Click to view details."
- If data exists: "Your pitch profile. Average threshold: X cents. Click to view details."

### 8.3 Comparison Feedback Indicator

**Visual specification:**
- Thumbs up / thumbs down icon (Unicode or SVG)
- Green (correct) / red (incorrect)
- Centered overlay, large enough for peripheral vision
- Appears for ~400ms, then disappears
- No animation required (simple show/hide is fine; subtle fade acceptable)

### 8.4 Pitch Matching Feedback Indicator

**Visual specification:**
- Directional arrow (up/down) or dot, colored by proximity band
- Signed cent offset text alongside
- Same position, timing, and transition as comparison feedback
- Green dot for dead center, green/yellow/red arrows for short/medium/long

### 8.5 Vertical Pitch Slider

**Visual specification:**
- Vertical orientation, occupying most of the training view height
- Large thumb/handle for comfortable dragging
- No markings, no tick marks, no center indicator
- Starts at the same physical position regardless of pitch offset

**Implementation:** Custom HTML element with mouse drag, touch drag, and keyboard support.

**States:** Inactive (dimmed, during reference), active (enabled), dragging (following input), released (disabled, result recorded).

**Accessibility:**
- ARIA role: slider
- ARIA label: "Pitch adjustment"
- Keyboard operable: Arrow Up/Down for fine adjustment, Enter/Space to commit
- Announce result after commit

---

## 9. Responsive Design

### Strategy

The app should work well on both desktop and mobile browsers. No breakpoints for training functionality — the core interaction works at any reasonable viewport size.

### Desktop (primary)

- Training buttons are large, centered
- Keyboard shortcuts are the primary input method for power users
- Profile visualization uses full available width

### Mobile (supported)

- Touch-friendly button sizes (minimum 44x44px tap targets)
- Vertical slider works with touch drag
- Single-column layout, no horizontal scrolling
- Viewport meta tag for proper mobile rendering

### Orientation

- **Portrait (phone):** Natural layout, vertical slider fills height
- **Landscape (phone):** Reduced slider height, still functional
- **Desktop:** Width varies; training views center content at a comfortable maximum width

---

## 10. Accessibility

### Strategy

Follow WCAG 2.1 AA guidelines. Use semantic HTML, ARIA attributes where needed, and keyboard navigation throughout.

### Requirements

| Area | Approach |
|---|---|
| Keyboard navigation | All interactive elements reachable and operable via keyboard |
| Screen reader | Semantic HTML + ARIA labels for custom components |
| Color contrast | Minimum 4.5:1 for text, 3:1 for large text and UI components |
| Focus indicators | Visible focus rings on all interactive elements |
| Motion | Respect `prefers-reduced-motion` — minimize or eliminate animations |
| Dark mode | Respect `prefers-color-scheme` — all colors work in both modes |
| Text scaling | Layout doesn't break at 200% browser zoom |

### Screen Reader Announcements

| Event | Announcement |
|---|---|
| Comparison feedback | "Correct" or "Incorrect" |
| Pitch matching feedback | "4 cents sharp" / "27 cents flat" / "Dead center" |
| Training started | "Training started" (or simply begin audio) |
| Training stopped | "Training stopped" |
| Interval change | "Target interval: Perfect Fifth Up" |

### Audio Dependency

Peach requires audio output to function. This is a fundamental constraint of the domain. Users who cannot hear audio cannot use the core training functionality. This is acknowledged as a limitation, not a design oversight.

---

## 11. Anti-Patterns to Avoid

These patterns are explicitly rejected:

1. **Score-driven design** — Any element that frames outcomes as a score, percentage, or pass/fail.
2. **Transition theater** — Animated transitions, "preparing next challenge" delays between comparisons. Every second of non-training time is a design failure.
3. **Engagement guilt mechanics** — Streaks, daily goals, "you haven't trained in X days," declining statistics presented as warnings.
4. **Complexity creep** — Visible algorithm parameters, per-comparison statistics, or "advanced mode" during training.
5. **Onboarding tutorials** — The interaction is self-explanatory. A tutorial implies complexity that doesn't exist.
6. **Session framing** — No session summaries, no "X comparisons today" counters, no session-complete screens.
7. **Gamification** — No badges, no levels, no achievements, no leaderboards, no confetti.
