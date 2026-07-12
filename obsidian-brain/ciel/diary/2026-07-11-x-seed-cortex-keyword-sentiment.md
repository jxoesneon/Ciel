---
title: "2026-07-11 X-Seed Cortex Keyword & Sentiment Heuristics"
type: diary
tags: [diary, session]
created: 2026-07-11
status: active
---

# 2026-07-11 X-Seed Cortex Keyword & Sentiment Heuristics

## Summary
Implemented local heuristic keyword extraction and sentiment analysis for the X-Seed Cortex feature.

## Changes
- Created lib/src/features/cortex/heuristic_text_analysis.dart with:
  - extractKeywords(String text) — lowercases, strips non-letters, removes stopwords, returns top 10 frequent words.
  - nalyzeSentiment(String text) — counts positive/negative words and returns a normalized SentimentResult.
- Updated lib/src/features/cortex/heuristic_cortex_service.dart to delegate extractKeywords and nalyzeSentiment to the helper functions via a heuristics import prefix.
- Added unit tests in 	est/cortex/heuristic_text_analysis_test.dart.
- Updated existing 	est/cortex/heuristic_cortex_service_test.dart to reflect the new real behavior (still no changes to ecommend or elevanceScore).

## Verification
- lutter analyze --fatal-infos lib/src/features/cortex -> No issues found.
- lutter test test/cortex -> All 72 tests passed.

## Notes
- Initial run showed unrelated/flaky failures in provider_quality_scorer_test.dart and transient state in heuristic_cortex_service_test.dart; re-run after updating outdated placeholder tests passed cleanly.
- No blockers remain.