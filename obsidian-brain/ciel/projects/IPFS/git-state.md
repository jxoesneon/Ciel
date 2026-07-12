---
title: IPFS — Git State
project_note: update
type: project-note
tags: [project, IPFS]
created: 2026-07-11
status: active
---

# IPFS — Git State

Current release, recent history, working-tree status, and notable recovery events for the local `dart_ipfs` clone.

## Latest release

- **Version:** `dart_ipfs` **v1.11.7** (released 2026-07-11).
- **Tag:** `v1.11.7`.
- **Status:** multi-platform production ready. Published to pub.dev, GitHub release, and GHCR.
- **pub.dev:** `dart_ipfs 1.11.7`, `dart_ipfs_core 1.11.5`, `dart_ipfs_quic 0.2.0`.
- **Docker:** `ghcr.io/jxoesneon/dart-ipfs:1.11.7`, `:1.11`, `:1`, `:latest`.

## Tags

All tags `v1.11.1` through `v1.11.7` verified present on local and remote. No gaps.

## Recent commits (v1.11.6–v1.11.7 stabilization)

| Hash | Message |
|------|---------|
| `23aeeb0` | chore: remove temporary build log |
| `ee22ec9` | fix(pubspec): remove direct xml/dart_udx deps that break downstream consumers |
| `4c44553` | ci(docker): skip SBOM release asset upload to avoid permission failure |
| `687c847` | chore(release): bump version to 1.11.6 and clean publishing artifacts |
| `3c22338` | chore: remove unnecessary dart_style dev dependency |
| `6991c7a` | fix(gateway): include leading slash in HTTPS redirect location |
| `3adbb88` | ci: use expanded test reporter to capture failing test name (temporary) |
| `b3fb689` | fix(test): apply dart_style formatting to interop tests and add dart_style dev dependency |
| `dfd79667` | ci: show formatting diff on Ubuntu (temporary) |
| `a723e24` | fix(test): run IPFSWebNode tests offline and fix interop/gateway CI assertions |
| `65a45457` | fix(web,test): make dart_ipfs web-compatible and harden CI tests |
| `c6845cc` | fix(web): make dart_ipfs Flutter-web compatible |
| `a50e3d8` | chore(deps): update GitHub Actions, patch Helia DHT vulnerability, and refresh Dart packages |
| `f1f5d97` | refactor: professionalize documentation and remove internal tooling references |
| `f0cde92` | fix: resolve Kubo/Helia interop tests and finalize release readiness |

## Working tree status (2026-07-11)

**Clean.** All changes committed as of v1.11.7. No uncommitted modifications or untracked files in the working tree.

## Version history highlights

### v1.11.7 (2026-07-11)
- Fixed: Removed direct `xml` and `dart_udx` dependencies that conflicted with downstream consumers (e.g. `port_forwarder`). Security pins remain as `dependency_overrides`.

### v1.11.6 (2026-07-11)
- Fixed: `IPFSWebNode` starts offline by default, fixing CI platform-specific router startup failures.
- Fixed: HTTPS redirect `Location` header includes leading slash.
- Fixed: Interop test files reformatted for Linux CI `dart format`.
- Changed: CI test workflow green on Ubuntu, macOS, Windows (3477 passed, 8 skipped).

### v1.11.5 (2026-06-23)
- Added: Monorepo with `packages/dart_ipfs_core/` and `packages/dart_ipfs_quic/`.
- Added: CLI hardened with `CommandRunner`, QUIC RFC evaluation.
- Changed: `dart_ipfs_quic` migrated from quiche FFI to pure-Dart `quic_lib`.
- Deprecated: Deep imports of `package:dart_ipfs/src/...` (removal in v3.0.0).

### v1.11.0 (2026-05-09)
- Added: Full Web P2P transport suite (WebRTC, WebRTC-Direct, WebTransport).
- Fixed: ~30 critical regressions in Bitswap, DHT, IPNS.

### v2.0.0-dev.1 (2026-04-30)
- Changed: Dismantled `IPFSNode` "God Object" into specialized managers.
- Changed: IPLD refactored with Strategy pattern for codecs.

### v1.0.0 (2025-12-12)
- Initial release with complete IPFS protocol implementation.

## Notable prior events

- **2026-07-08 modularization incident**: a WP-07 subagent replaced local CID/Block/UnixFS imports with `package:dart_ipfs_core` imports, causing 231 analysis errors. Recovery restored `lib/src/core/cid.dart`, `crypto_utils.dart`, `ed25519_signer.dart`, added missing `RouterInterface.unregisterProtocolHandler`, converted ~260 test files to `package:dart_ipfs/src/...` imports, and deleted invalid subagent-created files. Final state: `dart analyze` clean, 3232 tests passing, 6 environment-dependent failures.
- **2026-07-08 coverage push**: added 87 generated-protobuf unit tests, reaching **86.61% line coverage** and exceeding the 80% target.
- **2026-07-09 WP-06/08/09 completion**: all non-Council work packages closed. DHT provide/find and IPNS resolution interoperate with Kubo.
- **2026-07-09 WP-07 abandoned**: Council of Five final decision — adoption-first strategy over modularization. See `ciel/kg/decisions/2026-07-09-wp07-final-decision.md`.
- **2026-07-11 v1.11.6–v1.11.7 release**: CI hardened (web compatibility, offline tests, formatting), publishing regression fixed.

## Release checklist (from `MAINTAINER_GUIDE.md`)

Before any release:

1. Update version in `pubspec.yaml`, `CHANGELOG.md`, `README.md`, `ROADMAP.md`.
2. Ensure no uncommitted debug files.
3. Verify remote tag availability.
4. Run the full test suite.
5. Run `dart analyze`.
6. Run `pana` for pub.dev score check.

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]]
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]]
