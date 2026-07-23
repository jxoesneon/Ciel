# Obsidian Memory Backend for Ciel

Implements the Ciel abstract memory API (`CielMemoryBackend`) over an Obsidian vault.

## Responsibilities

| Abstract Method | Obsidian Service Used |
| ----------------- | ---------------------- |
| `put` | `obsidian-local-rest-api` PUT `/vault/{path}` |
| `get` | `obsidian-local-rest-api` GET `/vault/{path}` |
| `query` | `obsidian-hybrid-search` with scope filters |
| `search` | `obsidian-hybrid-search` semantic/hybrid search |
| `delete` | `obsidian-local-rest-api` DELETE `/vault/{path}` |
| `list` | `obsidian-local-rest-api` list directory |
| `compact` | re-index hybrid search + knowledge graph |
| `snapshot` / `restore` | filesystem copy |
| `stats` | count files in partition |

## Storage Format

Every value is stored as a markdown file with YAML frontmatter:

```markdown
---
_ciel_backend: obsidian
_ciel_enc: utf8
_ciel_updated: 2026-07-08T...
title: ...
tags: [...]
---

<value body>
```

Values are stored as UTF-8 text when possible, otherwise base64-encoded with `_ciel_enc: base64`.

## Configuration

```yaml
memory:
  backend: custom
  custom:
    entry: "ciel.skill/memory/backends/obsidian/cli.mjs"
    runtime: node
    endpoint: null
    auth_env: OBSIDIAN_API_KEY
```

Environment variables:

- `OBSIDIAN_API_URL` — Local REST API URL (default `http://127.0.0.1:27123`).
- `OBSIDIAN_API_KEY` — Bearer token from Obsidian Settings → Local REST API.
- `OBSIDIAN_VAULT_PATH` — Absolute path to the Obsidian vault.
- `OBSIDIAN_HYBRID_SEARCH_URL` — URL of the hybrid search server (default `http://127.0.0.1:3939`).
- `KG_DATA_DIR` — Knowledge-graph SQLite directory (default `~/.local/share/knowledge-graph`).
- `KG_VAULT_PATH` — Optional; falls back to `OBSIDIAN_VAULT_PATH`.

## CLI Usage

```bash
npm install
node cli.mjs --self-test
node cli.mjs put ciel/projects/ciel/ciel "Initial project context"
node cli.mjs get ciel/projects/ciel/ciel
node cli.mjs search ciel/kg/decisions "obsidian backend"
```

## Required Obsidian Plugins

1. **Local REST API** — provides authenticated CRUD over the vault.
2. **Templater** — optional; powers the templates in `obsidian-brain/templates/`.
3. **Dataview** — optional; powers the tables in `index.md` and project overviews.
4. **obsidian-hybrid-search** — external CLI/MCP server for semantic/hybrid retrieval.

## Integration with Ciel

Ciel's `obsidian-memory` skill dispatches to this backend when `memory.config.backend` is set to `custom` and the `entry` points to this adapter. The backend transparently implements the memory API:

- `mempalace mine` → write raw sources to `raw/`; synthesize wiki pages.
- `mempalace_diary_write` → append to `ciel/diary/`.
- `mempalace_kg_add` → write decision/concept/person notes to `ciel/kg/`.
- `mempalace_kg_query` → use `obsidian-hybrid-search` + `knowledge-graph`.

## Tests

```bash
npm test
```

Run only the self-test:

```bash
npm run self-test
```
