---
title: "Research synthesis — WP-07 and the right architecture for dart_ipfs"
type: decision
tags: [decision, wp-07, dart_ipfs, research, council]
status: completed
synthesis_date: 2026-07-09
created: 2026-07-09
---

# Research synthesis — WP-07 and the right architecture for dart_ipfs

## Research questions answered

1. How do other IPFS implementations modularize core primitives?
2. What does the Dart ecosystem expect from stable primitive packages?
3. Does any downstream consumer actually need a standalone `dart_ipfs_core`?
4. Where do official IPFS/libp2p specs draw the line between core CID primitives and protocol serialization?

## Key findings

### 1. IPFS implementations separate core primitives from protocol serialization

Across Kubo, Helia, rust-ipfs, js-ipfs, and go-libp2p, the pattern is identical:

- **Core primitives** (`CID`, `Multihash`, `Multibase`, `Multicodec`, `BlockStore`) live in dedicated, protocol-agnostic packages.
- **Protobuf serialization** lives in protocol-specific packages (Bitswap, DAG-PB, IPNS, DHT) or higher-level facades.
- The reference Go CID library (`go-cid`) exposes `Bytes()`, `Cast()`, `Decode()`, `String()` — but **no** `fromProto()`/`toProto()`.

This means the current dart_ipfs umbrella CID, which mixes protobuf methods into the same class, is architecturally inconsistent with the rest of the IPFS ecosystem.

### 2. The CID specification does not include protobuf

The official multiformats/CID spec defines only two encodings:

- Binary: `<version><codec><multihash>`
- String: multibase-encoded binary

Protobuf appears only as a **container format** in individual protocols. Bitswap's `bytes block` field, DAG-PB's `bytes Hash`, and IPNS records all carry raw CID bytes — not a protobuf-encoded CID abstraction. There is no such thing as "the protobuf encoding of a CID" in the specs; there are only protocol messages that happen to contain CID bytes.

Therefore, `CID.fromProto()`/`toProto()` are **protocol-layer convenience methods**, not core CID primitives.

### 3. The Dart ecosystem has strong precedent for minimal primitive packages

- atproto.dart's `multiformats` package is a clean Dart multiformats implementation with minimal deps.
- `dart_multihash` keeps only a `buffer` runtime dependency.
- dart.dev guidance says `lib/src/` is private and umbrella packages should re-export public APIs.
- Pub workspaces + Melos is the modern standard for Dart monorepos.

There is no precedent for putting protobuf serialization inside a CID/multiformats primitive package.

### 4. There is zero downstream demand for `dart_ipfs_core`

- `dart_ipfs` on pub.dev has **0 dependents**.
- `dart_ipfs_core` does not exist as a published package.
- No GitHub issues, discussions, StackOverflow questions, Reddit threads, or blogs request modularization or core-only imports.
- The Dart/Flutter IPFS community overwhelmingly uses HTTP API clients, not full nodes.

Creating a standalone core package today would be a solution in search of a problem.

## Strategic options

### Option A — Defer WP-07 indefinitely (status quo, Council's initial verdict)

Keep the monolithic core. `lib/src/core/cid.dart` remains the canonical CID implementation, protobuf methods and all.

- Pros: zero churn, preserves green state, no risk of regression.
- Cons: technical debt grows; the architecture diverges from the IPFS ecosystem; future package extraction becomes harder.

### Option B — Redesign WP-07: extract a protocol-agnostic core, move protobuf out

Refactor dart_ipfs to match the rest of the IPFS ecosystem:

1. Create `dart_ipfs_core` containing only protocol-agnostic primitives:
   - CID: `fromBytes()`, `toBytes()`, `fromString()`, `toString()`, `version`, `codec`, `multihash`
   - Multihash, Multibase, Multicodec, UnsignedVarint
   - Block and BlockStore interfaces
2. Remove or relocate protobuf methods from the CID class:
   - `CID.fromProto()`/`toProto()` become helper functions in protocol-specific packages (`dart_ipfs_bitswap`, `dart_ipfs_dag_pb`, etc.) or stay in the umbrella.
3. Re-export core from the umbrella package.

- Pros: aligns with official specs and reference implementations; enables a real multi-package ecosystem if demand ever appears; removes the conceptual inconsistency.
- Cons: massive refactor across ~43 source files and ~260 test files; high risk of another 231-analysis-error event; no existing consumer to benefit; requires redesigning protocol packages that do not exist yet.

### Option C — Abandon WP-07 and redirect effort toward adoption

Accept that dart_ipfs has no users today. Instead of restructuring code, invest in:

- Tutorials, examples, and documentation that explain when to use dart_ipfs vs. HTTP API clients.
- Interop and stability improvements that make the existing monolith credible.
- A lightweight HTTP API wrapper for Flutter/Dart developers who want IPFS without running a full node.
- Community engagement (Reddit, Dart/Flutter forums, IPFS Discord).

Revisit modularization only after dart_ipfs has dependents and a concrete use case for `dart_ipfs_core`.

- Pros: addresses the actual blocker (adoption), not an architectural hypothetical.
- Cons: defers the long-term ecosystem architecture indefinitely.

## Preliminary synthesis

The original WP-07 (extract core by moving the umbrella CID, including protobuf methods, into `dart_ipfs_core`) is **architecturally wrong**. It would create a core package that violates the boundary every other IPFS implementation respects.

The Council's initial deferral is therefore correct, but the reasoning can be sharpened: WP-07 should not proceed as designed. If modularization is ever undertaken, it must be Option B — a proper separation of protocol-agnostic primitives from protobuf-bearing protocol packages — not the half-measure that caused 231 analysis errors.

Given zero downstream demand and a green test suite, **Option C** (adoption-first, with architecture deferred) appears to be the most responsible use of effort. Option B is the principled architecture, but doing it now would be premature and high-risk.

## Recommended next step

Send these findings to a second-pass Council of Five. Ask the Council to ratify one of:

1. Option A (defer as-is),
2. Option B (redesign WP-07 into a proper spec-aligned modularization), or
3. Option C (abandon WP-07 and redirect to adoption).

---

## Sources (research subagent reports)

- `ciel/kg/decisions/2026-07-09-wp07-council-proposal.md`
- `doc/specs/OPERATIONS_ECOSYSTEM_SPEC.md`
- `doc/specs/features/MODULARIZATION_SPEC.md`
- Official specs: https://github.com/multiformats/cid, https://specs.ipfs.tech/bitswap-protocol/, https://ipld.io/specs/codecs/dag-pb/spec/, https://ipld.io/specs/transport/car/carv1/
- Reference repos: ipfs/go-cid, multiformats/js-multiformats, ipfs/boxo, multiformats/rust-cid
- dart.dev package/workspace guidance
- pub.dev pages for `dart_ipfs`, `multiformats`, `dart_multihash`, `dcid`, `dart_libp2p`
