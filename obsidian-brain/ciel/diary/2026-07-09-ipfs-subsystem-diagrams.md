---
title: IPFS subsystem drill-down and Mermaid diagrams
type: diary
tags: ["diary","session"]
status: active
created: "2026-07-09T00:00:00Z"
---

# IPFS subsystem drill-down and Mermaid diagrams

## What was done

Continued the comprehensive expansion of the `dart_ipfs` knowledgebase by drilling into every `lib/src/` subsystem and producing per-subsystem architecture notes with Mermaid diagrams.

1. **Waited for and collected** all 9 background subagent reports covering:
   - Core node & managers
   - Network, protocols, and routing
   - Gateway, RPC, and pinning services
   - Storage architecture
   - Platform abstraction, utilities, and CLI
   - Monorepo / `dart_ipfs_core` / `dart_ipfs_quic`
   - Protobuf messaging system
   - Transport layer

2. **Created 7 new subsystem notes** under `ciel/projects/IPFS/subsystems/`:
   - [[ciel/projects/IPFS/subsystems/core.md|Core Node & Managers]]
   - [[ciel/projects/IPFS/subsystems/network-protocols-routing.md|Network, Protocols & Routing]]
   - [[ciel/projects/IPFS/subsystems/services.md|Gateway, RPC & Pinning]]
   - [[ciel/projects/IPFS/subsystems/storage.md|Storage]]
   - [[ciel/projects/IPFS/subsystems/platform-utils-cli.md|Platform, Utilities & CLI]]
   - [[ciel/projects/IPFS/subsystems/proto.md|Protobuf Messaging]]
   - [[ciel/projects/IPFS/subsystems/transport.md|Transport Layer]]

3. **Generated diagrams** in each note:
   - Component diagrams for every subsystem.
   - Sequence diagrams for node startup, block retrieval, gateway request handling, and storage flows.
   - Class relationships and data-flow diagrams where appropriate.

4. **Added a general system architecture diagram** to [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]] showing how all subsystems (public API, core node, managers, storage, network/protocols/routing, transport, platform/primitives, protobuf) relate to one another.

5. **Updated navigation hubs**:
   - Added a "Subsystem drill-down" section to [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]].
   - Added subsystem links to [[ciel/projects/IPFS/IPFS.md|IPFS overview]] and [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]].

## Key takeaways

- The **Manager-Handler** pattern is the dominant organizing principle: `IPFSNode` → managers → handlers → platform/storage/transport primitives.
- **LifecycleManager** is the central orchestrator with ordered startup/shutdown and rollback on failure.
- **Transport layer** is cleanly abstracted via `RouterInterface`/`Libp2pRouter`, with platform-specific implementations for TCP/PNET, QUIC, WebRTC, WebTransport, and Circuit Relay.
- **Storage** separates content-addressed `BlockStore` from general `Datastore`, with CAR v1/v2 import/export and platform backends (native FS vs IndexedDB).
- **Services** (gateway/RPC/pinning) are modular and Kubo-compatible, including trustless gateway formats and AutoTLS via ACME.
- **Protobuf** is the wire-format backbone for Bitswap, DHT, IPNS, Gossipsub, Circuit Relay, GraphSync, and core data structures.

## Next steps / recommended follow-ups

- Verify that all internal Obsidian links resolve in the vault.
- Consider rendering the Mermaid diagrams with an Obsidian Mermaid plugin to catch syntax issues.
- If the user wants implementation work, use these subsystem notes as the source of truth for scoping changes to a single work-package (WP-06 to WP-09 per `AGENTS.md`).

## Files changed / created

- Created `ciel/projects/IPFS/subsystems/core.md`
- Created `ciel/projects/IPFS/subsystems/network-protocols-routing.md`
- Created `ciel/projects/IPFS/subsystems/services.md`
- Created `ciel/projects/IPFS/subsystems/storage.md`
- Created `ciel/projects/IPFS/subsystems/platform-utils-cli.md`
- Created `ciel/projects/IPFS/subsystems/proto.md`
- Created `ciel/projects/IPFS/subsystems/transport.md`
- Updated `ciel/projects/IPFS/architecture.md` (general diagram + subsystem links)
- Updated `ciel/projects/IPFS/knowledgebase.md` (subsystem drill-down section)
- Updated `ciel/projects/IPFS/IPFS.md` (subsystem links)
