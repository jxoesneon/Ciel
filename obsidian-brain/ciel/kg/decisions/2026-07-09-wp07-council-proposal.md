---
title: "Council of Five Proposal — WP-07: Core Modularization Redesign"
type: decision
tags: [decision, wp-07, council, dart_ipfs]
status: deferred
proposal_date: 2026-07-09
created: 2026-07-09
---

# Council of Five Proposal — WP-07: Core Modularization Redesign

## Proposal

Request deliberation and a binding decision from the Council of Five on the future of **WP-07: core modularization redesign** for `dart_ipfs`.

A prior subagent attempted a raw import-replacement from the local `lib/src/core/cid.dart` implementation to `package:dart_ipfs_core/dart_ipfs_core.dart`. That change cascaded into **231 analysis errors** because the umbrella CID exposes methods (`fromProto`, `toProto`, `computeForData`, `hashType`, `version`) that the core package currently lacks. Recovery required restoring `lib/src/core/cid.dart` and approximately 30 source files, adding the missing `RouterInterface.unregisterProtocolHandler`, and converting roughly 260 test files from relative imports to `package:dart_ipfs/src/...` imports.

The project is now stable and green, but WP-07 remains unmerged. This note lays out the viable paths forward and asks the Council to choose one.

## Options

### Option 1 — Extend `dart_ipfs_core` first, then migrate incrementally

- Audit the full umbrella CID API surface and implement the missing methods in `dart_ipfs_core`.
- Use re-exports (`export 'package:dart_ipfs_core/dart_ipfs_core.dart'`) so existing imports continue to resolve.
- Migrate consumers in small, reviewable phases (core types, router interfaces, codec utilities, tests) with verification after each phase.
- Pros: clean architecture, eventual removal of the monolithic core.
- Cons: larger upfront cost; requires coordination with the core package release cycle.

### Option 2 — Defer WP-07 indefinitely and keep the monolithic core

- Abandon modularization for the foreseeable future.
- Continue maintaining `lib/src/core/` as the canonical core implementation.
- Pros: zero churn, lowest risk, preserves current green state.
- Cons: technical debt accumulates; long-term separation of concerns remains blocked.

### Option 3 — Introduce a compatibility adapter / extension layer

- Keep the current monolithic core as the source of truth.
- Add a thin adapter or extension layer in `dart_ipfs` that exposes the same API via the core package where possible, and delegates to the local implementation where necessary.
- Mark the adapter as deprecated with a sunset date so both APIs can coexist during a transition window.
- Pros: avoids a big-bang rewrite; allows incremental adoption.
- Cons: adds an extra abstraction layer that must be maintained and eventually removed.

## Context / Prior Attempt

- **Project:** `dart_ipfs` at `c:/Users/josee/IPFS`.
- **Work package:** WP-07 — core modularization redesign.
- **Failed change:** wholesale replacement of `lib/src/core/cid.dart` imports with `package:dart_ipfs_core/dart_ipfs_core.dart`.
- **Failure mode:** umbrella CID methods missing from core package, causing 231 Dart analysis errors across the tree.
- **Recovery steps:**
  - Restored `lib/src/core/cid.dart` and approximately 30 affected files.
  - Added missing `RouterInterface.unregisterProtocolHandler`.
  - Converted approximately 260 test files from relative imports to `package:dart_ipfs/src/...` imports.
- **Current verification:**
  - `dart analyze` — clean (0 errors).
  - `dart test` — 3,478 passed, 8 skipped.
  - Line coverage — 85.79%.

## Constraints

1. No regression in `dart analyze` or `dart test`.
2. Preserve the umbrella CID API surface (`fromProto`, `toProto`, `computeForData`, `hashType`, `version`).
3. Minimize churn in the ~30 source files and ~260 test files that were just recovered.
4. Maintain >= 80% line coverage (current 85.79%).
5. Avoid breaking downstream package consumers who import `package:dart_ipfs_core/dart_ipfs_core.dart`.
6. Any modularization work must be reviewed by the Council of Five before it is merged.

## Questions for the Council

1. Which of the three options should be approved as the official path for WP-07?
2. If **Option 1** is chosen, what is the recommended sequencing (CID parity, router interfaces, codec utilities, tests, final removal of monolithic core)?
3. If **Option 3** is chosen, where should the adapter/extension layer live, and what is the target deprecation/sunset date?
4. What is the definition of done for WP-07, and what verification gates must be met before it can be considered complete?
5. Should the core package (`dart_ipfs_core`) receive a parallel work package, or should the work remain scoped within `dart_ipfs`?
6. Are there any additional constraints (e.g., release timeline, backward-compatibility window, documentation requirements) the Council wishes to impose?

---

## Council Verdict

**Status:** deferred  
**Decision date:** 2026-07-09  
**Binding decision:** Option 2 (Defer WP-07 indefinitely and keep the monolithic core).

### Scores

| Option | Description | Total Score |
|--------|-------------|-------------|
| Option 1 | Extend `dart_ipfs_core` first, then migrate incrementally | 31/50 |
| **Option 2** | **Defer WP-07 indefinitely and keep the monolithic core** | **32/50** |
| Option 3 | Introduce a compatibility adapter / extension layer | 20/50 |

### Council Vote Split

- **Coherence** — preferred Option 2
- **Safety** — preferred Option 2
- **Efficiency** — preferred Option 2
- **Capability** — preferred Option 1
- **Evolution** — preferred Option 1

Majority: **3 of 5** for Option 2.

### Key Reasons

- Protobuf-specific methods in the umbrella CID implementation (`fromProto`, `toProto`) do not belong in a protocol-agnostic core package.
- The current green state (`dart analyze` clean, `dart test` passing, coverage 85.79%) should not be risked for a non-critical architectural change.
- No downstream consumer currently requires the full `dart_ipfs_core` API, so the modularization cost has no immediate payoff.

### Conditions for Reopening

Revisit WP-07 only if one or more of the following occur:

1. A downstream consumer appears that depends on a shared `dart_ipfs_core` API.
2. The monolithic core becomes a measurable bottleneck for maintenance or feature velocity.
3. A protobuf-free core design is established and accepted by the Council.

### Decision Record

A formal decision record has been written to `2026-07-09-wp07-council-decision.md`.
