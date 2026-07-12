---
title: "2026-07-08: Obsidian Backend Self-Test Pass"
type: diary
date: 2026-07-08
session_id: run-selftest-pass-1
project: ciel
tags: [diary, session, deployment, obsidian]
status: active
created: "2026-07-08T00:00:00Z"
---

# 2026-07-08: Obsidian Backend Self-Test Pass

## Summary

The Obsidian memory backend is fully operational. After the user disabled restricted mode and restarted Obsidian, the Local REST API plugin loaded and all backend services passed the self-test.

## Final state

- **Obsidian desktop app**: running with `obsidian-brain` vault open.
- **Restricted mode**: disabled.
- **Local REST API plugin**: enabled and listening on `http://127.0.0.1:27123` (insecure) and `https://127.0.0.1:27124` (secure).
- **API authentication**: working with the generated API key.
- **obsidian-hybrid-search**: running on `http://127.0.0.1:3939`.
- **obra/knowledge-graph**: installed at `~/.ciel/tools/knowledge-graph`, indexed, and reachable from the adapter.
- **kepano/obsidian-skills**: cloned to `~/.ciel/skills/obsidian-skills/` and referenced in `_CLAUDE.md` and `obsidian-memory/SKILL.md`.
- **defuddle**: installed in `scripts/obsidian/` for web-to-markdown extraction.

## Bugs fixed

- The Obsidian window had the wrong vault open (`C:\Users\josee\Ciel` instead of `C:\Users\josee\Ciel\obsidian-brain`). Updated `%APPDATA%\Obsidian\obsidian.json` and relaunched Obsidian with the correct vault.
- The adapter self-test used a different `Date.now()` key for `put`, `get`, and `delete`, causing false negatives. Fixed to use a single `testKey` variable.
- The self-test status check looked for `status.ok`, but the Local REST API returns `status: "OK"`. Updated to `status.status === 'OK'`.
- The knowledge graph command failed on Windows with `spawn EINVAL` because `.cmd` files cannot be spawned directly. Updated the adapter to use `cmd /c npx` on Windows.
- Updated the unit test mock to return `{ status: 'OK' }` for the `/` endpoint.

## Verification

```powershell
. .\scripts\obsidian\setup-env.ps1
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test
node tests/obsidian-memory/adapter.test.mjs
```

Both exit with code 0 and all checks/tests pass.

## Configuration

`.ciel/project.json` updated:

```json
{
  "memory_backend": "custom-obsidian",
  "obsidian_backend_ready": true,
  "obsidian_components": {
    "desktop_app": "installed-running",
    "local_rest_api_plugin": "installed-enabled",
    "api_key": "generated",
    "hybrid_search": "running",
    "knowledge_graph": "installed-indexed",
    "restricted_mode": "disabled"
  }
}
```

## Next steps

- Switch `ciel.skill/configuration/global/memory.config.md` to `backend: custom` if not already done.
- Begin using the Obsidian backend for memory operations.
- Use `defuddle` via `npx --prefix scripts/obsidian defuddle parse <url> --md` when ingesting web sources.
