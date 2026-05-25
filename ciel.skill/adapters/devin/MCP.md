# MCP — Devin for Terminal

Devin for Terminal has first-class MCP support via the `devin mcp` CLI subcommand and `mcpServers` config key.

## Adding MCP Servers

### CLI

```bash
# stdio server
devin mcp add github -- npx -y @modelcontextprotocol/server-github

# HTTP server
devin mcp add notion https://mcp.notion.com/mcp

# With scope
devin mcp add -s project sentry https://mcp.sentry.dev/mcp
devin mcp add -s user linear https://mcp.linear.app/mcp
```

### Config File

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "ghp_..." }
    },
    "notion": {
      "url": "https://mcp.notion.com/mcp",
      "transport": "http"
    }
  }
}
```

## Ciel's MCP Integration

Ciel uses MCP servers for:
- **Research**: `seed_skills/research/SKILL.md` leverages web search MCPs
- **Memory**: MemPalace-rs MCP for persistent cross-session knowledge
- **External APIs**: GitHub, Linear, databases via MCP rather than shell commands

## MCP Permissions

Control MCP tool access in `permissions`:

```json
{
  "permissions": {
    "allow": ["mcp__github__*"],
    "deny": ["mcp__github__delete_repo"],
    "ask": ["mcp__linear__*"]
  }
}
```

## Ciel-Recommended MCP Servers for Faithful

| Server | Purpose | Config |
|--------|---------|--------|
| GitHub | Issue/PR management | `devin mcp add github -- npx -y @modelcontextprotocol/server-github` |
| Linear | Task tracking | `devin mcp add linear https://mcp.linear.app/mcp` |

## MCP + Hooks

When an MCP tool is invoked, the tool name seen by hooks is `mcp__<server>__<tool>`. Ciel's preflight hook can match these:

```bash
# In ciel_preflight.sh — block destructive MCP operations
if echo "$TOOL_NAME" | grep -qE '^mcp__.*__(delete|remove|drop|destroy)'; then
  decide "block" "Ciel Safety: destructive MCP operation blocked"
  exit 0
fi
```

## Authentication

OAuth-based servers require separate login per Devin client:

```bash
devin mcp login notion
devin mcp login linear
```

Tokens are stored locally and refreshed automatically.
