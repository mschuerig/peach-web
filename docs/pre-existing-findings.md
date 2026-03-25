# Pre-Existing Findings Catalog

Single source of truth for all known pre-existing issues. Every finding has a unique ID and a disposition.

**Dispositions:** OPEN — needs fixing or a story. WONT-FIX — accepted risk with rationale. Closed findings are removed; see git history for details.

## Findings

### PEF-005: merged_statistics rebuilds from scratch on every query

- **Status:** WONT-FIX — premature optimization; no user-visible perf issue yet. Revisit when rhythm data volume grows.
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/profile.rs` — `merged_statistics()`
- **Description:** `merged_statistics()` collects all metrics from requested keys, sorts O(n log n), and rebuilds Welford/EWMA/trend from scratch on every call. `trend()`, `current_ewma()`, and `discipline_statistics()` all invoke it for multi-key (rhythm) disciplines. No caching.
- **Recommendation:** Cache merged results and invalidate on `add_point`. Matters once rhythm disciplines are actively recording data.

### PEF-007: New statistics types lack Serialize/Deserialize

- **Status:** WONT-FIX — no persistence story requires it yet. Add serde derives when one does.
- **Surfaced:** Story 16.2 code review (2026-03-24)
- **Location:** `domain/src/statistics_key.rs`, `domain/src/types/tempo_range.rs`, `domain/src/types/rhythm_direction.rs`
- **Description:** `StatisticsKey`, `TempoRange`, and `RhythmDirection` have no serde derives. Not needed today (profile is rebuilt from records at hydration), but constrains future persistence strategies. `StatisticsKey` as a HashMap key would need a custom `Serialize` impl or string-based representation.
- **Recommendation:** Add serde derives when a persistence story requires it. No action needed now.

### PEF-013: Merge import dedup uses timestamp-only key, losing sub-second records

- **Status:** WONT-FIX — the training loop physically cannot produce two records of the same type within one second (each round involves listening, thinking, answering). The coarse timestamp key correctly deduplicates re-imports without needing field-by-field matching.
- **Surfaced:** Story 19.1 code review (2026-03-25)
- **Location:** `web/src/adapters/csv_export_import.rs` — `import_merge()`
- **Description:** Merge deduplication keys on `truncate_timestamp_to_second(&timestamp)` only. Two distinct records of the same training type within the same calendar second share a key, so the second is silently skipped as a duplicate. Affects all 4 record types.
- **Recommendation:** Use a composite dedup key including distinguishing fields (e.g., reference_note, target_note for pitch; tempo_bpm, offset_ms for rhythm).
