---
title: dart_ipfs verification after test fixes
type: diary
tags: [diary, session]
created: 2026-07-09
status: active
---

# dart_ipfs verification after test fixes

Date: 2026-07-09
Project: `c:/Users/josee/IPFS` (dart_ipfs)

## Analysis

`dart analyze` (run from `c:/Users/josee/IPFS`):
- **No issues found.**

## Tests

Command: `dart test --reporter=compact --no-color`

Result: **Aborted / did not complete.**
- Progress reached `+3150 ~7 -27` before the runner crashed.
- All 27 reported failures are **"Failed to load"** errors for test files, not test-logic failures.
- Root cause: **C: drive out of disk space** (`0 GB free`).
- The test runner exhausted `C:\Users\josee\AppData\Local\Temp\dart_test.kernel.*` while copying incremental `.dill` files and threw:

```
FileSystemException: Cannot copy file to ... (OS Error: There is not enough space on the disk, errno = 112)
```

Failing test files (failed to load due to disk space):
- `test\transport\http_gateway_client_test.dart`
- `test\transport\libp2p_router_test.dart`
- `test\transport\pnet\pnet_test.dart`
- `test\transport\quic_transport_test.dart`
- `test\transport\router_events_test.dart`
- `test\transport\stream_controller_lifecycle_test.dart`
- `test\transport\webrtc_signaling_test.dart`
- `test\transport\webrtc_transport_test.dart`
- `test\transport\webtransport\webtransport_datagram_test.dart`
- `test\transport\webtransport\webtransport_session_test.dart`
- `test\transport\webtransport\webtransport_transport_test.dart`
- `test\transport\webtransport_parser_test.dart`
- `test\umbrella_reexports_test.dart`
- `test\utils\base58_test.dart`
- `test\utils\car_test.dart`
- `test\utils\dnslink_resolver_test.dart`
- `test\utils\encoding_test.dart`
- `test\utils\encoding_utils_test.dart`
- `test\utils\generate_message_id_test.dart`
- `test\utils\generic_lru_cache_test.dart`
- `test\utils\keystore_test.dart`
- `test\utils\logger_test.dart`
- `test\utils\message_id_verified_test.dart`
- `test\utils\private_key_test.dart`
- `test\utils\varint_test.dart`
- `test\web\ipfs_web_node_test.dart`
- `test\bin\ipfs_cli_test.dart` (subsequently failed to reload with a socket/pipe error after the runner ran out of temp space)

## Coverage

Not run because the test suite did not complete. The runner crashed before reaching the final summary, so no coverage data was collected.

## Blockers

- **C: drive is full** (0 GB free). Free up disk space and rerun `dart test` before coverage can be computed.
- No code changes were made during this verification run.
