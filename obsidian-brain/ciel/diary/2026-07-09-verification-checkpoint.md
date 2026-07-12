---
title: dart_ipfs Verification Checkpoint — 2026-07-09
type: diary
tags: [diary, session]
created: 2026-07-09
status: active
---

# dart_ipfs Verification Checkpoint — 2026-07-09

Run from `C:\Users\josee\IPFS` with no code changes made.

## Git Status

17 working-tree changes (13 modified, 4 untracked):

- Modified:
  - `lib/src/core/config/network_config.dart`
  - `lib/src/core/ipfs_node/content_routing_handler.dart`
  - `lib/src/protocols/dht/dht_protocol_handler.dart`
  - `lib/src/protocols/dht/rate_limiter.dart`
  - `lib/src/protocols/pubsub/gossipsub/gossipsub.dart`
  - `lib/src/routing/reframe_routing.dart`
  - `lib/src/services/gateway/gateway_content_handler.dart`
  - `lib/src/services/gateway/gateway_directory_handler.dart`
  - `lib/src/transport/circuit_relay_client_io.dart`
  - `test/core/ipfs_node/content_routing_handler_test.dart`
  - `test/protocols/dht/rate_limiter_test.dart`
  - `test/services/gateway/gateway_content_handler_test.dart`
  - `test/services/gateway/gateway_directory_handler_test.dart`
  - `test/transport/circuit_relay_client_test.dart`
- Untracked:
  - `lib/src/protocols/pubsub/gossipsub/gossipsub_pubsub_adapter.dart`
  - `test/protocols/dht/dht_protocol_handler_test.dart`
  - `test/protocols/pubsub/gossipsub/gossipsub_pubsub_adapter_test.dart`

## Dart Analysis

```text
Analyzing IPFS...
No issues found!
```

Result: 0 errors, 0 warnings.

## Unit Tests

Command: `dart test --reporter=compact --no-color`

The suite did **not terminate normally**. It reached a stable count, then hung on `test\transport\circuit_relay_client_test.dart` at the test:

> `CircuitRelayClient STOP handling incoming STOP CONNECT replies with STATUS OK and emits event`

Final counts observed before the hang:

- **Passed:** 3476
- **Skipped:** 7
- **Failed:** 2

Two tests failed with the same assertion error:

1. `test\core\ipfs_node\network_handler_io_test.dart` — `NetworkHandler start and stop`
2. `test\core\ipfs_node\network_impl_test.dart` — `NetworkHandler IO Implementation Initialization and start`

Both failures:

```text
Expected: <2>
  Actual: <3>
Unexpected number of calls
```

Both are mock-call-count failures (a handler/initializer is being invoked one more time than expected).

## Coverage

Command: `dart test --coverage=coverage --timeout=30s --no-color`

Coverage collection could not be completed. The VM crashed with:

```text
../../runtime/platform/allocation.cc: 22: error: Out of memory.
version=3.12.2 (stable) on "windows_x64"
```

No fresh `coverage/lcov.info` was produced. The existing `coverage/test/` directory contains stale JSON coverage files from an earlier run, so a line-coverage percentage cannot be reliably reported at this checkpoint.

## Summary

- **Analysis:** clean (0 issues).
- **Tests:** 3476 passed, 7 skipped, 2 failed, and the runner hung on the circuit-relay STOP test.
- **Coverage:** blocked by an out-of-memory crash during collection.
- **Action:** No code changes were made. The next step should be to investigate the two NetworkHandler mock-count failures and the circuit-relay hang before re-running coverage on a machine with more available memory.
