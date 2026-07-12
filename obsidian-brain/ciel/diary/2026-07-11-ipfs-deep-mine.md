---
title: 2026-07-11 — Deep-mine dart_ipfs project into Obsidian brain
type: diary
tags: [diary, session]
created: 2026-07-11
status: active
---

# 2026-07-11 — Deep-mine dart_ipfs project into Obsidian brain

## What happened

The user activated Ciel and requested a full mining of the `dart_ipfs` project at `C:/Users/josee/IPFS` into the Obsidian brain. A previous mining run (run-mrfzkrup) had been created but was blocked and never executed.

## Actions taken

### 1. State verification
- Ran `dart analyze`: 0 issues
- Ran `dart test`: 3478 passed, 8 skipped, 0 failed
- Checked git log, tags, pubspec version, CI status — all current at v1.11.7
- Verified all CI workflows green (test, build, docker, docs, publish, interop, k8s, codeql)

### 2. Parallel subagent mining (3 subagents)
Spawned three read-only subagents to deep-mine different aspects:

- **Subagent 1 (lib/src/)**: Explored the full source tree, identified all subsystems, key files, key classes, architectural patterns, new/changed files since v1.11.5, and gaps/TODOs.
- **Subagent 2 (test/interop/packages/CI)**: Explored test infrastructure, interop tests, monorepo packages, all 10 CI workflows, Docker/K8s deployment configs, and full dependency list.
- **Subagent 3 (docs/specs/ROADMAP/CHANGELOG)**: Explored all documentation, specs, roadmap, changelog, compliance status, and audit/decision documents.

### 3. Obsidian brain update

Updated 7 main knowledgebase notes from stale v1.11.5 state to current v1.11.7:

- `ciel/projects/IPFS/IPFS.md` — version, tests, coverage, working tree, CI, pub.dev, Docker, tags
- `ciel/projects/IPFS/git-state.md` — v1.11.7 release, clean tree, recent commits, version history, prior events
- `ciel/projects/IPFS/knowledgebase.md` — summary, quick status, top-level structure, architecture, key files
- `ciel/projects/IPFS/dependencies-and-monorepo.md` — full dependency list, monorepo packages, security overrides
- `ciel/projects/IPFS/security-and-traps.md` — crypto libs, gateway security, plugin security, traps, WP status
- `ciel/projects/IPFS/build-test-ci.md` — test structure, tags/presets, CI workflows, Docker/K8s, CI status
- `ciel/projects/IPFS/specs-and-compliance.md` — all 26 specs, Council artifacts, roadmap highlights

### 4. New notes created

- `ciel/projects/IPFS/releases/v1.11.6-v1.11.7-stabilization.md` — release summary for v1.11.6 and v1.11.7
- `ciel/diary/2026-07-11-ipfs-deep-mine.md` — this diary entry

### 5. Cross-linking

- Added cross-link from `ciel/projects/IPFS/IPFS.md` to `ciel/projects/dart_ipfs/dart_ipfs.md`
- Added cross-link from `ciel/projects/IPFS/knowledgebase.md` to `ciel/projects/dart_ipfs/dart_ipfs.md`
- Added cross-link from `ciel/projects/IPFS/security-and-traps.md` to WP-07 final decision

## Key findings

### Staleness identified and fixed
- All notes were stale at v1.11.5 with "3232 passing, 6 failing, extensive uncommitted changes"
- Updated to v1.11.7 with "3478 passing, 8 skipped, 0 failing, clean working tree"
- Added new dependencies not previously documented: `catalyst_cose`, `jose`, `cipherlib`, `murmurhash`, `base32`, `markdown`, `synchronized`, `intl`, `web`, `fixnum`, `convert`, `async`, `uuid`
- Added new security features: ACME/AutoTLS, plugin security, PNET, dependency override trap
- Added new WP status: WP-06/08/09 closed, WP-07 abandoned
- Added new CI workflows: `coverage.yml`, `interop_nightly.yml`
- Added new subsystems: `routing/` (IPNI, delegated routing, Reframe, DNSLink), `storage/` (Hive datastore), `core/peer/`, `core/peering/`, `protocols/connection_manager/`

### Discoveries
- `ENGINEERING_NOTES.md` exists alongside `AGENTS.md` with overlapping but slightly different content
- ROADMAP.md still shows "Current Version: 1.11.5" — needs update to 1.11.7
- Only 13 TODO/FIXME markers in the entire codebase (very clean)
- Plugin system is implemented but no actual plugins exist yet
- QUIC transport is stubbed for web, native QUIC via `quic_lib` in `dart_ipfs_quic`
- Docker images use Chainguard hardened base images
- All interop tests pass in CI with Kubo v0.42.0 and Helia

## Vault stats after update

- 142 notes, 367 links, 121 tags, 35 components
- IPFS knowledgebase is the #2 hub note (degree 35) after projects index (degree 140)

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/releases/v1.11.6-v1.11.7-stabilization|v1.11.6–v1.11.7 release]]
- [[ciel/projects/dart_ipfs/dart_ipfs|dart_ipfs operational overview]]
