---
title: WP-09 Competitor Parity — dart_ipfs
project_note: goal
type: project-note
tags: [goal, wp-09, dart_ipfs, agentic-loop]
status: active
created: "2026-07-09T00:00:00.000Z"
updated: "2026-07-09T00:00:00.000Z"
---

# Goal: WP-09 Competitor Parity

Close the competitor-parity gaps listed for WP-09 in [[ciel/projects/dart_ipfs/dart_ipfs|overview]] and `AGENTS.md`:

1. IPNI client (`lib/src/routing/ipni_client.dart`).
2. Reframe routing client (`lib/src/routing/reframe_routing.dart`).
3. Circuit relay HOP/STOP client (`lib/src/transport/circuit_relay_client*.dart` and `circuit_relay_service.dart`).

## Constraints

- Work only on WP-09 scope. Do not touch WP-07 modularization or unrelated files.
- Preserve existing code style and conventions.
- Maintain the Iron Law of verification: `dart analyze`, `dart test --reporter=compact`, and coverage after changes. Target 0 new analysis errors, all existing tests pass, coverage >= 80%.
- Do not replace local `lib/src/core/cid.dart` imports with `package:dart_ipfs_core/dart_ipfs_core.dart`.
- Use `package:dart_ipfs/src/...` imports in tests, not relative imports.

## Task list

1. Retrieve existing implementations and tests for the three targets.
2. Identify parity gaps (spec compliance, edge cases, missing test coverage, integration with `ContentRouting`/`IPFSNode`).
3. Implement IPNI client parity improvements with tests.
4. Implement Reframe routing client parity improvements with tests.
5. Implement circuit relay HOP/STOP client parity improvements with tests.
6. Verify with `dart analyze`, `dart test`, and coverage.
7. Persist: diary entry + update `overview.md` WP-09 status.

## Acceptance criteria

- `dart analyze` reports 0 new errors in WP-09 files.
- `dart test --reporter=compact` reports all existing tests still pass (3459+ passed, 7 skipped interop).
- Line coverage stays >= 80%.
- New tests cover the added parity behavior without using external network calls.
