# Obsidian Brain Migration Guide

This branch (`Obsidian`) has replaced Ciel's default memory stack with a local-first, markdown-native Obsidian vault.

## What Was Implemented

### 1. Obsidian Vault Starter Pack (`obsidian-brain/`)

A ready-to-open Obsidian vault with:

- `_CLAUDE.md` — Ciel's operating manual, loaded at session start.
- `index.md` — catalog with Dataview tables for projects, decisions, concepts, and diary entries.
- `AGENTS.md` — the agentic loop protocol (decompose → retrieve → execute → verify → persist).
- `ciel/identity.md` — core identity and preferences.
- `ciel/diary/` — session-by-session audit trail.
- `ciel/kg/concepts/` — reusable ideas and frameworks.
- `ciel/kg/decisions/` — architecture decision records.
- `ciel/kg/people/` — people and organizations.
- `ciel/projects/` — per-project workspaces.
- `raw/` — unprocessed source material.
- `wiki/` — synthesized knowledge pages.
- `templates/` — daily notes, concept notes, and decision records.

### 2. Obsidian Memory Backend Adapter (`ciel.skill/memory/backends/obsidian/`)

A Node.js implementation of the abstract `CielMemoryBackend` interface:

- `adapter.mjs` — maps `put/get/query/search/delete/list/compact/snapshot/restore/stats` to Obsidian services.
- `cli.mjs` — command-line interface and self-test runner.
- `package.json` — dependencies (`js-yaml`).
- `README.md` — backend reference.

Services used:

- `obsidian-local-rest-api` for CRUD, periodic notes, and commands.
- `obsidian-hybrid-search` for semantic + full-text + hybrid retrieval.
- `obra/knowledge-graph` (optional) for graph traversal and community analysis.

### 3. `obsidian-memory` Skill (`skills/obsidian-memory/SKILL.md`)

A Ciel skill that provides Obsidian-backed equivalents for the previous memory API:

- `mempalace mine` → write raw sources to `raw/` and synthesize wiki pages.
- `mempalace_diary_write` → write to `ciel/diary/`.
- `mempalace_kg_add` → write atomic notes to `ciel/kg/`.
- `mempalace_kg_query` → hybrid search + graph traversal.

### 4. Agentic Loop (`scripts/obsidian/agentic-loop.mjs`)

A controller script that turns goals into tasks and subtasks, retrieves vault context, executes subtasks, and writes results back:

```bash
node scripts/obsidian/agentic-loop.mjs "<goal>" --project <project> --execute
```

Default is dry-run. Pass `--execute` to run actions.

### 5. Configuration Example

Updated `ciel.skill/configuration/global/memory.config.md` with an Obsidian backend example.

### 6. Tests (`tests/obsidian-memory/adapter.test.mjs`)

Node-native test suite that verifies the adapter against a mock Obsidian REST API server.

## Setup

### 1. Install Obsidian and Plugins

1. Download [Obsidian](https://obsidian.md/) and install it.
2. Open the `obsidian-brain/` folder as a vault.
3. Enable community plugins and install:
   - `Local REST API` by coddingtonbear
   - `Templater`
   - `Dataview`
4. In Settings → Local REST API, generate an API key and enable the HTTP server.

### 2. Install the Backend Adapter

```bash
cd ciel.skill/memory/backends/obsidian
npm install
```

### 3. Set Environment Variables

On Windows (PowerShell):

```powershell
$env:OBSIDIAN_API_URL = "http://127.0.0.1:27123"
$env:OBSIDIAN_API_KEY = "<your-api-key>"
$env:OBSIDIAN_VAULT_PATH = "c:\Users\josee\Ciel\obsidian-brain"
$env:OBSIDIAN_HYBRID_SEARCH_URL = "http://127.0.0.1:3939"
```

On macOS/Linux:

```bash
export OBSIDIAN_API_URL=http://127.0.0.1:27123
export OBSIDIAN_API_KEY=<your-api-key>
export OBSIDIAN_VAULT_PATH=/path/to/Ciel/obsidian-brain
export OBSIDIAN_HYBRID_SEARCH_URL=http://127.0.0.1:3939
```

### 4. Run the Self-Test

```bash
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test
```

Expected output includes checks for:

- Local REST API status
- Read/write round-trip
- Hybrid search reachability
- Knowledge-graph reachability (optional)

### 5. Install Hybrid Search

```bash
npm install -g obsidian-hybrid-search
obsidian-hybrid-search reindex
```

Or use the MCP server:

```bash
npx -y -p obsidian-hybrid-search@latest obsidian-hybrid-search serve
```

### 6. Switch Ciel to the Obsidian Backend

Edit `ciel.skill/configuration/global/memory.config.md`:

```yaml
memory:
  backend: custom
  auto_update: false
  isolation_strict: true
  custom:
    entry: "ciel.skill/memory/backends/obsidian/cli.mjs"
    runtime: node
    endpoint: null
    auth_env: OBSIDIAN_API_KEY
```

Restart Ciel. On the next session, Ciel will use the Obsidian vault as her brain.

## Running the Tests

```bash
cd ciel.skill/memory/backends/obsidian
npm test
```

Or from the repo root:

```bash
node --test tests/obsidian-memory/adapter.test.mjs
```

## Running the Agentic Loop

Dry-run a goal:

```bash
node scripts/obsidian/agentic-loop.mjs "Plan the next Ciel feature" --project ciel
```

Execute the generated subtasks:

```bash
node scripts/obsidian/agentic-loop.mjs "Plan the next Ciel feature" --project ciel --execute
```

Use a pre-written plan:

```bash
node scripts/obsidian/agentic-loop.mjs "Plan the next Ciel feature" --project ciel --plan plan.json
```

## Migration from the previous backend

| Previous Backend | Obsidian Equivalent |
|--------------|---------------------|
| `mempalace mine` | Write raw sources to `raw/`; synthesize `wiki/` pages. |
| `mempalace_diary_write` | Append to `ciel/diary/YYYY-MM-DD.md`. |
| `mempalace_kg_add` | Write to `ciel/kg/concepts/`, `ciel/kg/decisions/`, or `ciel/kg/people/`. |
| `mempalace_kg_query` | Use `obsidian-hybrid-search` + `obra/knowledge-graph`. |
| `mempalace wakeup` | Read `_CLAUDE.md`, `index.md`, `ciel/identity.md`, and recent diary entries. |
| `mempalace compact` | Re-index hybrid search and knowledge graph. |

## Security Notes

- The Local REST API uses a bearer token. Never commit the API key.
- Keep the HTTP endpoint on `127.0.0.1` only.
- The hybrid search and knowledge-graph indexes are regenerable and gitignored in `obsidian-brain/.gitignore`.
- All memory writes are plain markdown, so the user can audit and edit every note.

## Next Steps

1. Open `obsidian-brain/` in Obsidian and verify the vault renders correctly.
2. Run the self-test and fix any missing services.
3. Update `ciel/identity.md` with user-specific preferences.
4. Start using the agentic loop for new goals.
5. Backfill old MemPalace data into the corresponding vault folders.
