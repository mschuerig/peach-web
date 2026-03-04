# Story 5.1: Interval Training Mode

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to train comparison and pitch matching with musical intervals,
so that I can develop my ability to hear pitch differences within specific interval contexts.

## Acceptance Criteria

1. **Given** the Start Page **When** it loads **Then** I see "Interval Comparison" and "Interval Pitch Matching" buttons below a visual separator, with secondary prominence (FR2, FR10).

2. **Given** I click "Interval Comparison" **When** the training view loads **Then** the ComparisonSession starts with the user's selected intervals from Settings **And** the route is `/training/comparison?intervals=<comma-separated interval codes>` (e.g., `?intervals=M3u,M3d,m6u,M6d`).

3. **Given** I click "Interval Pitch Matching" **When** the training view loads **Then** the PitchMatchingSession starts with the user's selected intervals **And** the route is `/training/pitch-matching?intervals=<comma-separated interval codes>`.

4. **Given** training is in interval mode **When** a comparison or challenge is generated **Then** a random interval is selected from the user's configured interval set **And** the target note is transposed from the reference by that interval.

5. **Given** training is in interval mode **When** the training view renders **Then** a target interval label is visible showing the current interval name and direction (e.g. "Perfect Fifth Up") **And** this label is hidden in unison mode.

6. **Given** interval mode training **When** I answer a comparison or commit a pitch **Then** the record includes the correct interval semitone distance **And** the next challenge may use a different interval from the selected set.

7. **Given** the user has selected no non-prime intervals in Settings **When** they click an interval training button **Then** training starts with only prime/up (effectively unison mode — same as regular mode).

## Tasks / Subtasks

- [x] Task 1: Create shared interval code encode/decode utility (AC: #2, #3)
  - [x] 1.1 Add `web/src/interval_codes.rs` with `encode_intervals()` and `decode_intervals()` functions
  - [x] 1.2 Encoding format: `P1`, `m2u`, `m2d`, `M2u`, `M2d`, `m3u`, `m3d`, `M3u`, `M3d`, `P4u`, `P4d`, `d5u`, `d5d`, `P5u`, `P5d`, `m6u`, `m6d`, `M6u`, `M6d`, `m7u`, `m7d`, `M7u`, `M7d`, `P8u`, `P8d`
  - [x] 1.3 Unit tests for round-trip encode/decode

- [x] Task 2: Extract `interval_label()` to shared location (AC: #5)
  - [x] 2.1 Move `interval_label()` from `settings_view.rs` to `interval_codes.rs` and make it `pub`
  - [x] 2.2 Update `settings_view.rs` to import from new location

- [x] Task 3: Update Start Page for interval mode navigation (AC: #1, #2, #3, #7)
  - [x] 3.1 Replace static `<A>` links with `<button>` handlers that read intervals from localStorage and encode as query params
  - [x] 3.2 "Interval Comparison" → navigates to `/training/comparison?intervals=<encoded>`
  - [x] 3.3 "Interval Pitch Matching" → navigates to `/training/pitch-matching?intervals=<encoded>`
  - [x] 3.4 Fallback: if no non-prime intervals selected, navigate with `?intervals=P1`

- [x] Task 4: Update ComparisonView to support interval mode (AC: #2, #4, #5, #6)
  - [x] 4.1 Parse `?intervals=` query param using `leptos_router` query access
  - [x] 4.2 If `?intervals=` present: decode intervals, pass to `session.start(intervals, &settings)`
  - [x] 4.3 If `?intervals=` absent: pass `{Prime/Up}` only (unison mode) — fixes current bug
  - [x] 4.4 Add interval label element that shows current interval name+direction when in interval mode
  - [x] 4.5 Hide interval label in unison mode (when only Prime/Up or no query param)

- [x] Task 5: Update PitchMatchingView to support interval mode (AC: #3, #4, #5, #6)
  - [x] 5.1 Same query param parsing as ComparisonView
  - [x] 5.2 Same interval-based session start logic
  - [x] 5.3 Same interval label display logic
  - [x] 5.4 If `?intervals=` absent: pass `{Prime/Up}` only — fixes current bug

- [x] Task 6: Verify and test (AC: all)
  - [x] 6.1 `cargo test -p domain` — confirm no regressions (293 tests pass)
  - [x] 6.2 `trunk build` — confirm WASM compilation
  - [x] 6.3 `cargo clippy` — zero warnings
  - [x] 6.4 Manual browser test: click Interval Comparison, verify query param in URL, verify interval label shows, verify different intervals are randomly selected
  - [x] 6.5 Manual browser test: click regular Comparison, verify NO query param, verify NO interval label, verify unison behavior
  - [x] 6.6 Manual browser test: same tests for Pitch Matching and Interval Pitch Matching

## Dev Notes

### Architecture — Domain Layer Is Already Complete

Both `ComparisonSession` and `PitchMatchingSession` already accept `HashSet<DirectedInterval>` in their `start()` methods and fully support interval mode:

- `ComparisonSession::start(intervals, &settings)` stores intervals, randomly selects one per comparison
- `PitchMatchingSession::start(intervals, &settings)` stores intervals, randomly selects one per challenge
- `KazezNoteStrategy::next_comparison()` takes a `DirectedInterval` and transposes the target note accordingly
- `PitchMatchingSession::generate_challenge()` respects interval bounds (adjusts min/max note to keep transposition in MIDI range)
- `session.current_interval()` returns `Option<DirectedInterval>` from the current comparison/challenge

**Zero domain crate changes needed.** This story is entirely web-layer work.

### What Already Exists (DO NOT Rebuild)

| Component | File | Status |
|-----------|------|--------|
| `DirectedInterval` type + operations | `domain/src/types/interval.rs` | Complete with tests |
| `Interval` enum (13 variants, 0-12 semitones) | `domain/src/types/interval.rs:9-23` | Complete |
| `Direction` enum (Up/Down) | `domain/src/types/interval.rs:60-63` | Complete |
| `ComparisonSession::start(intervals, settings)` | `domain/src/session/comparison_session.rs:128` | Accepts `HashSet<DirectedInterval>` |
| `PitchMatchingSession::start(intervals, settings)` | `domain/src/session/pitch_matching_session.rs:122` | Accepts `HashSet<DirectedInterval>` |
| `session.current_interval()` | Both session files (~line 112) | Returns `Option<DirectedInterval>` |
| `KazezNoteStrategy::next_comparison()` with interval | `domain/src/strategy.rs:83-144` | Full interval transposition logic |
| `PitchMatchingSession::generate_challenge()` | `domain/src/session/pitch_matching_session.rs:280-309` | Interval-aware bounds |
| `ComparisonRecord.interval` field (semitone distance) | `domain/src/records.rs:7-52` | Already stores interval |
| `PitchMatchingRecord.interval` field | `domain/src/records.rs:54-96` | Already stores interval |
| `LocalStorageSettings::get_selected_intervals()` | `web/src/adapters/localstorage_settings.rs:44-54` | Reads `peach.intervals` from localStorage |
| `interval_label()` function | `web/src/components/settings_view.rs:41-62` | Formats "Perfect Fifth Up" etc. |
| `all_directed_intervals()` | `web/src/components/settings_view.rs:64-87` | Lists all 25 intervals |
| Start page with 4 training buttons + separator | `web/src/components/start_page.rs` | Buttons exist, links need fixing |
| Interval selection UI in Settings | `web/src/components/settings_view.rs:371-396` | Full multi-select checkbox UI |

### Current Bug to Fix

Both `ComparisonView` and `PitchMatchingView` currently call `LocalStorageSettings::get_selected_intervals()` to get intervals for session start. This means if the user has selected intervals in Settings, even the "regular" Comparison button uses them. The fix: unison mode buttons must always pass `{Prime/Up}` only, ignoring Settings intervals. Interval mode buttons read Settings intervals.

### Task 1: Interval Code Encode/Decode

Create `web/src/interval_codes.rs`. The compact encoding format matches the project convention from `project-context.md`:

```
P1   → Prime (always Up)
m2u  → Minor Second Up       m2d  → Minor Second Down
M2u  → Major Second Up       M2d  → Major Second Down
m3u  → Minor Third Up        m3d  → Minor Third Down
M3u  → Major Third Up        M3d  → Major Third Down
P4u  → Perfect Fourth Up     P4d  → Perfect Fourth Down
d5u  → Tritone Up            d5d  → Tritone Down
P5u  → Perfect Fifth Up      P5d  → Perfect Fifth Down
m6u  → Minor Sixth Up        m6d  → Minor Sixth Down
M6u  → Major Sixth Up        M6d  → Major Sixth Down
m7u  → Minor Seventh Up      m7d  → Minor Seventh Down
M7u  → Major Seventh Up      M7d  → Major Seventh Down
P8u  → Octave Up             P8d  → Octave Down
```

Functions:
- `pub fn encode_intervals(intervals: &HashSet<DirectedInterval>) -> String` — sorted, comma-separated
- `pub fn decode_intervals(code: &str) -> HashSet<DirectedInterval>` — parse comma-separated codes, skip invalid
- `pub fn interval_label(interval: Interval, direction: Direction) -> String` — moved from settings_view.rs

### Task 3: Start Page Changes

Current start page (`web/src/components/start_page.rs`):
- Lines 46-54: "Interval Comparison" and "Interval Pitch Matching" are `<A>` links to the same routes as regular training. They need to become `<button>` elements with click handlers that:
  1. Read `LocalStorageSettings::get_selected_intervals()`
  2. Encode as query string with `encode_intervals()`
  3. Navigate to `/training/comparison?intervals=<encoded>` or `/training/pitch-matching?intervals=<encoded>`

Pattern to follow: the existing `on_comparison` and `on_pitch_matching` closures (lines 10-21) already use `navigate()` — add the same pattern for interval buttons.

### Task 4 & 5: Training View Changes

**Query param parsing:** Use `leptos_router`'s query parameter access. In Leptos 0.8, use `use_query_map()` from `leptos_router::hooks` to read URL query parameters reactively.

**Session start logic change:**
```rust
// Current (buggy — uses Settings intervals even in unison mode):
let intervals = LocalStorageSettings::get_selected_intervals();
session.borrow_mut().start(intervals, &settings);

// Fixed:
let intervals = parse_intervals_from_query(); // from ?intervals= param
let intervals = if intervals.is_empty() {
    // Unison mode — no query param
    let mut set = HashSet::new();
    set.insert(DirectedInterval::new(Interval::Prime, Direction::Up));
    set
} else {
    intervals
};
session.borrow_mut().start(intervals, &settings);
```

**Interval label:** Both views need a conditionally-visible label. Derive from the session's `current_interval()`:
- If interval is `Some(di)` where `di.interval != Prime` → show label with `interval_label(di.interval, di.direction)`
- If interval is `None` or `Prime` → hide label

In `comparison_view.rs`, add a signal that reads `session.current_interval()` after each comparison generation (in the state machine's `PlayingReferenceNote` state). Display in the view template as a `<p>` element above the answer buttons, with `class="hidden"` when in unison mode.

In `pitch_matching_view.rs`, same pattern — show interval label above the slider during `PlayingReference` and `PlayingTunable` states.

### Leptos Router Query Param Access

In Leptos 0.8 with `leptos_router`, query parameters are accessed via:
```rust
use leptos_router::hooks::use_query_map;

let query = use_query_map();
let intervals_param = move || query.read().get("intervals").unwrap_or_default();
```

This is reactive — it reads the current URL query params. Call once at component mount to determine the interval set for the session.

### Project Structure Notes

**New file:**
- `web/src/interval_codes.rs` — shared encode/decode/label utilities

**Modified files:**
- `web/src/components/start_page.rs` — interval button navigation with query params
- `web/src/components/comparison_view.rs` — query param parsing, interval-based session start, interval label display
- `web/src/components/pitch_matching_view.rs` — same changes as comparison_view
- `web/src/components/settings_view.rs` — import `interval_label` from new location
- `web/src/components/mod.rs` — no changes needed (interval_codes is not a component)
- `web/src/main.rs` or `web/src/lib.rs` — add `mod interval_codes;`

**No changes to:**
- Any domain crate files
- `web/src/app.rs` (routes stay the same — query params are handled by views)
- `web/src/adapters/` (no adapter changes)
- `web/src/bridge.rs` (no observer changes)
- `Cargo.toml` (no new dependencies)

### Testing Strategy

**Automated:** `cargo test -p domain` (no domain changes, but verify no regressions — 293+ tests). Unit tests for `encode_intervals`/`decode_intervals` round-trip in web crate.

**Manual browser tests:**
1. Click "Comparison" → URL is `/training/comparison` (no query param), no interval label shown, unison behavior
2. Click "Interval Comparison" → URL is `/training/comparison?intervals=...`, interval label visible, intervals randomly selected
3. Same for Pitch Matching / Interval Pitch Matching
4. Select different intervals in Settings → interval buttons reflect new selection
5. Deselect all non-prime intervals → interval buttons use `P1` (effectively unison)

### References

- [Source: docs/planning-artifacts/epics.md#Story 5.1 — Acceptance criteria and BDD]
- [Source: docs/planning-artifacts/architecture.md#Frontend Architecture — routing, signals, component architecture]
- [Source: docs/planning-artifacts/ux-design-specification.md#Start Page — button hierarchy, interval buttons]
- [Source: docs/planning-artifacts/ux-design-specification.md#Comparison Training View — interval label display]
- [Source: docs/planning-artifacts/ux-design-specification.md#Pitch Matching Training View — interval label display]
- [Source: docs/planning-artifacts/ux-design-specification.md#Route Paths — query param convention]
- [Source: docs/project-context.md#Routing — interval mode via query parameter convention]
- [Source: domain/src/session/comparison_session.rs:128 — start() accepts HashSet<DirectedInterval>]
- [Source: domain/src/session/pitch_matching_session.rs:122 — start() accepts HashSet<DirectedInterval>]
- [Source: domain/src/strategy.rs:83-144 — next_comparison() with interval transposition]
- [Source: domain/src/types/interval.rs — DirectedInterval, Interval, Direction types]
- [Source: web/src/adapters/localstorage_settings.rs:44-54 — get_selected_intervals()]
- [Source: web/src/components/settings_view.rs:41-62 — interval_label() to extract]
- [Source: web/src/components/start_page.rs — current button layout to modify]
- [Source: web/src/components/comparison_view.rs:325-326 — current session start to fix]
- [Source: web/src/components/pitch_matching_view.rs:401-402 — current session start to fix]

## Previous Story Intelligence

### From Story 4.4 (Pitch Matching Persistence)

- Pattern for mirroring existing code: Story 4.4 mirrored `fetch_all_comparisons()` → `fetch_all_pitch_matchings()` with minimal changes. Apply same "mirror with targeted changes" approach when making parallel changes to ComparisonView and PitchMatchingView.
- All 293 domain tests pass, trunk build and clippy clean as of story 4.4 completion.
- Code review caught redundant `.abs()` call and ambiguous log messages — be precise in log messages and don't add unnecessary transformations.
- Two-borrow pattern in pitch_matching_view.rs (pre-checks state then mutably borrows) — be aware of RefCell borrow patterns in view components.

### From Story 4.3 Debug Log

- Document-level Enter/Space handler was removed (unreachable, passed wrong value) — avoid document-level key handlers that conflict with component-level ones.
- `let _ = adjust_frequency()` replaced with proper error logging — never silently discard Results.
- Tunable note timing was fixed (waits for slider touch) — the pitch matching state machine transitions are well-tested now.

## Git Intelligence

Recent commits (newest first):
```
6d29983 Apply code review fixes for story 4.4 and mark epic 4 as done
895d216 Implement story 4.4 Pitch Matching Persistence and fix pitch matching bugs
9d2cb25 Add story 4.4 Pitch Matching Persistence and mark as ready-for-dev
dd922d7 Apply code review fixes for story 4.3 and mark as done
f4f8901 Implement story 4.3 Pitch Matching Training UI
```

Convention: story creation commit → implementation commit → code review fixes commit. All recent work in epic 4 is comparison/pitch matching domain — directly relevant foundation for interval mode.

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- `LocalStorageSettings::get_selected_intervals()` is a static method (not trait method) — must call with `::` not `.`

### Completion Notes List

- Created `web/src/interval_codes.rs` with `encode_intervals()`, `decode_intervals()`, and `interval_label()` functions plus 9 unit tests covering round-trip encode/decode for all 25 intervals
- Extracted `interval_label()` from `settings_view.rs` to shared `interval_codes.rs` module
- Replaced static `<A>` links with `<button>` handlers on Start Page that read intervals from localStorage, encode as query params, and navigate with `?intervals=` parameter
- Updated ComparisonView and PitchMatchingView to parse `?intervals=` query param using `use_query_map()`, decode intervals, and pass to session start. When no query param present, defaults to `{Prime/Up}` only (unison mode) — fixes the bug where Settings intervals were used even in regular mode
- Added interval label display (conditionally visible) showing current interval name+direction in both training views
- Updated heading text to reflect mode: "Interval Comparison" vs "Comparison Training", "Interval Pitch Matching" vs "Pitch Matching Training"
- All 293 domain tests pass, trunk build succeeds, cargo clippy zero warnings

### File List

- `web/src/interval_codes.rs` (new) — interval encode/decode/label utilities with unit tests
- `web/src/main.rs` (modified) — added `mod interval_codes`
- `web/src/components/start_page.rs` (modified) — replaced `<A>` links with `<button>` handlers for interval navigation with query params
- `web/src/components/comparison_view.rs` (modified) — query param parsing, interval-based session start, interval label display
- `web/src/components/pitch_matching_view.rs` (modified) — same changes as comparison_view
- `web/src/components/settings_view.rs` (modified) — replaced local `interval_label()` with import from `interval_codes`
- `docs/implementation-artifacts/sprint-status.yaml` (modified) — status updated to in-progress then review
- `docs/implementation-artifacts/5-1-interval-training-mode.md` (modified) — task checkboxes, dev agent record, file list, change log

## Change Log

- 2026-03-04: Implemented story 5.1 Interval Training Mode — added interval code encode/decode utilities, updated Start Page with interval navigation buttons using query params, updated ComparisonView and PitchMatchingView to support interval mode with query param parsing and interval label display, fixed bug where regular mode used Settings intervals instead of unison
