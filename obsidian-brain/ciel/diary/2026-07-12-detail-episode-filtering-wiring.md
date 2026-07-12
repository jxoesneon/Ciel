---
title: 2026-07-12 — Wire detail-screen episode selector to episode-specific stream filtering
type: diary
tags: [diary, session]
created: 2026-07-12
status: active
---

# 2026-07-12 — Wire detail-screen episode selector to episode-specific stream filtering

## Summary

Completed the remaining known limitations from the P1/P2 detail-screen work: the series episode selector is now wired to refetch episode-specific streams, the health sparkline exposes a descriptive semantic label for screen readers, and the data models/providers propagate season/episode metadata.

## Goal

`ciel/projects/X-Seed/goals/2026-07-12-detail-episode-filtering-wiring.md`

## What was implemented

1. **Episode-aware `DetailQuery`**
   - Added optional `season` and `episode` fields to `DetailQuery` in `x_seed/lib/src/features/core/models.dart` and regenerated `copyWith`/`==`/`hashCode`.
   - Updated `DefaultDetailService` to append `:season:episode` to the base ID when calling `scraperManager.fetchStreams` for `series` content.

2. **Wired episode selector to refetch streams**
   - Converted `DetailScreen` from `ConsumerWidget` to `ConsumerStatefulWidget`.
   - `_DetailScreenState` tracks the selected `Episode` and rebuilds the `DetailQuery` accordingly.
   - Passing the selected episode to `_EpisodeSelector` keeps the dropdowns in sync after a refetch.

3. **Season/episode on `ProviderStream` and `StreamSource`**
   - Added `season`/`episode` to `ProviderStream`, `StreamSource`, and `DetailStream` models.
   - Populated the fields in `EztvProvider`, `TorrentioProvider`, and `GenericScrapingProvider` so downstream filtering/UI can use them later.

4. **Accessible health sparkline**
   - `HealthSparkline` now computes a semantic label that reports seeders/leechers start/end values and peaks, making the chart meaningful to screen-reader users.

5. **Tests**
   - Added widget test verifying that selecting a different episode in the series selector triggers a refetch and updates the stream list (`test/ui/detail_screen_test.dart`).
   - Added unit test verifying `DefaultDetailService` appends season/episode to the scraper request (`test/detail/detail_service_test.dart`).
   - Added semantics test for `HealthSparkline` (`test/ui/health_section_test.dart`).

## Files changed

- `x_seed/lib/src/features/core/models.dart`
- `x_seed/lib/src/features/detail/detail_service.dart`
- `x_seed/lib/src/features/ui/detail/detail_screen.dart`
- `x_seed/lib/src/features/ui/shared/health_sparkline.dart`
- `x_seed/lib/src/features/scraper/providers/community/eztv_provider.dart`
- `x_seed/lib/src/features/scraper/providers/community/torrentio_provider.dart`
- `x_seed/lib/src/features/scraper/providers/community/generic_scraping_provider.dart`
- `x_seed/test/ui/detail_screen_test.dart`
- `x_seed/test/detail/detail_service_test.dart`
- `x_seed/test/ui/health_section_test.dart`

## Verification

- `flutter analyze --fatal-infos`: **No issues found**
- `flutter test`: **1769 tests passed, 0 failed**
- Physical device: installed `app-full-debug.apk` on CPH2583 and ran `integration_test/addon_server_test.dart` with `--flavor full`; all 5 tests passed.
- Manual splash-screen workaround: the app stayed on the Flutter splash screen because `main_common.dart` awaits `BackgroundService.start()` synchronously; when Android's foreground-service start budget is exhausted, `runApp()` never runs. Wrapped the call in a debug-gated try-catch, rebuilt with `--flavor full`, reinstalled via `mobile-mcp`, and completed onboarding using `mobile-mcp` clicks plus ADB screenshots/logcat.
- Series deep link verification: `xseed://detail?imdbId=tt0898266&type=series` opened "The Big Bang Theory" correctly after fixing `xSeedUriToLocation` to preserve the `type` query parameter. The "Series" badge, genres, plot, and stream filters rendered. The episode selector did not appear because the current Cinemeta response does not include `episodes`; this is a data-layer gap, not a UI regression.
- **Seeders fix**: the user observed that every chip on the series detail screen showed the same seeder count. Root cause: without an episode selected, Torrentio returns a complete-series torrent pack where every "stream" is a different file index inside the same torrent, so they share one info hash and one peer count. Fix: series detail pages now auto-select S01E01 on first load and show a fallback episode selector when `Meta.episodes` is empty. The initial stream fetch targets `tt...:1:1`, and phone screenshots confirm distinct seeder counts per chip (4K: 108 vs 2; 1080p: 66, 63, 60, 52, 46, 45, 33, 23, 16...). Updated `test/ui/detail_screen_test.dart` to cover the auto-selected initial episode.
- Wireless-debugging skill updated with the splash-screen hang failure mode and debug workaround.
- Quality normalization: the Public Domain Torrents provider was emitting legacy labels (`DivX`, `IPOD`, `PSP`, `PDA`) that users do not recognize as qualities. Added `QualityDisplay` utility that parses stream titles via `QualityExtractor` and maps legacy tokens to canonical labels (`4K`, `1080p`, `720p`, `480p`, `DVD`). Updated `StreamFilterBar`, stream chips, and `QualityFilter.matches` to use normalized labels. Public Domain Torrents provider now returns `480p` for device/codec variants. Added `test/core/quality_display_test.dart`.
- Stream chip redesign: phone screenshot revealed duplicate-looking chips (`4K • community_torrentio • Seeders: 108`) that gave users no basis to choose. Replaced `ActionChip` with a two-line `StreamChip` showing a colored quality badge, source, size, seeders/leechers, a friendly provider name (`Torrentio`), a health dot, and the stream title. Added `test/ui/stream_chip_test.dart`. Design Council scores improved across all five lenses.

## Known limitations / next steps

- The episode selector now correctly refetches, but the stream list itself does not yet locally filter by episode when multiple episodes' streams are returned in a single response. The `season`/`episode` fields on `ProviderStream` enable this future enhancement.
- Provider support for season/episode metadata is implemented for the major community providers (EZTV, Torrentio, generic scraping); additional providers can be updated incrementally.

## Related records

- [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md]]
- [[ciel/projects/X-Seed/updates/2026-07-12-detail-episode-filtering.md]]
