# Pre-Existing Findings Catalog

Single source of truth for all known pre-existing issues. Every finding has a unique ID and a disposition.

**Dispositions:** OPEN — needs fixing or a story. WONT-FIX — accepted risk with rationale. Closed findings are removed; see git history for details.

## Findings

### PEF-005: merged_statistics rebuilds from scratch on every query

- **Status:** OPEN
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/profile.rs` — `merged_statistics()`
- **Description:** `merged_statistics()` collects all metrics from requested keys, sorts O(n log n), and rebuilds Welford/EWMA/trend from scratch on every call. `trend()`, `current_ewma()`, and `discipline_statistics()` all invoke it for multi-key (rhythm) disciplines. No caching.
- **Recommendation:** Cache merged results and invalidate on `add_point`. Matters once rhythm disciplines are actively recording data.

### PEF-007: New statistics types lack Serialize/Deserialize

- **Status:** OPEN
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/statistics_key.rs`, `domain/src/types/tempo_range.rs`, `domain/src/types/rhythm_direction.rs`
- **Description:** `StatisticsKey`, `TempoRange`, and `RhythmDirection` have no serde derives. Not needed today (profile is rebuilt from records at hydration), but constrains future persistence strategies. `StatisticsKey` as a HashMap key would need a custom `Serialize` impl or string-based representation.
- **Recommendation:** Add serde derives when a persistence story requires it. No action needed now.

### PEF-010: Suspended AudioContext causes click burst on resume

- **Status:** OPEN
- **Surfaced:** Story 17.2 code review (2026-03-25)
- **Location:** `web/src/adapters/rhythm_scheduler.rs` — `RhythmScheduler::start()`
- **Description:** When the `AudioContext` is in `suspended` state (common before user gesture on mobile), `currentTime` is frozen at 0. The scheduler sets `next_step_time = 0 + 0.050` and the lookahead loop schedules all pattern steps immediately since all step times fall within the lookahead window. When the context later resumes, all scheduled clicks fire at once in a burst rather than being spaced out. This is a broader AudioContextManager concern — callers should ensure the context is resumed before starting the scheduler.
- **Recommendation:** Guard `RhythmScheduler::start()` or the calling code to verify `AudioContext.state() == "running"` before scheduling, or defer scheduling until a resume event.

### PEF-011: AudioContext resume recovery uses unscoped spawn_local

- **Status:** OPEN
- **Surfaced:** Story 17.4 code review (2026-03-25)
- **Location:** `web/src/components/rhythm_offset_detection_view.rs` — audiocontext state change handler, `web/src/components/pitch_discrimination_view.rs` — same pattern
- **Description:** The `onstatechange` handler for AudioContext `Suspended` state spawns an unscoped `spawn_local` to attempt `resume()` and check recovery. This task is not tied to the component's lifecycle and can fire after the component is fully disposed, potentially calling `navigate()` and signal setters on disposed signals. The `interrupted` guard inside the closure mitigates most cases, but the spawned future itself persists. Shared pattern across both training views.
- **Recommendation:** Replace with `spawn_local_scoped_with_cancellation` if possible within a `Closure` context, or add a `terminated` guard inside the spawned future.

### PEF-013: Merge import dedup uses timestamp-only key, losing sub-second records

- **Status:** OPEN
- **Surfaced:** Story 19.1 code review (2026-03-25)
- **Location:** `web/src/adapters/csv_export_import.rs` — `import_merge()`
- **Description:** Merge deduplication keys on `truncate_timestamp_to_second(&timestamp)` only. Two distinct records of the same training type within the same calendar second share a key, so the second is silently skipped as a duplicate. Affects all 4 record types.
- **Recommendation:** Use a composite dedup key including distinguishing fields (e.g., reference_note, target_note for pitch; tempo_bpm, offset_ms for rhythm).
