---
title: X-Seed search filtering fix
type: diary
tags: [diary, session]
created: 2026-07-12
status: active
---

# X-Seed search filtering fix

## What happened

User reported that X-Seed search was returning unrelated results, including adult content, even when queries were clearly non-adult. They asked whether filters and sorting were being applied correctly.

## Investigation

- **Sorting is fine**: the UI showed `Sort: Alphabetical` and results were correctly ordered alphabetically. The perceived "randomness" came from the *content* being fed into the sort, not from the sort itself.
- **Root cause 1 — stale adult provider blocklist**: `SearchController._adultProviderIds` listed `community_torlock` and `community_zooqle`, which are not even registered, while missing adult-capable registered providers such as `community_1337x` and `community_torrentgalaxy`.
- **Root cause 2 — no result-level adult filter**: adult titles could leak through any general/community provider even when adult providers were excluded.
- **Root cause 3 — no query-relevance filter**: providers return broad matches; `SearchController` accepted every result the providers returned, so unrelated titles surfaced for unrelated queries.

## Changes

- `lib/src/features/providers/search_controller.dart`
  - Imported `TitleNormalizer` for title normalisation.
  - Moved `adultAllowed` read earlier so cached results also respect the setting.
  - Updated `_adultProviderIds` to the actually registered adult-heavy community providers.
  - Added `_adultTitlePattern` and `filterAdultResults()` for result-level adult title filtering.
  - Added `filterRelevantResults()` to require titles contain the full query phrase or a threshold of query words (all words for queries of 2 words or fewer; half rounded up for longer queries).
  - Wired both filters into `SearchController.search()` before quality filtering, and re-applied adult filtering on cached results.

- `test/ui/fake_content_provider.dart`
  - Added optional `filterByQuery` flag so fakes can behave more like real providers.

- `test/ui/search_controller_test.dart`
  - Updated existing SearchController tests to use query-matching fake titles.
  - Added tests for adult provider exclusion, adult title filtering, relevance filtering (full phrase, threshold, short queries, tiny queries).

- `test/ui/search_interactions_test.dart` and `test/ui/golden/search_golden_test.dart`
  - Updated fake result titles to contain the query used in the tests so relevance filtering does not empty the result list.
  - Regenerated `test/ui/golden/goldens/search_results.png`.

## Verification

- `flutter analyze --fatal-infos`: `No issues found!`
- `flutter test` (full suite): `All tests passed!` (1743 tests at time of run)
- Relevant search tests passed individually before the full run.

## Notes / next steps

- Browse screen is intentionally not filtered by relevance because it uses neutral seed queries (`public domain film`, etc.) and expects broad public-domain results.
- The adult keyword list is conservative to avoid false positives on legitimate titles (e.g., "Sex and the City", "The Naked Gun"). If new adult keywords leak through, expand `_adultTitlePattern` and add regression tests.
- Consider making the relevance threshold configurable per provider in the future, or exposing a "strict match" toggle in search settings.
