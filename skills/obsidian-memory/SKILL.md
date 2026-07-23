---
name: obsidian-memory
version: 1.0.0
format: skill/1.0
description: Ciel's persistent memory and knowledge graph system backed by an Obsidian vault.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems", "memory"]
triggers:
  - pattern: "(remember|save|persist|store).*memory"
    confidence: 0.9
  - pattern: "what did we.*(last|previous|before)"
    confidence: 0.9
  - pattern: "search.*(history|memory|palace|vault)"
    confidence: 0.9
  - pattern: "knowledge graph|temporal fact|vault query|obsidian brain"
    confidence: 0.9
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [obsidian-local-rest-api, obsidian-hybrid-search] }
---

# CIEL ADAPTATION: obsidian-memory

This skill provides an Obsidian-based persistent memory layer. It exposes the same abstract memory API as the previous backend manager, but every durable value is stored as a plain markdown file in an Obsidian vault.

## Why Obsidian

- **Local-first**: all notes live on the device as `.md` files.
- **Human-readable**: the user can open, edit, and browse the brain directly.
- **Agent-agnostic**: any AI agent with MCP or REST access can read and write the same vault.
- **Composable**: semantic search, knowledge graphs, and daily notes are separate tools that all operate on the same files.

## Memory Stack (L0-L3)

- **L0 (IDENTITY)**: `_CLAUDE.md` + `ciel/identity.md` — loaded at session start.
- **L1 (ESSENTIAL)**: `ciel/diary/` + `CRITICAL_FACTS.md` — recency-biased context.
- **L2 (ON-DEMAND)**: `ciel/kg/` + `ciel/projects/<project>/` — retrieved via hybrid search and graph traversal.
- **L3 (SEARCH)**: full vault search over `raw/` and `wiki/` via `obsidian-hybrid-search`.

## Core Capabilities

### 1. Semantic Mining

When Ciel ingests a conversation, codebase, or document, the raw source goes into `raw/` and a synthesized note goes into `wiki/` or `ciel/kg/`.

### 2. Knowledge Graph

Concepts, decisions, and people are stored as atomic markdown notes in `ciel/kg/`. Wikilinks and `related:` frontmatter create the graph structure. Graph traversal uses `obra/knowledge-graph` or `obsidian-mcp-ultra`.

### 3. Agent Diary

Every significant session produces a dated diary entry in `ciel/diary/` using the `daily-note` template.

### 4. Context Compression

Large contexts are synthesized into dense markdown wiki pages rather than AAAK binary blobs. The `_CLAUDE.md` file governs the compression style and retention rules.

## Orchestration Logic

### 1. Context Hydration

On every session start, the orchestration skill must read:

1. `_CLAUDE.md`
2. `index.md`
3. `ciel/identity.md`
4. Recent `ciel/diary/` entries (last 7 days or most relevant via hybrid search).

### 2. Persistence Gate

Every Council of Five decision must be written to `ciel/kg/decisions/` as a decision record.

### 3. Retrieval-Augmented Deliberation

Before any high-stakes Council meeting, Ciel must search `ciel/kg/decisions/` and `ciel/kg/concepts/` for relevant precedents.

## Abstract API

The backend adapter at `ciel.skill/memory/backends/obsidian/adapter.mjs` implements:

```text
mem.put(partition, key, value, metadata?)
mem.get(partition, key)
mem.query(partition, filter)
mem.search(partition, query, top_k)
mem.delete(partition, key)
mem.list(partition, prefix)
mem.compact(partition)
mem.snapshot(partition, path)
mem.restore(partition, path)
mem.stats(partition)
```

## Configuration

Switch Ciel to the Obsidian backend by updating `ciel.skill/configuration/global/memory.config.md`:

```yaml
memory:
  backend: custom
  custom:
    entry: "ciel.skill/memory/backends/obsidian/cli.mjs"
    runtime: node
    endpoint: null
    auth_env: OBSIDIAN_API_KEY
```

## Required Environment

- `OBSIDIAN_API_URL` — Local REST API URL.
- `OBSIDIAN_API_KEY` — Bearer token.
- `OBSIDIAN_VAULT_PATH` — Path to the `obsidian-brain/` vault.
- `OBSIDIAN_HYBRID_SEARCH_URL` — Optional; defaults to `http://127.0.0.1:3939`.
- `KG_VAULT_PATH` and `KG_DATA_DIR` — Optional for knowledge-graph traversal.

## Related Skills

- `kepano/obsidian-skills` (cloned to `.ciel/obsidian-skills/`) provides agent conventions for Obsidian Flavored Markdown, the Obsidian CLI, `.base` files, `.canvas` files, and `defuddle` web-to-markdown extraction. Use these when writing vault content or interacting with the live Obsidian app.

## Migration from the previous memory backend

1. Create the `obsidian-brain` vault and open it in Obsidian.
2. Install the Local REST API plugin and generate an API key.
3. Set the environment variables above.
4. Run `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test`.
5. Update `memory.config.md` to use `backend: custom`.
6. Over time, migrate old MemPalace partitions into the corresponding vault folders.

## Anti-Patterns

- **Binary dumps**: do not store opaque blobs without a markdown wrapper; use the adapter's encoding.
- **Orphan notes**: every durable note must be linked from an index or project overview.
- **Cross-project bleed**: keep project-specific notes in `ciel/projects/<project>/` unless explicitly lifted to global.
