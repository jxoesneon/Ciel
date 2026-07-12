---
title: X-Seed — IPFS / libp2p Node
project_note: subsystem
type: project-note
project: X-Seed
tags: [subsystem, x-seed, ipfs, libp2p, p2p, swarm]
status: active
created: "2026-07-12T07:44:58.464Z"
---

# X-Seed — IPFS / libp2p Node

On-device IPFS/libp2p node integration, adaptive governor, and swarm health tracking.

## Summary

X-Seed runs a full IPFS/libp2p node directly on Android to enable decentralized content addressing, peer exchange, and Bitswap. The node acts as a DHT server with adaptive limits, participating in swarms for content the user is actively streaming while respecting battery, network, and thermal constraints. The node runs in the same background isolate as the addon server, and exposes a local HTTP gateway on `127.0.0.1:8080` for converting CIDs to Stremio-playable URLs.

## Key files

| File | Purpose |
|------|---------|
| `x_seed/lib/src/features/ipfs/ipfs_service.dart` | `IpfsService` interface abstraction |
| `x_seed/lib/src/features/ipfs/swarm_service.dart` | Per-swarm health tracking and classification |
| `x_seed/lib/src/features/ipfs/adaptive_governor.dart` | Battery/network/thermal throttling (per spec) |
| `x_seed/lib/src/features/security/identity_key_provider.dart` | Signing delegate for IPFS identity |
| `docs/specs/IPFS_NODE_SPEC.md` | IPFS node specification (draft) |
| `docs/adr/002-always-on-p2p-node.md` | ADR for always-on P2P node |

## Node configuration

- **Identity**: Ed25519 key generated once on first launch; stored AES-wrapped in `flutter_secure_storage`, with the wrapping key in Android Keystore. The AES key is intentionally unrestricted so the background IP2P service can decrypt the seed without user interaction. Peer ID derived from the public key via multihash identity hash.
- **Bootstrap**: 4 default Kubo/PL bootstrap multiaddrs (configurable in Settings); custom bootstrap nodes allowed.
- **Repository**: `<ApplicationDocumentsDirectory>/ipfs_repo/`, default 2 GB cap, min 512 MB.
- **GC**: Automatic when usage > 90% of cap; oldest unpinned blocks evicted first. Only explicit user pins (watchlist items) preserved.
- **Local gateway**: `http://127.0.0.1:8080/` (configurable), loopback only, CORS `*`.
- **Protocols**: Bitswap, Gossipsub, DHT server (adaptive), Identify, Ping.
- **Transports**: TCP (primary), UDX (UDP-based), WebSocket (fallback). QUIC was a Sprint 10 stretch goal; TCP-only is sufficient for v1.0.0.

## Background lifecycle

The IPFS node runs inside the same isolate as the addon server via `flutter_background_service`:

1. Load identity key from Keystore.
2. Initialize IPFS repo.
3. Bootstrap to DHT.
4. Start local gateway on `127.0.0.1`.
5. Start addon server on `127.0.0.1`.
6. Update notification: "Ready".

Idle power management: after 30 minutes of no Stremio queries, enter low-power mode (DHT maintenance only, 1 query/minute).

## Adaptive governor

Triggers and actions (per `IPFS_NODE_SPEC.md`):

| Condition | Action |
|-----------|--------|
| Battery < 20% | Pause DHT queries; Bitswap only; reduce connection limit to 10 |
| Battery < 10% | Full node pause; notify user |
| Mobile data active | Disable Bitswap; DHT metadata only; disable scraping (configurable) |
| Wi-Fi active | Full operation; normal connection limit (50) |
| Storage > 90% cap | Trigger GC; if still > 95%, pause new Bitswap wants |
| Thermal throttling | Reduce background priority; pause non-essential DHT |
| Doze/App Standby | WorkManager resurrection every 15 min; foreground service keeps node alive when screen on |

Thermal mapping: `none` → full, `light` → -25% DHT, `moderate` → -50% DHT + pause non-essential Bitswap, `severe` → pause DHT, `critical` → full pause + notify.

## Swarm health service

`SwarmService` tracks per-content-swarm health:

- `SwarmStatus.active` — >10 peers
- `SwarmStatus.degraded` — 3–10 peers
- `SwarmStatus.dead` — 0–2 peers and last seen >1 hour ago
- `SwarmStatus.unknown` — insufficient data

Currently, the `IpfsService` interface does not expose per-CID peer counts, so `_supportsPerSwarmRefresh` is `false` and `refreshHealth` is a no-op that logs a warning. The service is wired for future `dart_ipfs` APIs and can be manually updated via `updateHealth()` for testing and UI development.

## Key contract

The IPFS/libp2p layer consumes identity through `IdentityKeyProvider`:

- `sign(Uint8List message)` — preferred signing delegate; private key never leaves the provider.
- `getPublicKeyBytes()` — returns 32-byte Ed25519 public key.
- `exportPrivateKeyBytes()` — controlled, short-lived export of the 32-byte private seed for consumers that require a raw seed (e.g., `dart_ipfs` initialization). Returned buffer must be zeroed by the caller; the provider zeroes temporary buffers before returning.

> **Security note**: `dart_ipfs` currently requires the raw `libp2pIdentitySeed` and does not accept a signer callback. If a signer callback becomes available, the service should switch to `sign()` and never export the seed.

## Error handling

| Error | Handling |
|-------|----------|
| `RepoLockError` | Show error; offer reset or exit |
| `BootstrapTimeout` | Retry with exponential backoff; notify user if all bootstrap fails |
| `QuicLoadError` | Log warning; degrade to TCP; one-time snackbar |
| `KeystoreError` | Fallback to `flutter_secure_storage`; prompt biometric re-enrollment |
| `DiskFullError` | Halt Bitswap; notify user; offer cache purge |

## Battery and memory targets

- Idle drain: < 1.5%/hour
- Active streaming drain: < 8%/hour
- UI isolate: < 150 MB
- Background isolate: < 200 MB
- Total app: < 400 MB

## Test coverage

- `test/ipfs/swarm_service_test.dart` — swarm registration, health classification, manual updates, monitoring lifecycle.
- IPFS node integration tests require an Android emulator and are documented in `IPFS_NODE_SPEC.md` but not yet automated in CI.

## Quirks and issues

- Per-swarm health refresh is currently a no-op because `dart_ipfs` does not expose per-CID peer counts yet.
- QUIC transport was a Sprint 10 stretch goal; TCP-only is used for v1.0.0.
- The private seed must be exported to `dart_ipfs` today; migrating to a signer callback would improve security.

## Related

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]]
- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|Providers / Scraper]]
- [[ciel/projects/X-Seed/subsystems/background-services.md|Background Services]]
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|Security / Build / CI]]
- [[ciel/projects/IPFS/IPFS.md|IPFS]] (upstream dependency)
