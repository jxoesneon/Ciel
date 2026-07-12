---
title: WP-09 Competitor Parity — dart_ipfs
type: diary
tags: [diary, wp-09, dart_ipfs, verification]
date: 2026-07-09
status: active
created: "2026-07-09T00:00:00Z"
---

# WP-09 Competitor Parity — Implementation Diary

## Goal

Close the WP-09 competitor-parity gaps listed in [[ciel/projects/dart_ipfs/dart_ipfs|overview]] and `AGENTS.md`:

- IPNI client
- Reframe routing client
- Circuit relay HOP/STOP client

## What changed

### 1. IPNI client integration (`lib/src/routing/ipni_client.dart`, `lib/src/core/ipfs_node/content_routing_handler.dart`)

- Wired `IPNIClient` into `ContentRoutingHandler` as a fallback provider-discovery strategy.
- `findProviders` now tries DHT first, then IPNI (if configured), then Reframe, then the existing delegated router.
- Added optional `ipniClient` constructor parameter so tests can inject fakes.
- Added `NetworkConfig.ipniEndpoints` so users can opt into IPNI lookups.

### 2. Reframe routing client parity (`lib/src/routing/reframe_routing.dart`)

- Added `useGetApi` flag to `ReframeRoutingClient`.
- When `useGetApi` is `true`, the client uses the modern Delegated Routing V1 HTTP GET API (`/routing/v1/providers/{cid}`) instead of the legacy Reframe POST body.
- Wired `ReframeRoutingClient` into `ContentRoutingHandler` as another fallback layer.
- Added `NetworkConfig.reframeEndpoints` for opt-in configuration.

### 3. Circuit relay HOP/STOP client parity (`lib/src/transport/circuit_relay_client_io.dart`)

- Extended `CircuitRelayClient` to register and handle the STOP protocol (`/libp2p/circuit/relay/0.2.0/stop`).
- Incoming `STOP CONNECT` messages are accepted with a `STATUS OK` response and emit a `circuit_relay_incoming` event, enabling the node to act as a relay destination.
- Renamed the existing HOP handler to `_handleHopMessage` for clarity and added `_handleStopMessage`.

### 4. Configuration updates (`lib/src/core/config/network_config.dart`)

- Added `ipniEndpoints` and `reframeEndpoints` fields with JSON serialization support.
- Default endpoints are empty (opt-in), with `defaultIpniEndpoints` and `defaultReframeEndpoints` constants available for callers.

### 5. Tests updated

- `test/routing/reframe_routing_test.dart`: added GET API tests.
- `test/core/ipfs_node/content_routing_handler_test.dart`: added IPNI and Reframe fallback tests, plus `getStatus` coverage.
- `test/transport/circuit_relay_client_test.dart`: added STOP handling test, updated `_MockRouter` to route STOP messages separately.
- `test/core/ipfs_node/network_handler_io_test.dart` and `test/core/ipfs_node/network_impl_test.dart`: updated expected `registerProtocolHandler` calls from 2 to 3 (HOP + STOP + dialback/autonat).

## Verification

| Command | Result |
|--------|--------|
| `dart analyze` | No issues found |
| `dart test --reporter=compact` | **3482 passed, 7 skipped** (all host unit tests pass) |
| Coverage | **85.77%** line coverage (>= 80% target) |

> Note: the first `dart test` run hit a pre-existing, concurrency-sensitive `TimeoutException` in `test/protocols/pubsub/gossipsub_pubsub_adapter_test.dart`; a re-run passed cleanly. A coverage run also hit a transient load error on `test/protocols/pubsub/gossipsub_test.dart`; re-running produced the reported 85.77% coverage.

## WP-09 status

WP-09 is now **completed** and the gaps for IPNI, Reframe, and circuit relay HOP/STOP have been closed.

## Next steps

- Run the Docker interop preset (`dart test --preset interop`) when the environment is available.
- Consider exposing `IPNIClient` and `ReframeRoutingClient` through the public `lib/dart_ipfs.dart` API if downstream users need direct access.
- Continue WP-08 spec-compliance work and WP-06 autonat/DCUtR/peering integration without touching WP-07 core modularization.
