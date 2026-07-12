---
title: "Coverage suite after adding `cli` tag with extended timeout"
type: diary
tags: [diary, session]
created: 2026-07-09
status: active
---

# Coverage suite after adding `cli` tag with extended timeout

Date: 2026-07-09
Project: dart_ipfs

## Commands run

```bash
# 1. Static analysis
dart analyze --fatal-infos

# 2. Host unit tests
dart test --reporter=compact --no-color

# 3. Coverage collection
dart test --coverage=coverage --no-color
 dart pub global run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info --packages=.dart_tool/package_config.json --report-on=lib
```

## Results

| Step | Result | Details |
|------|--------|---------|
| `dart analyze --fatal-infos` | Pass | No issues found |
| `dart test --reporter=compact --no-color` | Pass | 3478 passed, 8 skipped |
| `dart test --coverage=coverage --no-color` | Pass | 3478 passed, 8 skipped |

## Line coverage

From `coverage/lcov.info` (reported on `lib` only):

- `LH:` (lines hit) = 21,538
- `LF:` (lines found) = 25,106
- **Line coverage = 85.79%**

This is a hair below the previous 85.81% baseline, likely because the newly included `cli`-tagged tests exercise slightly more uncovered library surface.

## Notes

- No source code changes were made during this run.
- The `cli` tag tests are now completing reliably with the extended timeout.
- All host unit tests pass; Docker-dependent interop tests remain skipped by default.
