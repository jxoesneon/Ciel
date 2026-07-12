---
title: Cortex features implementation
project_note: update
type: project-note
tags: [project, update, X-Seed]
created: 2026-07-12
status: active
---

# Cortex features implementation

## Summary

Implemented every Cortex capability in the X-Seed app using an agentic loop with five revolving subagent slots. The work focused on local, heuristic (non-ML) implementations that satisfy the v1.0.0 roadmap while keeping the codebase free of on-device LLM or cloud-AI dependencies. All changes are wired through Riverpod and covered by tests.

## Agentic loop execution

- Started the official `scripts/obsidian/agentic-loop.mjs` orchestrator, but it could not authenticate to the `claude` CLI or the Obsidian Local REST API.
- Switched to manual orchestration with 5 revolving subagent slots.
- Tracked the full run in `x_seed/.scratch/cortex_agentic_loop.md`.

## Tasks completed

| # | Feature | Implementation |
|---|---------|----------------|
| 1 | `CortexService` provider + `HeuristicCortexService` skeleton | New barrel file `lib/src/features/cortex/cortex.dart` and provider `cortex_service_provider.dart` |
| 2 | `relevanceScore` | Token overlap + substring-match bonus with stopword filtering in `heuristic_cortex_service.dart` |
| 3 | Provider quality scoring | `ProviderQualityScorer` + `SqliteTrackerStatsProvider` reading the existing `tracker_stats` table |
| 4 | Search result ranking | `lib/src/features/search/search_ranking.dart` helper; integrated into `SearchController` |
| 5 | Browse result ranking | Integrated into `BrowseController` |
| 6 | Content `recommend` | Seeds from watchlist (most-recent-by-`addedAt`), queries `ScraperManager`, ranks by relevance, deduplicates |
| 7 | `extractKeywords` + `analyzeSentiment` | Heuristic word-list implementations in `heuristic_text_analysis.dart` |
| 8 | `ReputationManager` persistence | Hive-backed storage + `reputationManagerProvider` |
| 9 | Reputation event wiring | Callback injected into `TrackerScrapeClient`, `UdpScrapeClient`, and `DhtPeerDiscovery`; wired through `health_controller.dart` |
| 10 | Tests | New tests for relevance, keywords, sentiment, provider scoring, SQLite tracker stats, search ranking, browse ranking, recommendations, reputation wiring |
| 11 | Verification | `flutter analyze --fatal-infos`: No issues. `flutter test`: 1858 passed, 0 failed. |

## Files changed

- `lib/src/features/cortex/cortex_service_provider.dart`
- `lib/src/features/cortex/cortex.dart` (new barrel)
- `lib/src/features/cortex/heuristic_cortex_service.dart`
- `lib/src/features/cortex/heuristic_text_analysis.dart` (new)
- `lib/src/features/cortex/provider_quality_scorer.dart`
- `lib/src/features/cortex/provider_quality_scorer_provider.dart` (new)
- `lib/src/features/cortex/sqlite_tracker_stats_provider.dart` (new)
- `lib/src/features/cortex/reputation_manager.dart`
- `lib/src/features/cortex/reputation_provider.dart`
- `lib/src/features/core/peer_reputation_callback.dart` (new)
- `lib/src/features/providers/browse_controller.dart`
- `lib/src/features/providers/search_controller.dart`
- `lib/src/features/search/search_ranking.dart` (new)
- `lib/src/features/scraper/dht_peer_discovery.dart`
- `lib/src/features/scraper/tracker_scrape_client.dart`
- `lib/src/features/scraper/udp_scrape_client.dart`
- `lib/src/services/sqflite_service.dart`
- `lib/main_common.dart`
- `lib/src/services/hive_service.dart`
- `test/cortex/*.dart` (new/extended)
- `test/providers/browse_controller_test.dart` (new)
- `test/search/search_ranking_test.dart` (new)

## Design Council note

| Lens | Impact | Rationale |
|---|---|---|
| Clarity | improved | Search/browse results are now ranked by relevance and provider quality, surfacing better matches first. |
| Efficiency | improved | Recommendations and sentiment/keyword analysis run entirely on-device with no backend latency. |
| Aesthetics | neutral | No UI changes yet; intelligence is under the hood. |
| Inclusion | maintained | All features respect existing reduced-motion and opt-out patterns; reputation data is local only. |

## Known limitations / next steps

- `SearchResult` currently has no `providerId` field, so provider-quality scoring in ranking uses a neutral 0.7 fallback. TODOs are left in `search_ranking.dart` and `browse_controller.dart` to wire real per-result provenance once the scraper layer exposes it.
- Reputation wiring for HTTP/UDP trackers uses the tracker URL as a peer-id proxy; DHT uses real `host:port` identifiers.
- The background-isolate `ProviderAggregator` path is not yet feeding reputation events; this requires setting up a `ReputationManager` inside the isolate.
- The current implementations are heuristic. The roadmap's on-device Gemma / cloud-AI provider integration remains future work for Sprint 10.
