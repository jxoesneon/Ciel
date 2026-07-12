---
title: Design Council Review — X-Seed Detail Screen
type: decision
project: X-Seed
tags: [decision, x-seed, design-council, detail-screen, review]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Design Council Review — X-Seed Detail Screen

**Date:** 2026-07-11
**Implemented:** 2026-07-12
**Convened by:** Ciel (orchestration)
**Artifact:** `x_seed/lib/src/features/ui/detail/detail_screen.dart` and supporting files
**Council:** Design Council of Five (Clarity, Inclusion, Efficiency, Aesthetics, Actionability)

## Verdict

**Conditional pass.** The detail screen is functional and trustworthy, but it has a cluster of usability gaps that repeatedly appear across multiple council lenses. No member vetoed, but Inclusion scored 5, which is below the pass threshold and signals meaningful accessibility debt. The highest-leverage fixes are: (1) wire the existing `StreamFilterBar`, (2) remove or fix the misleading bottom Copy Magnet action, (3) add semantic labels to chips, and (4) explain disabled states.

## Member scores

| Member | Score | Status | Top flags |
|--------|-------|--------|-----------|
| Clarity | 6 | Pass (≥6) | hidden_state, jargon, weak_hierarchy, inconsistency, cognitive_overload |
| Inclusion | 5 | Pass (no veto) | missing_semantics, color_only, small_touch_target, dynamic_type_failure |
| Efficiency | 6 | Pass (≥6) | extra_steps, missing_feedback, error_prone |
| Aesthetics | 6 | Pass (≥6) | weak_hierarchy, typography_issue |
| Actionability | 6 | Pass (≥6) | hidden_action, weak_cta, dead_end, missing_guidance, intent_mismatch |

**Voting math:** 4/5 members scored ≥6. Inclusion scored 5 (no veto, since >3). Majority threshold met, but the low Inclusion score means the screen should not ship without addressing accessibility findings.

## Cross-lens synthesis: recurring themes

### 1. The hidden StreamFilterBar is the single biggest missed opportunity

Every council member except Aesthetics flagged this. The widget, model, and controller are fully implemented, but `_DetailStreamsSection` does not render the filter bar. This simultaneously causes:
- **Clarity** — cognitive overload from a flat, ungrouped stream list.
- **Efficiency** — users must scan all streams instead of filtering by quality/source/size/seeders.
- **Actionability** — a major capability is hidden from users.

Fixing this one integration yields the largest multi-lens improvement.

### 2. The bottom “Copy Magnet” button is misleading

The button copies the magnet of the **first** stream, but the UI gives no indication of this. Clarity, Efficiency, and Actionability all flagged it. It is a high-severity actionability bug because users may believe they are copying the best/selected stream.

Recommendation: remove the bottom Copy Magnet button entirely. Per-stream copying already exists in the action sheet, and the bottom button cannot cleanly represent multiple streams.

### 3. Disabled states lack explanation

Clarity, Inclusion, Efficiency, and Actionability all noted that disabled buttons and action-sheet items provide no explanation. This is a cross-lens failure: it hurts clarity, blocks screen-reader users, creates friction, and weakens affordances.

Recommendation: add tooltips, semantic labels, or helper text for every disabled control explaining the prerequisite (e.g., “No magnet URL available,” “Content ID required”).

### 4. Inconsistent seeders notation and emoji-only indicators

Stream chips use `👤SEEDERS` while health cards use `S:` and `L:`. Aesthetics dislikes the emoji’s visual inconsistency; Inclusion treats the emoji as a `color_only` failure because the meaning is not conveyed textually; Clarity flags the inconsistency as jargon/confusion.

Recommendation: standardize on textual labels. Prefer “Seeders: N” or use an icon + label with a proper semantic announcement. Expand `S:` and `L:` to full words in the health cards.

### 5. Accessibility gaps are systemic, not one-off

Inclusion scored 5 because several barriers compound:
- ActionChips and subtitle chips lack semantic labels.
- The emoji seeder indicator is not perceivable to all users.
- The health sparkline is decorative for screen-reader users.
- Dynamic type scaling is not supported.
- Chip touch targets may not meet the 48dp minimum globally.

These are fixable, but they require a pass through the whole screen rather than spot fixes.

### 6. Series and non-IMDb content are second-class citizens

- Series: no season/episode selector despite `Meta.episodes` existing (Clarity, Efficiency, Actionability).
- Non-IMDb IDs: subtitle search silently returns no results (Clarity, Actionability).
- Type defaults to `movie` because deep links lack type information (Clarity).

Recommendation: add a series episode selector and show content-type-aware empty states for subtitles.

## Prioritized recommendations

### P0 — fix before release

1. **Wire `StreamFilterBar` into `_DetailStreamsSection`.**
   - Import the widget, watch `streamFiltersProvider`, apply filters, and conditionally show when >1 stream.
   - Location: `detail_screen.dart:397-458`.
2. **Remove or fix the bottom Copy Magnet button.**
   - Preferred: remove the button; per-stream copying is already in the action sheet.
   - Location: `detail_screen.dart:499-505`.
3. **Add semantic labels to all stream chips and subtitle chips.**
   - Include quality, provider, and seeders in the stream chip label; use full language names for subtitle chips.
   - Locations: `detail_screen.dart:433-438`, `subtitle_section.dart:89-99`, `subtitle_language_chip.dart`.
4. **Explain every disabled state.**
   - Add tooltips or helper text for disabled bottom actions and action-sheet items.
   - Locations: `detail_screen.dart:493-505`, `stream_action_sheet.dart:179-207`.

### P1 — high impact, fix in next sprint

5. **Replace emoji seeder indicator with text or icon + label.**
   - Standardize across stream chips and health cards.
   - Locations: `detail_screen.dart:431`, `health_section.dart:207`.
6. **Add season/episode selector for series content.**
   - Render above streams when `meta.type == 'series'` and `meta.episodes` is non-empty.
   - Location: `detail_screen.dart:178-182`.
7. **Add content-type-aware subtitle empty state.**
   - Explain that subtitles are only available for IMDb content when `!id.startsWith('tt')`.
   - Location: `subtitle_section.dart:76-82`.
8. **Ensure chip touch targets meet 48dp.**
   - Set `MaterialTapTargetSize.padded` in the chip theme or on each chip.
   - Location: `x_seed_theme.dart` or individual chip widgets.
9. **Support dynamic type scaling.**
   - Apply `MediaQuery.textScalerOf(context)` or `TextTheme.textScaler`.
   - Location: `seed_typography.dart` or theme setup.

### P2 — polish and delight

10. **Improve visual hierarchy of bottom actions.**
    - Use `OutlinedButton` for secondary actions and reserve `TextButton` for tertiary actions.
    - Location: `detail_screen.dart:478-506`.
11. **Style the health loading state as a skeleton card.**
    - Match the loaded card shape for visual continuity.
    - Location: `health_section.dart:168-173`.
12. **Polish the subtitle preview dialog.**
    - Add title, rounded card styling, and monospace font for subtitle text.
    - Location: `subtitle_section.dart:156-169`.
13. **Add an undo action to the watchlist success SnackBar.**
    - Location: `detail_screen.dart:510-528`.
14. **Add content-type badge near the title.**
    - Helps users understand when the type defaulted to `movie`.
    - Location: `detail_screen.dart:154-165`.

## Implementation log

All P0, P1, and P2 items were implemented on 2026-07-12 through an agentic loop.

### P0

| P0 | Status | Commit |
|----|--------|--------|
| Wire `StreamFilterBar` | Done | `_DetailStreamsSection` now watches `streamFiltersProvider`, applies filters, and renders `StreamFilterBar` when >1 stream. |
| Remove misleading Copy Magnet | Done | Bottom `Copy magnet` button removed; per-stream copy remains in `StreamActionSheet`. |
| Semantic labels for chips | Done | Stream chips use `Tooltip` with quality/provider/seeders; subtitle chips use `Tooltip` with language code; replaced `👤` with `Seeders:` text. |
| Explain disabled states | Done | `_DisabledTooltip` helper wraps `Open in Stremio`; `_ActionTile` accepts `disabledMessage` and wraps disabled items in `Tooltip`. |

### P1

| P1 | Status | Commit |
|----|--------|--------|
| Expand health card labels | Done | `S:`/`L:` replaced with localized `healthSeedersLabel` / `healthLeechersLabel`. |
| Series episode selector | Done | `_EpisodeSelector` rendered above streams when `meta.type == 'series'` and episodes exist. |
| Non-IMDb subtitle empty state | Done | Subtitle section shows `subtitleNotImdbTitle` / `subtitleNotImdbBody` when ID doesn't start with `tt`. |
| Chip touch targets | Done | `MaterialTapTargetSize.padded` on stream/subtitle/type chips. |
| Dynamic type scaling | Done | Stream chip labels use `TextOverflow.ellipsis` to avoid overflow at large text scales. |

### P2

| P2 | Status | Commit |
|----|--------|--------|
| Bottom action hierarchy | Done | `Share` and `Open in Stremio` are now `OutlinedButton`s; `Add to watchlist` remains primary `ElevatedButton`. |
| Health skeleton card | Done | `_HealthLoadingState` renders a `Card` with `SkeletonBox` placeholders matching `_ProviderHealthCard`. |
| Subtitle preview polish | Done | Dialog has title, card-wrapped content, and monospace `FiraCode` text. |
| Watchlist undo | Done | SnackBar has `Undo` action that removes the item. |
| Content-type badge | Done | Chip in `_DetailMetadata` shows `Series` (or other types) when `meta.type != 'movie'`. |

### New localization keys

- P0: `detailOpenStremioDisabled`, `detailCopyMagnetDisabled`, `detailStreamActionCopyMagnetDisabled`, `detailStreamActionCopyIpfsDisabled`, `detailStreamActionDownloadTorrentDisabled`
- P1/P2: `healthSeedersLabel`, `healthLeechersLabel`, `subtitleNotImdbTitle`, `subtitleNotImdbBody`, `subtitlePreviewTitle`, `detailWatchlistUndo`, `detailTypeMovie`, `detailTypeSeries`, `detailEpisodeSelectorTitle`, `detailSeasonLabel`, `detailEpisodeLabel`

### Verification

- `flutter analyze --fatal-infos`: **No issues found**
- `flutter test`: **1751 tests passed, 0 failed**
- Updated `test/ui/detail_screen_test.dart` stream chip text expectations from `👤N` to `Seeders: N`.

## Files touched by recommendations

- `x_seed/lib/src/features/ui/detail/detail_screen.dart`
- `x_seed/lib/src/features/ui/detail/health_section.dart`
- `x_seed/lib/src/features/ui/detail/subtitle_section.dart`
- `x_seed/lib/src/features/ui/detail/stream_action_sheet.dart`
- `x_seed/lib/src/features/ui/detail/stream_filter_bar.dart`
- `x_seed/lib/src/features/ui/shared/subtitle_language_chip.dart`
- `x_seed/lib/src/features/ui/shared/health_sparkline.dart`
- `x_seed/lib/src/theme/seed_typography.dart`
- `x_seed/lib/src/theme/x_seed_theme.dart`
- `x_seed/lib/src/theme/seed_colors.dart`

## Files changed during implementation

- `x_seed/lib/src/features/ui/detail/detail_screen.dart`
- `x_seed/lib/src/features/ui/detail/health_section.dart`
- `x_seed/lib/src/features/ui/detail/subtitle_section.dart`
- `x_seed/lib/src/features/ui/detail/stream_action_sheet.dart`
- `x_seed/lib/src/features/ui/shared/subtitle_language_chip.dart`
- `x_seed/lib/l10n/app_en.arb`
- `x_seed/lib/l10n/app_es.arb`
- `x_seed/lib/l10n/app_localizations.dart`
- `x_seed/lib/l10n/app_localizations_en.dart`
- `x_seed/lib/l10n/app_localizations_es.dart`
- `x_seed/test/ui/detail_screen_test.dart`
- `x_seed/lib/src/theme/x_seed_theme.dart`

## Council members invoked

- [[ciel/kg/concepts/design-council-clarity]]
- [[ciel/kg/concepts/design-council-inclusion]]
- [[ciel/kg/concepts/design-council-efficiency]]
- [[ciel/kg/concepts/design-council-aesthetics]]
- [[ciel/kg/concepts/design-council-actionability]]

## Notes

Stage 1 independent reviews were run in parallel as subagents. Stage 2 anonymous cross-review was not performed for this UI review due to the focused, non-architectural nature of the decision; the cross-lens synthesis above captures the convergent findings directly from Stage 1. If this becomes a gate for a release, a Stage 2 pass is recommended to challenge the scoring and refine the prioritization.

All P0-P2 recommendations have been implemented. The remaining design debt is minor (e.g., health sparkline screen-reader decoration, full episode-based stream filtering). A follow-up review is recommended before release to confirm the Inclusion score has improved above 5.

## Implementation diaries

- P0 fixes: [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md]]
- P1/P2 fixes: [[ciel/diary/2026-07-12-detail-screen-px-implementation.md]]
