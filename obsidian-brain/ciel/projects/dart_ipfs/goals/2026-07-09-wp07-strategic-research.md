---
title: "Strategic research goal — WP-07 and the future of dart_ipfs"
project_note: goal
type: project-note
tags: [goal, wp-07, dart_ipfs, council, research]
status: completed
created: 2026-07-09
---

# Strategic research goal — WP-07 and the future of dart_ipfs

## Objective

Determine the right long-term architectural direction for `dart_ipfs` regarding WP-07 (core modularization redesign), not based on local convenience, but based on what is in the best interest of the project, its users, and the broader IPFS ecosystem.

## Context

WP-07 proposes extracting stable core primitives (CID, multihash, blockstore, codecs, key utilities) into a separate `dart_ipfs_core` package. A previous attempt at raw import replacement caused 231 analysis errors because the umbrella CID exposes protobuf-specific methods (`fromProto`, `toProto`, `computeForData`, `hashType`, `version`) that the core package lacks. The Council of Five initially deferred WP-07 by majority vote on 2026-07-09, citing the risk to the current green state and the lack of a downstream consumer.

The user has challenged us to go deeper: use an agentic loop with revolving subagent slots to research the question globally and decide what is *right*, not merely what is easy.

## Research questions

1. How do other major IPFS/libp2p implementations (Kubo, Helia, rust-ipfs, js-ipfs, go-libp2p) modularize core primitives vs. protocol layers?
2. What are Dart ecosystem best practices for stable primitive packages and monorepos? Are there precedents for `dart_ipfs_core`-style packages?
3. Who actually uses `dart_ipfs` and `dart_ipfs_core` on pub.dev? Are there downstream dependents or community requests?
4. Where do IPFS/libp2p specifications draw the line between core content-addressing primitives and protocol-specific serialization (especially protobuf)?
5. Under what conditions does the long-term benefit of modularization outweigh the short-term risk for a project in dart_ipfs's current state?

## Success criteria

- Produce a synthesis of findings from primary sources (official specs, source repositories, pub.dev, public discussions).
- Present at least three strategic options with evidence-based pros/cons.
- Reconvene the Council of Five for a second-pass decision informed by the research.
- Record the final recommendation and any updated decision in the Obsidian vault.

## Related

- [[ciel/kg/decisions/2026-07-09-wp07-council-decision|Initial WP-07 Council decision (deferred)]]
- [[ciel/projects/dart_ipfs/dart_ipfs|dart_ipfs project overview]]
