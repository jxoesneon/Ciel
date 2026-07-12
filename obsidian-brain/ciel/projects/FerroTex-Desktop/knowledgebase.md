---
title: FerroTex-Desktop — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-09
source: "https://github.com/jxoesneon/FerroTex-Desktop"
---

# FerroTex-Desktop — Knowledgebase

Synthesized expansion from the read-only subagent exploration of the local clone.

## Summary

FerroTex Desktop is the desktop companion application for the FerroTeX LaTeX engine ecosystem. It is a hybrid Rust + TypeScript monorepo: a Tauri v2 desktop host running a React 19 frontend that communicates over IPC with 9 modular Rust crates. It provides local-first, peer-to-peer collaborative LaTeX editing with real-time collaboration (Yjs), an AI assistant (Ollama), Jujutsu VCS integration, and Zotero bibliography sync.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/FerroTex-Desktop` |
| Local path | `C:/Users/josee/FerroTex Desktop` |
| Version | 0.25.0 |
| Visibility | PRIVATE |
| VCS | Jujutsu (jj) with Git compatibility layer |
| Stack | Rust 1.85 (edition 2024), TypeScript/React 19, Tauri v2 |

## Top-level structure

- `Cargo.toml` — Rust workspace with 9 domain crates + Tauri host; MSRV 1.85.
- `pnpm-workspace.yaml` / `package.json` — pnpm monorepo scripts.
- `AGENTS.md` — single source of truth for context, build/test, verification checklist.
- `.jjignore` / `.gitignore` — Jujutsu and Git ignore patterns.
- `apps/desktop/` — Tauri v2 + React 19 app (100+ components).
- `crates/` — 9 Rust domain crates.
- `packages/` — shared TypeScript (sync, ts-config, UI).
- `tests/e2e/` — 63 Playwright tests.
- `docs/spec/` — 40+ FTX specification documents.

## Architecture

### Rust domain crates

| Crate | Responsibility |
|-------|----------------|
| `ferrotex-core` | Types, VFS, document/workspace abstractions |
| `ferrotex-syntax` | Rowan-based lossless LaTeX parser (CST) |
| `ferrotex-build` | Compiler DAG scheduler with Tectonic integration |
| `ferrotex-analysis` | Semantic diagnostics, delimiter checking |
| `ferrotex-package` | LaTeX package streamer (CAS, prefetch) |
| `ferrotex-dap` | Debug Adapter Protocol server |
| `ferrotex-lsp` | Language Server Protocol library |
| `ferrotex-jj` | Jujutsu VCS sidecar bridge |
| `ferrotex-zotero` | Zotero SQLite parser / BibTeX sync |

### Frontend / IPC

- `apps/desktop/src-tauri/src/lib.rs` — 16+ Tauri commands, workspace registry, LSP/DAP lifecycle, collaboration room management.
- `apps/desktop/src/App.tsx` — root React component, FerroTexContext, state wiring.
- `apps/desktop/src/hooks/useTauri.ts` — TypeScript IPC contract wrappers with browser backend fallback.
- `apps/desktop/src/ferroui/` — FerroUI server-driven UI integration (component registry, default layout, renderer).
- `tests/e2e/mocks/tauriMock.ts` — Playwright injectable mock for all 16 IPC commands.
- 100+ React components including Editor (CodeMirror), PDFViewer (PDF.js), BuildLog, GitSyncPanel, AiAssistant.

### Deployment target

Aims to supplant Overleaf with local-first, offline-capable, peer-to-peer collaborative typesetting.

## Build / test / deploy

```bash
# Rust
cargo build --workspace
cargo test --workspace

# Frontend / Tauri
cd apps/desktop
pnpm install
pnpm build      # tsc + vite build
pnpm tauri build
pnpm test       # Vitest: 41 files, 399 tests

# E2E
cd ../../tests/e2e
pnpm test       # Playwright: 63 tests

# Full verification
cargo build --workspace && cargo test --workspace && \
  cd apps/desktop && pnpm build && pnpm test && \
  cd ../../tests/e2e && pnpm test
```

CI (`ci.yml`): Rust build/test/clippy on Ubuntu 1.85; frontend build/test; E2E tests.

## Recent git state (manual snapshot)

- **Version:** 0.25.0.
- **Primary VCS:** Jujutsu with Git compatibility; `.jjignore` and `.gitignore` both maintained.
- **Alignment audit:** `alignment-audit.md` verdict is **FULLY ALIGNED** (16 IPC commands, 0 gaps; component props 0 mismatches; 1 low-severity E2E semantic drift).
- **Phase:** Phase 2 (Engine & Tooling) complete; Phase 3+ advanced features in progress.
- **Recent commits:**
  - `5d85c01` chore(gitignore): add .playwright-mcp/ directory
  - `5aacc4e` chore(gitignore): ignore test screenshots and archive directory
  - `f340263` feat(ai-assistant): full AI assistant with thought trails, artifacts, and tool approval
  - `689368c` feat(orchestrator): multi-stage AI pipeline with genre-aware model routing
  - `e6b8f46` feat(registry): add source reliability registry and prompt classifier

## Related ecosystem repos

- [FerroTeX](https://github.com/jxoesneon/FerroTeX) — main LaTeX engine (v0.22.0)
- [FerroUI](https://github.com/jxoesneon/FerroUI) — UI framework (private)
- [mempalace-rs](https://github.com/jxoesneon/mempalace-rs) — working memory system

## Key files for deeper context

1. `AGENTS.md` — context, build/test, verification, phase roadmap.
2. `apps/desktop/src-tauri/src/lib.rs` — Tauri command handlers and state.
3. `apps/desktop/src/App.tsx` — root React component.
4. `apps/desktop/src/hooks/useTauri.ts` — IPC contract.
5. `apps/desktop/src/ferroui/ferroTexRegistry.tsx` — FerroUI component registry.
6. `apps/desktop/src/ferroui/defaultLayout.ts` — root layout JSON.
7. `alignment-audit.md` — IPC/component/E2E alignment verdict.
8. `Cargo.toml` — workspace members and lint rules.
9. `apps/desktop/vite.config.ts` — dev proxies for Tectonic and Ollama.
10. `docs/spec/FTX-001.md` — Tauri IPC & Command Bridge spec.

## Related

- [[ciel/projects/FerroTex-Desktop/FerroTex-Desktop.md|FerroTex-Desktop overview]]
- [[ciel/projects.md|Projects index]]
