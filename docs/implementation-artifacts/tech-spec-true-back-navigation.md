---
title: 'True back navigation with in-app history guard'
type: 'feature'
created: '2026-03-30'
status: 'done'
baseline_commit: 'a0ca58e'
context: []
---

# True back navigation with in-app history guard

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The back button on every view always navigates to the start page (`/`), ignoring the user's actual navigation history. Users who go Settings → Profile expect back to return to Settings, not start.

**Approach:** Track in-app navigation depth via a context counter. Increment on each in-app navigation, decrement on back. When depth > 0, back uses `window.history.back()`; when depth == 0 (entered from external link or direct URL), back navigates to `/`. Training views continue firing their stop-session handlers before navigating.

## Boundaries & Constraints

**Always:** Training `on_back` handlers must run before any navigation. The counter must only track navigations within the app (not external referrers). `navigate()` (not `base_href`) must be used for programmatic navigation since it handles the base path internally.

**Ask First:** Changes to the escape-key behavior in InfoView (currently hardcoded to `/`).

**Never:** Use `popstate` listener approaches (fragile with SPA routers). Expose browser history outside the app. Break deep-link behavior — direct URL entry must work as before.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Normal in-app nav | Start → Settings → Profile, press back | Goes to Settings | N/A |
| Deep chain | Start → Training → Settings → Profile, back×3 | Profile→Settings→Training→Start | N/A |
| External entry | User opens /settings from bookmark, press back | Goes to `/` (depth=0) | N/A |
| Training stop | Start → PitchDiscrimination, press back | Stops session, then goes to `/` via history.back() | N/A |
| Refresh mid-chain | Start → Settings → (refresh) → press back | Goes to `/` (depth reset to 0) | N/A |

</frozen-after-approval>

## Code Map

- `web/src/app.rs` -- Provide `NavDepth` context (counter signal), `go_back()`, `nav_push()`, `popstate` listener to reset depth
- `web/src/components/nav_bar.rs` -- Replace `back_href` with `show_back: bool`, back button calls `go_back()`
- `web/src/components/pitch_discrimination_view.rs` -- Update on_back to use `go_back()`; `interrupt_and_navigate` uses `navigate("/")`
- `web/src/components/pitch_matching_view.rs` -- Same pattern
- `web/src/components/rhythm_offset_detection_view.rs` -- Same pattern
- `web/src/components/continuous_rhythm_matching_view.rs` -- Same pattern
- `web/src/components/settings_view.rs` -- Remove `back_href`, use `show_back=true`
- `web/src/components/profile_view.rs` -- Same
- `web/src/components/info_view.rs` -- Update escape key and button to use `go_back()`
- `web/Cargo.toml` -- Add `History` to web_sys features

## Tasks & Acceptance

**Execution:**
- [ ] `web/Cargo.toml` -- Add `"History"` to web_sys features
- [ ] `web/src/app.rs` -- Add `NavDepth(RwSignal<u32>)` newtype, provide as context. Add `go_back()`: if depth > 0, decrement + `history.back()`; else `navigate("/")`. Add `nav_push()`: increment depth. Add `popstate` listener in `App` that resets depth to 0 on browser-initiated back/forward (since we cannot know if the target page was in-app).
- [ ] `web/src/components/nav_bar.rs` -- Replace `back_href: Option<String>` with `show_back: bool`. Back button becomes `<button>` calling `on_back` then `go_back()`. `on_back` type: `Callback<()>`. `NavIconButton` `<A>` variant calls `nav_push()` on click.
- [ ] `web/src/components/settings_view.rs` -- Replace `back_href=base_href("/")` with `show_back=true`
- [ ] `web/src/components/profile_view.rs` -- Same
- [ ] `web/src/components/info_view.rs` -- Update back button and escape handler to use `go_back()`
- [ ] `web/src/components/pitch_discrimination_view.rs` -- on_back: stop logic only (no navigate). `interrupt_and_navigate`: stop logic + `navigate("/")` (NOT `go_back()` — interrupts are system events, not user back navigation).
- [ ] `web/src/components/pitch_matching_view.rs` -- Same pattern
- [ ] `web/src/components/rhythm_offset_detection_view.rs` -- Same pattern
- [ ] `web/src/components/continuous_rhythm_matching_view.rs` -- Same pattern
- [ ] `web/src/components/start_page.rs` -- `TrainingCard` `<A>` links call `nav_push()` on click

**Acceptance Criteria:**
- Given user navigated Start → Training → Profile (via icon), when pressing back on Profile, then Training is shown
- Given user opened /profile from a bookmark, when pressing back, then start page is shown
- Given user is in a training session and presses app back button, then session stops and previous page is shown
- Given user refreshed the page, when pressing back, then start page is shown (depth resets to 0)
- Given user presses browser back button, then depth resets to 0 (safe fallback for next app-back press)

## Design Notes

**NavDepth counter + popstate reset:** A simple counter tracks in-app depth. On browser-initiated back/forward (`popstate` event), reset depth to 0. This is conservative — the next app-back will go to `/` — but it's safe and avoids complex history tracking. The `popstate` listener must distinguish app-initiated `history.back()` (from `go_back()`) vs browser-initiated: use a flag `NavBackInProgress` set before calling `history.back()` and cleared in the `popstate` handler.

**`interrupt_and_navigate` must use `navigate("/")`, not `go_back()`:** System-initiated interrupts (visibility change, audio context closed) are not user back-navigations. They should always go to start page regardless of depth. This avoids depth/history misalignment when interrupts fire from secondary pages.

**KEEP from v1:** `NavBar` `show_back` prop, `NavIconButton` calling `nav_push()`, `Callback<()>` type for `on_back`, core `NavDepth`/`go_back()`/`nav_push()` structure.

## Verification

**Commands:**
- `cargo clippy --workspace` -- expected: no warnings
- `cargo test -p domain` -- expected: all pass (no domain changes)
- `trunk serve` -- expected: manual verification of back navigation scenarios

## Spec Change Log

- **v2 (review loop 1):** Triggered by: (1) browser back button doesn't decrement NavDepth — depth drifts permanently; (2) `interrupt_and_navigate` calling `go_back()` causes depth/history misalignment when fired from secondary pages. Amendments: added `popstate` listener with `NavBackInProgress` flag to reset depth on browser-initiated navigation; changed `interrupt_and_navigate` to use `navigate("/")` instead of `go_back()`; updated AC1 example to use achievable navigation path; added AC5 for browser back behavior. Known-bad state avoided: permanent depth counter drift after any browser back/forward use. KEEP: NavBar `show_back` prop, `NavIconButton` calling `nav_push()`, `Callback<()>` type, core `NavDepth`/`go_back()`/`nav_push()` structure.
