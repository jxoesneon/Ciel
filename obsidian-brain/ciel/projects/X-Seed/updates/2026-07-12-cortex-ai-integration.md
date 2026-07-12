---
title: Cortex AI integration and remaining backend wiring
project_note: update
type: project-note
tags: [project, update, X-Seed]
created: 2026-07-12
status: active
---

# Cortex AI integration and remaining backend wiring

## Summary

Completed all remaining Cortex work requested by the user: integrated SeedSphere's multi-provider AI service into X-Seed, added opt-in user-provided API-key storage, built a hybrid AI/heuristic `CortexService`, wired it dynamically through Riverpod, added a Cortex AI settings section, surfaced content recommendations on the Browse screen, wired `providerId` into `SearchResult` for real provider-quality scoring, and wired reputation events through the background isolate.

## AI integration

### SeedSphere AI port

- Ported `AiService` and AI models from `SeedSphere/router/lib/services/ai_service.dart` and `SeedSphere/router/lib/models/ai_models.dart` into `lib/src/features/cortex/ai/`.
- Supports 8 providers: DeepSeek (default, free tier allows empty key), OpenAI, Anthropic, Google Gemini, xAI, Mistral, Meta (via Together AI), and Cohere.
- Generic `generateText(AiTextRequest)` method with in-memory caching, timeouts, and structured response parsing.

### Settings and secure storage

- `AiSettingsStorage` persists the enable toggle, provider, and model in `SharedPreferences` and the API key in `FlutterSecureStorage` under `xseed_cortex_ai_api_key`.
- `AiSettingsController` / `aiSettingsProvider` / `aiReadyProvider` manage Riverpod state; the public `AiSettings` model never exposes the raw key.
- Added a Cortex AI section to `SettingsScreen` with master toggle, provider dropdown, model text field, obscured API-key field, and clear-key button.
- Added English and Spanish localizations for all new strings.

### Hybrid CortexService

- `HybridCortexService` implements all four `CortexService` methods and uses the AI backend only when the user has enabled AI and a key is present (or DeepSeek is selected).
- On any AI failure or when AI is disabled, it transparently falls back to `HeuristicCortexService`.
- `cortexServiceProvider` now watches `aiSettingsControllerProvider` and `aiApiKeyProvider`, so switching settings at runtime rebuilds the service.

### Recommendations UI

- Added `recommendationsProvider` and `RecommendationsSection` widget.
- "Recommended for you" horizontal carousel appears at the top of the Browse screen.
- Shows shimmer placeholders while loading and hides silently on error or empty results.
- Tapping a recommendation navigates to the detail route via its `imdbId`.

## Remaining backend wiring

### SearchResult providerId

- Added optional `providerId` to `SearchResult`.
- Updated all built-in and community scraper providers to pass their registered ID or tracker URL.
- Search/browse ranking now calls `qualityScorer.score(result.providerId ?? '')` instead of the neutral 0.7 fallback.

### Background-isolate reputation

- `ProviderAggregator` accepts an optional `PeerReputationCallback` and wires it into `TrackerScrapeClient`/`UdpScrapeClient`.
- `BackgroundService` forwards isolate reputation events via the service event channel to a main-isolate callback.
- `main_common.dart` creates a shared `ReputationManager` with `HiveReputationStorage`, loads persisted reputations, and sets `BackgroundService.onReputationEvent` to record events in the same manager used by the UI provider graph.

## Verification

- `flutter analyze --fatal-infos`: **No issues found!**
- `flutter test`: **1930 passed, 0 failed**
- Fixed pre-existing `test/ui/settings_interactions_test.dart` `pumpAndSettle` timeouts by:
  - Adding non-animated loading/error states to `CortexAiSettingsSection`.
  - Providing a synchronous fake `AiSettingsStorage` by default in `test/ui/test_helpers.dart`.

## Key decisions

- AI is strictly opt-in; default behavior is unchanged heuristic logic.
- DeepShip remains the default provider because SeedSphere marks it as the free default.
- The raw API key is never exposed in public provider state; only an internal `aiApiKeyProvider` reads it for service-layer calls.
- `HybridCortexService.recommend` always fetches heuristic candidates first so a fallback list is guaranteed.

## Files changed

- `lib/src/features/cortex/ai/ai_service.dart`
- `lib/src/features/cortex/ai/ai_models.dart`
- `lib/src/features/cortex/ai/ai_settings_storage.dart`
- `lib/src/features/cortex/ai/ai_settings_provider.dart`
- `lib/src/features/cortex/ai/cortex_ai_settings.dart`
- `lib/src/features/cortex/cortex.dart`
- `lib/src/features/cortex/hybrid_cortex_service.dart`
- `lib/src/features/cortex/recommendations_provider.dart`
- `lib/src/features/cortex/cortex_service_provider.dart`
- `lib/src/features/ui/settings/cortex_ai_settings_section.dart`
- `lib/src/features/ui/settings/settings_screen.dart`
- `lib/src/features/ui/shared/recommendations_section.dart`
- `lib/src/features/ui/browse/browse_screen.dart`
- `lib/src/features/core/models.dart`
- `lib/src/features/search/search_ranking.dart`
- `lib/src/features/providers/browse_controller.dart`
- `lib/src/features/scraper/providers/**/*.dart` (all `SearchResult` constructors)
- `lib/src/features/addon/provider_aggregator.dart`
- `lib/src/services/background_service.dart`
- `lib/main_common.dart`
- `lib/l10n/app_en.arb`, `lib/l10n/app_es.arb`, generated localizations
- `test/ui/test_helpers.dart`
- `test/cortex/ai/*.dart`, `test/cortex/hybrid_cortex_service_test.dart`, `test/cortex/reputation_isolate_wiring_test.dart`, `test/ui/recommendations_section_test.dart`, `test/ui/settings/cortex_ai_settings_section_test.dart`, plus updated existing model/ranking/browse tests.
