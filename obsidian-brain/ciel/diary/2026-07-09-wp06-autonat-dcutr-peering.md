---
title: WP-06 — AutoNAT + DCUtR + Peering Lifecycle Integration
type: diary
date: 2026-07-09
tags: [dart_ipfs, wp-06, autonat, dcutr, peering, lifecycle]
status: active
created: "2026-07-09T00:00:00Z"
---

# WP-06 — AutoNAT + DCUtR + Peering Lifecycle Integration

## Goal
Close the remaining lifecycle-integration gaps for AutoNAT, DCUtR, and peering in the dart_ipfs node, verify all existing tests still pass, and keep coverage above 80%.

## What was changed

### Code
- `lib/src/core/ipfs_node/ipfs_node.dart` (line ~196)
  - Registered `AutoNATHandler` with the node's `LifecycleManager` alongside `DCUtRHandler` and `PeeringService`.
  - This was the only missing lifecycle wire: `DCUtRHandler` and `PeeringService` were already registered, but `AutoNATHandler` was only created as a singleton in `IPFSNodeBuilder` and never started/stopped by the node lifecycle, so its AutoNAT server was never active in a running node.
  - Updated the adjacent comment to reflect that WP-06 services are now lifecycle-managed when registered.

### Tests
- `test/core/ipfs_node/ipfs_node_coverage_test.dart`
  - Added `MockDCUtRHandler` and `MockPeeringService` implementations.
  - Registered both mocks in the test container alongside the existing `MockAutoNATHandler`.
  - Added a dedicated WP-06 integration test: `WP-06 lifecycle: AutoNAT, DCUtR, and Peering are managed by node` that asserts all three services are started on `IPFSNode.start()` and stopped on `IPFSNode.stop()`.
  - Updated the stale comment in the existing `Full start and stop sequence` test that claimed external handlers were not managed by the node.

## Verification results

| Check | Command | Result |
|-------|---------|--------|
| Analysis | `dart analyze --fatal-infos` | 0 issues found |
| Unit tests | `dart test --reporter=compact` | 3460 passed, 7 skipped (Docker-dependent interop tests) |
| Coverage | `dart test --coverage=coverage` + `coverage:format_coverage` | 85.76% line coverage (threshold: ≥80%) |

Targeted WP-06 tests all pass:
- `test/protocols/autonat/autonat_protocol_test.dart`
- `test/protocols/dcutr/dcutr_handler_test.dart`
- `test/core/peering/peering_service_test.dart`
- `test/core/ipfs_node/auto_nat_handler_test.dart`
- `test/core/ipfs_node/node_handlers_test.dart`
- `test/integration/full_nat_test.dart`
- `test/core/ipfs_node/ipfs_node_coverage_test.dart` (new WP-06 assertion)

Note: the first two `dart test --coverage=coverage` runs hit pre-existing flaky timeouts in `test/transport/libp2p_router_coverage_test.dart` and a transient load issue in `test/services/gateway/gateway_directory_handler_test.dart`. Both tests pass individually and in the compact run; the third coverage run completed cleanly and produced the reported 85.76% coverage.

## Next steps
- WP-06 is now closed from a lifecycle-integration standpoint. Any future AutoNAT/DCUtR/peering work should focus on interop behavior (e.g., real libp2p hole-punch smoke tests, Kubo AutoNAT round-trip tests) rather than wiring.
- The working tree currently contains unrelated uncommitted WP-08 changes (gateway content/directory handlers, DHT rate limiter, gossipsub adapter). Those should be committed by their owner work-package before a final release snapshot.
- Consider adding an interop-level test that spins up two nodes (one relayed, one public) and verifies DCUtR hole-punch via the libp2p host, once a stable Docker fixture is available.
