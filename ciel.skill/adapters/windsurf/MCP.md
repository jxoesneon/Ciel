# MCP — Windsurf Adapter

Windsurf supports Model Context Protocol servers via configuration.

## Configuration Path

- Settings UI: `Cmd/Ctrl + Shift + P` → "Windsurf: Open Settings"
- Or workspace settings in `.vscode/settings.json` (Windsurf inherits VS Code conventions)

## MCP Server Registration

Add to settings:

```json
{
  "mcp.servers": {
    "fetch": {
      "command": "uvx",
      "args": ["mcp-server-fetch"]
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/project"]
    }
  }
}
```

## Ciel's Default MCP Stack

Recommended for full functionality:

| Server | Purpose |
| --- | --- |
| `fetch` | Web retrieval |
| `filesystem` | Structured file access |
| `git` | Repository operations |

## Runtime Detection

Ciel detects available MCP servers by checking the active toolset. If MCP is unavailable, falls back to shell equivalents.

## Tool Invocation

Windsurf exposes MCP tools as native tools to Cascade. Ciel uses them transparently.

## Error Handling

MCP failures fall back to shell commands:

- `fetch` failure → `curl` or `wget`
- `filesystem` failure → direct `fs_read`/`fs_write` via shell
