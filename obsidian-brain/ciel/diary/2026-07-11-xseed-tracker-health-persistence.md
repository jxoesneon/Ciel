---
title: X-Seed — Tracker health SQLite persistence
type: diary
tags: [diary, session]
created: 2026-07-11
status: active
---

# X-Seed — Tracker health SQLite persistence

## Goal

Implement SQLite persistence for `TrackerHealthMonitor` stats so tracker health data survives app restarts.

## What was done

- Added `saveTrackerStats`, `loadAllTrackerStats`, `deleteTrackerStats`, and `clearTrackerStats` helpers to `lib/src/services/sqflite_service.dart`.
  - Reused the existing `tracker_stats` table (schema v7), which already stored the full `TrackerStats` fields used by `SqliteTrackerStatsProvider`.
- Updated `lib/src/features/scraper/tracker_health_monitor.dart`:
  - Added an optional `SqfliteService? sqfliteService` constructor parameter.
  - Added `load()` / `_load()` to hydrate `_stats` from SQLite on construction.
  - Implemented `_persist(TrackerStats)` to write each updated `TrackerStats` row to SQLite.
  - Ensured `_persist()` is awaited after every `_stats` assignment in `_probeTracker` and `_drainQueue`.
  - Updated `pruneDeadTrackers()` to delete the persisted row when a dead tracker is evicted.
- Added unit tests:
  - `test/scraper/tracker_health_monitor_test.dart`: persistence load, successful-probe save, failed-probe save, prune deletion, and in-memory fallback when no DB is wired.
  - `test/services/sqflite_service_test.dart`: `saveTrackerStats` insert/update, `deleteTrackerStats`, and null-timestamp round-trip.

## Schema note

The requested column names (`successCount`, `failCount`, `avgLatencyMs`, `lastSeenAt`, etc.) differ from the existing `tracker_stats` table columns (`success_count`, `total_count`, `ewma_latency`, `last_scrape_at`, etc.). I kept the existing richer schema because it is already used by `SqliteTrackerStatsProvider` and maps directly to the `TrackerStats` model. Functionally, persistence now stores the same data.

## Verification

- `flutter analyze --fatal-infos lib/src/features/scraper/tracker_health_monitor.dart lib/src/services/sqflite_service.dart test/scraper/tracker_health_monitor_test.dart test/services/sqflite_service_test.dart` → No issues found.
- `flutter test test/scraper/tracker_health_monitor_test.dart test/scraper/tracker_scorer_test.dart test/services/sqflite_service_test.dart` → All 113 tests passed.
- Full `flutter analyze lib test` still shows pre-existing errors in `community_plugins_section.dart` and `settings_screen.dart` unrelated to this change.

## Recommended next steps

- Wire `SqfliteService` into the eventual `TrackerHealthMonitor` provider/constructor when the production integration is built.
- Consider adding migration columns (`created_at`, `updated_at`) to `tracker_stats` if future tooling needs explicit audit timestamps.
