---
title: 'Pitch Comparison Note Gap'
slug: 'pitch-comparison-note-gap'
created: '2026-03-15'
status: 'implementation-complete'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['Rust stable', 'Leptos 0.8', 'leptos-fluent (Fluent i18n)', 'std::time::Duration', 'gloo-timers (TimeoutFuture)']
files_to_modify: ['domain/src/ports.rs', 'web/src/adapters/localstorage_settings.rs', 'web/src/adapters/default_settings.rs', 'web/src/components/pitch_comparison_view.rs', 'web/src/components/settings_view.rs', 'web/locales/en/main.ftl', 'web/locales/de/main.ftl']
code_patterns: ['UserSettings trait in domain/src/ports.rs with implementations in web crate', 'LocalStorageSettings reads f64 via get_f64(key, default) helper', 'Settings UI uses RwSignal<f64> + Stepper component with Callback on_decrement/on_increment', 'SettingsRowDynamic for reactive labels with Signal<String>', 'Training loop polls with TimeoutFuture::new(POLL_INTERVAL_MS) and checks cancelled flag', 'i18n uses Fluent .ftl files with move_tr!/tr! macros and {$param} interpolation']
test_patterns: ['Domain: inline #[cfg(test)] mod tests with test_ prefix', 'cargo test -p domain for domain tests', 'Web: manual browser testing']
---

# Tech-Spec: Pitch Comparison Note Gap

**Created:** 2026-03-15

## Overview

### Problem Statement

In pitch comparison ("Hear & Compare") training, the reference and target notes play back-to-back with no silence between them. Users may want a configurable pause to better process each note before hearing the next.

### Solution

Add a configurable gap (0.0–5.0s, default 0.0, step 0.1) that inserts a silence between the reference note and the target note in pitch comparison training. Surfaced in Settings in the existing Difficulty section with a Stepper control.

### Scope

**In Scope:**
- New `note_gap()` method on `UserSettings` trait returning `std::time::Duration` + implementations
- Gap delay in the web-layer training loop (`pitch_comparison_view.rs`) between reference note completion and target note start
- Stepper in the Difficulty section of `settings_view.rs` with label "Note Gap (Hear & Compare): {value}s"
- i18n strings (English + German) in `.ftl` locale files
- Update help text for Difficulty section to mention note gap
- Domain tests for the new trait method

**Out of Scope:**
- Pitch matching training (has natural gap via slider interaction)
- New domain session state — delay stays in web layer
- Separate "Pitch Comparison" settings section (placed in existing Difficulty section per iOS design)

## Context for Development

### Codebase Patterns

- **Settings trait:** `UserSettings` in `domain/src/ports.rs` (lines 47-53) defines getters returning domain types (`NoteDuration`, `Frequency`, `NoteRange`, `TuningSystem`, `f64`). Implemented by `LocalStorageSettings` (web) and `DefaultSettings` (web/test).
- **localStorage pattern:** `LocalStorageSettings::get_f64(key, default)` reads from browser localStorage. Keys use `peach.` prefix + `snake_case` (e.g. `peach.note_duration`). Values stored as strings, parsed to `f64`.
- **Stepper component:** Reusable `Stepper` at `settings_view.rs:83-115` with `label: Signal<String>`, `on_decrement/on_increment: Callback<()>`, `decrement_disabled/increment_disabled: Signal<bool>`. Used with `SettingsRowDynamic` for reactive labels.
- **Duration stepper pattern:** `note_duration` at lines 532-548 shows the exact pattern: `RwSignal<f64>`, decrement/increment by `(val * 10.0 ± 1.0).round() / 10.0`, clamped to min/max, persisted via `LocalStorageSettings::set()`.
- **Training loop:** `pitch_comparison_view.rs` lines 554-656. Async loop with `TimeoutFuture::new(POLL_INTERVAL_MS)` polling. Cancellation via `cancelled: Rc<Cell<bool>>` checked at each await point. Gap insertion point: after line 592 (`sync()` after `on_reference_note_finished()`) and before line 595 (target note playback).
- **i18n:** Fluent `.ftl` files at `web/locales/{en,de}/main.ftl`. Parameters via `{$value}`. Reactive text: `move_tr!("key")`. One-shot text in closures: `tr!("key")`.
- **Help sections:** Static arrays in `web/src/help_sections.rs`. `SETTINGS_HELP` includes `help-difficulty-body` which currently only describes loudness variation.

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `domain/src/ports.rs` | Add `note_gap()` to `UserSettings` trait (line 52) |
| `web/src/adapters/localstorage_settings.rs` | Implement `note_gap()` reading `peach.note_gap` (after line 96) |
| `web/src/adapters/default_settings.rs` | Implement `note_gap()` returning `Duration::ZERO` (after line 26) |
| `web/src/components/pitch_comparison_view.rs` | Insert gap delay after line 592, before line 594 |
| `web/src/components/settings_view.rs` | Add signal + stepper in Difficulty section (after line 620) |
| `web/locales/en/main.ftl` | Add `note-gap-label` and update `help-difficulty-body` |
| `web/locales/de/main.ftl` | Add German translations |

### Technical Decisions

- **Web-layer gap:** The delay between notes lives in `pitch_comparison_view.rs` async loop, not in the domain session state machine. This matches the existing pattern where timing/polling is a web-layer concern.
- **`std::time::Duration`:** Used as the return type for `UserSettings::note_gap()`. Rust's standard library type — no custom newtype needed since 0.0 is a valid value (unlike `NoteDuration` which clamps to 0.3 minimum).
- **Storage:** `f64` seconds in localStorage with key `peach.note_gap`, consistent with `peach.note_duration` pattern.
- **Default 0.0 seconds:** Preserves current behavior (no gap) for existing users.
- **Placement:** Inside the existing Difficulty section, after the loudness slider, matching iOS layout.
- **Label format:** `"Note Gap (Hear & Compare): {value}s"` / `"Pause (Hören & Vergleichen): {value}s"` — matches iOS final localization.
- **Gap delay pattern:** Polling loop identical to the existing reference note wait (lines 578-584), with `cancelled` check at each poll interval. If `cancelled` is set during the gap, breaks to `'training` label cleanly.

## Implementation Plan

### Tasks

- [x] Task 1: Add `note_gap()` to `UserSettings` trait
  - File: `domain/src/ports.rs`
  - Action: Add `use std::time::Duration;` import. Add `fn note_gap(&self) -> Duration;` to the `UserSettings` trait after line 52 (`vary_loudness`).

- [x] Task 2: Implement `note_gap()` in `LocalStorageSettings`
  - File: `web/src/adapters/localstorage_settings.rs`
  - Action: Add `use std::time::Duration;` import. Add method to the `impl UserSettings for LocalStorageSettings` block:
    ```rust
    fn note_gap(&self) -> Duration {
        Duration::from_secs_f64(Self::get_f64("peach.note_gap", 0.0))
    }
    ```

- [x] Task 3: Implement `note_gap()` in `DefaultSettings`
  - File: `web/src/adapters/default_settings.rs`
  - Action: Add `use std::time::Duration;` import. Add method to the `impl UserSettings for DefaultSettings` block:
    ```rust
    fn note_gap(&self) -> Duration {
        Duration::ZERO
    }
    ```

- [x] Task 4: Update mock in domain port tests
  - File: `domain/src/ports.rs`
  - Action: In the `test_traits_compile_with_mock` test, the `MockPlayer` struct doesn't implement `UserSettings`, so no change needed. However, any other test files that implement `UserSettings` must be updated. Search for `impl UserSettings` across the codebase and update all implementations.

- [x] Task 5: Add i18n strings for note gap
  - File: `web/locales/en/main.ftl`
  - Action: In the `## Settings` section, after `loudness-variation-aria`, add:
    ```
    note-gap-label = Note Gap (Hear & Compare): { $value }s
    note-gap-aria = Note gap for pitch comparison
    ```
    Update `help-difficulty-body` to:
    ```
    help-difficulty-body = **Vary Loudness** changes the volume of notes randomly. This makes training harder but more realistic — in real music, notes are rarely played at the same volume.{"\u000A\u000A"}**Note Gap** adds a pause between the two notes in Hear & Compare training. At zero, notes play back-to-back.
    ```

- [x] Task 6: Add German i18n strings
  - File: `web/locales/de/main.ftl`
  - Action: In the `## Settings` section, after `loudness-variation-aria`, add:
    ```
    note-gap-label = Pause (Hören & Vergleichen): { $value }s
    note-gap-aria = Pause für Tonhöhenvergleich
    ```
    Update `help-difficulty-body` to:
    ```
    help-difficulty-body = **Lautstärke variieren** verändert die Lautstärke der Töne zufällig. Das macht das Training schwieriger, aber realistischer — in echter Musik werden Töne selten gleich laut gespielt.{"\u000A\u000A"}**Pause** fügt eine Stille zwischen den beiden Tönen im Hören & Vergleichen-Training ein. Bei null werden die Töne direkt nacheinander gespielt.
    ```

- [x] Task 7: Add note gap stepper to Settings UI
  - File: `web/src/components/settings_view.rs`
  - Action:
    1. Add signal initialization after `vary_loudness_pct` (line 129):
       ```rust
       let note_gap = RwSignal::new(LocalStorageSettings::get_f64("peach.note_gap", 0.0));
       ```
    2. Add reactive label (near the `duration_label` signal, around line 127-129 area):
       ```rust
       let note_gap_label = Signal::derive(move || {
           tr!("note-gap-label", {"value" => format!("{:.1}", note_gap.get())})
       });
       ```
    3. Inside the Difficulty `SettingsSection` (after the loudness slider `</div>` at line 620), add a divider and stepper row:
       ```rust
       <div class="border-t border-gray-200 dark:border-gray-700"></div>
       <SettingsRowDynamic label=note_gap_label>
           <Stepper
               label=move_tr!("note-gap-aria")
               on_decrement=Callback::new(move |_| {
                   let val = note_gap.get();
                   let new_val = ((val * 10.0 - 1.0).round() / 10.0).max(0.0);
                   note_gap.set(new_val);
                   LocalStorageSettings::set("peach.note_gap", &format!("{new_val:.1}"));
               })
               on_increment=Callback::new(move |_| {
                   let val = note_gap.get();
                   let new_val = ((val * 10.0 + 1.0).round() / 10.0).min(5.0);
                   note_gap.set(new_val);
                   LocalStorageSettings::set("peach.note_gap", &format!("{new_val:.1}"));
               })
               decrement_disabled=Signal::derive(move || note_gap.get() <= 0.0)
               increment_disabled=Signal::derive(move || note_gap.get() >= 5.0)
           />
       </SettingsRowDynamic>
       ```

- [x] Task 8: Insert gap delay in training loop
  - File: `web/src/components/pitch_comparison_view.rs`
  - Action: After line 592 (`sync();` following `on_reference_note_finished()`) and before line 594 (comment `// === PlayingTargetNote phase`), insert the gap delay:
    ```rust
    // === Note gap phase (silence between reference and target) ===
    let note_gap_ms = settings.note_gap().as_millis() as u32;
    if note_gap_ms > 0 {
        let mut gap_elapsed = 0u32;
        while gap_elapsed < note_gap_ms {
            if cancelled.get() {
                break 'training;
            }
            TimeoutFuture::new(POLL_INTERVAL_MS).await;
            gap_elapsed += POLL_INTERVAL_MS;
        }
        if cancelled.get() {
            break;
        }
    }
    ```
  - Notes: `settings` is already `LocalStorageSettings` (line 90), which implements `UserSettings`. The gap is read fresh each iteration so changes in Settings take effect on the next comparison without restarting training.

- [x] Task 9: Add domain tests for `note_gap`
  - File: `domain/src/ports.rs`
  - Action: The existing `test_traits_compile_with_mock` test doesn't use `UserSettings`. No additional domain tests needed beyond verifying compilation. The `UserSettings` trait has no default implementation to test — it's just a method signature. Implementations are tested in the web crate via manual testing.

### Acceptance Criteria

- [ ] AC 1: Given default settings (note_gap = 0.0), when a pitch comparison plays, then note 2 starts immediately after note 1 finishes (no behavioral change from current behavior).
- [ ] AC 2: Given note_gap is set to 1.5s, when a pitch comparison plays, then there is a ~1.5-second silence between note 1 ending and note 2 starting.
- [ ] AC 3: Given note_gap is set to a non-zero value, when the user stops training during the gap (by navigating away, tab hide, or help modal), then the session stops cleanly and no target note plays.
- [ ] AC 4: Given the Settings screen is open, when the user scrolls to the Difficulty section, then a "Note Gap (Hear & Compare)" stepper is visible below the loudness slider, with range 0.0–5.0s and 0.1s step increment.
- [ ] AC 5: Given the user changes the Note Gap setting, when they start a new pitch comparison training, then the configured gap is applied between notes.
- [ ] AC 6: Given the Settings screen help modal is open, when the user views the Difficulty section, then it explains both loudness variation and the note gap setting.
- [ ] AC 7: Given the app language is German, when the Settings screen is displayed, then the note gap label reads "Pause (Hören & Vergleichen): {value}s" and help text is in German.

## Additional Context

### Dependencies

None — purely additive feature, no external crate dependencies.

### Testing Strategy

**Unit tests:**
- `cargo test -p domain` — verifies trait compiles with new method signature (existing `test_traits_compile_with_mock` must still pass, though it tests `NotePlayer`, not `UserSettings`)

**Manual testing:**
- Verify stepper appears in Settings under Difficulty section, below loudness slider
- Set gap to 0.0 and confirm notes play back-to-back (unchanged behavior)
- Set gap to 2.0s and confirm audible silence between reference and target notes
- Stop training during the gap (navigate away, hide tab, open help) and confirm clean transition
- Switch to German and confirm localized labels and help text
- Verify pitch matching training is unaffected by the note_gap setting
- Verify gap setting persists across page reloads (localStorage)

### Notes

- The gap is read from `settings` (a `LocalStorageSettings` instance) on each training loop iteration, so mid-session changes to the setting in another tab would take effect on the next comparison. This is acceptable — the iOS version snapshots at session start, but reading live is simpler here and the edge case is negligible.
- The polling-based delay (50ms intervals) means the actual gap duration has up to ±50ms jitter. This is consistent with the existing note duration timing and is imperceptible.
- Future consideration: if more Hear & Compare-specific settings are added, they can go in the same Difficulty section or a new section could be introduced later.
