---
title: IPFS
project_note: hub
type: project
tags: [project, IPFS]
created: 2026-07-12
status: active
---

# IPFS

Production-ready IPFS implementation in Dart with full protocol support, P2P networking, Gateway mode, and offline capabilities. Supports mobile (Flutter) and web platforms.

## Metadata

| Field | Value |
|-------|-------|
| Owner | jxoesneon |
| Repository | https://github.com/jxoesneon/IPFS |
| Homepage | — |
| Default branch | master |
| Primary language | Dart |
| Visibility | PUBLIC |
| License | MIT License |
| Stars | 10 |
| Forks | 1 |
| Created | 2025-10-02T01:06:17Z |
| Updated | 2026-07-11T06:22:17Z |
| Archived | false |
| Fork | false |

## Topics

bitswap, content-addressed, dht, flutter, ipfs, p2p, pubsub, web

## Use and scope

A full pure-Dart implementation of IPFS: CID, UnixFS, MerkleDAG, Bitswap, DHT, IPNS, PubSub/Gossipsub, HTTP gateway, RPC API, MFS, GraphSync, CAR, plugin system, and Docker/Kubernetes deployment. Targets Flutter mobile, Dart VM desktop/server, and Dart web via a `IPFSWebNode` abstraction.

## Local clone snapshot

- Path: `C:/Users/josee/IPFS`
- Version: **v1.11.7** (released 2026-07-11)
- Tags: `v1.11.1` through `v1.11.7` all present on local and remote
- Architecture: Manager-Handler pattern with LifecycleManager, ContentManager, NetworkManager, ProtocolManager, SecurityManager, MFSManager, Reprovider, PluginManager.
- Platforms: iOS, Android, Windows, macOS, Linux, Web.
- Tests: **3478 passing, 8 skipped, 0 failing** on Windows VM; interop tests requiring Kubo/Helia run separately in CI.
- Coverage: **85.79%** line coverage (exceeds 80% target).
- `dart analyze`: 0 issues.
- Working tree: **clean** — all changes committed as of v1.11.7.
- pub.dev: `dart_ipfs 1.11.7`, `dart_ipfs_core 1.11.5`, `dart_ipfs_quic 0.2.0`.
- Docker images: `ghcr.io/jxoesneon/dart-ipfs:1.11.7`, `:1.11`, `:1`, `:latest` (plus `-debug` and `-builder` variants).
- CI: all workflows green (test, build, docker, docs, publish, interop, k8s, codeql).
- Monorepo: `packages/dart_ipfs_core` (stable core), `packages/dart_ipfs_quic` (QUIC transport via `quic_lib`), `melos.yaml` workspace.
- Specs: 26/26 tracked specs Complete.

## Expanded knowledge

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]] (hub)
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]]
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]]
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]]

## Subsystem drill-down

- [[ciel/projects/IPFS/subsystems/core.md|Core Node & Managers]]
- [[ciel/projects/IPFS/subsystems/network-protocols-routing.md|Network, Protocols & Routing]]
- [[ciel/projects/IPFS/subsystems/services.md|Gateway, RPC & Pinning]]
- [[ciel/projects/IPFS/subsystems/storage.md|Storage]]
- [[ciel/projects/IPFS/subsystems/platform-utils-cli.md|Platform, Utilities & CLI]]
- [[ciel/projects/IPFS/subsystems/proto.md|Protobuf Messaging]]
- [[ciel/projects/IPFS/subsystems/transport.md|Transport Layer]]

## Related

- [[ciel/projects.md|Projects index]]
- [[ciel/projects/quic_lib/quic_lib.md|quic_lib]] (QUIC dependency)
- [[ciel/projects/X-Seed/X-Seed.md|X-Seed]] (downstream consumer)
- [[ciel/projects/dart_ipfs/dart_ipfs|dart_ipfs operational overview]] (operational note)
