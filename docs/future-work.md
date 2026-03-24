# Future Work

Deferred findings from code reviews and implementation that need attention in future stories.

## Hardcoded "cents" unit in sparkline and profile components

**Source:** Story 15.2 code review (D1)
**Date:** 2026-03-24

`ProgressSparkline` (`web/src/components/progress_sparkline.rs:87`) unconditionally formats the EWMA value using `tr!("value-cents", ...)`, which produces "X.X cents". For rhythm disciplines the unit is "% of 16th" (per `unit_label` in `TrainingDisciplineConfig`), not cents.

The same issue exists in:
- `current-trend` fluent message (`web/locales/en/main.ftl:133`) — hardcodes "cents" in the aria-description used by `ProgressCard`
- `value-trend` fluent message — same pattern

**Impact:** Screen readers and visible UI will show "cents" for rhythm disciplines once they produce training data. Currently no rhythm data exists, so the bug is latent.

**Fix:** Use `config.unit_label` (already available via `mode.config()`) to select the correct fluent message or parameterize the unit in the existing messages.
