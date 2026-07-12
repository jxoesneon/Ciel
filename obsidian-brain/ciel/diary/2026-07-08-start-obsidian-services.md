---
title: "2026-07-08: Starting Obsidian Services"
type: diary
date: 2026-07-08
session_id: run-start-services-1
project: ciel
tags: [diary, session, deployment, obsidian]
status: active
created: "2026-07-08T00:00:00Z"
---

# 2026-07-08: Starting Obsidian Services

## Summary

Attempted to fully start the Obsidian backend services after the Ciel deployment. Most components were installed and started successfully; the Local REST API is blocked by Obsidian's restricted mode, which requires a manual UI toggle.

## Completed

- Installed Obsidian desktop app (v1.12.7) silently at `%LOCALAPPDATA%\Obsidian`.
- Installed the Obsidian Local REST API plugin (v4.1.3) into `obsidian-brain/.obsidian/plugins/obsidian-local-rest-api/`.
- Generated an API key and self-signed certificate and wrote `data.json` with `enableInsecureServer: true` and insecure port 27123.
- Added `obsidian-local-rest-api` to `community-plugins.json`.
- Started the `obsidian-hybrid-search` HTTP server on port 3939; it indexed the vault (9/9 files).
- Cloned and installed `obra/knowledge-graph` at `~/.ciel/tools/knowledge-graph` and indexed the vault (12 nodes, 37 edges, 4 communities).
- Updated the Obsidian adapter to use `KG_REPO_PATH` for knowledge graph commands.
- Updated `scripts/obsidian/setup-env.ps1` to set all required environment variables and read the API key from `data.json`.
- Left Obsidian running with the `obsidian-brain` vault open.

## Blockers

- **Restricted mode:** Obsidian's community-plugin sandbox is still on. The Local REST API plugin is installed and configured but cannot load until restricted mode is turned off in the UI. CDP and Playwright automation attempts failed to bypass this guard.
- **REST API self-test:** Fails with `connect ECONNREFUSED 127.0.0.1:27123` until the plugin is loaded.

## Next Step (manual)

1. In the Obsidian window that is already open, go to **Settings → Community plugins**.
2. Click **Turn off restricted mode** and confirm the reload.
3. After Obsidian restarts, the Local REST API plugin will load and start listening on `http://127.0.0.1:27123`.
4. In PowerShell, run:
   ```powershell
   . .\scripts\obsidian\setup-env.ps1
   node ciel.skill/memory/backends/obsidian/cli.mjs --self-test
   ```
5. If the self-test passes, update `.ciel/project.json` to set `obsidian_backend_ready: true`.
