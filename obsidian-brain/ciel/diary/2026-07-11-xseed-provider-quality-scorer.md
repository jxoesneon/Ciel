---
title: X-Seed — Provider quality scorer
type: diary
tags: [diary, session]
created: 2026-07-11
status: active
---

# X-Seed — Provider quality scorer

## What was done

Implemented `ProviderQualityScorer` in `lib/src/features/cortex/provider_quality_scorer.dart` for Cortex search/browse ranking.

- Defines `TrackerStatsProvider` / `TrackerQualityStats` so the scorer can be wired to the SQLite `tracker_stats` table later.
- `score(providerId)` returns:
  - `0.7` for non-tracker provider IDs (e.g. `public_domain_torrents`, `archive_org`, `yts`).
  - For tracker URLs (`http://`, `https://`, `udp://`):
    - base = `success_count / total_count` (or `0.5` if no data).
    - latency penalty = `min(0.3, ewma_latency / 5000)`.
    - consecutive-failure penalty = `min(0.3, consecutive_failures * 0.1)`.
    - result clamped to `0.0`–`1.0`.

Added 13 unit tests in `test/cortex/provider_quality_scorer_test.dart` covering neutral non-tracker scores, cold-start trackers, success-ratio scoring, latency/failure penalties + caps, combined clamping, case-insensitive protocol detection, trimming, zero-total-count handling, and scorer usage without a stats provider.

## Verification

- `flutter analyze --fatal-infos lib/src/features/cortex/provider_quality_scorer.dart` — No issues found.
- `flutter test test/cortex/provider_quality_scorer_test.dart` — 13/13 passed.
- `flutter test test/cortex` — 78/78 passed (existing + new cortex tests).
- `flutter analyze --fatal-infos` on the whole project reports one pre-existing `avoid_redundant_argument_values` info in `test/services/hive_service_test.dart`, unrelated to this change.

## Wiring status

No existing `TrackerStatsRepository` abstraction was found; only `TrackerCacheRepository` (peer-state/tracker-list cache) and the raw `tracker_stats` schema in `SqfliteService` exist. The scorer accepts a `TrackerStatsProvider` dependency so the repository can be wired later without changing the scorer.

## Next steps / blockers

- Add a concrete `TrackerStatsProvider` implementation that queries `tracker_stats` (tracker_url, protocol, success_count, total_count, ewma_latency, consecutive_failures) and wire it into the scorer provider.
- Decide where provider IDs come from for `SearchResult` ranking (the current `SearchResult` model does not carry a provider/source field).
