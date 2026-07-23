# INSTALL — Obsidian Backend

Ciel installs, maintains, and updates the Obsidian memory backend.

## Prerequisites

- Obsidian desktop app installed and running.
- `obsidian-local-rest-api` plugin enabled with a generated API key.
- Node.js (`node` + `npm`) available.
- Optional: `obsidian-hybrid-search` for semantic/hybrid retrieval.

## Install / Update Dependencies

```bash
cd ciel.skill/memory/backends/obsidian
npm install
```

This installs `js-yaml` and other adapter dependencies.

## Verify Obsidian Backend

```bash
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test
```

Expected checks:

- Local REST API status responds with `OK`.
- Read/write round-trip succeeds.
- Hybrid search server is reachable.
- Knowledge-graph server is reachable (optional).

## Upgrade Cadence

On every `init/INIT.md` invocation:

1. Verify Obsidian backend dependencies (`npm install` if `package.json` changed).
2. Run the adapter self-test.
3. If the self-test fails, propose a fix or fall back per `FALLBACK.md`.
4. On pass, run migration and integrity check.

## Migration

Schema version lives in `obsidian-brain/.obsidian/` and vault structure. Each upgrade runs migrations in order. Failed migration → auto-rollback to previous snapshot and backup restore (`BACKUP.md`).

## Fallback Installation Failure

If Obsidian is not available or the self-test fails, Ciel falls back per `FALLBACK.md`:

1. Try SQLite backend (`backends/SQLITE.md`).
2. Try filesystem KV (`backends/FILESYSTEM.md`).
3. Inform user; continue in degraded mode.

## User Overrides

```yaml
memory:
  backend: custom
  auto_update: false
  custom:
    entry: "ciel.skill/memory/backends/obsidian/cli.mjs"
    runtime: node
    endpoint: null
    auth_env: OBSIDIAN_API_KEY
```

## Binary / Entry Location

- Adapter entry: `ciel.skill/memory/backends/obsidian/cli.mjs`.
- Node runtime must be on PATH.
