---
title: X-Seed — UI / UX / Routing
project_note: subsystem
type: project-note
tags: [subsystem, project, X-Seed]
created: 2026-07-11
status: active
---

# X-Seed — UI / UX / Routing

Mobile-first Flutter user interface, navigation, and design system.

## Summary

X-Seed implements a mobile-first, dark-first Flutter app with a 5-tab bottom navigation shell powered by `go_router`'s `StatefulShellRoute.indexedStack`. The app emphasizes explicit user action, neutral default sorting, comprehensive but tier-gated filtering, and a rich detail screen with real-time stream health and Stremio integration. Navigation supports the custom `xseed://` deep link scheme with security sanitization. Localization uses Flutter's ARB format with English as the primary locale.

## Key files

| File | Purpose |
|------|---------|
| `x_seed/lib/src/app.dart` | Root `XSeedApp` widget, `MaterialApp.router`, theme/locale, root detection, share handling |
| `x_seed/lib/src/routing/app_router.dart` | `GoRouter` with `StatefulShellRoute.indexedStack`, onboarding gate, deep link redirect |
| `x_seed/lib/src/routing/app_routes.dart` | Route path constants, `xseed://` deep link sanitization and validation |
| `x_seed/lib/src/features/ui/search/search_screen.dart` | Search UI with explicit trigger, provider chips, sort control |
| `x_seed/lib/src/features/ui/search/search_filters_sheet.dart` | Modal bottom sheet with filter sections |
| `x_seed/lib/src/features/ui/detail/detail_screen.dart` | Content detail view with hero poster, metadata, stream list, episode selector, actions |
| `x_seed/lib/src/features/ui/detail/stream_filter_bar.dart` | Compact stream filters (quality, source, size, seeders) — wired into `_DetailStreamsSection` as of 2026-07-12 |
| `x_seed/lib/src/features/ui/detail/stream_action_sheet.dart` | Action sheet for stream actions (open, copy, share) |
| `x_seed/lib/src/features/ui/detail/health_section.dart` | Provider health sparkline cards |
| `x_seed/lib/src/features/ui/detail/subtitle_section.dart` | OpenSubtitles search, download, and preview |
| `x_seed/lib/src/features/ui/settings/settings_screen.dart` | Grouped settings with 9 sections |
| `x_seed/lib/src/features/ui/settings/community_plugins_section.dart` | Community plugin management |
| `x_seed/lib/src/features/ui/settings/tracker_optimization_section.dart` | 7 tracker optimization settings |
| `x_seed/lib/src/features/ui/onboarding/stremio_setup_view.dart` | Stremio addon setup helper |
| `x_seed/lib/src/features/ui/status_dashboard.dart` | Real-time node status tiles |
| `x_seed/lib/src/features/ui/shared/safe_network_image.dart` | Network image loader with HTTPS upgrade and retry |
| `x_seed/lib/l10n/app_en.arb` | English localization strings |
| `docs/specs/UI_UX_SPEC.md` | Design system specification |

## App shell and navigation

- **5-tab navigation**: Search, Browse, Watchlist, Status, Settings.
- **`StatefulShellRoute.indexedStack`**: Each tab owns its own navigator state, preserving scroll position and tab state.
- **Detail route**: Full-screen route above the shell (`parentNavigatorKey: rootKey`) for deep-linkable content views.
- **Onboarding gate**: `OnboardingController` forces first-run users through `/onboarding` before accessing the shell.
- **Deep links**: Custom `xseed://` scheme with sanitized query parameters (max 256 chars, regex validation for IMDb IDs). `xseed://player` and `xseed://stremio` handled for external launches; navigation deep links converted to in-app routes.

## Search flow

- **Explicit search**: No debounced auto-search; user taps the search button or submits via keyboard.
- **Provider filtering**: Multi-select checkboxes in the filter sheet, provider chips in the search screen.
- **Neutral sorting**: Default is now relevance-based (2026-07-11) to surface core title matches; user can switch to alphabetical, date added, response time, or seed count.
- **Optional sorts**: Date added, response time, seed count (with a disclaimer that it "may highlight content with higher engagement. This does not constitute an endorsement").
- **Filter sheet**: Draggable modal with 6 sections (providers, type, year, IMDb rating, genre, runtime). Tier 3 filters (IMDb rating, genre, runtime) are gated by `tier3FiltersAvailableProvider`.
- **Static genre list**: `_commonGenres` (18 genres) is used until `cachedGenresProvider` from `MetaCacheRepository` is implemented.

## Detail screen

- **Hero poster**: Full-width 16:9 backdrop with `SafeNetworkImage` (HTTPS upgrade, retry logic).
- **Metadata sections**: Title, year, runtime, genres, plot, cast/director, plus a content-type chip when type is not `movie`.
- **Series episode selector**: Season and episode dropdowns rendered above the stream list when `meta.type == 'series'` and `meta.episodes` is non-empty.
- **Stream list**: `Wrap` of quality `ActionChip`s; health-enriched via `healthControllerProvider`.
- **Stream filter bar**: `StreamFilterBar` is wired into `_DetailStreamsSection`. It watches `streamFiltersProvider` and applies quality/source/size/seeders filters when more than one stream is present.
- **Health dashboard**: Real-time peer counts from trackers via `StreamHealthService`. Loading state matches the loaded card shape using `SkeletonBox`.
- **Actions**: Watchlist (primary elevated), share and open in Stremio (secondary outlined). Copy magnet was moved to the per-stream `StreamActionSheet` to avoid ambiguity.
- **Subtitles**: OpenSubtitles search by `imdbId`; requires API key in settings. Non-IMDb IDs show a content-type-aware empty state.

## Design Council review (2026-07-12)

The detail screen was reviewed by the new Design Council of Five. **Verdict: conditional pass.**

Scores:
| Member | Score | Flags |
|--------|-------|-------|
| Clarity | 6 | hidden_state, jargon, weak_hierarchy, inconsistency, cognitive_overload |
| Inclusion | 5 | missing_semantics, color_only, small_touch_target, dynamic_type_failure |
| Efficiency | 6 | extra_steps, missing_feedback, error_prone |
| Aesthetics | 6 | weak_hierarchy, typography_issue |
| Actionability | 6 | hidden_action, weak_cta, dead_end, missing_guidance, intent_mismatch |

**All P0-P2 fixes implemented on 2026-07-12:**

P0:
1. ✅ Wire `StreamFilterBar` into the streams section.
2. ✅ Remove misleading bottom "Copy Magnet" button.
3. ✅ Add semantic labels to stream and subtitle chips (replaced `👤` with "Seeders:").
4. ✅ Explain disabled states with tooltips (Open in Stremio, action-sheet items).

P1:
5. ✅ Expand health card labels from `S:`/`L:` to full words.
6. ✅ Add season/episode selector for series.
7. ✅ Add content-type-aware subtitle empty state.
8. ✅ Ensure chip touch targets meet 48dp.
9. ✅ Support dynamic type scaling.

P2:
10. ✅ Improve visual hierarchy of bottom actions (`OutlinedButton` for secondary).
11. ✅ Style health loading state as a skeleton card.
12. ✅ Polish subtitle preview dialog (title, card, monospace).
13. ✅ Add undo action to watchlist success SnackBar.
14. ✅ Add content-type badge near metadata.

Verification: `flutter analyze` 0 issues, `flutter test` 1751 passed.

Full decision record: [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]
Implementation diaries: [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md]], [[ciel/diary/2026-07-12-detail-screen-px-implementation.md]]

## Settings screens

- **Grouped layout**: Material 3 `ListTile` groups with section headers.
- **9 groups**: Security, Addon, IPFS, Providers, Playback, Subtitles, Tracker Optimization, Network, App.
- **Biometric gating**: `_authenticateForCriticalAction()` protects export/import key, clear cache, and other critical actions when biometric auth is enabled.
- **Community plugins**: JSON config dialog for Torznab indexers and HTTP providers; file-based discovery from `community_plugins` directory.
- **Tracker optimization**: 7 settings with master toggle; child controls are null-disabled when the master toggle is off.

## Onboarding and Stremio setup

- `StremioSetupView` displays the hardcoded loopback URL `http://127.0.0.1:7979/manifest.json` with a copy button.
- "Auto-detect Stremio" button is a placeholder and not yet functional.
- Static instructions guide manual addon installation.

## Localization

- Flutter ARB format (`app_en.arb`, `app_es.arb`).
- `AppLocalizations.delegate` plus global Material/Cupertino delegates.
- Live switching via Riverpod `themeModeControllerProvider` and `localeControllerProvider`.

## Shared components

- **SafeNetworkImage**: Sanitizes HTTP→HTTPS, handles protocol-relative URLs (`//`), adds `User-Agent` header, retries failed loads (max 2, 500ms delay).
- **Status dashboard**: Adaptive bento grid with 7 tiles (DHT peers, swarms, repo usage, bandwidth, battery, addon, activity); log tail bottom sheet.

## Recent decisions

- Explicit search trigger (no auto-search) to avoid unnecessary provider load.
- Relevance-based default sort for search results (2026-07-11).
- Tier 3 filters gated by availability provider.
- Stream filter bar, episode selector, and Design Council P0-P2 fixes integrated into detail screen (2026-07-12).
- `xseed://` deep links sanitized and validated before navigation.

## Quirks and issues

- `search_filters_sheet.dart` uses a hardcoded `_commonGenres` list with a TODO to swap in `cachedGenresProvider` once implemented.
- `StremioSetupView` "Auto-detect Stremio" button is not yet functional.
- `tier3FiltersAvailableProvider` gates some filters; implementation not visible in reviewed files.
- `app.dart` sets `themeMode: ThemeMode.dark` despite watching a theme mode provider (likely intentional dark-first design).
- `SafeNetworkImage` retry is limited to 2 attempts with 500ms delay; may not recover from persistent network issues.
- Health sparkline remains decorative for screen-reader users (potential future Inclusion pass item).
- Episode selector is UI-only; episode-specific stream filtering requires season/episode metadata on `ProviderStream`.

## Test coverage

- `test/ui/*` — detail screen, search interactions, settings, status dashboard, onboarding, golden tests.
- `test/routing/*` — app router and route updates.
- `test/widget_test.dart` — full `XSeedApp` smoke test with `FakeBackgroundService` and fake root detection.
- Overall coverage: 89.05% line coverage.

## Related

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]]
- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|Providers / Scraper]]
- [[ciel/projects/X-Seed/subsystems/background-services.md|Background Services]]
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|Security / Build / CI]]
- [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md|Design Council detail screen review]]
- [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md|P0 implementation diary]]
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md|P1/P2 implementation diary]]
