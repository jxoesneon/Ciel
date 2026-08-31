---
name: laravel-plugin-discovery
version: 1.0.0
format: skill/1.0
description: CIEL's framework for MCP-based package discovery and health evaluation. Finds, filters, and assesses Laravel packages via LaraPlugins.io MCP with health scoring and version compatibility checks.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(find|discover|search).*(laravel|plugin|package).*(healthy|compatible|maintained)"
    confidence: 0.9
  - pattern: "laravel.*(plugin|package).*(discovery|search|evaluate|health)"
    confidence: 0.85
  - pattern: "laraplugins"
    confidence: 0.9
source: { tier: 2, origin: "laravel-plugin-discovery" }
dependencies: { skills: [], mcp: ["laraplugins"], system: [] }
side_effects: ["network"]
---

# CIEL ADAPTATION: Laravel Plugin Discovery

This skill discovers and evaluates Laravel packages via the LaraPlugins.io MCP server. While originally Laravel-specific, the discovery-plus-health-check pattern is reusable beyond Laravel — the MCP-based search, health-band filtering, and compatibility matrix approach applies to any package ecosystem with an MCP endpoint.

## MCP Configuration

Add to `~/.claude.json` mcpServers:

```json
"laraplugins": {
  "type": "http",
  "url": "https://laraplugins.io/mcp/plugins"
}
```

No API key required — the server is free for the Laravel community.

## MCP Tools

### SearchPluginTool

Search packages by keyword, health score, vendor, and version compatibility.

- `text_search` (string, optional): Keyword (e.g. "permission", "admin", "api")
- `health_score` (string, optional): `Healthy`, `Medium`, `Unhealthy`, or `Unrated`
- `laravel_compatibility` (string, optional): `"5"` through `"13"`
- `php_compatibility` (string, optional): `"7.4"` through `"8.5"`
- `vendor_filter` (string, optional): Vendor name (e.g. "spatie", "laravel")
- `page` (number, optional): Pagination

### GetPluginDetailsTool

Fetch detailed metrics, readme content, and version history.

- `package` (string, required): Full Composer name (e.g. "spatie/laravel-permission")
- `include_versions` (boolean, optional): Include version history

## Health Bands

| Band | Meaning |
|------|---------|
| `Healthy` | Active maintenance, recent updates |
| `Medium` | Occasional updates, may need attention |
| `Unhealthy` | Abandoned or infrequently maintained |
| `Unrated` | Not yet assessed |

## Best Practices

- Always filter by `health_score: "Healthy"` for production projects.
- Match `laravel_compatibility` to the target project's Laravel version.
- Prefer packages from known vendors (spatie, laravel, etc.).
- Use `GetPluginDetailsTool` for comprehensive assessment before recommending.
- Combine filters: search by keyword + health + version for precise results.

## Anti-Patterns

- **Blind search without health filter**: Recommending unrated or unhealthy packages for production use.
- **Ignoring version compatibility**: Adding a package that doesn't support the project's Laravel or PHP version.
- **Skipping detail lookup**: Recommending based on search results alone without checking vendor reputation and risk score.
- **Assuming free = no limits**: While no API key is needed, respect the MCP endpoint's rate limits and pagination.
