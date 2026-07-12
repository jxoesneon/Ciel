---
title: "Search result ranking: relevance-based default sort"
type: decision
project: X-Seed
tags: [decision, x-seed, search, relevance, ranking]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Search result ranking: relevance-based default sort

## Context

After tightening the search relevance filter (whole-word matching + Levenshtein similarity), live searches for queries like "The Lion King" still felt off. Providers were returning results that legitimately contained the query words, but those words appeared in secondary contexts: anime episode titles, release group prefixes, soundtracks, etc. The previous default sort was alphabetical, which pushed those secondary results above the actual movies.

## Decision

Implement a dedicated `SearchSort.relevance` mode and make it the default search sort. Results are scored by a new `scoreRelevance` function that rewards:

1. Whole-word phrase matches (highest weight).
2. Query phrase appearing early in the title (small position penalty, capped).
3. Levenshtein similarity to the query (small bonus).
4. Word-level matches when the full phrase is not contiguous.

A light-normalized title is used for position scoring so that release-group prefixes like `[MNFansubs] Inception` are treated as secondary to the bare title `Inception`.

## Consequences

- **Positive:** Core titles now rank above secondary mentions; search results feel dramatically more relevant.
- **Positive:** Users can still switch to alphabetical, date added, seed count, or response time if needed.
- **Neutral:** The filtering logic remains unchanged; we only re-rank the kept results.
- **Risk:** Very short or ambiguous queries may still rank a release-group title high if its normalized title is identical to the query; this is mitigated by the position penalty but worth monitoring.
- **Follow-up:** Consider structured title parsing (year/episode/release group extraction) and/or external metadata signals for further improvements.

## Related

- [[ciel/projects/X-Seed/updates/2026-07-11-search-relevance-ranking.md]]
- [[ciel/diary/2026-07-11-search-relevance-ranking.md]]
- [[ciel/kg/decisions/xseed-search-relevance-adult-filtering.md]]
