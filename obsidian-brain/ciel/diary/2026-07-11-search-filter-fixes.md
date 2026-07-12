---
type: diary
date: 2026-07-11
project: X-Seed
title: Search filter fixes — follow-up to phone verification
status: active
created: "2026-07-11T00:00:00Z"
tags: ["diary","session"]
---

# Search filter fixes — follow-up to phone verification

## Goal

Investigate and fix the user's observation that "filters not always apply" during phone search verification.

## What was found

Three related issues in the search/filter pipeline:

1. **Cache-hit path skipped `filterRelevantResults`.** Cached results stored before the relevance filter existed were not re-filtered when read back, so unrelated results could appear.
2. **Stop-word substring false positives.** `filterRelevantResults` matched query words as substrings. A query like "The Lion King" counted the word "the" as a match, so titles such as "Beyblade X 08 - The Mask and the King" or "Re:PETIT - Starting Life in Another World" passed the relevance threshold.
3. **Filter sheet Apply button did not re-run search.** Tapping Apply only dismissed the sheet; filter changes were not reflected until the user searched again, and provider changes were not reflected from cache at all.

## Changes made

- `lib/src/features/providers/search_controller.dart`
  - Extracted shared `_applyFilters(...)` pipeline (adult → relevance → quality → Tier 1) used by both fresh scrapes and cache hits.
  - Cache-hit path now re-applies the full deterministic pipeline.
- `lib/src/features/search/search_result_filters.dart`
  - Added a stop-word set and removed stop words from the word threshold matcher.
  - Full phrase check still uses the original query, preserving exact-title matches.
- `lib/src/features/ui/search/search_filters_sheet.dart`
  - Apply button now re-runs the current query with `forceRefresh: true` so all active filters (including provider selection) are immediately reflected.
- `test/ui/search_controller_test.dart`
  - Added regression tests for cache-path relevance filtering and stop-word handling.

## Verification

- `flutter analyze --fatal-infos` — no issues.
- `flutter test` — **1,746 tests passed**.
- Re-deployed to the OnePlus CPH2583 (after uninstalling to clear stale cache) and verified fresh searches for "Blade Runner", "Harry Potter", and "Interstellar" return relevant results.
- Onboarding needs to be completed on the device before further manual testing.

## Screenshots

Saved in `x_seed/.scratch/wireless-debug/2026-07-11/`.

## Recommended next steps

- Complete onboarding on the physical device and re-run the original queries ("The Matrix", "Inception", "Naruto", "The Lion King") to confirm the cache-re-filtering and stop-word fixes.
- Consider whether stricter title matching (e.g., whole-word or stemming) is desired if low-quality substring matches remain visible.
