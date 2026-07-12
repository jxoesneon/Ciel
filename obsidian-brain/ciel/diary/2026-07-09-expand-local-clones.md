---
title: 2026-07-09 — Expanded knowledgebase for all local clones
type: diary
tags: ["diary","session"]
status: completed
created: 2026-07-09
updated: 2026-07-09
---

# 2026-07-09 — Expanded knowledgebase for all local clones

## What happened

All nine read-only subagents completed their exploration of the locally cloned projects. I synthesized each report into a new `knowledgebase.md` note under the corresponding `ciel/projects/<repo>/` workspace, then updated the project `overview.md` with a local-clone snapshot and a link to the expanded note. Finally, I updated `ciel/projects.md` with a "Local clones expanded today" section.

## Projects expanded

| Local folder | GitHub repo | Key takeaway |
|--------------|-------------|--------------|
| `C:/Users/josee/blindsight` | `jxoesneon/blindsight` | FastAPI + React 19/Nx privacy-first survey platform, 45+ routers, 90% coverage target |
| `C:/Users/josee/Ciel` | `jxoesneon/Ciel` | Autonomous partner-intelligence `.skill`, Council of Five, Obsidian memory backend |
| `C:/Users/josee/Faithful` | `jxoesneon/Faithful-HD2D` | God-sim game with HD-2D+ deferred rendering and Rust/WASM core; several gameplay systems still stubs |
| `C:/Users/josee/FerroTex Desktop` | `jxoesneon/FerroTex-Desktop` | Tauri v2 + React 19 + 9 Rust crates for local-first collaborative LaTeX |
| `C:/Users/josee/IPFS` | `jxoesneon/IPFS` | Production-ready Dart IPFS v1.11.5, monorepo with `dart_ipfs_core` and `dart_ipfs_quic` |
| `C:/Users/josee/mempalace-rs` | `jxoesneon/mempalace-rs` | Offline-first AI memory in Rust, vector + KG storage, 20 MCP tools, 0.5.0 |
| `C:/Users/josee/dart_quic` | `jxoesneon/quic_lib` | Pure-Dart QUIC/HTTP-3/WebTransport/libp2p v1.13.0, final v1.x release |
| `C:/Users/josee/SeedSphere` | `jxoesneon/SeedSphere` | Decentralized Stremio addon ecosystem: core + router + gardener + bridge |
| `C:/Users/josee/X-Seed` | `jxoesneon/X-Seed` | Android-native P2P stream aggregator and Stremio addon host v1.0.0-rc.1 |

## Notes created / updated

Created:

- `ciel/projects/blindsight/knowledgebase.md`
- `ciel/projects/Ciel/knowledgebase.md`
- `ciel/projects/Faithful-HD2D/knowledgebase.md`
- `ciel/projects/FerroTex-Desktop/knowledgebase.md`
- `ciel/projects/IPFS/knowledgebase.md`
- `ciel/projects/mempalace-rs/knowledgebase.md`
- `ciel/projects/quic_lib/knowledgebase.md`
- `ciel/projects/SeedSphere/knowledgebase.md`
- `ciel/projects/X-Seed/knowledgebase.md`

Updated:

- All corresponding `ciel/projects/<repo>/overview.md` notes.
- `ciel/projects.md` index.

## Observations

- IPFS and X-Seed have large working trees with many uncommitted changes and untracked files; they appear to be in active stabilization toward their next releases.
- Faithful-HD2D has no `.git` directory in the local folder; it may be a detached working copy or exported project.
- FerroTex Desktop uses Jujutsu (jj) as its primary VCS with a Git compatibility layer.
- Several repos are interconnected: `quic_lib` → `dart_ipfs` → `X-Seed`/`SeedSphere`; `SeedSphere` concepts inform `X-Seed`; `mempalace-rs` is a working-memory dependency for Ciel.

## Next steps

- If needed, break `knowledgebase.md` notes into atomic architecture / state / build notes.
- Add cross-links between related project knowledgebase notes.
- Keep snapshots fresh as working trees evolve.
