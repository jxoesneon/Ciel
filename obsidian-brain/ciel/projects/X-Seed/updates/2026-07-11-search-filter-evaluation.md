---
project_note: update
type: project-note
date: 2026-07-11
project: X-Seed
title: Search filter effectiveness evaluation - 1000 live queries
status: active
created: "2026-07-11T00:00:00Z"
tags: ["project-note","update"]
---

# Search filter effectiveness evaluation - 1000 live queries

## Summary

Ran the new `tool/search_filter_evaluation.dart` harness against 1000 real-world search queries and 14 providers. The evaluation measured the impact of each filter stage: adult-provider exclusion, adult-title filtering, relevance filtering, and quality filtering.

## Key results

- **Raw results:** 298,425 (avg 298.4 per query)
- **Adult-provider filter:** removed 236,327 (79.19%)
- **Adult-title filter:** removed 48 (0.02%)
- **Relevance filter:** removed 38,698 (12.97%)
- **Quality filter:** removed 3,444 (1.15%)
- **Final results:** 19,908 (6.67%)
- **Overall reduction:** 278,517 results (93.33%)

## Observations

1. Provider-level adult exclusion is the dominant filter, accounting for nearly 80% of raw result volume.
2. Result-level adult-title filtering catches a small but non-zero number of adult titles that leak through non-adult providers (48 titles across 36 queries).
3. Relevance filtering removes ~13% of results that do not contain the query terms, preventing unrelated content from reaching the user.
4. Quality filtering mostly removes non-video audio releases (FLAC/OST) and a few garbage titles.
5. Average final results per query dropped from ~298 to ~20.

## Harness details

- Tool: `x_seed/tool/search_filter_evaluation.dart`
- Query list: `x_seed/tool/search_queries.dart` (1000 curated queries)
- Providers: 14 registered providers (excludes Torznab, which requires API URL config)
- Runtime: 1:31:32
- Cache: `.scratch/search-eval/raw_cache.json`
- Report: `.scratch/search-eval/report.md`

## Follow-ups

- Consider a future run with `--include-adult` to measure what the provider-level filter actually blocks.
- Evaluate whether the quality filter should be stricter on soundtrack/OST releases.
- Consider a user-facing "strict match" toggle for relevance threshold.
