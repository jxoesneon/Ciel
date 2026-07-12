---
title: Council of Five — WP-07 Deferral Decision
type: diary
date: 2026-07-09
tags: [diary, dart_ipfs, wp-07, council, decision]
project: dart_ipfs
status: active
created: "2026-07-09T00:00:00Z"
---

# WP-07 Council Deferral — 2026-07-09

## Context

The user requested proceeding with the Council of Five evaluation for **WP-07** (core modularization redesign) in the `dart_ipfs` project.

## Council Invocation

Five Council members were invoked in parallel:

1. **Coherence** — architectural consistency
2. **Capability** — technical feasibility
3. **Safety** — risk posture
4. **Efficiency** — effort and maintenance cost
5. **Evolution** — future adaptability

## Scoring

Each member scored three options on a 10-point scale (50 points total):

| Option | Description | Score |
|--------|-------------|-------|
| Option 1 | Proceed with core modularization now | **31/50** |
| Option 2 | Defer WP-07 indefinitely, keep monolith | **32/50** |
| Option 3 | Abandon modularization entirely | **20/50** |

## Council Preferences

- **Majority preferred Option 2 (defer):**
  - Coherence
  - Safety
  - Efficiency

- **Preferred Option 1 (proceed):**
  - Capability
  - Evolution

## Verdict

**Defer WP-07 indefinitely and keep the monolithic core.**

## Rationale

- The umbrella CID implementation in the monolith exposes protobuf-specific methods (`fromProto`, `toProto`, `computeForData`, `hashType`, `version`) that are not suitable for a protocol-agnostic `dart_ipfs_core` package.
- The project is still green and has no downstream consumers requiring a standalone full-core API.
- Splitting the core prematurely would introduce library-URI churn, test import mismatches, and ongoing maintenance overhead without an immediate payoff.

## Actions Taken

- Updated `doc/specs/features/MODULARIZATION_SPEC.md` with **section 10** documenting the deferral, rationale, and revisit conditions.
- Updated `AGENTS.md` WP-07 status to **Deferred by Council of Five decision (2026-07-09)**.
- Committed both documentation files.
- Created a formal decision record at `ciel/kg/decisions/2026-07-09-wp07-council-decision.md`.

## Conditions for Revisit

Re-open WP-07 only if one or more of the following become true:

1. A downstream consumer requires a standalone `dart_ipfs_core` package with full CID/protobuf capability.
2. The monolithic core becomes a measurable bottleneck for build, test, or release velocity.
3. A protobuf-free core design is established and validated against existing functionality.

## Note

The Obsidian Local REST API at `http://127.0.0.1:27123` was unavailable earlier, so this entry was written directly to the vault filesystem. Backticks and em-dashes have been preserved as requested.
