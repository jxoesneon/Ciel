---
type: diary
date: 2026-07-11
project: x-seed
tags: "agentic-loop, security, logging"
title: X-Seed Security/Logging Fixes
status: active
created: "2026-07-11T00:00:00Z"
---

# X-Seed Security/Logging Fixes

## Summary

Applied two small defensive security/logging fixes requested for the X-Seed codebase.

## Changes

1. lib/src/features/bridge/open_subtitles_subtitle_service.dart
   - Removed full esponse.body from dart:developer log() calls on HTTP errors.
   - Search failure log now only includes status code.
   - Download request failure log now only includes status code.
   - Existing normal-flow logs and exception logs remain unchanged.

2. lib/src/features/core/blocklist_service.dart
   - Replaced TODO about silently swallowed parse errors with structured debugPrint logging.
   - Log includes the file path (ssets/dmca_blocklist.json) and the error.
   - Wrapped in kDebugMode guard per project logging conventions.
   - Added optional AssetBundle? assetBundle parameter to init() for testability while preserving production behavior (defaults to ootBundle).
   - Graceful degradation preserved: returns empty blocklist on failure.

## Tests

- Created 	est/bridge/open_subtitles_subtitle_service_test.dart with focused tests covering normal search/download and HTTP error paths.
- Updated 	est/core/blocklist_service_test.dart to verify parse failures are logged via captured debugPrint output.

## Verification

- lutter analyze --fatal-infos lib/src/features/bridge/open_subtitles_subtitle_service.dart lib/src/features/core/blocklist_service.dart: No issues found.
- lutter test test/bridge/open_subtitles_subtitle_service_test.dart test/core/blocklist_service_test.dart: 12 tests passed.
- Also ran lutter test test/bridge/subtitle_service_test.dart to ensure existing subtitle tests still pass: 8 tests passed.

## Notes

No blockers. The optional AssetBundle injection in BlocklistService.init() was added to avoid Flutter asset-cache cross-test contamination when verifying error logging.