# Story 8.3: SoundFont Loading UX

Status: ready-for-dev

## Story

As a user,
I want the app to wait for my chosen SoundFont to load before I can start training,
so that I always hear the sound I selected instead of an unexpected oscillator fallback.

## Acceptance Criteria

1. When the user's `sound_source` setting starts with `"sf2:"`, the Start Page training buttons are disabled (visually and functionally) until SoundFont assets (Phase 1) have finished loading
2. A loading indicator is visible on the Start Page while SoundFont assets are being fetched, clearly communicating "please wait" to the user
3. Once Phase 1 fetch completes successfully, training buttons become enabled and the loading indicator disappears
4. If Phase 1 fetch fails, the app falls back to oscillator, enables training buttons, and shows a brief non-blocking notification explaining that the selected sound could not be loaded
5. When the user's `sound_source` is `"oscillator:sine"` (or any non-SF2 value), training buttons are enabled immediately with no loading gate
6. On Firefox (and all browsers), the app shell and Start Page render immediately — the SF2 fetch does not block initial page rendering
7. All existing training functionality continues to work — no regressions

## Tasks / Subtasks

- [ ] Task 1: Add explicit SoundFont loading status signal (AC: 1, 2, 3, 4)
  - [ ] Define a `SoundFontLoadStatus` enum in `app.rs`: `NotNeeded`, `Fetching`, `Ready`, `Failed(String)`
  - [ ] Create `RwSignal<SoundFontLoadStatus>` in `App` component, provide via context
  - [ ] Set initial value based on user's `sound_source` setting: if starts with `"sf2:"` → `Fetching`, else → `NotNeeded`
  - [ ] In `fetch_worklet_assets()` success path: set signal to `Ready`
  - [ ] In `fetch_worklet_assets()` error path: set signal to `Failed(error_message)`

- [ ] Task 2: Gate training buttons on SoundFont readiness (AC: 1, 5)
  - [ ] In Start Page component, consume `SoundFontLoadStatus` from context
  - [ ] Derive a `can_start_training` memo: `true` when status is `NotNeeded` or `Ready`, `false` when `Fetching`
  - [ ] For `Failed`: set `can_start_training = true` (fallback to oscillator is acceptable)
  - [ ] Apply `disabled` attribute and `aria-disabled="true"` to all training start buttons when `can_start_training` is `false`
  - [ ] Prevent navigation on click when disabled (not just visual — block the router navigation)

- [ ] Task 3: Add loading indicator to Start Page (AC: 2, 3)
  - [ ] When status is `Fetching`, show a loading indicator near or on the training buttons
  - [ ] Use a simple text-based indicator (e.g., "Loading sounds..." with a CSS animation) — no heavy spinner library
  - [ ] The indicator disappears reactively when status transitions away from `Fetching`
  - [ ] Ensure the indicator is accessible: `role="status"` and `aria-live="polite"` so screen readers announce it

- [ ] Task 4: Show notification on SoundFont load failure (AC: 4)
  - [ ] When status transitions to `Failed`, display a non-blocking notification on the Start Page: "Selected sound could not be loaded. Using default sound."
  - [ ] Auto-dismiss after 5 seconds (reuse the existing `storage_error` / `audio_error` notification pattern from stories 8.2 / 6.3)
  - [ ] Log the actual error at `warn` level for debugging

- [ ] Task 5: Ensure SF2 fetch does not block rendering (AC: 6)
  - [ ] Verify that `fetch_worklet_assets()` in `app.rs` runs via `spawn_local()` (non-blocking async)
  - [ ] Verify the app shell, router, and Start Page render before the fetch completes
  - [ ] Test on Firefox specifically — the original bug report says the app doesn't show until SF2 loads
  - [ ] If Firefox blocking is caused by the fetch itself (unlikely with `spawn_local`), investigate whether the SF2 response handling (ArrayBuffer conversion) is blocking the main thread and break it up if needed
  - [ ] If the issue is that the fetch triggers layout reflows or large memory allocation that freezes the UI, consider adding a `yield_now()` (via `gloo_timers::future::TimeoutFuture::new(0).await`) after the fetch before processing the response

- [ ] Task 6: Manual browser testing (AC: 7)
  - [ ] Test Chrome with cleared cache: verify loading indicator shows, buttons disabled, then enabled after load
  - [ ] Test Chrome with cached SF2: verify near-instant enable (loading indicator may flash briefly or not appear)
  - [ ] Test Firefox with cleared cache: verify app renders immediately, loading indicator shows, buttons gate correctly
  - [ ] Test with `sound_source = "oscillator:sine"`: verify no loading gate, buttons enabled immediately
  - [ ] Test SF2 fetch failure (e.g., rename file temporarily): verify fallback notification and oscillator playback works
  - [ ] Test all three training modes still work correctly after loading completes

## Dev Notes

### Current Behavior (Problem)

The two-phase worklet init from story 8.2 works well for *audio reliability* but has a UX gap:

1. **Phase 1** (`fetch_worklet_assets()`) runs async at app mount via `spawn_local()` — fetches and compiles WASM module + fetches the 31MB SF2 file
2. **Training views** check `worklet_assets.get_untracked()` at training start. If Phase 1 hasn't finished yet, `worklet_assets` is `None`, so `connect_worklet()` is skipped
3. `create_note_player()` in `note_player.rs:82-102` falls through to `OscillatorNotePlayer` when `worklet_bridge` is `None`
4. The user hears an oscillator instead of their selected SoundFont — with no indication that anything is wrong
5. This oscillator fallback persists for the entire training session. The note player is not recreated mid-session.

On Firefox specifically, the SF2 fetch may block rendering of the entire app (reported by user), though the `spawn_local()` pattern should be non-blocking. This needs investigation — it may be related to how Firefox handles large `fetch()` responses or `ArrayBuffer` conversion on the main thread.

### Implementation Strategy

The fix is a **loading gate on the Start Page**, not in the training views:

1. Add an explicit `SoundFontLoadStatus` signal that reflects Phase 1 progress
2. The Start Page reads this signal and disables training buttons while `Fetching`
3. When Phase 1 completes (or fails), buttons become enabled
4. This ensures the user *never* starts training before their SoundFont is ready

This approach is simpler and more correct than trying to show a loading state inside the training view or recreating the note player mid-session.

### Key Code Locations

| File | What to change |
|---|---|
| `web/src/app.rs` | Add `SoundFontLoadStatus` enum and signal, update `fetch_worklet_assets()` to set status |
| `web/src/components/start_page.rs` (or equivalent) | Consume status signal, gate buttons, show loading indicator |
| `web/src/adapters/note_player.rs` | No changes needed — existing fallback logic is correct |
| Training views | No changes needed — Phase 2 connect still happens at training start |

### SoundFontLoadStatus Enum

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum SoundFontLoadStatus {
    NotNeeded,       // User selected oscillator, no SF2 fetch needed
    Fetching,        // Phase 1 in progress
    Ready,           // Phase 1 complete, assets available
    Failed(String),  // Phase 1 failed, will fall back to oscillator
}
```

### Reading User's Sound Source Setting

The `sound_source` setting is stored in localStorage under key `peach.sound_source`. Read it with `LocalStorageSettings::get_string("peach.sound_source")`. Default is `"oscillator:sine"`. SF2 values have format `"sf2:<bank>:<preset>"`.

### Existing Patterns to Reuse

- **Notification pattern**: `audio_error: RwSignal<Option<String>>` with auto-dismiss Effect (5s) from story 8.2 training views. Same pattern can be used on Start Page for the failure notification.
- **Disabled button pattern**: Check existing button disabled patterns in the codebase. Use both `disabled` HTML attribute and conditional `class` for visual styling.
- **Context signal pattern**: `provide_context()` / `use_context()` — already used for `worklet_bridge`, `worklet_assets`, `sf2_presets`.

### Architecture Constraints

- Domain crate has no browser code — `SoundFontLoadStatus` lives in `web/` crate only
- Signal setters (`.set()`) for domain state go through `UIObserver` bridge — but this is a UI-only signal, so direct `.set()` in `app.rs` is fine
- No `tokio` — async via `wasm-bindgen-futures::spawn_local()` only
- Accessibility: loading indicator needs `role="status"` + `aria-live="polite"`

### Previous Story Intelligence (8.2)

- `fetch_worklet_assets()` is in `app.rs` lines ~221-267, runs via `spawn_local()` at mount
- `WorkletAssets` struct holds pre-compiled WASM module + raw SF2 buffer
- `worklet_assets: RwSignal<Option<Rc<WorkletAssets>>>` transitions from `None` to `Some` on success
- Phase 2 `connect_worklet()` is called from training views after `ensure_running()`
- Error notification pattern: `RwSignal<Option<String>>` + Effect with `set_timeout` for auto-dismiss
- `audio_error` signal renders a red notification banner at bottom of training views

### Git Intelligence

Recent commits follow pattern: imperative mood, story reference in parentheses.
- `3cfe282` Fix audio playback reliability: ensure_running, two-phase worklet init, soft onstatechange (story 8.2)
- Story 8.2 introduced `WorkletAssets`, `fetch_worklet_assets()`, `connect_worklet()` — all in `app.rs`

### Project Structure Notes

- Start Page component: check `web/src/components/` for the start/home page component
- All components use `#[component]` macro with `PascalCase` function names
- CSS: check existing patterns — likely inline styles or a CSS file in `web/assets/`
- Alignment with project conventions: loading indicator should be minimal, non-intrusive — "training, not testing" philosophy

### References

- [Source: docs/implementation-artifacts/8-2-audio-playback-reliability-fix.md] — Two-phase worklet init, WorkletAssets, connect_worklet
- [Source: docs/implementation-artifacts/8-1-audio-playback-reliability-research.md] — Root cause analysis, SoundFont fallback behavior
- [Source: web/src/app.rs#fetch_worklet_assets] — Phase 1 fetch implementation
- [Source: web/src/adapters/note_player.rs#create_note_player] — SF2 vs oscillator decision logic
- [Source: docs/project-context.md] — Coding conventions, signal patterns, error handling
- [Source: docs/planning-artifacts/architecture.md] — Audio architecture, SoundFont loading design

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
