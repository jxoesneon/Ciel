---
title: Search Relevance Ranking Implementation
type: diary
tags: [diary, session]
created: 2026-07-11
status: active
---

# Search Relevance Ranking Implementation

## Summary

Deepened the search relevance work by discovering that the main problem was not the keyword filter but the **alphabetical sort order**. Providers return many results that legitimately contain the query words, but those words are often in secondary contexts (episode titles, release group prefixes, etc.). Sorting A→Z pushed those results to the top, making the list feel irrelevant.

Implemented a new `SearchSort.relevance` mode that scores each result by:

1. Whole-word phrase match — highest score.
2. Position of the phrase in the title — earlier is better, capped so phrase matches always beat plain word matches.
3. Levenshtein similarity — small bonus for exact matches.
4. Word-level matches — fallback when the full phrase is not contiguous.
5. A light-normalized title is used for position scoring so release-group prefixes like `[MNFansubs] Inception` are treated as secondary to the bare title `Inception`.

Made relevance the default search sort and kept alphabetical, date added, seed count, and response time as options.

## Files changed

- `lib/src/features/search/search_result_filters.dart` — added `scoreRelevance`, `_lightNormalizeForScoring`, and `SearchSort.relevance` handling in `sortResults`.
- `lib/src/features/providers/search_controller.dart` — default sort is now `SearchSort.relevance`; all sort calls pass the current query.
- `lib/src/features/ui/search/search_screen.dart` — added relevance label and menu item.
- `lib/l10n/app_en.arb`, `app_es.arb`, `app_localizations*.dart` — added `sortRelevance` translations.
- `test/ui/search_controller_test.dart` — updated default-sort test and added two new relevance tests.
- `test/ui/golden/goldens/search_results.png` — regenerated for the new default sort label.

## Verification

- `flutter analyze --fatal-infos` → no issues.
- `flutter test` → 1,751 tests pass.
- Physical device (OnePlus CPH2583) re-deployed and tested with live searches:
  - "The Lion King" — Disney movies now at top, anime episode references pushed down.
  - "The Matrix" — core Matrix films at top.
  - "Inception" — 2010 movie at top, `[MNFansubs] Inception` second.
  - "Naruto" — main Naruto movie at top.

## Next steps

- Monitor user searches for further false positives or misrankings.
- Consider a structured title parser (e.g., extracting year/episode/release group) for even stronger main-title detection.
- Consider using external metadata (Cinemeta, popularity) as an additional ranking signal.
