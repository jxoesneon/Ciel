# MCP — opencode

OpenCode has a first-class MCP client. Ciel never assumes servers exist; she
discovers them at init and registers capabilities from whatever is present.

## Configuration

Servers live under `mcp` in `~/.config/opencode/opencode.jsonc` (global) or
`.opencode/opencode.jsonc` (project). Managed Console workspaces inject
additional servers through the workspace policy config.

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "servers": {
      "ultramac":   { "type": "local",  "command": ["/Users/me/.bun/bin/bun", "run", "/Users/me/ultramac-mcp/index.ts", "--stdio"] },
      "playwright": { "type": "local",  "command": ["npx", "-y", "@playwright/mcp@latest"] }
    }
  }
}
```

Manage from the CLI:

```sh
opencode mcp add <name> --global -- <command...>
opencode mcp list
```

## Route Map

| Ciel capability | MCP server (typical) |
| --- | --- |
| Desktop vision / input / clipboard (computer use) | `ultramac` |
| Browser automation, screenshots, DOM assertions | `playwright` |
| GitHub issues / PRs / code search | `github` |

## Discovery Contract

At init, Ciel runs `opencode mcp list` and enumerates each server's tools. A
missing capability triggers `adapters/../acquisition` (registry → MCP → web), not
a hard failure. Plugins may also manage servers via `ctx.mcp.transform`, but
Ciel treats `opencode.jsonc` as the source of truth.
