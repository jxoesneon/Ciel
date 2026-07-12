---
title: Ciel — Memory (Obsidian Backend)
project_note: subsystem
type: project-note
tags: ["project-note","subsystem"]
status: active
created: 2026-07-11
updated: 2026-07-11
---

# Ciel — Memory (Obsidian Backend)

## Architecture

- Primary memory: Obsidian vault at `C:\Users\josee\Ciel\obsidian-brain` via the custom backend adapter.
- Access is abstracted through `skills/obsidian-memory/SKILL.md`; direct backend calls are forbidden.
- Storage format: Markdown files with YAML frontmatter (`_ciel_backend`, `_ciel_enc`, `_ciel_updated`).
- Partition model: `ciel-global` (cross-project) + `ciel-project-<hash>` (per-project isolation).

## Configuration

- **Config:** `ciel.skill/configuration/global/memory.config.md`
- `backend: custom` pointing to `ciel.skill/memory/backends/obsidian/cli.mjs`
- `isolation_strict: true` (Constitutional, locked)
- Health check interval: 60 min; reinstall check: 7 days.

## Environment variables

- `OBSIDIAN_API_URL` — default `http://127.0.0.1:27123`
- `OBSIDIAN_API_KEY` — Bearer token from Obsidian Local REST API plugin
- `OBSIDIAN_VAULT_PATH` — absolute path to vault
- `OBSIDIAN_HYBRID_SEARCH_URL` — default `http://127.0.0.1:3939`
- `KG_VAULT_PATH`, `KG_DATA_DIR`, `KG_REPO_PATH` — knowledge-graph paths

## Adapter API

- `adapter.mjs` implements `CielMemoryBackend`: put, get, delete, list, query, search, compact, snapshot, restore, stats.
- Extended: `kgSearch`, `kgRelated`, `kgPath`, `kgCommunities`.
- Self-test: `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test` checks Local REST API, read/write, hybrid search, and knowledge graph.

## Fallback order

1. Obsidian (custom) — primary
2. SQLite — single-file, FTS5 full-text, no embeddings
3. Filesystem KV — key-per-file, no embeddings
4. Custom — user-supplied adapter

## Key scripts

- `scripts/obsidian/setup-env.ps1` — sets environment variables, loads API key from plugin data.json.
- `scripts/obsidian/generate-rest-api-key.mjs` — auto-generates API key and self-signed cert.
- `scripts/obsidian/init-ciel-project.mjs` — creates project overview in vault.
- `scripts/obsidian/agentic-loop.mjs` — goal → tasks → vault context → execution → diary.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
