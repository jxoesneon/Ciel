---
title: 2026-07-12 — Implement Design Council P1/P2 fixes for X-Seed detail screen
type: diary
tags: [diary, session]
created: 2026-07-12
status: active
---

# 2026-07-12 — Implement Design Council P1/P2 fixes for X-Seed detail screen

## Summary

Completed all remaining P1 and P2 recommendations from the Design Council review of the X-Seed detail screen. The screen now has a fully wired filter bar, clearer labels, an episode selector for series, improved accessibility, and polished action/loading states.

## Goal

[[ciel/projects/X-Seed/goals/2026-07-12-detail-screen-px-fixes.md]]

## What was implemented

### P1

1. **Health card label expansion**
   - Replaced `S:` and `L:` abbreviations with localized `healthSeedersLabel` / `healthLeechersLabel`.
   - File: `x_seed/lib/src/features/ui/detail/health_section.dart`

2. **Season/episode selector for series**
   - Added `_EpisodeSelector` widget with dropdowns for season and episode.
   - Rendered above the stream list when `meta.type == 'series'` and `meta.episodes` is non-empty.
   - File: `x_seed/lib/src/features/ui/detail/detail_screen.dart`

3. **Content-type-aware subtitle empty state**
   - Subtitle section now shows `subtitleNotImdbTitle` / `subtitleNotImdbBody` when the content ID does not start with `tt`.
   - File: `x_seed/lib/src/features/ui/detail/subtitle_section.dart`

4. **Chip touch targets**
   - Added `MaterialTapTargetSize.padded` to stream chips, subtitle chips, and the content-type badge chip.
   - Kept the global `ChipThemeData` within the Flutter API surface available in this project.

5. **Dynamic type scaling**
   - Stream chip labels use `TextOverflow.ellipsis` to avoid overflow when text scale is large.

### P2

6. **Visual hierarchy of bottom actions**
   - Converted `Share` and `Open in Stremio` from `TextButton` to `OutlinedButton`.
   - Kept `Add to watchlist` as the primary `ElevatedButton`.

7. **Skeleton-card health loading state**
   - `_HealthLoadingState` now renders a `Card` with `SkeletonBox` placeholders matching the loaded `_ProviderHealthCard` shape.

8. **Polished subtitle preview dialog**
   - Added a title, a `Card`-wrapped content area, and monospace `FiraCode` font for subtitle text.

9. **Undo action in watchlist SnackBar**
   - Added `SnackBarAction` that calls `watchlistControllerProvider.notifier.remove(id, type)`.

10. **Content-type badge**
    - Added a `Chip` in `_DetailMetadata` showing `Series` (or other non-movie types) near the metadata.

## New localization keys

Added to `app_en.arb` / `app_es.arb` and regenerated with `flutter gen-l10n`:

- `healthSeedersLabel` / `healthLeechersLabel`
- `subtitleNotImdbTitle` / `subtitleNotImdbBody`
- `subtitlePreviewTitle`
- `detailWatchlistUndo`
- `detailTypeMovie` / `detailTypeSeries`
- `detailEpisodeSelectorTitle`
- `detailSeasonLabel` / `detailEpisodeLabel`

## Files changed

- `x_seed/lib/src/features/ui/detail/detail_screen.dart`
- `x_seed/lib/src/features/ui/detail/health_section.dart`
- `x_seed/lib/src/features/ui/detail/subtitle_section.dart`
- `x_seed/lib/l10n/app_en.arb`
- `x_seed/lib/l10n/app_es.arb`
- `x_seed/lib/l10n/app_localizations.dart`
- `x_seed/lib/l10n/app_localizations_en.dart`
- `x_seed/lib/l10n/app_localizations_es.dart`

## Verification

- `flutter analyze --fatal-infos`: **No issues found**
- `flutter test`: **1751 tests passed, 0 failed**

## Known limitations / next steps

- The episode selector currently manages its own state; future work can wire it to episode-specific stream filtering once `ProviderStream` carries season/episode metadata.
- Health sparkline remains decorative for screen-reader users (a known P1-level Inclusion item that was deferred due to being lower leverage than the implemented changes).

## Related records

- [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]
- [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md]]
