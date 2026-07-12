---
title: "README Known Limitations & Troubleshooting Review"
type: diary
date: 2026-07-11
tags: ["diary","session"]
status: active
created: "2026-07-11T00:00:00Z"
---

# README Known Limitations & Troubleshooting Review

## Goal
Review `README.md` lines 475-537 (Known Limitations and Troubleshooting) against the current `dart_ipfs` implementation and produce updated Markdown sections.

## Work performed
- Read the existing Known Limitations and Troubleshooting sections in `C:\Users\josee\IPFS\README.md`.
- Verified each limitation against current code:
  - **Web**: WebRTC/WebTransport transports exist but browser-to-browser still requires signaling/relay; direct TCP/UDP listening remains unavailable.
  - **Mobile**: No iOS/Android battery/background optimizations found.
  - **QUIC**: Pure-Dart `dart_ipfs_quic` package exists and is wired via `NetworkConfig.enableQuic`, but defaults to `false` and production interop is still being hardened.
  - **MFS**: Core operations are implemented and exposed via RPC; advanced semantics (symlinks, hard links, chmod/chown, atomic snapshots) are missing.
- Verified each troubleshooting entry against code/config:
  - **Windows libsodium**: `LibsodiumSetup.ensureAvailable()` auto-installs via winget; manual fallback and offline mode added to docs.
  - **Bonjour/mDNS**: `MDNSHandler` already logs the Windows 11 tip; `NetworkConfig.enableMDNS` field confirmed.
  - **AutoNAT**: Spec `NATStatus` only reports `public`/`private`/`unknown`; local `NATType.symmetric` is defined but never assigned. Updated symptom to "private/restricted NAT" and clarified relay fallback.
  - **DHT slow**: `dhtDifficulty` is in `SecurityConfig` (default `0`), not `DHTConfig`. Updated config example.
  - **Gateway 404**: Gateway now falls back to Bitswap (and HTTP gateway fallback when configured). Updated explanation to distinguish local vs. online retrieval and the role of pinning.
- Wrote revised sections to `C:\Users\josee\AppData\Local\Temp\ciel-readme-limits-trouble.md`.
- Did **not** modify `README.md` directly per instructions.

## Key findings
- Config field `dhtDifficulty` is under `SecurityConfig`, not `DHTConfig` (`lib/src/core/config/security_config.dart`, line 13).
- `NetworkConfig.enableQuic` defaults to `false` and the transport is still being hardened (`packages/dart_ipfs_quic/README.md`, lines 8-18).
- `AutoNATHandler` maps `NATStatus.private` to `NATType.restricted`; the `symmetric` enum value exists but is never assigned (`lib/src/core/ipfs_node/auto_nat_handler.dart`, lines 172-187; `NATType` enum line 318-319).
- `GatewayHandler._getBlockByCid()` now falls back to Bitswap before returning 404 (`lib/src/services/gateway/gateway_handler.dart`, lines 1243-1271).

## Blockers / next steps
- None. Parent agent can review the temp file and decide how to merge into `README.md`.

## Files touched
- `C:\Users\josee\AppData\Local\Temp\ciel-readme-limits-trouble.md` (created)
