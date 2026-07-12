---
title: dart_ipfs
project_note: hub
type: project
tags: [project, dart_ipfs]
created: 2026-07-11
status: active
---

# dart_ipfs

A comprehensive IPFS implementation in Dart (v1.11.7). Repository: https://github.com/jxoesneon/IPFS

## Local paths

- Codebase: `C:\Users\josee\IPFS`
- Pubspec: `pubspec.yaml`
- README: `README.md`

## Verification commands

- Analysis: `dart analyze` — target 0 errors. **Current: 0 issues.**
- Host tests: `dart test --reporter=compact` — **3478 passed, 8 skipped, 0 failed**. The skipped tests are Docker-dependent interop scenarios.
- Interop tests: inside `test/interop/docker-compose`, run `dart test --preset interop test/interop`.
- Coverage: run `dart test --coverage=coverage`, format to `coverage/lcov.info`, target 80% line coverage; **current 85.79%**.

## Current state

- Branch: `master`
- Latest commit: `23aeeb0` — chore: remove temporary build log
- Package version: `1.11.7`
- Latest tag: `v1.11.7` (also `v1.11.6`, `v1.11.5`)
- GitHub release: `v1.11.7`
- pub.dev: `dart_ipfs 1.11.7`, `dart_ipfs_core 1.11.5`, `dart_ipfs_quic 0.2.0`
- Docker images: `ghcr.io/jxoesneon/dart-ipfs:1.11.7`, `:1.11`, `:1`, `:latest` (plus `-debug` and `-builder` variants)
- Platform: Dart VM (Windows, macOS, Linux) and Web (Chrome, Firefox, Safari)
- Working tree: **clean** — all changes committed as of v1.11.7.
- CI: all workflows green (test, build, docker, docs, publish, interop, k8s, codeql).

## Recent changes

- **v1.11.7** (2026-07-11): Fixed publishing regression — removed direct `xml` and `dart_udx` dependencies that broke downstream consumers. Security pins remain as `dependency_overrides`.
- **v1.11.6** (2026-07-11): CI hardening — `IPFSWebNode` starts offline by default, HTTPS redirect fix, interop test formatting, Flutter web compatibility. CI green on Ubuntu, macOS, Windows.
- WP-08 partially closed: gateway content/directory handlers, UnixFS HAMT sharding traversal, DHT protocol handler with rate limiting, and Gossipsub `IPubSub` adapter are implemented and tested. Remaining gaps: full recursive HAMT shard root listing and explicit trustless handler paths.
- DHT provide/find and IPNS resolution now interoperate with Kubo.
- RPC `/api/v0/dag/export` and `/api/v0/dag/import` handlers implemented for CAR exchange.
- WP-09 competitor parity closed: IPNI and Reframe routing clients integrated, circuit relay client handles HOP/STOP.
- WP-06 closed: AutoNAT, DCUtR, and peering wired into lifecycle with end-to-end coverage test.
- WP-07 abandoned after Council of Five final decision. Adoption-first strategy adopted.

## Known traps

- Do not replace local `lib/src/core/cid.dart` imports with `package:dart_ipfs_core/dart_ipfs_core.dart`; the umbrella CID has methods the core package lacks.
- Test files must use `package:dart_ipfs/src/...` imports, not relative `../../../lib/src/...` imports.
- On Windows, restoring files with `git show HEAD:path > file` can corrupt them to UTF-16; use `git checkout HEAD -- <path>` instead.
- IPNS V2 signatures are computed over `ipns-signature:` + the raw DAG-CBOR `data` bytes. Verify using the original serialized `data` bytes and accept both prefixed and raw V2 signatures for interop.
- `xml` and `dart_udx` security pins must stay as `dependency_overrides`, not direct deps. `port_forwarder` constrains `xml ^6.5.0`.

## Active work packages

- None. The project is shifting to an adoption-first strategy.

## Abandoned work packages

- WP-07 — core modularization redesign (abandoned by Council of Five final decision 2026-07-09 after external research; see `ciel/kg/decisions/2026-07-09-wp07-final-decision.md`).

## Recently completed work packages

- WP-08 — spec compliance (partially closed 2026-07-09; implemented and tested, host suite 3478 passed / 8 skipped, coverage 85.79%; gaps: full recursive HAMT shard listing, explicit trustless handler paths)
- WP-09 — competitor parity (closed 2026-07-09; integration + tests pass, coverage 85.77%)
- WP-06 — autonat + DCUtR + peering lifecycle integration (closed 2026-07-09; lifecycle wired end-to-end, tests pass, coverage 85.76%)

## Decisions

- [[ciel/kg/decisions/2026-07-09-wp07-final-decision|WP-07 abandoned by Council of Five]] — adopt an adoption-first strategy (docs, examples, HTTP API wrapper, community outreach). The original WP-07 design is discredited; any future modularization must be spec-aligned and protobuf-free in core.

## Comprehensive knowledgebase

The full knowledgebase with architecture diagrams, subsystem drill-downs, spec compliance, and security details is at:

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]] (hub)
- [[ciel/projects/IPFS/IPFS.md|IPFS — Overview]]
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]]
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]]
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]]
- [[ciel/projects/IPFS/releases/v1.11.6-v1.11.7-stabilization|v1.11.6–v1.11.7 Release]]

## Related

- [[ciel/kg/decisions/obsidian-brain-migration-audit|Obsidian brain migration audit]]
- [[ciel/projects/ciel/ciel|Ciel project overview]]
