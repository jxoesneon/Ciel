---
title: "Final decision — WP-07 abandoned, adoption-first strategy adopted"
type: diary
date: 2026-07-09
tags: [diary, dart_ipfs, wp-07, council, decision, research]
project: dart_ipfs
status: active
created: "2026-07-09T00:00:00Z"
---

# Final decision — WP-07 abandoned, adoption-first strategy adopted

## Context

The user challenged Ciel to do what is right, not what is easy, regarding WP-07. We ran an agentic-loop research phase with four parallel subagents, then convened a second-pass Council of Five.

## Research findings

1. **IPFS implementations separate core primitives from protocol serialization.** Kubo, Helia, rust-ipfs, js-ipfs, and go-libp2p all keep CID/multihash/multibase in protocol-agnostic packages with no protobuf dependency. Protobuf lives in protocol-specific packages.
2. **The CID spec does not include protobuf.** It defines only binary and multibase-string encodings. Protobuf is a container used by Bitswap, DAG-PB, IPNS, and DHT messages — not a CID encoding.
3. **Dart ecosystem precedent supports minimal primitive packages.** There is no precedent for putting protobuf serialization inside a CID/multiformats core package.
4. **Zero downstream demand.** `dart_ipfs` has 0 pub.dev dependents. No issues, discussions, or forum posts request modularization. The Dart/Flutter IPFS community uses HTTP API clients, not full nodes.

## Second-pass Council of Five vote

| Member | A (defer) | B (redesign properly) | C (adoption-first) | Preferred |
|--------|-----------|------------------------|--------------------|-----------|
| Coherence | 6 | 4 | 9 | C |
| Capability | 6 | 3 | 9 | C |
| Safety | 6 | 3 | 8 | C |
| Efficiency | 8 | 2 | 9 | C |
| Evolution | 3 | 7 | 8 | C |
| **Total** | **29** | **19** | **43** | |

**Unanimous verdict: Option C.**

## Decision

WP-07 as originally specified is **abandoned**. The project will pursue an **adoption-first strategy**:

- Stop modularization work.
- Redirect effort to docs, examples, community outreach, and a lightweight HTTP API wrapper.
- Revisit architecture only after real downstream consumers exist.

If modularization is ever revisited, it must be spec-aligned: a protobuf-free `dart_ipfs_core`, with `CID.fromProto`/`toProto` remaining in protocol/umbrella packages.

## Actions taken

- Updated `doc/specs/features/MODULARIZATION_SPEC.md` with the final decision.
- Updated `AGENTS.md` to mark WP-07 abandoned and document the architectural inconsistency.
- Created/updated Obsidian notes:
  - `ciel/kg/decisions/2026-07-09-wp07-research-synthesis.md`
  - `ciel/kg/decisions/2026-07-09-wp07-final-decision.md`
  - `ciel/projects/dart_ipfs/goals/2026-07-09-wp07-strategic-research.md`
- Committed: `8b01dc6 doc: record final Council of Five decision to abandon WP-07 in favor of adoption-first strategy`.

## Next steps

Pending user approval, begin scoping adoption work: a lightweight HTTP API wrapper, getting-started guides, or community outreach. Track pub.dev dependents monthly.
