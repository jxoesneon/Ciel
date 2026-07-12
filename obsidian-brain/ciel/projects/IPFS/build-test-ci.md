---
title: "IPFS — Build, Test & CI"
project_note: update
type: project-note
tags: [project, IPFS]
created: 2026-07-11
status: active
---

# IPFS — Build, Test & CI

How to verify `dart_ipfs` locally and how the CI pipelines are organized.

## Local verification commands

All commands assume the repo root (`C:/Users/josee/IPFS`).

### Static analysis

```bash
dart analyze
```

- Target: **0 errors**.
- Current: **0 issues** (0 errors, 0 warnings, 0 infos) as of v1.11.7.

### Unit tests

```bash
dart test --reporter=compact
```

- Latest result (2026-07-11): **3478 passing, 8 skipped, 0 failing** on Windows VM.
- The 8 skipped tests are Docker-dependent interop scenarios that run with `dart test --preset interop` inside `test/interop/docker-compose`.

### Interop tests

```bash
cd test/interop
docker compose up -d --build
docker compose exec -T test-runner sh -c "cd /app && dart test --preset interop test/interop"
```

- P0 tests (release-blocking): CAR, Bitswap, Gateway — run in `interop.yml`.
- P1 tests (non-blocking): DHT, IPNS — run in `interop.yml` with `continue-on-error`.
- Helia tests: run nightly in `interop_nightly.yml`.
- All interop tests pass in CI with Kubo v0.42.0 and Helia.

### Coverage

```bash
dart test --coverage=coverage
dart pub global activate coverage
dart pub global run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info --packages=.dart_tool/package_config.json --report-on=lib
```

- Target: **80% line coverage**.
- Achieved: **85.79%** as of 2026-07-09.

### Formatting

```bash
dart format .
```

## Test structure

| Directory | Contents |
|-----------|----------|
| `test/core/` | CID, blocks, managers, handlers, data structures, config, crypto, DI, errors, events, interfaces, IPLD, metrics, MFS, peer, peering, plugins (~60+ files) |
| `test/protocols/` | Bitswap, DHT, IPNS, PubSub, GraphSync, AutoNAT, DCUtR, Identify, Ping (~50+ files) |
| `test/services/` | Gateway, RPC, pinning (~20+ files) |
| `test/transport/` | WebRTC, WebTransport, QUIC, PNET, circuit relay |
| `test/routing/` | Content routing, delegated routing, IPNI, Reframe (4 files) |
| `test/network/` | Connection management, NAT traversal, mDNS (6 files) |
| `test/interop/` | Docker-based interoperability tests with Kubo/Helia (6 test files) |
| `test/proto_generated/` | Protobuf message unit tests (87+ files) |
| `test/bin/` | CLI tests |
| `packages/dart_ipfs_core/test/` | Core package tests (7 files) |
| `packages/dart_ipfs_quic/test/` | QUIC package tests (4 files) |

Total: **~150+ test files** (excluding mocks), **3478 passing tests**.

## Test tags and presets (dart_test.yaml)

| Tag | Purpose |
|-----|---------|
| `p0` | Release-blocking interop tests (skipped by default) |
| `p1` | Non-blocking interop tests (skipped by default) |
| `helia` | Helia-specific interop tests (skipped by default) |
| `cli` | CLI tests (2x timeout for subprocess spawning) |

**Preset:** `interop` — enables p0, p1, and helia tags.

## Makefile targets

| Target | Purpose |
|--------|---------|
| `analyze` | Run `dart analyze` |
| `test` | Run `dart test` |
| `doc` | Generate documentation |
| `format` | Format the codebase |
| `protos` | Regenerate protobuf files |
| `clean` | Clean build artifacts |

## Melos workspace scripts

From `melos.yaml`:

```bash
melos bootstrap          # dart pub get in all packages
melos run analyze        # dart analyze in all packages
melos run test           # dart test in all packages
melos run test:all       # pub get + analyze + test in sequence
```

## GitHub Actions workflows

Located in `.github/workflows/`:

| Workflow | Purpose | Triggers | Platforms |
|----------|---------|----------|-----------|
| `test.yml` | Unit tests + formatting + analysis | push/PR to master | Ubuntu, Windows, macOS |
| `build.yml` | AOT executable + Flutter dashboard | push/PR to master | Ubuntu, Windows, macOS |
| `coverage.yml` | Coverage generation + Codecov upload | push/PR to master | Ubuntu |
| `codeql.yml` | CodeQL security analysis | push/PR + weekly | Ubuntu |
| `interop.yml` | Kubo interop tests (P0 + P1) | PR + daily 3am UTC | Ubuntu |
| `interop_nightly.yml` | Helia interop tests | daily 4am UTC | Ubuntu |
| `docs.yml` | API docs → GitHub Pages | push to master | Ubuntu |
| `docker.yml` | Docker build, scan (Trivy), sign (cosign), publish to GHCR | push + release | Ubuntu (multi-arch) |
| `k8s.yml` | K8s manifest validation (kubeconform, Helm lint) | push/PR + dispatch | Ubuntu |
| `publish.yml` | Publish to pub.dev with pana check | tag push `v*` | Ubuntu |

## Docker / Kubernetes

- `Dockerfile` — multi-stage: builder (dart:stable), runtime (cgr.dev/chainguard/glibc-dynamic, hardened, no shell), debug (cgr.dev/chainguard/bash). Non-root user (uid=1000), read-only rootfs, dropped capabilities.
- Ports: 4001/tcp+udp (libp2p), 5001 (RPC), 8080 (gateway), 8081 (metrics).
- `docker-compose.yml` — single daemon service with healthcheck.
- `k8s/` — Kustomize base + development/production overlays.
- `helm/dart-ipfs/` — Helm chart with NetworkPolicy, ServiceMonitor, HPA, PDB, Ingress, PVC.
- `doc/ACME_CERTIFICATE_ISSUANCE.md` — automatic TLS certificate management.

## CI status (2026-07-11)

All workflows are **green**:
- Test: 3477 passed, 8 skipped (Ubuntu)
- Docker: build + scan + sign + publish successful for v1.11.7
- Publish: `dart_ipfs 1.11.7` live on pub.dev
- Docs: deployed to GitHub Pages

## Common developer tasks

```bash
# Regenerate protobufs
make protos

# Full local verification
make analyze && make test

# Bootstrap monorepo
melos bootstrap

# Full monorepo verification
melos run test:all
```

## Notes

- `public_member_api_docs` is set to `warning` in `analysis_options.yaml`.
- Proto/generated directories are excluded from analysis.
- The example dashboard uses `lucide_icons_flutter` instead of the abandoned `lucide_icons` package.

## Related

- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/architecture.md|IPFS — Architecture]]
- [[ciel/projects/IPFS/specs-and-compliance.md|IPFS — Specs & Compliance]]
- [[ciel/projects/IPFS/dependencies-and-monorepo.md|IPFS — Dependencies & Monorepo]]
- [[ciel/projects/IPFS/security-and-traps.md|IPFS — Security & Traps]]
- [[ciel/projects/IPFS/git-state.md|IPFS — Git State]]
