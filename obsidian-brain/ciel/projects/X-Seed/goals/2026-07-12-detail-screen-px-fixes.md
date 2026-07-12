---
title: "Goal: Implement remaining Design Council P1/P2 fixes for X-Seed detail screen"
project_note: goal
type: project-note
tags: [project, X-Seed]
created: 2026-07-12
status: active
---

# Goal: Implement remaining Design Council P1/P2 fixes for X-Seed detail screen

## Objective

Complete all remaining recommendations from the Design Council review of the X-Seed detail screen, moving the screen beyond the conditional pass state and closing the identified UX gaps.

## Status

**Completed** on 2026-07-12.

## Acceptance criteria

### P1

1. ✅ Health cards use full words instead of `S:`/`L:` abbreviations.
2. ✅ Series content displays a season/episode selector above the stream list.
3. ✅ Subtitle section shows a content-type-aware empty state for non-IMDb IDs.
4. ✅ Chip touch targets meet the 48dp minimum (padded tap target on stream/subtitle/type chips; global chip theme kept within available API surface).
5. ✅ Dynamic type scaling supported — stream chip labels use ellipsis to avoid overflow with large text.

### P2

6. ✅ Bottom actions use improved visual hierarchy (`OutlinedButton` for secondary actions).
7. ✅ Health loading state is styled as a skeleton card matching the loaded card shape.
8. ✅ Subtitle preview dialog has a title, rounded card styling, and monospace subtitle text.
9. ✅ Watchlist success SnackBar includes an undo action.
10. ✅ A content-type badge is shown near the metadata when the type is not `movie`.

### Verification

11. ✅ `flutter analyze` reports zero issues.
12. ✅ `flutter test` passes (1751 tests, 0 failures).

## Related

- [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]
- [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md]]
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md]]
- `x_seed/lib/src/features/ui/detail/detail_screen.dart`
- `x_seed/lib/src/features/ui/detail/health_section.dart`
- `x_seed/lib/src/features/ui/detail/subtitle_section.dart`
- `x_seed/lib/src/theme/x_seed_theme.dart`
