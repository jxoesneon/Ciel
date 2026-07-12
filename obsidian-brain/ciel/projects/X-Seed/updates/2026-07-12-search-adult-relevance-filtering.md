---
title: Search adult/relevance filtering update
tags: [project, update, X-Seed]
created: 2026-07-12
status: active
type: project-note
project_note: update
---

# Search adult/relevance filtering update

**Date:** 2026-07-12  
**Scope:** X-Seed search pipeline (`SearchController`)

## Problem

Search was returning unrelated and adult content even for clearly unrelated queries. The UI sort appeared to behave oddly because the *input* to the sort was garbage, not because sort was broken.

## Fix summary

- Updated the adult-provider blocklist in `SearchController` to match actually registered community providers.
- Added result-level adult title filtering so adult content cannot leak through general/community providers.
- Added query-relevance title filtering so broad provider matches that do not contain the query words are dropped.
- Re-applied adult filtering to cached results so toggling the adult setting takes effect without waiting for a fresh search.

## Affected files

- `lib/src/features/providers/search_controller.dart`
- `test/ui/fake_content_provider.dart`
- `test/ui/search_controller_test.dart`
- `test/ui/search_interactions_test.dart`
- `test/ui/golden/search_golden_test.dart`
- `test/ui/golden/goldens/search_results.png`

## Verification

- `flutter analyze --fatal-infos`: no issues.
- `flutter test`: all 1743 tests pass.

## Links

- [[ciel/kg/decisions/xseed-search-relevance-adult-filtering.md|Decision record — search relevance & adult filtering]]
- [[ciel/diary/2026-07-12-xseed-search-filtering.md|Session diary]]
