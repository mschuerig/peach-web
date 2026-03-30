---
title: 'Refine rhythm spectrogram: 6 tempo bands, 5 accuracy levels, help text'
type: 'feature'
created: '2026-03-30'
status: 'done'
baseline_commit: 'f7ec888'
context: []
---

# Refine rhythm spectrogram: 6 tempo bands, 5 accuracy levels, help text

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The web spectrogram uses 3 tempo bands and 3 accuracy levels while iOS now uses 6 bands and 5 levels with refined thresholds. iOS also added spectrogram help sections to the profile screen. The platforms should match.

**Approach:** Expand `TempoRange` from 3 to 6 variants, `SpectrogramAccuracyLevel` from 3 to 5, update thresholds to match iOS, add teal/orange colors, and add two help sections to the profile help modal.

## Boundaries & Constraints

**Always:** Match iOS band boundaries, threshold values, and color scheme exactly. Keep existing cell popover and keyboard navigation working.

**Ask First:** If any test changes reveal unexpected behavioral regressions beyond the expected threshold/band updates.

**Never:** Change stored data format, session logic, or CSV export/import.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| BPM 59 classification | `TempoBPM::new(59)` | `TempoRange::VerySlow` | N/A |
| BPM 60 classification | `TempoBPM::new(60)` | `TempoRange::Slow` | N/A |
| BPM 100 boundary | `TempoBPM::new(100)` | `TempoRange::Brisk` | N/A |
| BPM 120 boundary | `TempoBPM::new(120)` | `TempoRange::Fast` | N/A |
| BPM 160 boundary | `TempoBPM::new(160)` | `TempoRange::VeryFast` | N/A |
| Excellent accuracy | 3% of 16th at Slow | `Excellent` | N/A |
| Precise accuracy | 6% of 16th at Slow | `Precise` | N/A |
| Loose accuracy | 20% of 16th at Slow | `Loose` | N/A |

</frozen-after-approval>

## Code Map

- `domain/src/types/tempo_range.rs` -- Enum definition, `ALL`, `from_bpm()`, midpoints
- `domain/src/spectrogram.rs` -- `SpectrogramAccuracyLevel` enum, `SpectrogramThresholds` struct + defaults, accuracy classification
- `web/src/components/rhythm_spectrogram_chart.rs` -- `cell_fill()` colors, `tempo_range_i18n_key()` mapping
- `web/src/help_sections.rs` -- `PROFILE_HELP` array
- `web/locales/en/main.ftl` -- English tempo-range labels + help text
- `web/locales/de/main.ftl` -- German tempo-range labels + help text
- `domain/src/training_discipline.rs` -- Uses `TempoRange::ALL` for key enumeration
- `web/src/components/progress_card.rs` -- Iterates `TempoRange::ALL` for spectrogram computation

## Tasks & Acceptance

**Execution:**
- [ ] `domain/src/types/tempo_range.rs` -- Expand enum to 6 variants (VerySlow/Slow/Moderate/Brisk/Fast/VeryFast), update `ALL`, `from_bpm()`, `midpoint_bpm()`, fix tests
- [ ] `domain/src/spectrogram.rs` -- Add Excellent and Loose to `SpectrogramAccuracyLevel`, expand `SpectrogramThresholds` with excellent/loose fields, update defaults to iOS values (excellent 4%/8-15ms, precise 8%/12-30ms, moderate 15%/20-40ms, loose 25%/30-55ms), update classification logic, fix tests
- [ ] `web/src/components/rhythm_spectrogram_chart.rs` -- Add teal and orange to `cell_fill()`, add new i18n keys to `tempo_range_i18n_key()`
- [ ] `web/locales/en/main.ftl` -- Add 6 tempo-range labels + 2 spectrogram help entries
- [ ] `web/locales/de/main.ftl` -- Add 6 tempo-range labels + 2 spectrogram help entries (matching iOS translations)
- [ ] `web/src/help_sections.rs` -- Append spectrogram + colors sections to `PROFILE_HELP`

**Acceptance Criteria:**
- Given a rhythm spectrogram, when rendered, then 6 tempo bands appear (only those with data) with correct labels
- Given varying accuracy values, when classified, then 5 distinct color levels are shown (teal/green/yellow/orange/red)
- Given the profile help modal is opened, when scrolled, then spectrogram explanation and color legend sections are visible

## Verification

**Commands:**
- `cargo test -p domain` -- expected: all tests pass with updated thresholds and bands
- `cargo clippy --workspace` -- expected: no warnings
- `cargo fmt --check` -- expected: no formatting issues
