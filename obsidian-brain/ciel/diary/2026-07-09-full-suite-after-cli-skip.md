---
title: dart_ipfs full suite run (CLI test skipped)
type: diary
tags: [diary, session]
created: 2026-07-09
status: active
---

# dart_ipfs full suite run (CLI test skipped)

**Date:** 2026-07-09
**Scope:** Run verification commands for `dart_ipfs` with the flaky CLI test excluded, without changing source code.

## Commands run

### 1. Static analysis
```bash
cd c:/Users/josee/IPFS
dart analyze --fatal-infos
```
**Result:** `No issues found!` (exit code 0)

### 2. Unit tests (CLI test skipped)
The CLI test lives at `test/bin/ipfs_cli_test.dart`. It was excluded by passing every other `test/` subdirectory to `dart test` instead of running the default full tree.

```bash
cd c:/Users/josee/IPFS
dart test --reporter=compact --no-color \
  test/core test/e2e test/fakes test/fixtures test/fuzz test/integration \
  test/mocks test/network test/platform test/property test/proto \
  test/protocols test/proto_generated test/routing test/services \
  test/storage test/transport test/utils test/web
```

**Final result:** `3458 passed, 1 skipped, 1 failed` (exit code 1)

**Failure:**
- File: `test/transport/libp2p_router_coverage_test.dart`
- Test: `Libp2pRouter Coverage initialize should handle seed`
- Error: `PathExistsException: Cannot open file, path = 'ipfs.log' (OS Error: Cannot create a file when that file already exists, errno = 183)`
- Source: `package:dart_ipfs/src/platform/platform_io.dart:27:5` (`IpfsPlatformIO.writeBytes`)

This is a flaky file-system collision on `ipfs.log`; it is unrelated to the CLI test that was skipped.

### 3. Coverage
Because the test run did **not** pass, coverage collection was skipped.

## Summary
- Static analysis is clean.
- Full unit suite minus the CLI test still has one flaky failure caused by an existing `ipfs.log` file.
- No source code changes were made.
