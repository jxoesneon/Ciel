# MCP — Claude Code

Model Context Protocol invocation. Claude Code supports lazy tool schema loading via `ToolSearch` — Ciel relies on this for context efficiency.

## Lazy Schema Loading

On session start, Claude Code loads only tool *names* for connected MCP servers. Full schemas are fetched on demand via:

```text
mcp.search(query) -> [matched_tool_names]
mcp.describe(tool_name) -> full_schema
mcp.invoke(tool_name, args) -> result
```

Ciel uses this as follows:

1. Router needs a capability → queries registry first.
2. If registry miss, `acquisition/TIER_2_MCP.md` issues `mcp.search(capability_description)`.
3. Top matches are described just-in-time.
4. A matched tool can be wrapped as a seed skill via `seed_skills/skill_builder/SKILL.md`.

## Configuration

Claude Code MCP servers are declared in `~/.claude/settings.json`. Ciel does **not** auto-add MCP servers without Council approval — each candidate passes through `council/invocation_scopes/SKILL_INTEGRATION.md` because adding a server is effectively adding a capability bundle.

## Known MCP Servers (discovery hints)

Populated at init if present:

- `sequential-thinking`
- `git` / GitHub MCP
- `mongodb-mcp-server`
- `supabase-mcp-server`
- `mcp-playwright`

Seeded into `acquisition/SOURCES.md` as Tier 2 candidates.

## Error Handling

- MCP call timeouts default to 30s; router-level budget may tighten this per call.
- Transient failures retried once; sustained failures mark the server `degraded` in `router/ROUTE_REGISTRY.md`.
- Schema mismatch (field missing) triggers a `self_improvement` task — registry entry for that MCP is out of date.

## Security

All MCP calls are logged in `observability/ACTIVITY_LOG.md` with args hashed (no raw secrets written). Safety member enforces this at registration time.
