---
title: "Search result filtering: relevance + adult content"
type: decision
project: X-Seed
tags: [decision, x-seed, search, filtering, adult-content, relevance]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Search result filtering: relevance + adult content

## Context

`SearchController` queries every enabled provider in parallel, applies a small set of quality filters (garbage titles, audio keywords), Tier 1 filters (type/year), and a user-selected sort. Two gaps were reported by users:

1. Adult content appeared in search results even when the adult setting was disabled.
2. Results were often unrelated to the query (e.g., searching "NASA" returned anime and unrelated movies).

The user also asked whether sort/filter logic was being applied correctly. Investigation showed sorting *was* correct; the problem was that the data being sorted was noisy and unsafe.

## Decision

Add two new pure filter steps inside `SearchController.search()` *before* quality filtering:

1. **Adult title filter** (`filterAdultResults`): a conservative regex of adult-content keywords applied to every `SearchResult.title` whenever the adult-content setting is off. This is a second line of defence after provider-level adult-provider exclusion.
2. **Relevance filter** (`filterRelevantResults`): titles must contain either the full normalised query phrase or a minimum number of query words:
   - Queries of 2 words or fewer: all query words must appear in the title.
   - Queries of 3+ words: at least half the words (rounded up) must appear.

Both filters use `TitleNormalizer` to strip release-group brackets, quality/source tags, and separators before matching.

The existing adult-provider blocklist (`_adultProviderIds`) was also updated to match the set of actually registered community providers.

## Rationale

- Provider-level adult filtering alone is insufficient because general/community trackers return adult torrents for non-adult queries.
- Broad query matches from torrent trackers are a known source of noise; requiring some word overlap dramatically reduces unrelated results without needing a full search index or ML model.
- Filtering in `SearchController` (rather than in each provider) keeps the rule centralised, testable, and easy to tune.
- Browse uses `ScraperManager` directly with neutral seed queries, so relevance filtering is intentionally scoped to `SearchController` only.

## Consequences

- Search results are now safer and more relevant by default.
- Very broad queries (e.g., single common words) may return fewer results; this is acceptable because the previous behaviour was returning garbage.
- Titles that legitimately contain adult keywords (e.g., "Sex and the City", "The Naked Gun") are preserved by the conservative pattern.
- Future work: consider a user-facing "strict match" toggle or per-provider relevance thresholds.

## Verification

- Added unit tests for both filter functions covering adult removal, false-positive preservation, full-phrase matching, word-threshold matching, and short/trivial queries.
- Added `SearchController` integration tests verifying adult provider exclusion, adult title filtering, and unrelated title removal.
- Updated existing UI/golden tests so fake provider titles contain the query used, preventing empty-result golden regressions.
- `flutter analyze --fatal-infos`: clean.
- `flutter test`: all tests pass.
