---
title: IPFS — Knowledgebase
project_note: knowledgebase
type: project-note
tags: [project, knowledgebase, IPFS]
created: 2026-07-11
status: active
---

# IPFS — Knowledgebase

Hub note for the comprehensive `dart_ipfs` knowledgebase. Synthesized from read-only subagent exploration and direct source reading.

## Summary

`dart_ipfs` (v1.11.7) is a production-oriented, pure-Dart IPFS implementation that targets Flutter/mobile, web, and server deployments. It provides content-addressable storage, UnixFS, Bitswap, Kademlia DHT, IPNS, HTTP gateway, RPC API, and browser support via `IPFSWebNode`. The codebase uses a monorepo layout with `packages/dart_ipfs_core` as the stable core and `packages/dart_ipfs_quic` binding the `quic_lib` QUIC stack. All 26 tracked specs are Complete, and the project has shifted to an adoption-first strategy after abandoning WP-07 (modularization).

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/IPFS` |
| Local path | `C:/Users/josee/IPFS` |
| Package | `dart_ipfs` v1.11.7 |
| Core package | `dart_ipfs_core` v1.11.5 |
| QUIC package | `dart_ipfs_quic` v0.2.0 |
| License | MIT |
| Visibility | PUBLIC |
| Stars | 10 |

## Quick status

- **Latest release:** v1.11.7 (2026-07-11)
- **Test status:** 3478 passing, 8 skipped, 0 failing
- **Coverage:** 85.79% line coverage (exceeds 80% target)
- **`dart analyze`:** 0 issues
- **Working tree:** clean — all changes committed
- **CI:** all workflows green (test, build, docker, docs, publish, interop, k8s, codeql)
- **pub.dev:** `dart_ipfs 1.11.7`, `dart_ipfs_core 1.11.5`, `dart_ipfs_quic 0.2.0`
- **Docker:** `ghcr.io/jxoesneon/dart-ipfs:1.11.7` (+ `:1.11`, `:1`, `:latest`, `-debug`, `-builder`)
- **Specs:** 26/26 tracked specs Complete
- **Tags:** `v1.11.1` through `v1.11.7` all present

## Detailed expansion notes

- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]] — Manager-Handler pattern, LifecycleManager, managers, platform abstraction, data flow, monorepo tiers.
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]] — 26-spec implementation inventory, Council artifacts, roadmap highlights.
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]] — verification commands, test structure, coverage, Makefile/Melos, CI workflows.
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]] — pubspec, key dependencies, `dart_ipfs_core`/`dart_ipfs_quic`, stability tiers, migration guide.
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]] — security policy, cryptography, network security, content blocking, known traps, work-package boundaries.
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]] — release history, recent commits, working tree snapshot, notable prior incidents.

## Subsystem drill-down (with Mermaid diagrams)

Each note below maps an `lib/src/` subsystem and includes component, sequence, and data-flow diagrams.

- [[ciel/projects/IPFS/subsystems/core.md|Core Node & Managers]] — `IPFSNode`, lifecycle, managers, DI, plugins, metrics, MFS, config.
- [[ciel/projects/IPFS/subsystems/network-protocols-routing.md|Network, Protocols & Routing]] — `Router`, `MessageHandler`, Bitswap, DHT, PubSub/Gossipsub, IPNS, GraphSync, AutoNAT, DCUtR, Identify, Ping, content/delegated/IPNI/Reframe routing.
- [[ciel/projects/IPFS/subsystems/services.md|Gateway, RPC & Pinning]] — HTTP gateway (path/subdomain/trustless/AutoTLS), Kubo-compatible RPC API, MFS endpoints, remote pinning, IPFS Cluster client.
- [[ciel/projects/IPFS/subsystems/storage.md|Storage]] — `BlockStore`, `WebBlockStore`, `Datastore`, CAR v1/v2, platform backends.
- [[ciel/projects/IPFS/subsystems/platform-utils-cli.md|Platform, Utilities & CLI]] — `IpfsPlatform`, HTTP server adapters, network handler platform switching, utils, `bin/ipfs.dart`.
- [[ciel/projects/IPFS/subsystems/proto.md|Protobuf Messaging]] — `.proto` definitions, generated Dart layout, protocol mappings.
- [[ciel/projects/IPFS/subsystems/transport.md|Transport Layer]] — `RouterInterface`/`Libp2pRouter`, TCP/PNET, QUIC, WebRTC, WebTransport, Circuit Relay.

## Top-level structure

- `pubspec.yaml` — package manifest, depends on `dart_ipfs_core` and `dart_ipfs_quic`.
- `README.md` — 974-line user documentation.
- `AGENTS.md` — AI-agent rules, verification commands, work-package boundaries, known traps.
- `ENGINEERING_NOTES.md` — verification commands, WP boundaries, known traps (replaces some AGENTS.md content).
- `CHANGELOG.md` — history from v1.0.0 through v1.11.7.
- `ROADMAP.md` — v1.10–v3.0 timeline (note: shows v1.11.5 as current, needs update to v1.11.7).
- `Makefile` — `analyze`, `test`, `doc`, `format`, `protos`, `clean`.
- `analysis_options.yaml` — strict mode.
- `melos.yaml` — monorepo workspace scripts.
- `doc/` — architecture guides, specs, audits, decisions, features.
- `lib/src/core/` — node, config, data structures, IPLD, crypto, interfaces, plugins, MFS, Bitswap, peer, peering.
- `lib/src/protocols/` — Bitswap, DHT, IPNS, PubSub/Gossipsub, GraphSync, AutoNAT, DCUtR, Identify, Ping, peering, connection manager.
- `lib/src/services/` — Gateway, RPC, pinning, block store service, content service.
- `lib/src/transport/` — libp2p router, WebRTC, WebTransport, Circuit Relay, PNET, HTTP gateway client, QUIC stub.
- `lib/src/routing/` — content routing, delegated routing, DNSLink, IPNI, Reframe.
- `lib/src/storage/` — Hive datastore.
- `packages/dart_ipfs_core/` — stable CID/block/codec/crypto primitives.
- `packages/dart_ipfs_quic/` — QUIC transport foundation backed by `quic_lib`.
- `test/` — 150+ test files across core, protocols, services, transport, interop, proto, routing, network.

## Architecture at a glance

The codebase follows a **Manager-Handler** pattern with dependency injection:

```
IPFSNode (Facade)
├── LifecycleManager (Orchestrator)
│   ├── ContentManager
│   │   ├── DatastoreHandler
│   │   ├── BitswapHandler (optional)
│   │   └── DenylistService (optional)
│   ├── NetworkManager
│   │   ├── NetworkHandler (IO/Web)
│   │   ├── DHTHandler (optional)
│   │   ├── ContentRoutingHandler (optional)
│   │   └── BitswapHandler (optional)
│   ├── ProtocolManager
│   │   ├── PubSubHandler (optional)
│   │   ├── DHTHandler (optional)
│   │   ├── IPNSHandler (optional)
│   │   └── ContentRoutingHandler (optional)
│   ├── MFSManager
│   ├── Reprovider (optional)
│   └── PluginManager
└── ServiceContainer (DI)
```

## Key files for deeper context

1. `ENGINEERING_NOTES.md` — rules, verification, traps, WPs.
2. `doc/ARCHITECTURE.md` — Manager-Handler pattern.
3. `lib/dart_ipfs.dart` — public barrel.
4. `lib/src/core/ipfs_node/ipfs_node.dart` — main facade (655 lines).
5. `lib/src/core/builders/ipfs_node_builder.dart` — builder pattern (246 lines).
6. `doc/specs/IMPLEMENTATION_INVENTORY.md` — 26/26 specs complete.
7. `ROADMAP.md` — v1.10–v3.0 timeline and Council verdicts.
8. `doc/monorepo.md` — monorepo tiers and import migration.
9. `pubspec.yaml` — dependencies and overrides.
10. `doc/MAINTAINER_GUIDE.md` — release checklist.
11. `doc/specs/PROTOCOL_COMPLIANCE_SPEC.md` — v2.0 compliance goals.

## Related

- [[ciel/projects/IPFS/IPFS.md|IPFS overview]]
- [[ciel/projects/dart_ipfs/dart_ipfs.md|dart_ipfs operational overview]]
- [[ciel/projects.md|Projects index]]
- [[ciel/projects/quic_lib/quic_lib.md|quic_lib]] (QUIC transport dependency)
- [[ciel/projects/X-Seed/X-Seed.md|X-Seed]] (downstream consumer)
