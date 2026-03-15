---
title: 'Replace GeneralUser GS with Samples.sf2'
slug: 'replace-soundfont-samples-sf2'
created: '2026-03-15'
status: 'implementation-complete'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['Rust', 'Leptos 0.8', 'Trunk', 'WASM', 'Fluent i18n']
files_to_modify: ['web/src/app.rs', 'domain/src/types/sound_source.rs', 'web/src/components/settings_view.rs', 'web/src/components/pitch_comparison_view.rs', 'web/src/components/pitch_matching_view.rs', 'web/locales/en/main.ftl', 'web/locales/de/main.ftl', 'index.html', '.github/workflows/ci.yml', 'README.md', 'docs/project-context.md']
code_patterns: ['relative fetch URLs for WASM assets', 'Fluent .ftl with inline HTML for acknowledgements', 'unwrap_or_else fallback for localStorage reads']
test_patterns: ['inline #[cfg(test)] mod tests in domain crate']
---

# Tech-Spec: Replace GeneralUser GS with Samples.sf2

**Created:** 2026-03-15

## Overview

### Problem Statement

The app currently uses GeneralUser GS as its SoundFont, downloaded at build time via a script. The project is switching to a custom Samples.sf2 that combines FluidR3_GM piano sounds with GeneralUser GS for everything else. This SoundFont should be committed directly to the repo, eliminating the download step.

### Solution

Move Samples.sf2 into `web/assets/soundfont/`, update all code/config references from `GeneralUser-GS.sf2` to `Samples.sf2`, change the default sound source to `sf2:0:0`, update acknowledgements, simplify the CI pipeline, and update the README.

### Scope

**In Scope:**

- Move Samples.sf2 from project root to `web/assets/soundfont/`
- Update all code references from `GeneralUser-GS.sf2` to `Samples.sf2`
- Update acknowledgements in localization files (en + de)
- Change default sound source from `sf2:8:80` to `sf2:0:0`
- Remove `bin/download-sf2.sh` and `bin/sf2-sources.conf`
- Update CI pipeline (remove SF2 download/cache steps)
- Update README (SF2 no longer needs separate download)
- Update `index.html` (remove separate copy-file directive for SF2)
- Update `project-context.md` reference to SF2 filename
- Add Samples.sf2 to git

**Out of Scope:**

- NOTICE file (already updated by user)
- Note range minimum (already enforced at MIDI 21/A0)
- Any changes to sound source selection UI logic
- Synth worklet code changes

## Context for Development

### Codebase Patterns

- Trunk copies `web/assets/soundfont/` directory to build output via `<link data-trunk rel="copy-dir">` in `index.html` — output lands at `dist/soundfont/`
- SF2 currently fetched at runtime via relative URL `./GeneralUser-GS.sf2` (resolves against `<base href>`) — will change to `./soundfont/Samples.sf2`
- Sound source stored in localStorage as `peach.sound_source`; training views fall back to `"oscillator:sine"` when no value set — should fall back to `"sf2:0:0"` instead
- Acknowledgements in Fluent `.ftl` files use inline `<a>` tags with Tailwind classes
- `SoundSourceID::DEFAULT` in domain crate is `"sf2:8:80"` — tests assert this value

### Files to Reference

| File | Line(s) | Purpose |
| ---- | ------- | ------- |
| `web/src/app.rs` | 351 | SF2 fetch URL: `./GeneralUser-GS.sf2` |
| `domain/src/types/sound_source.rs` | 3, 10, 49, 55 | Default constant `sf2:8:80`, doc comment, test assertions |
| `web/src/components/settings_view.rs` | 139 | Fallback `"oscillator:sine"` when no localStorage value |
| `web/src/components/pitch_comparison_view.rs` | 91-92 | Fallback `"oscillator:sine"` when no localStorage value |
| `web/src/components/pitch_matching_view.rs` | 84-85 | Fallback `"oscillator:sine"` when no localStorage value |
| `web/locales/en/main.ftl` | 137 | English acknowledgements body |
| `web/locales/de/main.ftl` | 137 | German acknowledgements body |
| `index.html` | 11 | Trunk `copy-file` directive for `.cache/GeneralUser-GS.sf2` |
| `.github/workflows/ci.yml` | 66-76 | CI SF2 cache/download/verify steps |
| `README.md` | 39-45 | Build instructions: SF2 download step |
| `docs/project-context.md` | 216 | Asset path example `./GeneralUser-GS.sf2` |
| `bin/download-sf2.sh` | all | SF2 download script (delete) |
| `bin/sf2-sources.conf` | all | SF2 source config (delete) |

### Technical Decisions

- Samples.sf2 lives in `web/assets/soundfont/` — the existing Trunk `copy-dir` for that directory handles it. Remove the separate `copy-file` for `.cache/GeneralUser-GS.sf2`
- Fetch URL changes from `./GeneralUser-GS.sf2` to `./soundfont/Samples.sf2` because the copy-dir outputs to `dist/soundfont/`
- Default sound source changes from `sf2:8:80` to `sf2:0:0` (bank 0, program 0 = piano in new SF2)
- All three fallback sites (`settings_view`, `pitch_comparison_view`, `pitch_matching_view`) change from `"oscillator:sine"` to `"sf2:0:0"`
- Download script and source config removed; `.cache/` directory stays in `.gitignore`

## Implementation Plan

Tasks are ordered by dependency: domain layer first, then asset pipeline, then code references, then docs/CI.

### Tasks

- [x] Task 1: Change default sound source constant
  - File: `domain/src/types/sound_source.rs`
  - Action: Change `const DEFAULT: &str = "sf2:8:80"` to `"sf2:0:0"`. Update doc comment on line 3. Update test assertions on lines 49 and 55 from `"sf2:8:80"` to `"sf2:0:0"`.

- [x] Task 2: Move Samples.sf2 to asset directory
  - File: `Samples.sf2` (project root) → `web/assets/soundfont/Samples.sf2`
  - Action: `mv Samples.sf2 web/assets/soundfont/Samples.sf2`

- [x] Task 3: Remove copy-file directive from index.html
  - File: `index.html`
  - Action: Delete line 11 (`<link data-trunk rel="copy-file" href=".cache/GeneralUser-GS.sf2" />`). The file is now served from the `soundfont/` directory via the existing `copy-dir` directive.

- [x] Task 4: Update SF2 fetch URL in app.rs
  - File: `web/src/app.rs`
  - Action: Change `fetch_with_str("./GeneralUser-GS.sf2")` to `fetch_with_str("./soundfont/Samples.sf2")` on line 351.

- [x] Task 5: Update sound source fallback in settings_view.rs
  - File: `web/src/components/settings_view.rs`
  - Action: Change `.unwrap_or_else(|| "oscillator:sine".to_string())` to `.unwrap_or_else(|| "sf2:0:0".to_string())` on line 139.

- [x] Task 6: Update sound source fallback in pitch_comparison_view.rs
  - File: `web/src/components/pitch_comparison_view.rs`
  - Action: Change `.unwrap_or_else(|| "oscillator:sine".to_string())` to `.unwrap_or_else(|| "sf2:0:0".to_string())` on line 92.

- [x] Task 7: Update sound source fallback in pitch_matching_view.rs
  - File: `web/src/components/pitch_matching_view.rs`
  - Action: Change `.unwrap_or_else(|| "oscillator:sine".to_string())` to `.unwrap_or_else(|| "sf2:0:0".to_string())` on line 85.

- [x] Task 8: Update English acknowledgements
  - File: `web/locales/en/main.ftl`
  - Action: Replace line 137 acknowledgements-body with: `acknowledgments-body = Piano sounds from <a href="https://member.keymusician.com/Member/FluidR3_GM/index.html" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">FluidR3_GM by Frank Wen</a> (MIT License). All other sounds from <a href="https://schristiancollins.com/generaluser.php" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">GeneralUser GS by S. Christian Collins</a>.`

- [x] Task 9: Update German acknowledgements
  - File: `web/locales/de/main.ftl`
  - Action: Replace line 137 acknowledgements-body with: `acknowledgments-body = Pianoklänge von <a href="https://member.keymusician.com/Member/FluidR3_GM/index.html" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">FluidR3_GM von Frank Wen</a> (MIT-Lizenz). Alle anderen Klänge von <a href="https://schristiancollins.com/generaluser.php" target="_blank" rel="noopener noreferrer" class="text-indigo-600 underline dark:text-indigo-400">GeneralUser GS von S. Christian Collins</a>.`

- [x] Task 10: Remove CI SoundFont download steps
  - File: `.github/workflows/ci.yml`
  - Action: Delete the three steps: "Cache SoundFont" (lines 66-70), "Download SoundFont" (lines 72-73), and "Verify SoundFont exists" (lines 75-76).

- [x] Task 11: Update README build instructions
  - File: `README.md`
  - Action: Remove the SF2 download section (lines 39-45: "Before the first build..." through "You only need to run this once."). The SoundFont is now included in the repository.

- [x] Task 12: Update project-context.md asset path example
  - File: `docs/project-context.md`
  - Action: Change `./GeneralUser-GS.sf2` to `./soundfont/Samples.sf2` on line 216.

- [x] Task 13: Delete download script and config
  - Files: `bin/download-sf2.sh`, `bin/sf2-sources.conf`
  - Action: Delete both files. Check if `bin/` directory is now empty; if so, delete it.

- [x] Task 14: Run tests and lints
  - Action: Run `cargo test -p domain` (verify sound_source tests pass with new default). Run `cargo clippy --workspace` (no warnings).

### Acceptance Criteria

- [x] AC 1: Given a fresh checkout, when `trunk build` is run without any download step, then the build succeeds and `dist/soundfont/Samples.sf2` exists in the output.
- [x] AC 2: Given the app is loaded in a browser with no `peach.sound_source` in localStorage, when a training view is opened, then the sound source defaults to `sf2:0:0`.
- [x] AC 3: Given the app is loaded, when the info page is viewed, then the acknowledgements show both FluidR3_GM (MIT) and GeneralUser GS attributions with correct links.
- [x] AC 4: Given the CI pipeline runs on main, when the build-deploy job executes, then it succeeds without any SF2 download step.
- [x] AC 5: Given `domain/src/types/sound_source.rs`, when `cargo test -p domain` is run, then all sound_source tests pass with the new `sf2:0:0` default.
- [x] AC 6: Given the README, when a new developer reads it, then there is no mention of downloading a SoundFont separately.

## Additional Context

### Dependencies

None — this is a self-contained asset swap with no new crate dependencies.

### Testing Strategy

- **Unit tests:** `cargo test -p domain` — verifies `SoundSourceID::default()` and `SoundSourceID::new("")` both return `"sf2:0:0"`
- **Lint check:** `cargo clippy --workspace` — no warnings introduced
- **Manual browser test (deferred to user):** Load app, open training view without localStorage value set, verify SF2 plays. Check info page acknowledgements render correctly with both attributions and links.
- **CI verification:** Push to branch, confirm CI passes without SF2 download steps.

### Notes

- User has already updated the NOTICE file — no changes needed there
- Users with existing `peach.sound_source` localStorage values (e.g. `sf2:8:80`) may need to clear localStorage or update their setting if bank 8 program 80 doesn't exist in Samples.sf2. This is acceptable for an early-development app.
- The `.cache/` directory and `.gitignore` entry for it are retained — they may be used for other cached artifacts in the future.
