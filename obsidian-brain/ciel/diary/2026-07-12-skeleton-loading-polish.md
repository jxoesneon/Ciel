---
title: Skeleton loading polish
type: diary
tags: [diary, session]
created: 2026-07-12
status: active
---

# Skeleton loading polish

## Goal

Polish the generic shimmer loading blocks to be element-specific and visually beautiful, following Material 3 and modern Flutter shimmer best practices.

## Research: shimmer direction best practices

- Material 3 / M2 guidance recommends a left-to-right "wave" as the canonical skeleton shimmer; it signals progress without implying fake content.
- Perceptual-motion research and common design systems (e.g., Figma community skeleton files, Shopify Polaris, Ant Design) suggest the shimmer should travel along the element's reading axis or longest axis so it doesn't look like a glitch crossing the shape at an odd angle.
- Reduced-motion support is mandatory: disable shimmer and fall back to a static block when the user requests reduced motion.
- Key standards used: match final layout dimensions, keep contrast low, use 1–2s duration, and keep the highlight band around 30–50% of the container.

## What changed

- Upgraded `SkeletonBox` from a basic horizontal gradient sweep to a 1.8s diagonal wave shimmer (`ShaderMask` + `LinearGradient` + `Curves.easeInOutSine`).
- Added named shape constructors for common bones: circle, chip, text line, card.
- Made shimmer direction orientation-aware: tall bones shimmer vertically, wide bones shimmer horizontally, square-ish bones keep the diagonal wave.
- Softened the shimmer gradient with transparent feathered ends (`Colors.transparent` → `highlightColor` at 35% alpha → 65% alpha → 35% alpha → transparent) and overlaid it on a solid bone via `BlendMode.srcOver` with `ClipRRect`/`ClipOval` clipping, replacing the previous hard `baseColor → highlightColor → baseColor` `srcATop` sweep. Fixed an initial inversion where the center alpha was lower than the edge mid-stops, causing the shimmer to vanish.
- Created element-specific skeleton widgets in `lib/src/features/ui/shared/specific_skeletons.dart`:
  - `SearchSkeleton` mirrors search result cards.
  - `WatchlistSkeleton` mirrors watchlist list tiles.
  - `DetailSkeleton` mirrors the full detail page (hero poster, title, metadata, plot, episode selector, stream filters, stream chips, health section).
  - `StreamChipSkeleton` and `HealthSectionSkeleton` for reusable pieces.
- Replaced generic `SkeletonList` usage in `SearchScreen`, `WatchlistScreen`, and `_DetailLoading`.
- Removed the now-unused generic `SkeletonList`.

## Tests

- Added `test/ui/specific_skeletons_test.dart` with 5 widget tests covering `DetailSkeleton`, `SearchSkeleton`, `WatchlistSkeleton`, `HealthSectionSkeleton`, and `StreamChipSkeleton`.
- Added adaptive navigation-bar tests in `test/routing/app_shell_test.dart`.
- Added scrollable `EmptyState` test and 4 shimmer-axis tests in `test/ui/shared_widgets_test.dart`.
- `flutter analyze --fatal-infos`: no issues.
- `flutter test`: 1781 passed, 0 failed.

## Emulator verification

- Used the local emulator `Medium_Phone_API_35` (`emulator-5554`) and resized it with `adb shell wm size/density` to cover display extremes from 2020–2025.
- **Smallest**: 480x854 @ 320dpi (Unihertz Jelly 2E, 3").
  - Bottom navigation labels overflowed/wrapped.
  - Search empty state overflowed by 3px vertically.
- **Largest**: 1848x2960 @ 480dpi (Galaxy Tab S9 Ultra, 14.6").
  - Layout looked good; all nav labels fit; detail hero stayed proportional.
- **Fixes applied**:
  - `DetailScreen` + `DetailSkeleton` hero height: changed from `size.width * 9/16` to `size.shortestSide * 9/16` so landscape no longer fills the screen.
  - `AppShell`: adaptive `NavigationBar.labelBehavior` — hide labels below 320dp, only selected label 320–420dp, all labels above 420dp.
  - `EmptyState`: wrapped content in `SingleChildScrollView` to prevent vertical overflow on tiny screens.
- Verified after fixes across small, default, and large resolutions in both portrait and landscape.
- Emulator stabilized after restart and successfully ran the rebuilt APK; detail skeleton renders correctly with the new soft shimmer overlay.

## Blockers / next steps

- Consider applying the same skeleton pattern to any future screens that currently use `CircularProgressIndicator` or generic placeholders.
