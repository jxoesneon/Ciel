---
title: Cortex features status check
type: diary
tags: [diary, session]
created: 2026-07-12
status: active
---

# Cortex features status check

## Goal

Answer the question "where are we with the cortex features?" by inspecting the codebase, tests, and roadmaps.

## What I found

### Implemented / scaffolded

- `lib/src/features/cortex/cortex_service.dart`
  - Defines `CortexService` abstract contract with four methods: `recommend`, `relevanceScore`, `extractKeywords`, `analyzeSentiment`.
  - Defines value objects: `ContentRecommendation`, `SentimentResult`, `SentimentLabel`.
  - Provides `NoOpCortexService` singleton that returns empty/neutral results.
  - **No Riverpod provider** exists yet; nothing wires the service into the UI or search flow.
- `lib/src/features/cortex/cortex_settings.dart`
  - `CortexSettings` immutable value object with four toggles (recommendations, keyword extraction, sentiment analysis, reputation tracking).
  - `CortexSettingsController` Notifier persists toggles to `SharedPreferences`.
  - `cortexSettingsProvider` is exposed and tested.
- `lib/src/features/cortex/reputation_manager.dart`
  - In-memory `ReputationManager` with `TrustLevel`, `PeerReputation`, score deltas, and clamping.
  - **Not persisted** and **not wired** to the P2P/DHT stack; described as a P2 stub.
- Tests
  - `test/cortex/cortex_service_test.dart` — verifies NoOp behavior and value equality.
  - `test/cortex/cortex_settings_test.dart` — verifies defaults, copyWith, persistence, and controller updates.
  - `test/cortex/reputation_manager_test.dart` — verifies trust classification, interactions, and overrides.

### Not implemented

- Real `CortexService` implementation (on-device LLM via `flutter_gemma`, heuristic, or cloud provider).
- `cortexServiceProvider` wiring.
- Search/browse result ranking using `relevanceScore`.
- Content recommendations surfaced in the UI.
- Keyword extraction and sentiment analysis consumers.
- `ReputationManager` persistence and integration with DHT/swarm peer events.

### Roadmap status

Per `docs/specs/AI_ML_ROADMAP.md` and `docs/specs/V1.0.0_ROADMAP.md`:

- **Sprint 9 (v0.9.x)**: Cortex core — provider quality scoring, search result ranking; Reputation Manager peer trust scoring.
- **Sprint 10 (v1.0.0)**: Cortex advanced — content relevance, preference learning, on-device Gemma integration, optional cloud AI provider fallback.

Cortex is currently **behind schedule** relative to the v1.0.0 roadmap: only the contract, settings toggles, and an in-memory reputation stub exist. The actual intelligence features and wiring are still pending.

## Blockers / next steps

- Decide on the first Cortex capability to ship (likely search result ranking or provider quality scoring, because they need no on-device LLM).
- Wire `CortexService` as a Riverpod provider so consumers can swap NoOp / heuristic / LLM implementations.
- Integrate `ReputationManager` with the DHT/tracker stack and persist scores to SQLite/Hive.
