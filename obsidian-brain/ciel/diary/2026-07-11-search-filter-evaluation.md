---
type: diary
date: 2026-07-11
title: 2026-07-11 — Search filter evaluation completed
status: active
created: "2026-07-11T00:00:00Z"
tags: ["diary","session"]
---

# 2026-07-11 — Search filter evaluation completed

## What was done

1. Extracted filter/sort functions from `search_controller.dart` into a new pure-Dart library: `lib/src/features/search/search_result_filters.dart`.
2. Updated `search_controller.dart` to import and re-export the new library.
3. Created `tool/search_filter_evaluation.dart` and `tool/search_queries.dart`.
4. Ran 1000 live searches across 14 providers and computed filter effectiveness metrics.
5. Verified `flutter analyze --fatal-infos` and search-related tests still pass.

## Results

- 298,425 raw results across 1000 queries.
- 93.33% overall reduction after all filters.
- 79.19% removed by adult-provider exclusion.
- 12.97% removed by relevance filtering.
- 1.15% removed by quality filtering.
- 0.02% removed by adult-title filter (36 queries affected).

## Files changed

- `lib/src/features/search/search_result_filters.dart` (new)
- `lib/src/features/providers/search_controller.dart`
- `tool/search_filter_evaluation.dart` (new)
- `tool/search_queries.dart` (new)

## Follow-up: stop-word fix

After the phone verification run, a user observed that "filters not always apply". One contributing factor was that the relevance threshold matcher counted common stop words (e.g., "the", "in") as substring matches, producing false positives on queries like "The Lion King" and "The Matrix". The follow-up session removed stop words from the word matcher while preserving the full-phrase check. See `2026-07-11-search-filter-fixes.md` for the full fix list and verification.

## Next steps

- Persist the full report in the vault.
- Consider running the adult-provider-included variant for comparison.
- Continue monitoring false positives and tune filters as needed.
- Re-run the filter-effectiveness evaluation after the stop-word fix to measure the change in false-positive rate.
