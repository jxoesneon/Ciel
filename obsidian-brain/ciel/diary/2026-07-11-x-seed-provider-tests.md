---
title: 2026-07-11 X-Seed provider/widget coverage tests
type: diary
tags: [diary, session]
created: 2026-07-11
status: active
---

# 2026-07-11 X-Seed provider/widget coverage tests

## Goal
Add unit and widget tests for the largest zero-coverage files in the X-Seed Flutter project to improve coverage.

## Tasks completed
- Created `test/scraper/cinemeta_test_helper.dart` to mock Cinemeta HTTP calls and prevent real network requests in provider tests.
- Added widget test `test/ui/debug_logs_screen_test.dart` covering rendering, filtering, real-time streaming, and clear behavior.
- Added unit tests for zero-coverage scraper providers:
  - `public_domain_torrents_provider_test.dart`
  - `configurable_http_provider_test.dart`
  - `generic_scraping_provider_test.dart`
  - `community_plugin_config_test.dart`
  - `x1337_provider_test.dart`
  - `piratebay_provider_test.dart`
  - `torznab_provider_test.dart`
  - `nyaa_provider_test.dart`
  - `yts_provider_test.dart`

## Source changes
Fixed small bugs discovered by the new tests (all covered by tests now):
- `lib/src/features/scraper/providers/community/generic_scraping_provider.dart`
  - `_htmlUnescape` helper handles missing dependency; only unescapes when `HtmlUnescape` is available.
- `lib/src/features/scraper/providers/community/nyaa_provider.dart`
  - `stream` now passes the proper `baseUrl` to `_parseSearchResultRows` so relative magnet links resolve correctly.
- `lib/src/features/scraper/providers/community/piratebay_provider.dart`
  - `search` uses the configured `baseUrl` when resolving relative `detLink` detail URLs instead of hard-coding the provider's default base URL.
- `lib/src/features/scraper/providers/community/x1337_provider.dart`
  - Magnet extraction in `stream` uses a wider CSS selector and parses the `href` attribute correctly.

## Verification
- Targeted test run: **113 tests passed**.
- `flutter analyze`: **No issues found**.
- Full `flutter test` suite: **All tests passed** (2042 tests).

## Tags
#x-seed #testing #flutter #diary
