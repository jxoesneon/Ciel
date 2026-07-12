---
title: SwarmService per-CID health refresh
type: diary
created: 2026-07-12
tags: [x-seed, ipfs, swarm, diary]
status: active
---

# SwarmService per-CID health refresh

## Goal

Resolve the TODO at `lib/src/features/ipfs/swarm_service.dart:280` so that `SwarmService.refreshHealth()` queries real per-CID peer data instead of being a no-op.

## Data source

`dart_ipfs` exposes `IPFSNode.findProviders(cid)`, which returns a `Future<List<String>>` of provider peer IDs for a given CID. This is the closest available per-CID peer metric; `dart_ipfs` does not expose a dedicated "peers interested in this CID" API.

## Changes made

- `lib/src/features/ipfs/ipfs_service.dart`
  - Added `Future<int> countPeersForCid(String cid)` to the `IpfsService` interface.
  - Implemented it in `DartIpfsService` via `_node?.findProviders(cid).length` with graceful fallback to `0` on error or when the node is not started.
  - Implemented a deterministic mock in `DevIpfsService` for dev/test builds.

- `lib/src/features/ipfs/swarm_service.dart`
  - Replaced the `_supportsPerSwarmRefresh` guard and TODO with a real `refreshHealth()` implementation.
  - Iterates all tracked swarms in parallel, calls `IpfsService.countPeersForCid`, and updates each `SwarmHealth` record via `updateHealth()`.
  - Sets `peerCount` and `seederCount` to the provider count; `leecherCount` remains `0` because `dart_ipfs` does not expose leecher data.
  - Failures for individual swarms are logged and do not block other swarms.

- Tests
  - `test/ipfs/swarm_service_test.dart`: expanded `refreshHealth` tests; added `_ThrowingIpfsService`; updated `_FakeIpfsService` to support `countPeersForCid`.
  - `test/ipfs/dev_ipfs_service_test.dart`: added deterministic mock value test.
  - `test/ipfs/dart_ipfs_service_test.dart`: added pre-start zero test.
  - `test/ipfs/dart_ipfs_service_extended_test.dart`: added mocked `findProviders` success/failure/not-started tests.
  - `test/bridge/background_status_provider_test.dart`: updated fake `IpfsService` to implement the new method.

## UI/dashboard

No UI file changes were required. `SwarmHealthHero` already consumes `SwarmHealthSummary.fromService(SwarmService)`, so once `refreshHealth()` populates the records the dashboard summary reflects real data. Existing dashboard widget tests continue to pass.

## External package changes

None. The implementation uses the existing `dart_ipfs` `IPFSNode.findProviders(cid)` API; no modifications to the `IPFS` package were necessary.

## Verification

- `flutter analyze lib/src/features/ipfs/ipfs_service.dart lib/src/features/ipfs/dev_ipfs_service.dart lib/src/features/ipfs/swarm_service.dart lib/src/features/ui/dashboard/swarm_health_hero.dart --fatal-infos` → `No issues found!`
- `flutter test test/ipfs/swarm_service_test.dart` → 44/44 passed.
- Related tests also passed:
  - `test/ipfs/dev_ipfs_service_test.dart`
  - `test/ipfs/dart_ipfs_service_test.dart`
  - `test/ipfs/dart_ipfs_service_extended_test.dart`
  - `test/bridge/background_status_provider_test.dart`
  - `test/ui/dashboard_widgets_test.dart`

## Blockers / next steps

None. The TODO is resolved.
