---
title: Cortex agentic-loop implementation
type: diary
tags: [diary, session]
created: 2026-07-12
status: active
---

# Cortex agentic-loop implementation

## Goal

Answered the user's question "where are we with the cortex features?" and then used an agentic loop with 5 revolving subagent slots to comprehensively implement every Cortex feature in the X-Seed app.

## Initial status

Before this session, Cortex was mostly scaffolding:

- `CortexService` contract and `NoOpCortexService` existed.
- `CortexSettings` and persistence existed.
- `ReputationManager` was in-memory only.
- No real implementations of recommendations, relevance scoring, keyword extraction, sentiment analysis, or provider quality scoring.
- No wiring into search/browse or the tracker/DHT stack.

## What changed

### Foundation

- Added `lib/src/features/cortex/cortex_service_provider.dart` exposing `cortexServiceProvider`.
- Added `lib/src/features/cortex/cortex.dart` barrel file.
- Created `HeuristicCortexService` implementing all four `CortexService` methods.

### Intelligence features (heuristic / non-ML)

- **Relevance scoring**: token overlap + substring-match bonus, with stopword filtering.
- **Provider quality scoring**: reads the existing `tracker_stats` SQLite table via `SqliteTrackerStatsProvider`; scores tracker URLs by success ratio, latency penalty, and consecutive-failure penalty.
- **Search/browse ranking**: new `SearchRanking` helper combining `0.6 * relevance + 0.4 * quality`; integrated into `SearchController` and `BrowseController`.
- **Recommendations**: seeds from the most recent watchlist item by `addedAt`, queries `ScraperManager` for candidates, ranks by relevance, deduplicates, excludes the seed.
- **Keyword extraction**: lowercases, strips non-letters, removes stopwords, returns top 10 by frequency.
- **Sentiment analysis**: positive/negative word-list counts with normalized score and thresholded label.

### Reputation system

- Persisted `ReputationManager` to Hive via `HiveReputationStorage`.
- Exposed `reputationManagerProvider`.
- Added `reputationInteractionCallbackProvider`.
- Wired the callback into `TrackerScrapeClient`, `UdpScrapeClient`, and `DhtPeerDiscovery`.
- `health_controller.dart` now constructs scrape/DHT clients with the reputation callback.

## Subagent delegation

Used 5 revolving slots across two waves:

| Wave | Slot | Subagent | Task |
|---|---|---|---|
| 1 | 1 | 0adcd3dd | Foundation provider + skeleton |
| 1 | 2 | 5ac5529e | `relevanceScore` |
| 1 | 3 | 0e6d22ae | Provider quality scorer |
| 1 | 4 | 41cea500 | Keywords + sentiment |
| 1 | 5 | 2310c228 | Reputation persistence + provider |
| 2 | 1 | a17f6b58 | Search ranking |
| 2 | 2 | 471d4097 | Browse ranking |
| 2 | 3 | 9be1f8a4 | Recommendations |
| 2 | 4 | ca5ac58a | Reputation event wiring |
| 2 | 5 | parent | Fix test data, analyze, full test suite |

Subagents could not run shell commands (`exec` denied), so the parent agent ran `flutter analyze` and `flutter test` and fixed the resulting issues.

## Verification

- `flutter analyze --fatal-infos`: No issues found.
- `flutter test`: 1858 passed, 0 failed.

Fixes applied by the parent agent during verification:

- Removed unused `qualityScorer` variable and import from `browse_controller.dart`.
- Removed unused imports in `heuristic_recommend_test.dart` and `reputation_tracker_wiring_test.dart`.
- Fixed `const`/redundant-argument infos in test files.
- Fixed `heuristic_recommend_test.dart` fake scraper manager to be case-insensitive.
- Updated test candidates to be actually relevant to the seed title.
- Made `HeuristicCortexService.recommend` sort the watchlist by `addedAt` descending when picking the seed.

## Blockers / next steps

- `SearchResult` has no `providerId` field; provider-quality ranking currently uses a neutral 0.7 fallback. Wire real provenance once the scraper layer exposes it.
- Background isolate `ProviderAggregator` does not yet feed reputation events; needs a `ReputationManager` inside the isolate.
- On-device ML (Gemma/Qwen) and optional cloud providers remain for Sprint 10 per the roadmap.
