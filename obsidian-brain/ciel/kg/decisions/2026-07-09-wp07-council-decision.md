---
title: "Council of Five Decision — WP-07: Core Modularization Redesign"
type: decision
tags: [decision, council, wp-07, dart_ipfs]
status: deferred
decision_date: 2026-07-09
created: 2026-07-09
---

# Council of Five Decision — WP-07: Core Modularization Redesign

## Proposal Summary

The Council of Five deliberated on **WP-07: core modularization redesign** for `dart_ipfs`. A prior attempt to replace the local `lib/src/core/cid.dart` implementation with imports from `package:dart_ipfs_core/dart_ipfs_core.dart` produced **231 analysis errors**, required recovery of roughly 30 source files and 260 test files, and added the missing `RouterInterface.unregisterProtocolHandler` method. The project is currently green and stable, and the Council was asked to choose among three paths forward.

## Options and Scores

| Option | Description | Total Score |
|--------|-------------|-------------|
| Option 1 | Extend `dart_ipfs_core` first, then migrate `dart_ipfs` incrementally | 31/50 |
| **Option 2** | **Defer WP-07 indefinitely and keep the monolithic core** | **32/50** |
| Option 3 | Introduce a compatibility adapter / extension layer | 20/50 |

### Council Vote Split

- **Coherence** — Option 2 (defer / keep monolith)
- **Safety** — Option 2 (defer / keep monolith)
- **Efficiency** — Option 2 (defer / keep monolith)
- **Capability** — Option 1 (extend core / incremental)
- **Evolution** — Option 1 (extend core / incremental)

**Majority:** 3 of 5 for **Option 2**.

## Verdict

WP-07 is **deferred**. The Council of Five selects **Option 2 — defer WP-07 indefinitely and keep the monolithic core** as the official path forward for `dart_ipfs`.

## Rationale

1. **Protocol-agnostic core mismatch.** The umbrella CID implementation exposes protobuf-specific methods (`fromProto`, `toProto`, `computeForData`, `hashType`, `version`) that do not belong in a protocol-agnostic `dart_ipfs_core` package. Moving them into core would either pollute core's abstraction boundary or require a broader redesign that has not been scoped.
2. **Preserve the green state.** The project currently passes `dart analyze` with zero errors, `dart test` with 3,478 passed / 8 skipped, and maintains 85.79% line coverage. The Council judged that risking this state for a non-critical modularization is not justified.
3. **No downstream consumer.** No downstream package or consumer currently requires the full `dart_ipfs_core` API. Without an external driver, the cost of modularization does not have an immediate return on investment.

## Conditions

WP-07 may be revisited only if one or more of the following conditions are met:

1. A downstream consumer appears that depends on a shared `dart_ipfs_core` API.
2. The monolithic core becomes a measurable bottleneck for maintenance velocity, build times, or feature delivery.
3. A protobuf-free core design is established and formally accepted by the Council of Five.

Until then, `lib/src/core/` remains the canonical core implementation for `dart_ipfs`.

## Related Notes

- `2026-07-09-wp07-council-proposal.md` — the original proposal note, updated with this verdict.
- Project context: `dart_ipfs` at `c:/Users/josee/IPFS`.
