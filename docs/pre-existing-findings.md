# Pre-Existing Findings Catalog

Single source of truth for all known pre-existing issues. Every finding has a unique ID and a disposition.

**Dispositions:** OPEN — needs fixing or a story. CLOSED — fixed, with commit/story reference. WONT-FIX — accepted risk with rationale.

## Findings

### PEF-001: NaN/infinity propagation through WelfordAccumulator

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/welford.rs` — `WelfordAccumulator::update()`
- **Description:** If a NaN or infinity `f64` is passed to `update()`, it permanently poisons `mean`, `m2`, and all downstream statistics (EWMA, trend, std dev). Callers currently pass values derived from `.abs()` on known-finite fields, but there is no guard in the accumulator itself.
- **Recommendation:** Add `debug_assert!(value.is_finite())` in `update()`, or return early / return `Result`.

### PEF-002: NaN timestamp from parse_iso8601_to_epoch breaks session bucketing

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/training_discipline_statistics.rs:82`, `web/src/app.rs`
- **Description:** If `parse_iso8601_to_epoch` returns NaN (e.g., malformed timestamp from IndexedDB), the session gap comparison `point.timestamp - prev.timestamp >= session_gap` evaluates to `false`, silently merging bad points into sessions. The `partial_cmp` sort also swallows NaN via `unwrap_or(Equal)`.
- **Recommendation:** Validate or filter out non-finite timestamps at the hydration boundary in `app.rs`.

### PEF-003: No non-negative validation on MetricPoint::value

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/metric_point.rs`
- **Description:** The statistics engine assumes values represent absolute errors (non-negative). Callers do call `.abs()` before constructing points, but `MetricPoint::new` accepts any `f64` including negative values. No compile-time or runtime signal that negative inputs are a logic error.
- **Recommendation:** Add `debug_assert!(value >= 0.0)` in `MetricPoint::new`, or document the contract.

### PEF-004: WelfordAccumulator::mean() returns 0.0 on empty accumulator

- **Status:** OPEN
- **Surfaced:** Story 16.1 code review (2026-03-24)
- **Location:** `domain/src/welford.rs` — `mean()`
- **Description:** `mean()` returns `0.0` when `count == 0` rather than `Option<f64>`. All current callers guard with `count > 0` checks, but the public API is a footgun for future callers.
- **Recommendation:** Change return type to `Option<f64>`, or add a doc comment making the 0.0 contract explicit.

### PEF-005: merged_statistics rebuilds from scratch on every query

- **Status:** OPEN
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/profile.rs` — `merged_statistics()`
- **Description:** `merged_statistics()` collects all metrics from requested keys, sorts O(n log n), and rebuilds Welford/EWMA/trend from scratch on every call. `trend()`, `current_ewma()`, and `discipline_statistics()` all invoke it for multi-key (rhythm) disciplines. No caching.
- **Recommendation:** Cache merged results and invalidate on `add_point`. Matters once rhythm disciplines are actively recording data.

### PEF-006: merged_statistics accepts cross-discipline key slices

- **Status:** OPEN
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/profile.rs` — `merged_statistics()`
- **Description:** The public `merged_statistics` API accepts arbitrary `&[StatisticsKey]`. If keys from different disciplines are passed, the config from whichever key has data first wins, producing wrong EWMA/trend parameters. No current caller does this.
- **Recommendation:** Add a `debug_assert!` that all keys share the same discipline, or make the method `pub(crate)`.

### PEF-007: New statistics types lack Serialize/Deserialize

- **Status:** OPEN
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/statistics_key.rs`, `domain/src/types/tempo_range.rs`, `domain/src/types/rhythm_direction.rs`
- **Description:** `StatisticsKey`, `TempoRange`, and `RhythmDirection` have no serde derives. Not needed today (profile is rebuilt from records at hydration), but constrains future persistence strategies. `StatisticsKey` as a HashMap key would need a custom `Serialize` impl or string-based representation.
- **Recommendation:** Add serde derives when a persistence story requires it. No action needed now.

### PEF-009: add_metric_for_discipline silently drops metrics for unregistered disciplines

- **Status:** OPEN
- **Surfaced:** Story 16.3 code review (2026-03-24)
- **Location:** `domain/src/progress_timeline.rs` — `add_metric_for_discipline()`
- **Description:** `if let Some(state) = self.disciplines.get_mut(&discipline)` silently discards metrics when the discipline isn't in the HashMap. Currently safe because `ProgressTimeline::new()` initializes all variants from `TrainingDiscipline::ALL`. A `debug_assert!` was added in the 16.3 review fix, but a log warning for release builds would catch future discipline/initialization mismatches.
- **Recommendation:** Add `log::warn!` alongside the `debug_assert!` when a discipline key is missing.

### PEF-008: rebuild_all silently drops mismatched keys

- **Status:** OPEN
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/profile.rs` — `rebuild_all()`
- **Description:** `rebuild_all` iterates only valid discipline-key combinations. Any entries in the input HashMap with keys that don't match a valid combination are silently ignored. All current callers construct keys correctly.
- **Recommendation:** Log or `debug_assert!` if input contains unrecognized keys.

### PEF-011: AudioContext resume recovery uses unscoped spawn_local

- **Status:** OPEN
- **Surfaced:** Story 17.4 code review (2026-03-25)
- **Location:** `web/src/components/rhythm_offset_detection_view.rs` — audiocontext state change handler, `web/src/components/pitch_discrimination_view.rs` — same pattern
- **Description:** The `onstatechange` handler for AudioContext `Suspended` state spawns an unscoped `spawn_local` to attempt `resume()` and check recovery. This task is not tied to the component's lifecycle and can fire after the component is fully disposed, potentially calling `navigate()` and signal setters on disposed signals. The `interrupted` guard inside the closure mitigates most cases, but the spawned future itself persists. Shared pattern across both training views.
- **Recommendation:** Replace with `spawn_local_scoped_with_cancellation` if possible within a `Closure` context, or add a `terminated` guard inside the spawned future.

### PEF-010: Suspended AudioContext causes click burst on resume

- **Status:** OPEN
- **Surfaced:** Story 17.2 code review (2026-03-25)
- **Location:** `web/src/adapters/rhythm_scheduler.rs` — `RhythmScheduler::start()`
- **Description:** When the `AudioContext` is in `suspended` state (common before user gesture on mobile), `currentTime` is frozen at 0. The scheduler sets `next_step_time = 0 + 0.050` and the lookahead loop schedules all pattern steps immediately since all step times fall within the lookahead window. When the context later resumes, all scheduled clicks fire at once in a burst rather than being spaced out. This is a broader AudioContextManager concern — callers should ensure the context is resumed before starting the scheduler.
- **Recommendation:** Guard `RhythmScheduler::start()` or the calling code to verify `AudioContext.state() == "running"` before scheduling, or defer scheduling until a resume event.
