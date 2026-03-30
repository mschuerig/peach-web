---
title: 'Fix chart session rendering: empty spectrogram cells, scroll position, spacing'
type: 'bugfix'
created: '2026-03-30'
status: 'done'
baseline_commit: '11fd609'
context: []
---

# Fix chart session rendering: empty spectrogram cells, scroll position, spacing

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Three related chart bugs when today's sessions exist: (1) spectrogram cells for session buckets appear empty because the time-range filter uses strict `<` on `period_end`, excluding the last record of every bucket and all records of single-record buckets; (2) spectrograms don't auto-scroll to the right edge on mount and aren't right-aligned when non-scrollable; (3) session buckets get equal horizontal width as day/month buckets, making dots too spread out.

**Approach:** Fix the off-by-one filter, right-align both chart types when non-scrollable (SVG sized proportionally + flex justify-end on container), and compact session bucket horizontal spacing via weighted x-positioning (sessions get fractional weight vs. 1.0 for non-session buckets).

## Boundaries & Constraints

**Always:** Keep both charts visually consistent. Session dots need visible whitespace around them but no grid-proportional width. Right-align non-scrollable charts so newest data is at the right edge.

**Ask First:** If the weighted spacing approach causes visual issues with the session bridge point or stddev band endpoint in the progress chart.

**Never:** Change bucket computation logic in `progress_timeline.rs`. Don't change the scrollable threshold (VISIBLE_BUCKETS = 8).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Single-record session bucket | 1 metric point in session bucket | Spectrogram cell shows colored (not gray) | N/A |
| Multi-record bucket last point | 5 points, last at period_end | All 5 included in spectrogram cell | N/A |
| Non-scrollable chart (3 cols) | col_count < VISIBLE_BUCKETS | SVG right-aligned, width = col_count/8 * 100% | N/A |
| Session-only chart (no day/month) | Only session buckets | Session dots compactly spaced, right-aligned | N/A |
| Scrollable spectrogram | col_count > 8 | Auto-scrolled to right edge on mount | N/A |

</frozen-after-approval>

## Code Map

- `domain/src/spectrogram.rs:268` -- off-by-one: `m.timestamp < bucket.period_end` should be `<=`
- `web/src/components/progress_chart.rs` -- x-mapping function, SVG sizing, container CSS
- `web/src/components/rhythm_spectrogram_chart.rs` -- cell_w sizing, SVG sizing, container CSS, scroll-to-right

## Tasks & Acceptance

**Execution:**
- [ ] `domain/src/spectrogram.rs` -- Change `m.timestamp < bucket.period_end` to `<=` on line 268 -- fixes empty cells for last-record and single-record buckets
- [ ] `domain/src/spectrogram.rs` -- Add unit test for single-record bucket cell computation -- validates the off-by-one fix
- [ ] `web/src/components/progress_chart.rs` -- Implement weighted x-positioning: session buckets get weight ~0.4, non-session get 1.0; compute cumulative weights for x() function -- compacts session spacing
- [ ] `web/src/components/progress_chart.rs` -- Right-align non-scrollable charts: set `svg_width` proportional to weighted bucket count vs VISIBLE_BUCKETS, add `justify-end` to container div -- newest data at right edge
- [ ] `web/src/components/rhythm_spectrogram_chart.rs` -- Same weighted cell width and right-alignment as progress chart -- visual consistency
- [ ] `web/src/components/rhythm_spectrogram_chart.rs` -- Verify scroll-to-right works for scrollable case; increase TimeoutFuture delay if needed -- ensures newest data visible on mount

**Acceptance Criteria:**
- Given a session bucket with 1 record, when the spectrogram renders, then the cell shows a colored fill (not gray/empty)
- Given fewer than 8 buckets, when the chart renders, then content is right-aligned with space on the left
- Given session + day buckets, when the chart renders, then session buckets occupy less horizontal space than day buckets
- Given a scrollable spectrogram, when it mounts, then it auto-scrolls to show the rightmost (newest) data

## Verification

**Commands:**
- `cargo test -p domain` -- expected: all tests pass including new spectrogram off-by-one test
- `cargo clippy --workspace` -- expected: zero warnings

**Manual checks:**
- Load profile page with today's session data: spectrogram cells should be colored, charts right-aligned, session dots compact
