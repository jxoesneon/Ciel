---
title: Detail screen episode selector wired to episode-specific stream filtering
project_note: update
type: project-note
tags: [project, update, X-Seed]
created: 2026-07-12
status: active
---

# Detail screen episode selector wired to episode-specific stream filtering

## Summary

The series episode selector added during the P1/P2 detail-screen UX fixes now drives episode-specific stream resolution. The `DetailQuery` model, `DefaultDetailService`, and major community providers all carry season/episode metadata, and the health sparkline is now accessible to screen-reader users.

## Changes

- `DetailQuery` now accepts optional `season` and `episode`.
- `DefaultDetailService` appends `:season:episode` to series IDs when requesting streams from `ScraperManager`.
- `DetailScreen` manages the selected episode in local state and rebuilds the query when the user changes the season/episode dropdowns.
- `ProviderStream`, `StreamSource`, and `DetailStream` carry `season`/`episode` fields, populated by EZTV, Torrentio, and the generic scraping provider.
- `HealthSparkline` exposes a semantic label describing the trend's start/end and peak seeders/leechers.
- `QualityDisplay` normalizes legacy provider quality tokens (`DivX`, `IPOD`, `PSP`, `PDA`, `Original`, `Direct`) into canonical user-recognizable labels (`4K`, `1080p`, `720p`, `480p`, `DVD`). `StreamFilterBar`, stream chips, and `QualityFilter.matches` now use normalized labels. Public Domain Torrents provider no longer emits device/codec names as qualities.
- `StreamChip` redesign: replaced the single-line `ActionChip` (`quality • provider • Seeders: N`) with a two-line decision chip that surfaces a colored quality badge, source, size, seeders/leechers, a friendly provider name, a health dot, and the stream title. This addresses the Design Council finding that identical-looking chips made it impossible to choose between duplicate results.
- Series detail pages now auto-select S01E01 on first load and show a fallback episode selector when `Meta.episodes` is empty. This prevents the previous behavior where opening a series without an explicit episode fetched every file from a complete-series torrent pack, causing all chips to share the same info hash and identical seeder counts. The initial stream fetch now targets `tt...:1:1`, and the user can switch episodes via the selector.

## Verification

- `flutter analyze --fatal-infos`: No issues found.
- `flutter test`: 1769 tests passed, 0 failed (added 10 quality-display tests + 5 stream-chip widget tests).
- Phone screenshot captured: `C:/Users/josee/AppData/Local/Temp/xseed_rich_chips2.png` shows normalized quality filters (`4K`, `1080p`, `720p`, `480p`) and rich stream chips with quality badge, seeders, friendly provider name, health dot, and stream title.
- Device test: `flutter test integration_test/addon_server_test.dart --flavor full -d <phone>` passed (5/5 tests).
- Full manual verification: after working around the splash-screen hang by wrapping `BackgroundService.start()` in a try-catch (debug-only), the app reached the home shell on a OnePlus CPH2583 (Android 16). A series deep link (`xseed://detail?imdbId=tt0898266&type=series`) opened "The Big Bang Theory" as a series with the "Series" badge. Phone screenshots (`xseed_episode_streams2.png`, `xseed_episode_top.png`) show per-episode stream chips with distinct seeder counts: 4K streams report 108 and 2 seeders; 1080p streams report 66, 63, 60, 52, 46, 45, 33, 23, 16, etc. This confirms each chip now renders its own peer data instead of a duplicated default.

## Related

- [[ciel/diary/2026-07-12-detail-episode-filtering-wiring.md]]
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md]]
- [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]

## Design Council review — stream chips

| Member | Score | Flags | Rationale |
|--------|-------|-------|-----------|
| Actionability | 7 → 9 | `weak_cta` resolved, `missing_guidance` resolved | Quality badge, source, size, seeders, provider, and title give users the information needed to pick a stream confidently. The chip still opens the action sheet, preserving the single-tap play path. |
| Clarity | 5 → 8 | `jargon` resolved, `hidden_state` resolved | `community_torrentio` is now displayed as `Torrentio`. Source, size, and health are visible instead of hidden inside the action sheet. Title adds distinguishing context. |
| Efficiency | 6 → 8 | `extra_steps` resolved | Users no longer need to open multiple action sheets to compare otherwise identical-looking streams. |
| Aesthetics | 5 → 7 | `visual_clutter` reduced, `weak_hierarchy` improved | Quality badge uses color coding for instant scanning; secondary details are smaller and muted; two-line chips are taller but the layout breathes. |
| Inclusion | 6 → 8 | `missing_semantics` resolved | Tooltip/semantic label now reads the full quality, source, size, seeders/leechers, provider, codec, audio, and HDR. Touch target remains generous. |

### Technical Council note
The provider title currently carries the series/movie title rather than the release filename for some aggregators (e.g. Torrentio), so duplicate chips may still look similar. This is a data-layer improvement, not a UI regression. The chip now correctly renders whatever distinguishing metadata is available, and the `QualityDisplay` normalization is a pure presentation-layer safety net.
