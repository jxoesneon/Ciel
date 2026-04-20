# TIER_2 — MCP Discovery

Scan connected MCP servers and installable MCP registries for tools that match the gap.

## Flow

1. **List connected MCP servers** — from `~/.claude/settings.json` / `~/.gemini/settings.json`.
2. **Search** — on Claude Code, `mcp.search(gap_description)`; on Gemini CLI, iterate `mcp.list_tools(server)` and rank by description similarity.
3. **Describe matches** — `mcp.describe(tool_name)` to get full schema.
4. **Wrap as skill** — `seed_skills/skill_builder/SKILL.md` instantiates a `.skill` wrapping the MCP call.

## MCP-as-Skill Wrapping

An MCP tool becomes a Ciel skill by:

- declaring `triggers` derived from tool name + description keywords,
- mapping `io_contract.input` to the tool's JSON schema,
- defining `side_effects` from the tool's declared side-effect hints,
- linking to the MCP server in `dependencies.mcp`.

## Server Installation

If a capability exists in a known-but-not-installed MCP registry (e.g., an npm MCP package), Ciel:

1. proposes the install via `seed_skills/mcp_manager/SKILL.md`,
2. classifies risk (usually mid-risk — installing system software),
3. routes through Council via `council/invocation_scopes/SKILL_INTEGRATION.md` with the server-as-skill-bundle,
4. on pass, installs and adds to `settings.json`.

## Degraded Mode

If the runtime does not support dynamic MCP connection (generic minimal), Ciel falls through to Tier 3.

## Trust

MCP tool origin:

- publisher verification (signed index, verified publisher),
- known-good server list in `SOURCES.md`,
- community trust signals (star count, maintenance cadence).
