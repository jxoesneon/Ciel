---
title: WP-08 Spec Compliance — Completed
type: diary
tags: [diary, dart_ipfs, wp-08, spec-compliance]
date: 2026-07-09
status: active
created: "2026-07-09T00:00:00Z"
---

# WP-08 Spec Compliance — Completed (2026-07-09)

Work package WP-08 (spec compliance) is closed.

## Scope implemented

- Gateway content handler (`lib/src/services/gateway/gateway_content_handler.dart`)
  - Serves UnixFS files, directories, and raw blocks via `serveContent`.
  - Detects content type via MIME sniffing and UTF-8 fallback.
  - Supports `Range` requests (`bytes=...`) with 206/416 handling.
  - Adds immutable cache headers and `X-Ipfs-Path` / `X-Content-Type-Options`.

- Gateway directory handler (`lib/src/services/gateway/gateway_directory_handler.dart`)
  - Renders HTML directory listings with parent navigation, name escaping, and human-readable sizes.
  - Resolves sub-paths recursively.
  - Supports `index.html` fallback.
  - Integrates UnixFS HAMT sharded directory traversal via `UnixFSNode` and `resolveHAMTSegment`.

- DHT protocol handler (`lib/src/protocols/dht/dht_protocol_handler.dart`)
  - Kademlia message handling for PING, FIND_NODE, GET_VALUE, and PUT_VALUE.
  - Uses `RateLimiter` for amplification protection.
  - Routes closest peers through the DHT routing table with fallback to connected peers.

- DHT rate limiter (`lib/src/protocols/dht/rate_limiter.dart`)
  - Token-bucket rate limiter with configurable window, queue size, and FIFO eviction.
  - `RateLimiter.fromConfig` builds limits from `RateLimitConfig`.

- PubSub GossipSub adapter (`lib/src/protocols/pubsub/gossipsub/gossipsub_pubsub_adapter.dart`) (new)
  - Bridges the spec-compliant `GossipsubHandler` to the legacy `IPubSub` interface.
  - UTF-8 string serialization, lifecycle management, subscribe/unsubscribe/publish/onMessage.

- `gossipsub.dart` export barrel updated to include the new adapter.

## Tests added/updated

- `test/services/gateway/gateway_content_handler_test.dart`
- `test/services/gateway/gateway_directory_handler_test.dart`
- `test/protocols/dht/rate_limiter_test.dart`
- `test/protocols/dht/dht_protocol_handler_test.dart` (new)
- `test/protocols/pubsub/gossipsub_pubsub_adapter_test.dart` (new)

## Verification status

- Host unit tests: 3478 passed, 8 skipped.
- Line coverage: 85.79%.
- Interop tests (Docker) are skipped in the host run; they run with `dart test --preset interop`.

## Gaps / next steps

- HAMT shard root currently renders only the direct links; full recursive shard listing remains future work.
- Gateway trustless handlers and `application/vnd.ipfs` content-type variants are partially covered by content handler MIME detection but may need explicit handler paths for full spec parity.
- No code changes were made to the dart_ipfs codebase as part of this note.