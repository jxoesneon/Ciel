---
title: Skeleton loading polish
tags: [project, update, X-Seed]
created: 2026-07-12
status: active
type: project-note
project_note: update
---

# Skeleton loading polish

## Summary

Replaced generic shimmer blocks with element-specific skeletons that mirror the final UI layout, and upgraded the base shimmer animation to a Material-3-style diagonal wave. This reduces perceived loading time and makes every screen's loading state instantly recognizable.

## Research

- Material 3 skeleton loaders use a soft horizontal or diagonal "wave" highlight moving across placeholder shapes (Flutter issue #120285, MUI Skeleton docs).
- Best practice: skeleton geometry should match the real content so the layout does not visually "jump" when data arrives.
- Reduced-motion support is required: static surface when `MediaQuery.disableAnimations` is true.

## Changes

- `lib/src/features/ui/shared/skeleton_shimmer.dart`
  - Rewrote `SkeletonBox` with a 1.8s diagonal wave shimmer using `ShaderMask` + animated `LinearGradient` and `Curves.easeInOutSine`.
  - Added named shape constructors: `SkeletonBox.circle`, `SkeletonBox.chip`, `SkeletonBox.text`, `SkeletonBox.card`.
  - Removed the old generic `SkeletonList`.
- `lib/src/features/ui/shared/specific_skeletons.dart` (new)
  - `SearchSkeleton`: search-result cards with poster thumbnail, title, subtitle, and trailing icon bones.
  - `WatchlistSkeleton`: watchlist list tiles with poster, title, year, and pin icon bones.
  - `DetailSkeleton`: full detail-page layout including hero poster, title, type chip, metadata, plot, episode selector, stream filters, stream chips, and health section.
  - `StreamChipSkeleton`: chip-shaped bone used inside detail streams.
  - `HealthSectionSkeleton`: health sparkline card bone.
- `lib/src/features/ui/detail/detail_screen.dart`
  - `_DetailLoading` now uses `DetailSkeleton` instead of `SkeletonList`.
- `lib/src/features/ui/search/search_screen.dart`
  - Loading state now uses `SearchSkeleton`.
- `lib/src/features/ui/watchlist/watchlist_screen.dart`
  - Loading state now uses `WatchlistSkeleton`.
- `lib/src/routing/app_shell.dart`
  - Added adaptive `NavigationBar.labelBehavior` based on screen width to prevent label overflow on narrow devices.
- `lib/src/features/ui/shared/empty_state.dart`
  - Wrapped the centered empty-state content in a `SingleChildScrollView` to avoid vertical overflow on very small screens.
- `lib/src/features/ui/shared/skeleton_shimmer.dart`
  - Added `axis` parameter to `SkeletonBox` and its named constructors.
  - Implemented auto-detection of shimmer direction from bone aspect ratio: tall bones (`height > width * 1.4`) shimmer vertically, wide bones (`width > height * 1.4`) shimmer horizontally, square-ish bones keep a diagonal wave.
  - Direction follows the longest axis so the motion feels aligned with the content shape and avoids distracting diagonal slashes on elongated elements.
  - Softened the shimmer gradient with transparent feathered ends and overlaid it on a solid bone using `BlendMode.srcOver` + shape clipping (`ClipRRect`/`ClipOval`), replacing the previous hard `baseColor → highlightColor → baseColor` `srcATop` sweep. Fixed an initial inversion bug where the gradient mid-points were brighter than the center, making the shimmer disappear.

## Verification

- `flutter analyze --fatal-infos`: No issues found.
- `flutter test`: 1781 tests passed, 0 failed (added 5 specific-skeleton widget tests in `test/ui/specific_skeletons_test.dart`, 3 adaptive-nav tests in `test/routing/app_shell_test.dart`, 1 scrollable-empty-state test, and 4 shimmer-axis tests in `test/ui/shared_widgets_test.dart`).
- Emulator verification on `Medium_Phone_API_35` (emulator-5554):
  - Skeleton detail loading screen renders correctly in both portrait and landscape.
  - **Horizontal layout fix**: `DetailScreen` hero banner previously used `width * 9/16`, filling the entire landscape screen and pushing content off-screen. Updated to use `shortestSide * 9/16` in both `detail_screen.dart` and `DetailSkeleton`, producing a consistent, reasonable banner height in landscape.
  - **Smallest-display test (480x854, Unihertz Jelly 2E, 3")**: bottom navigation labels overflowed and the search empty-state body overflowed by 3px.
    - Fixed `AppShell` to hide labels on very narrow screens (`<320dp`), show only the selected label on medium-narrow screens (`320dp–420dp`), and show all labels on wider screens.
    - Fixed `EmptyState` to wrap its content in a `SingleChildScrollView`, preventing body overflow when the empty state doesn't fit vertically.
  - **Largest-display test (1848x2960, Galaxy Tab S9 Ultra, 14.6")**: detail screen renders well; all bottom-nav labels fit; hero banner stays proportional; content is readable.
  - Verified after fixes across all three resolutions: portrait, landscape, smallest, and largest displays all show consistent, non-overflowing layouts.
  - Soft shimmer overlay verified on the restarted emulator: detail skeleton renders with feathered highlight edges instead of the previous hard gradient cut-off.

## Design Council note

| Lens | Impact | Rationale |
|------|--------|-----------|
| Clarity | improved | Loading placeholders now preview the actual layout, so users understand what is loading. |
| Efficiency | improved | Less visual re-layout when content arrives; users can scan placeholders while waiting. |
| Aesthetics | improved | Wave shimmer is softer and less mechanical than the previous horizontal sweep. |
| Inclusion | maintained | Reduced-motion fallback remains; bones are excluded from screen readers. |

## Related

- [[ciel/diary/2026-07-12-detail-episode-filtering-wiring.md|Detail episode filtering wiring diary]]
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md]]
