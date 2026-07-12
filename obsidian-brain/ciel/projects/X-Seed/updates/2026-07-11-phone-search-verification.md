---
project_note: update
type: project-note
date: 2026-07-11
project: X-Seed
title: Phone search verification — OnePlus CPH2583
status: active
created: "2026-07-11T00:00:00Z"
tags: ["project-note","update"]
---

# Phone search verification — OnePlus CPH2583

## Device setup

- Device: OnePlus CPH2583, Android 16 (API 36)
- Connection: Wireless ADB TLS via mDNS `_adb-tls-connect._tcp`
- Initial ADB port: `192.168.100.149:38485`
- Reconnected port: `192.168.100.149:33765` (TLS session rotated after timeout)
- Build flavor: `full`

## What was tested

Searched four queries on the physical device using live providers and captured screenshots:

1. **The Matrix** — returned relevant results:
   - `[Dodgy] Transformers Theme Song DVD ~Tra...`
   - `[PS2] Enter The Matrix[SLPS-25254][J...`
   - `A Glitch in the Matrix` (2021)
   - `The Animatrix` (2003)
   - `The Living Matrix` (2009)

2. **Inception** — returned relevant results:
   - `"It's Inception meets Weird Science, with kin..."` (2016)
   - `[MNFansubs] Inception`
   - `Inception` (2010)
   - `Inception Ariadne Learns How To Build Dreams`

3. **Naruto** — returned relevant results:
   - `2017/02/05 "AWAY" Live Performance - naruto2...`
   - `[ TR] TSUKUYOMISUBSN ARUTOCLASSICE 001`
   - `[Mo7tas] Boruto: Naruto Next Generations [BD 1...`
   - `[Naruto-Kun.Hu] Naruto - 114 [1080p].mkv`

4. **xyzqwerty12345** — correctly returned the empty state:
   - `No seeds found`
   - `Try disabling filters or checking provider health.`

Note: `asdfghjkl` returned several `asdfghjkl` (2022) movie entries from providers, so it is not a reliable empty-query probe.

## Bug fix: filters not always applied

User reported that filters did not always apply. Investigation found three related issues:

1. **Cache-hit path skipped the relevance filter.** The **cache-hit path** in `SearchController.search()` re-applied adult and quality filters plus Tier 1 filters, but it **skipped `filterRelevantResults`**. Cached entries that predated the relevance filter could therefore show unrelated results.
   - Fix: extracted a shared `_applyFilters(results, query, filters, adultAllowed)` pipeline in `SearchController` that runs adult → relevance → quality → Tier 1 filters in order. Both fresh-scrape and cache-hit paths now call the same pipeline.
   - Added a regression test: `cached results are re-filtered by relevance`.

2. **Stop words caused false-positive relevance matches.** The relevance filter used substring matching for query words. A query like "The Lion King" would treat "the" as a matching word, causing titles like "Beyblade X 08 - The Mask and the King" or "Re:PETIT - Starting Life in Another World" to pass because they contained the stop word "the" (or other stop words).
   - Fix: added a stop-word set to `search_result_filters.dart` and removed them from the word list used by the threshold matcher. The full phrase check still uses the original query, so exact titles like "The Lion King" are preserved.
   - Added a regression test: `filterRelevantResults ignores stop words when matching titles`.

3. **Filter sheet Apply button did not re-run the search.** Changing filters in the bottom sheet updated the filter state, but tapping **Apply** only dismissed the sheet. The user had to manually search again to see filter changes, and provider changes in particular would not be reflected from cache.
   - Fix: `SearchFiltersSheet` now re-runs the current query with `forceRefresh: true` when Apply is pressed, so all active filters (including provider selection) are immediately reflected in the results.

## Implementation notes

- Added `integration_test` to `dev_dependencies` so physical-device UI tests can be built.
- Added missing assets to `pubspec.yaml`: `assets/top100` and `assets/top100.json` (previously referenced at runtime but not declared).
- Added deep-link query handling to `SearchScreen` so `xseed://search?query=...` auto-populates the search field and submits when the route query changes.
- ADB coordinate taps on the OnePlus CPH2583 Flutter screen were unreliable for the `SearchBar`; deep-link launching proved to be the stable automation path.

## Verification

- `flutter analyze --fatal-infos` — no issues.
- `flutter test` — **1,746 tests passed**.
- Deep-link query handling was guarded with `GoRouterState.of(context)` try/catch so widget tests that render `SearchScreen` outside a `GoRouter` still pass.

## Screenshots

Saved in `x_seed/.scratch/wireless-debug/2026-07-11/`:

- `2026-07-11-092637.png` — The Matrix results
- `2026-07-11-092713.png` — Inception results
- `2026-07-11-092750.png` — Naruto results
- `2026-07-11-092900.png` — xyzqwerty12345 empty state
