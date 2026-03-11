---
title: 'Add German Localization with leptos-fluent'
slug: 'add-german-localization'
created: '2026-03-11'
status: 'ready-for-dev'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['leptos-fluent 0.3', 'fluent-templates 0.9', 'Leptos 0.8 CSR', 'Trunk', 'Python 3 (conversion script)']
files_to_modify:
  - 'web/Cargo.toml'
  - 'web/src/app.rs'
  - 'web/src/help_sections.rs'
  - 'web/src/components/start_page.rs'
  - 'web/src/components/settings_view.rs'
  - 'web/src/components/pitch_comparison_view.rs'
  - 'web/src/components/pitch_matching_view.rs'
  - 'web/src/components/profile_view.rs'
  - 'web/src/components/info_view.rs'
  - 'web/src/components/audio_gate_overlay.rs'
  - 'web/src/components/nav_bar.rs'
  - 'web/src/components/help_content.rs'
  - 'web/src/components/training_stats.rs'
  - 'web/src/components/progress_card.rs'
  - 'web/src/components/progress_sparkline.rs'
  - 'web/src/components/pitch_slider.rs'
  - 'web/src/bridge.rs'
  - 'web/src/adapters/csv_export_import.rs'
  - 'domain/src/training_mode.rs'
  - 'domain/src/types/interval.rs'
  - 'index.html'
  - 'web/locales/en/main.ftl (new)'
  - 'web/locales/de/main.ftl (new)'
  - 'bin/convert_xcstrings_to_ftl.py (new)'
code_patterns:
  - 'move_tr!("key") for reactive translated strings in views'
  - 'static_loader! macro for compile-time .ftl embedding'
  - 'leptos_fluent! macro in app.rs for i18n provider context'
  - 'Domain crate returns i18n keys (String), web layer translates'
  - 'Component label params: &'static str → String for i18n compatibility'
  - 'Markdown-in-ftl rendered via inner_html with md-to-html helper'
test_patterns:
  - 'Domain tests: cargo test -p domain (key strings are plain identifiers)'
  - 'Web i18n: manual browser testing (language switching, string rendering)'
  - 'Conversion script: manual verification of .ftl output'
---

# Tech-Spec: Add German Localization with leptos-fluent

**Created:** 2026-03-11

## Overview

### Problem Statement

All UI strings in peach-web are hardcoded in English as `&'static str` literals across ~15+ component files. German-speaking users get no localized experience, and there is no infrastructure for localization. The iOS app already has complete German translations that can be reused.

### Solution

Integrate `leptos-fluent` with Mozilla Fluent `.ftl` files to provide runtime localization. Convert existing German translations from the iOS `Localizable.xcstrings` via a Python script. Refactor all components to use reactive translation lookups. Add a language switcher in Settings with browser language auto-detection and localStorage persistence.

### Scope

**In Scope:**

- `leptos-fluent` 0.3 + `fluent-templates` 0.9 integration (compiled into WASM at build time)
- English (fallback) + German `.ftl` files with idiomatic kebab-case Fluent keys
- Python conversion script (`bin/convert_xcstrings_to_ftl.py`) from iOS `Localizable.xcstrings` → `.ftl`
- Refactor all component string literals to `move_tr!()` / `tr!()` macro calls
- Component signature changes (`&'static str` labels → reactive `String` or `Signal<String>`)
- Language switcher in Settings screen with localStorage persistence
- Browser language auto-detection (`navigator.languages`) on first visit
- Lightweight markdown-to-HTML helper for formatted translation strings (`**bold**` → `<strong>`)
- Manual German translations for any web-only strings not present in iOS

**Out of Scope:**

- Additional languages beyond English and German
- Full markdown parser (only bold and italic conversion)
- Build-time locale splitting (all translations compiled into single WASM binary)
- RTL language support
- SSR considerations (app is CSR-only)

## Context for Development

### Codebase Patterns

- Leptos context providers are set up in `app.rs` — the i18n provider goes here alongside existing contexts
- Components use `&'static str` for labels (e.g., `SettingsSection`, `SettingsRow`) — these need refactoring to accept dynamic strings
- Settings persistence uses `LocalStorageSettings` adapter with `peach.` prefix keys — language preference managed by `leptos-fluent` via its own localStorage key
- The `UIObserver` pattern does NOT apply here — i18n is a UI-layer concern, not a domain signal
- Domain crate returns i18n key strings (e.g., `"training-mode-hear-compare-single"`), web layer translates via `move_tr!()` — keeps domain crate free of browser dependencies
- Help sections in `help_sections.rs` use `&'static str` constants with markdown formatting — these become `.ftl` keys with markdown content, rendered via `inner_html` after md-to-html conversion

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `web/src/app.rs` | App root — insert i18n provider context here |
| `web/src/help_sections.rs` | All help modal content — 40 strings with markdown |
| `web/src/components/settings_view.rs` | Largest component (~911 lines), all settings labels + language switcher destination |
| `web/src/components/start_page.rs` | Training mode buttons and labels |
| `web/src/components/nav_bar.rs` | Navigation title and aria-labels |
| `web/src/components/info_view.rs` | About page — version, developer, license strings |
| `web/src/components/training_stats.rs` | Stats display — "Latest:", "Best:", trend labels |
| `web/src/components/pitch_comparison_view.rs` | "Higher"/"Lower" buttons, feedback announcements |
| `web/src/components/pitch_matching_view.rs` | Pitch matching training title |
| `web/src/components/profile_view.rs` | Profile page — loading/empty states |
| `web/src/components/audio_gate_overlay.rs` | "Tap to Start Training" |
| `web/src/components/help_content.rs` | HelpModal component — "Done" button |
| `web/src/components/progress_card.rs` | Progress card aria-labels |
| `web/src/components/progress_sparkline.rs` | Sparkline aria-labels |
| `web/src/components/pitch_slider.rs` | Slider aria-label |
| `web/src/bridge.rs` | Storage failure error message |
| `web/src/adapters/csv_export_import.rs` | Import/export error messages |
| `domain/src/training_mode.rs` | Training mode display names → i18n keys |
| `domain/src/types/interval.rs` | Interval display names + direction labels → i18n keys |
| `index.html` | Page title, noscript text |
| `../peach/Peach/Resources/Localizable.xcstrings` | iOS source for German translations |

### Technical Decisions

- **Crate choice:** `leptos-fluent` over `leptos_i18n` — more mature (74 releases), native CLDR plural support, better fit for Fluent format
- **File format:** Mozilla Fluent `.ftl` — designed for translators, handles plurals natively, human-readable
- **Key convention:** Idiomatic Fluent kebab-case identifiers (e.g., `settings`, `training-help`, `current-difficulty`)
- **Loading strategy:** `static_loader!` macro embeds all `.ftl` files into the WASM binary at compile time — no runtime HTTP requests, no Trunk config changes needed
- **Detection:** `navigator.languages` on first visit, localStorage persistence thereafter
- **Manual override:** Language picker in Settings, using `i18n.language.set(lang)`
- **Formatted strings:** Store markdown in `.ftl` files, convert `**bold**` / `*italic*` to `<strong>` / `<em>` at render time via a small helper function, render with `inner_html`
- **iOS string reuse:** One-time Python conversion script extracts EN/DE pairs from `Localizable.xcstrings` JSON, maps iOS format specifiers (`%@`, `%lld`, `%.1f`, positional `%1$lld`) to Fluent variables and `NUMBER()` functions, maps iOS plural variants to Fluent select expressions
- **Domain i18n keys:** Domain crate methods (e.g., `display_name()`, `interval_name()`) return kebab-case i18n key strings instead of English text. Web layer translates via `move_tr!()`. This preserves the domain crate's zero-browser-dependency rule.
- **Component signatures:** `&'static str` label parameters change to `String`. Components that currently accept static labels will accept owned strings from `move_tr!()` calls at the call site.

## Implementation Plan

### Tasks

- [ ] Task 1: Create Python conversion script
  - File: `bin/convert_xcstrings_to_ftl.py` (new)
  - Action: Write a Python 3 script that reads `Localizable.xcstrings` (JSON) and outputs `en/main.ftl` and `de/main.ftl` files. The script must:
    - Parse the JSON structure, iterating over all string entries
    - Generate kebab-case Fluent keys from the English source strings (e.g., `"Settings"` → `settings`, `"Hear & Compare"` → `hear-and-compare`, `"Export Training Data"` → `export-training-data`)
    - Map iOS format specifiers to Fluent variables: `%@` → `{ $arg }`, `%lld` → `{ $count }`, `%.1f` → `{ NUMBER($value, maximumFractionDigits: 1) }`, positional `%1$lld` → named `{ $arg1 }`
    - Map iOS plural variants (`one`/`other`) to Fluent select expressions
    - Preserve multiline strings (iOS `\n\n` → Fluent multiline with blank lines)
    - Preserve markdown formatting (`**bold**`, `*italic*`) as-is in `.ftl` values
    - Skip entries with `extractionState: "stale"` or flag them with a comment
    - Output a mapping file (JSON or comments) linking Fluent keys to iOS source strings for traceability
  - Notes: One-time script. Run it, review output, then manually curate the `.ftl` files. The script provides a starting point — expect manual adjustments for key naming and web-specific strings.

- [ ] Task 2: Create English `.ftl` locale file
  - File: `web/locales/en/main.ftl` (new)
  - Action: Starting from the conversion script output, create the canonical English `.ftl` file containing all user-visible strings from the web app. This includes:
    - All strings identified in the investigation (components, help sections, error messages, aria-labels)
    - Web-only strings not present in iOS (e.g., "Tap to Start Training", "Skip to main content", CSV import dialog text, "Page not found")
    - Idiomatic kebab-case keys grouped by feature area with comments (e.g., `## Start Page`, `## Settings`, `## Help - Comparison`, etc.)
    - Plural forms using Fluent select expressions where applicable
    - Markdown formatting preserved in help text values
  - Notes: This file is the source of truth for all translatable strings. Every user-visible string in the web app must have an entry here.

- [ ] Task 3: Create German `.ftl` locale file
  - File: `web/locales/de/main.ftl` (new)
  - Action: Starting from the conversion script output, create the German `.ftl` file. This includes:
    - All translations available from the iOS `Localizable.xcstrings`
    - German translations for web-only strings — delegate to Michael for any missing translations
    - Same key structure and comments as the English file
  - Notes: After initial generation, Michael reviews and provides missing translations.

- [ ] Task 4: Add dependencies to Cargo.toml
  - File: `web/Cargo.toml`
  - Action: Add `leptos-fluent = "0.3"` and `fluent-templates = { version = "0.9", features = ["fluent"] }` to `[dependencies]`
  - Notes: Verify version compatibility with Leptos 0.8 after adding. Run `cargo check -p web` to confirm.

- [ ] Task 5: Set up i18n provider in app.rs
  - File: `web/src/app.rs`
  - Action:
    - Add `static_loader!` macro call pointing to `./locales` with `fallback_language: "en"`
    - Add `leptos_fluent!` macro inside the app root component, configured with:
      - `locales: "./locales"`
      - `fallback_language: "en"`
      - `languages: { "en": "English", "de": "Deutsch" }`
      - `translations: [TRANSLATIONS]`
      - `sync_html_tag_lang: true`
      - `initial_language_from_navigator: true`
      - `set_language_to_local_storage: true`
      - `initial_language_from_local_storage: true`
    - Ensure the i18n provider wraps all routed content so `move_tr!()` is available in all components
  - Notes: The `leptos_fluent!` macro creates an `I18n` context. All child components can access it via `expect_context::<I18n>()`. Verify that the macro placement doesn't conflict with existing context providers.

- [ ] Task 6: Create markdown-to-HTML helper
  - File: `web/src/i18n_helpers.rs` (new) or inline in an existing utility module
  - Action: Write a function `fn md_to_html(s: &str) -> String` that converts:
    - `**text**` → `<strong>text</strong>`
    - `*text*` → `<em>text</em>`
    - `\n\n` → `<br><br>` (paragraph breaks in help text)
    - Passes all other content through unchanged
  - Notes: Keep it minimal — not a full markdown parser. Only bold, italic, and paragraph breaks. Used for help section content rendered via `inner_html`. Ensure the function escapes HTML entities in the non-markdown portions to prevent XSS (Fluent strings are translator-provided, treat as untrusted).

- [ ] Task 7: Update domain crate — training mode i18n keys
  - File: `domain/src/training_mode.rs`
  - Action: Change `display_name()` method to return i18n key strings:
    - `"Hear & Compare -- Single Notes"` → `"training-mode-hear-compare-single"`
    - `"Hear & Compare -- Intervals"` → `"training-mode-hear-compare-intervals"`
    - `"Tune & Match -- Single Notes"` → `"training-mode-tune-match-single"`
    - `"Tune & Match -- Intervals"` → `"training-mode-tune-match-intervals"`
  - Notes: Update any domain tests that assert on display name values. These are now opaque keys, not English strings.

- [ ] Task 8: Update domain crate — interval and direction i18n keys
  - File: `domain/src/types/interval.rs`
  - Action: Change `display_name()` method to return i18n key strings:
    - `"Prime"` → `"interval-prime"`, `"Minor Second"` → `"interval-minor-second"`, etc. for all 13 intervals
    - Change direction labels: `"Up"` → `"direction-up"`, `"Down"` → `"direction-down"`
    - Keep `short_label()` unchanged — `"P1"`, `"m2"`, etc. are universal music theory notation, not translatable
  - Notes: Update any domain tests that assert on these display names.

- [ ] Task 9: Refactor component signatures
  - Files: `web/src/components/nav_bar.rs`, `web/src/components/settings_view.rs`, `web/src/components/start_page.rs`, `web/src/components/help_content.rs`
  - Action: Change label/title parameters from `&'static str` to `String` (or `impl Into<String>`) in these components:
    - `NavBar`: `title: &'static str` → `title: String`
    - `SettingsSection`: `title: &'static str` → `title: String`
    - `SettingsRow`: `label: &'static str` → `label: String`
    - `Stepper`: `label: &'static str` → `label: String`
    - `TrainingCard`: `label: &'static str` → `label: String`, `aria_label: &'static str` → `aria_label: String`
    - `HelpModal`: `title: &'static str` → `title: String`
  - Notes: Update all call sites to pass `String` values. After this task, call sites can pass `move_tr!()` results directly.

- [ ] Task 10: Refactor help_sections.rs for i18n
  - File: `web/src/help_sections.rs`
  - Action: Replace all hardcoded `&'static str` help section titles and bodies with Fluent key references. Options:
    - Change `HelpSection` to store i18n keys (`title_key: &'static str`, `body_key: &'static str`) instead of content
    - At render time in `help_content.rs`, use `move_tr!(section.title_key)` and `md_to_html(&move_tr!(section.body_key))` with `inner_html`
  - Notes: The help sections contain markdown formatting (`**bold**`). The md-to-html helper (Task 6) handles conversion at render time.

- [ ] Task 11: Localize start_page.rs
  - File: `web/src/components/start_page.rs`
  - Action: Replace all hardcoded strings with `move_tr!()` calls:
    - `"Peach"` → `move_tr!("app-name")`
    - `"Loading sounds…"` → `move_tr!("loading-sounds")`
    - `"Single Notes"` → `move_tr!("single-notes")`
    - `"Intervals"` → `move_tr!("intervals")`
    - `"Hear & Compare"` → `move_tr!("hear-and-compare")`
    - `"Tune & Match"` → `move_tr!("tune-and-match")`
    - All aria-labels similarly
    - `"Info"`, `"Profile"`, `"Settings"` nav icon labels
    - Error notification for sound loading failure
  - Notes: Training mode keys from domain (`display_name()`) are now i18n keys — use `move_tr!()` to translate them.

- [ ] Task 12: Localize settings_view.rs and add language switcher
  - File: `web/src/components/settings_view.rs`
  - Action:
    - Replace all hardcoded section titles, labels, button text, error messages, and dialog text with `move_tr!()` calls
    - Add a "Language" `SettingsRow` (or `SettingsSection`) in the Settings view with a dropdown/selector showing available languages (`"English"`, `"Deutsch"`)
    - Wire the selector to `i18n.language.set(lang)` on change
    - The current language is read from `i18n.language.get()`
  - Notes: This is the largest component. Systematic replacement — go section by section (Pitch Range, Intervals, Sound, Difficulty, Data). The language switcher can use a simple `<select>` element iterating over `i18n.languages`.

- [ ] Task 13: Localize remaining components
  - Files: `web/src/components/pitch_comparison_view.rs`, `web/src/components/pitch_matching_view.rs`, `web/src/components/profile_view.rs`, `web/src/components/info_view.rs`, `web/src/components/audio_gate_overlay.rs`, `web/src/components/training_stats.rs`, `web/src/components/progress_card.rs`, `web/src/components/progress_sparkline.rs`, `web/src/components/pitch_slider.rs`
  - Action: Replace all hardcoded strings with `move_tr!()` calls in each component:
    - `pitch_comparison_view.rs`: "Higher", "Lower", "Correct", "Incorrect", training title
    - `pitch_matching_view.rs`: training title
    - `profile_view.rs`: "Profile", "Loading…", empty state message
    - `info_view.rs`: "Done", "Version", "Developer", "Project", "License:", "Copyright:", acknowledgments
    - `audio_gate_overlay.rs`: "Tap to Start Training"
    - `training_stats.rs`: "Latest:", "Best:", trend labels ("Improving", "Stable", "Declining"), "cents"
    - `progress_card.rs`: aria-label text with translated trend labels
    - `progress_sparkline.rs`: aria-label text
    - `pitch_slider.rs`: "Pitch adjustment" aria-label
  - Notes: For strings with variables (e.g., "Latest: X ¢"), use `move_tr!("latest-value", {"value" => x})`.

- [ ] Task 14: Localize error messages in bridge and adapters
  - Files: `web/src/bridge.rs`, `web/src/adapters/csv_export_import.rs`
  - Action: Replace hardcoded error message strings with `tr!()` calls:
    - `bridge.rs`: storage failure message
    - `csv_export_import.rs`: file validation errors, import status messages
  - Notes: These are non-reactive contexts (not inside views), so `tr!()` (non-reactive) may be appropriate. Verify that the i18n context is accessible from these modules — if not, pass translated strings from the calling component.

- [ ] Task 15: Localize index.html
  - File: `index.html`
  - Action: The `<title>Peach</title>` is static HTML, not Leptos-rendered. Options:
    - Keep "Peach" as-is (it's a brand name, same in all languages)
    - The noscript message can remain English (edge case, non-critical)
  - Notes: `leptos-fluent` with `sync_html_tag_lang: true` will update the `<html lang>` attribute reactively. The page title can be updated at runtime via `document.set_title()` if needed, but "Peach" is language-neutral.

- [ ] Task 16: Verify build and test
  - Action:
    - Run `cargo clippy --workspace` — no new warnings
    - Run `cargo test -p domain` — all domain tests pass (with updated key assertions)
    - Run `trunk serve` — app builds and loads successfully
    - Manual browser testing: verify English rendering, switch to German in Settings, verify all strings update

### Acceptance Criteria

- [ ] AC 1: Given the app loads in a browser with `Accept-Language: de`, when the user opens any page, then all UI text is displayed in German.
- [ ] AC 2: Given the app loads in a browser with `Accept-Language: en` (or any unsupported language), when the user opens any page, then all UI text is displayed in English (fallback).
- [ ] AC 3: Given the user is on the Settings page, when they change the language selector from English to Deutsch, then all visible UI text across the entire app updates to German without page reload.
- [ ] AC 4: Given the user has selected German in Settings, when they close and reopen the app, then German is still the active language (persisted in localStorage).
- [ ] AC 5: Given a help modal is open (Settings Help, Comparison Help, Pitch Matching Help, Info), when the language is German, then all help text is displayed in German with correct bold/italic formatting rendered as HTML.
- [ ] AC 6: Given the user is on the Start Page, when the language is German, then training mode names ("Hören & Vergleichen", "Stimmen & Treffen"), section headers ("Einzeltöne", "Intervalle"), and navigation labels are all in German.
- [ ] AC 7: Given the user is in a training view, when the language is German, then the training title, button labels ("Höher"/"Tiefer"), feedback text ("Richtig"/"Falsch"), and stats labels are in German.
- [ ] AC 8: Given the user opens the Profile page in German, when training data exists, then progress cards display German trend labels ("Verbessernd", "Stabil", "Nachlassend") and German unit labels ("Cent").
- [ ] AC 9: Given the conversion script is run against the iOS `Localizable.xcstrings`, when it completes, then it produces valid `.ftl` files with correct Fluent syntax for both English and German.
- [ ] AC 10: Given `cargo clippy --workspace` is run after all changes, when the check completes, then there are no new warnings.
- [ ] AC 11: Given `cargo test -p domain` is run after domain key changes, when tests complete, then all tests pass.

## Additional Context

### Dependencies

- `leptos-fluent` 0.3 — must be compatible with Leptos 0.8 CSR (verified in research)
- `fluent-templates` 0.9 — compile-time `.ftl` loader
- Python 3 — for the conversion script (no external packages required, uses only stdlib `json`)
- iOS `Localizable.xcstrings` file — located at `../peach/Peach/Resources/Localizable.xcstrings`

### Testing Strategy

- **Domain unit tests:** Update assertions in `training_mode.rs` and `interval.rs` tests to expect i18n key strings instead of English display names. Run via `cargo test -p domain`.
- **Conversion script:** Run script, manually inspect output `.ftl` files for correct syntax, key naming, variable mapping, and plural forms.
- **Build verification:** `cargo clippy --workspace` clean, `trunk build` succeeds.
- **Manual browser testing:**
  - Load app with browser language set to German → verify German UI
  - Load app with browser language set to English → verify English UI
  - Change language in Settings → verify all visible text updates reactively
  - Close and reopen app → verify language preference persisted
  - Open each help modal → verify markdown formatting renders correctly (bold text)
  - Navigate to all routes (Start, Settings, Profile, Info, Comparison Training, Pitch Matching) → verify no untranslated strings
  - Test with browser language set to an unsupported language (e.g., French) → verify English fallback

### Notes

- **High-risk area:** The component signature refactoring (Task 9) touches many files and may cause cascading compilation errors. Recommend doing this as a separate commit and verifying the build before proceeding with string replacement.
- **Partial coverage expected from iOS:** Not all web app strings exist in iOS (e.g., "Tap to Start Training", CSV import dialog text, "Skip to main content"). Michael will provide German translations for these gaps.
- **Future contributors:** To add a new language, a contributor copies `web/locales/en/main.ftl` to `web/locales/{lang}/main.ftl`, translates all values, and adds the language code to the `languages` map in `app.rs`. This should be documented in a brief section in the project README or a `CONTRIBUTING.md` note.
- **Stale iOS strings:** Some iOS strings have `extractionState: "stale"` — the conversion script should flag these so they can be reviewed rather than blindly included.
- **`inner_html` security:** The md-to-html helper must HTML-escape all content before applying markdown-to-HTML conversion, to prevent XSS from malicious translation strings. Since translations are developer-controlled (committed to repo), this is low risk but good hygiene.
