---
title: "dart_ipfs Final Test Suite & Coverage — 2026-07-09"
type: diary
tags: [diary, session]
created: 2026-07-09
status: active
---

# dart_ipfs Final Test Suite & Coverage — 2026-07-09

## Summary

Ran the final verification pass on `dart_ipfs` at `c:/Users/josee/IPFS`. No source code changes were made.

## 1. Static Analysis

```powershell
dart analyze --fatal-infos
```

- **Result:** `No issues found!`
- **Exit code:** `0`

## 2. Unit Test Suite

```powershell
dart test --reporter=compact --no-color
```

- **Result:** `+3478 ~8: All other tests passed!`
- **Passed:** 3,478
- **Skipped:** 8
- **Failed:** 0
- **Exit code:** 0

The 8 skipped tests are Docker-dependent interop scenarios (run with `dart test --preset interop` inside `test/interop/docker-compose`). All host unit tests pass.

## 3. Coverage Collection

```powershell
dart test --coverage=coverage --no-color
```

- **Result:** `+3476 ~8 -2: Some tests failed.`
- **Passed:** 3,476
- **Skipped:** 8
- **Failed:** 2
- **Exit code:** 1

### Failing tests under coverage run

1. `test\bin\ipfs_cli_test.dart: ls lists directory entries`
2. `test\bin\ipfs_cli_test.dart: pin pins a CID`

Both tests passed in the standard run but failed when the test runner was instrumented for coverage. This points to timing/flakiness introduced by coverage overhead in CLI tests.

### Coverage report (from partial run)

Formatted with:

```bash
dart pub global run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info --packages=.dart_tool/package_config.json --report-on=lib
```

Sums from `coverage/lcov.info`:

- **Lines Found (LF):** 25,106
- **Lines Hit (LH):** 21,539
- **Line coverage:** **85.79%**

This exceeds the project target of **80%** but is based on a coverage run where two tests failed. The percentage is therefore slightly under-counted relative to a fully passing suite.

## 4. Blockers

- Two CLI tests are flaky/failing under `dart test --coverage=coverage`:
  - `ls lists directory entries`
  - `pin pins a CID`
- These must be stabilized before the coverage gate can be declared fully clean.

## 5. No Source Changes

No files in the `dart_ipfs` project were modified during this run.

## Recommended Next Steps

1. Re-run `dart test --coverage=coverage` with `--chain-stack-traces` to capture full failure traces for the two CLI tests.
2. Inspect the CLI test setup/teardown for shared mutable state (directory handles, pin service mocks, temp directories) that coverage instrumentation may slow down enough to expose.
3. Once the two tests are green under coverage, re-compute `lcov.info` for the authoritative percentage.

## Commands Reference

```bash
# Analyze
dart analyze --fatal-infos

# Unit tests
dart test --reporter=compact --no-color

# Coverage
dart test --coverage=coverage --no-color
dart pub global run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info --packages=.dart_tool/package_config.json --report-on=lib

# Compute percentage (PowerShell)
$lf=0; $lh=0; foreach ($line in Get-Content coverage/lcov.info) { if ($line -match '^LF:(\d+)') { $lf += [int]$matches[1] } if ($line -match '^LH:(\d+)') { $lh += [int]$matches[1] } }; "{0:P2}" -f ($lh / $lf)
```

---

#diary #dart_ipfs #verification #coverage
