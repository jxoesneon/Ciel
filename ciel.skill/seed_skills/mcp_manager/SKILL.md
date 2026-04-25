---
name: mcp_manager
version: 1.0.0
description: MCP server discovery, installation, configuration, health checks.
triggers: [mcp, mcp server, install mcp, mcp search]
tags: [mcp, scope:both, runtime:any, risk:mid]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md, package_manager/SKILL.md] }
---

# mcp_manager

Manage MCP servers.

## Operations

- `mcp.list()` — installed servers.
- `mcp.search(capability)` — discover matches.
- `mcp.install(pkg_or_url)` — install server (Council-gated).
- `mcp.configure(server, config)` — edit settings.json entries.
- `mcp.invoke(server, tool, args)` — runtime-adapter call.
- `mcp.ping(server)` — health.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { result }
  idempotent: depends
  side_effects: [fs, network, state_mutation]
```

## Safety

- Installs are mid-risk minimum (Council).
- Server invocations respect the host's MCP policy and Ciel's permission rules.
- Added servers recorded in `~/.ciel/acquisition/sources.json` with trust tracking.
