---
title: "Goal: Implement Design Council P0 fixes for X-Seed detail screen"
project_note: goal
type: project-note
tags: [project, X-Seed]
created: 2026-07-12
status: active
---

# Goal: Implement Design Council P0 fixes for X-Seed detail screen

## Objective

Apply all P0 recommendations from the Design Council review of the X-Seed detail screen to bring the screen from a conditional pass to a stronger UX state.

## Status

**Completed** on 2026-07-12.

## Acceptance criteria

1. ✅ `StreamFilterBar` is wired into `_DetailStreamsSection` and filters streams by quality, source, size, and seeders.
2. ✅ The bottom "Copy Magnet" button is removed; users can still copy magnets from the per-stream action sheet.
3. ✅ Stream chips and subtitle language chips have explicit semantic labels (via `Tooltip`).
4. ✅ Disabled actions (Open in Stremio, action-sheet items) have explanatory tooltips.
5. ✅ `flutter analyze` reports zero issues.
6. ✅ `flutter test` passes (1751 tests, 0 failures).

## Approach

Followed the agentic loop: DECOMPOSE → RETRIEVE → EXECUTE → VERIFY → PERSIST.

## Implementation diary

[[ciel/diary/2026-07-12-detail-screen-p0-implementation.md]]

## Decision record

[[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]

## Related files

- `x_seed/lib/src/features/ui/detail/detail_screen.dart`
- `x_seed/lib/src/features/ui/detail/stream_action_sheet.dart`
- `x_seed/lib/src/features/ui/shared/subtitle_language_chip.dart`
- `x_seed/lib/l10n/app_en.arb`
- `x_seed/lib/l10n/app_es.arb`
- `x_seed/test/ui/detail_screen_test.dart`
