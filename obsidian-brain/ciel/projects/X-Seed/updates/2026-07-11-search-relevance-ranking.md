---
title: Search Relevance Ranking — 2026-07-11
project_note: update
type: project-note
tags: [project, update, X-Seed]
created: 2026-07-11
status: active
---

# Search Relevance Ranking — 2026-07-11

## What changed

Search results now sort by **relevance** by default instead of alphabetically. A new `scoreRelevance` function ranks each title based on phrase match, phrase position, and Levenshtein similarity, while keeping the previous filtering logic intact. This directly addresses the false-positive feeling where anime episodes or secondary mentions of a movie title appeared first.

## Why it matters

The previous fixes tightened the keyword filter, but many returned results still contain the query words in legitimate-but-secondary contexts (episode titles, release groups, soundtracks). With alphabetical sorting, those entries could appear above the actual movies. Relevance sorting pushes the core titles to the top.

## Verification snapshot

| Query | Top result | Status |
|-------|------------|--------|
| The Lion King | The Lion King (2019) | Good |
| The Matrix | The Matrix (1999) | Good |
| Inception | Inception (2010) | Good |
| Naruto | NARUTO (movie) | Good |

## Follow-up work

- Continue to monitor real-world queries for edge cases.
- Evaluate a structured title parser (year/episode/release-group extraction) if secondary-context matches remain problematic.

## Related diary entry

- [[ciel/diary/2026-07-11-search-relevance-ranking]]
