---
title: "Final decision — WP-07 abandoned in favor of adoption-first strategy"
type: decision
tags: [decision, council, wp-07, dart_ipfs, strategy]
status: decided
decision_date: 2026-07-09
effective_date: 2026-07-09
created: 2026-07-09
---

# Final decision — WP-07 abandoned in favor of adoption-first strategy

## Decision

WP-07 (core modularization redesign) is **abandoned as originally specified**. The project will adopt an **adoption-first strategy** (Option C):

- Stop all work aimed at extracting `dart_ipfs_core` or moving protobuf-bearing CID methods into a core package.
- Redirect effort toward documentation, examples, community outreach, and a lightweight HTTP API wrapper that matches how the Dart/Flutter ecosystem actually uses IPFS.
- Revisit architecture only after `dart_ipfs` has real downstream consumers and a concrete use case for protocol-agnostic core primitives.

## Council of Five second-pass vote

| Council Member | Option A (defer) | Option B (redesign properly) | Option C (adoption-first) | Preferred |
|----------------|------------------|------------------------------|---------------------------|-----------|
| Coherence      | 6                | 4                            | 9                         | **C**     |
| Capability     | 6                | 3                            | 9                         | **C**     |
| Safety         | 6                | 3                            | 8                         | **C**     |
| Efficiency     | 8                | 2                            | 9                         | **C**     |
| Evolution      | 3                | 7                            | 8                         | **C**     |
| **Total**      | **29**           | **19**                       | **43**                    |           |

**Result:** Unanimous — all five Council members prefer **Option C**.

## Why this is the right decision

1. **Architectural evidence.** Kubo, Helia, rust-ipfs, js-ipfs, and go-libp2p all keep CID/multihash/multibase in protocol-agnostic packages with no protobuf dependency. The official CID spec defines only binary and multibase-string encodings. The original WP-07 plan — moving `CID.fromProto()`/`toProto()` into `dart_ipfs_core` — would have created a core package that violates the boundary every other IPFS implementation respects.
2. **Zero demand.** `dart_ipfs` has 0 pub.dev dependents. No GitHub issues, discussions, StackOverflow questions, Reddit threads, or blogs request modularization or core-only imports. The Dart/Flutter IPFS community overwhelmingly uses HTTP API clients, not full nodes.
3. **Risk vs. reward.** A proper spec-aligned refactor (Option B) would touch ~43 source files and ~260 test files, risk another 231-analysis-error event, and benefit no one. The cost is not justified by the current state.
4. **Strategic priority.** The project cannot build a healthy ecosystem on architecture alone. Adoption must come first; architecture will follow from validated user needs.

## Conditions for revisiting WP-07

Reconsider modularization only when one or more of the following are true:

1. `dart_ipfs` reaches **≥5 pub.dev dependents** (or a similarly concrete adoption signal).
2. A downstream package explicitly requests protocol-agnostic CID/multihash/multibase primitives without the full IPFS protocol stack.
3. The project has the resources to execute Option B correctly: extract a protobuf-free `dart_ipfs_core` and move protobuf methods into protocol-specific packages, matching go-cid / js-multiformats / rust-cid.

## If modularization is revisited

The original WP-07 design is **discredited**. Any future modularization must follow the spec-aligned pattern:

- `dart_ipfs_core` contains only protocol-agnostic primitives:
  - CID: `fromBytes()`, `toBytes()`, `fromString()`, `toString()`, `version`, `codec`, `multihash`
  - Multihash, Multibase, Multicodec, UnsignedVarint
  - Block and BlockStore interfaces
- `CID.fromProto()` / `CID.toProto()` are **not** in core. They live in protocol-specific packages (Bitswap, DAG-PB, IPNS, DHT) or remain in the umbrella package as convenience helpers.

## Immediate next steps

1. Update `doc/specs/features/MODULARIZATION_SPEC.md` to reflect that WP-07 is abandoned and replaced by an adoption-first strategy.
2. Update `AGENTS.md` to remove WP-07 from active/deferred work packages and add the adoption-first note.
3. Optionally begin scoping an HTTP API wrapper or docs sprint, pending user approval.
4. Track pub.dev dependent count monthly.

## Related notes

- [[ciel/kg/decisions/2026-07-09-wp07-council-decision|Initial Council decision (deferred)]]
- [[ciel/kg/decisions/2026-07-09-wp07-research-synthesis|Research synthesis]]
- [[ciel/kg/decisions/2026-07-09-wp07-council-proposal|Original WP-07 proposal]]
- [[ciel/projects/dart_ipfs/dart_ipfs|dart_ipfs project overview]]
