---
title: "2026-07-08: Full Local + Global Ciel Deployment"
type: diary
date: 2026-07-08
session_id: run-deployment-1
project: ciel
tags: [diary, session, deployment, obsidian]
status: active
created: "2026-07-08T00:00:00Z"
---

# 2026-07-08: Full Local + Global Ciel Deployment

## Summary

Ran the full Ciel deployment on the `Obsidian` branch. The global `~/.ciel/` home was initialized/updated, the local project `.ciel/` domain was created, and the memory backend was switched from `mempalace` to the custom Obsidian adapter.

## Actions

- Ran `ciel.skill/init/scripts/install.ps1` to verify/update `~/.ciel/`.
- Verified `mempalace-rs 0.5.0` is available as a fallback/historical backend.
- Created local `.ciel/` domain with `project.json` (`id: 16adf6a4fd9ecf2e`).
- Ensured `.ciel/` is in `.gitignore`.
- Switched both repo and global `~/.ciel/configuration/global/memory.config.md` to `backend: custom` pointing to the Obsidian adapter.
- Installed `ciel.skill/memory/backends/obsidian/` npm dependencies.
- Ran `node --test tests/obsidian-memory/adapter.test.mjs` — all 6 tests passed.
- Created `scripts/obsidian/setup-env.ps1` to prompt for the Obsidian API key and set environment variables.

## Decisions

- [[ciel/kg/decisions/obsidian-brain-migration-audit]] — Council-approved migration plan.

## Project Updates

- `~/.ciel/configuration/global/memory.config.md` now uses the Obsidian custom backend.
- `.ciel/project.json` records `memory_backend: custom-obsidian` and `obsidian_backend_ready: false`.

## Open Tensions

- Obsidian desktop app, Local REST API plugin, and hybrid search server are not yet running.
- The live self-test (`cli.mjs --self-test`) will fail until the Obsidian services are started and `setup-env.ps1` is sourced.
- Old `.mempalace/` directory in the repo remains; migration of old data is pending.

## Next Steps

1. Open `obsidian-brain/` in Obsidian and enable the Local REST API plugin.
2. Generate an API key and run `.\scripts\obsidian\setup-env.ps1`.
3. Start `obsidian-hybrid-search` (or configure it as an MCP server).
4. Run `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test`.
5. If the self-test passes, mark `obsidian_backend_ready: true` in `.ciel/project.json`.
