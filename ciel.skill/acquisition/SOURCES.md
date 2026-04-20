# SOURCES

Known acquisition sources with trust scores, maintained by Ciel.

## Structure

`~/.ciel/acquisition/sources.json`:

```json
[
  {
    "id": "anthropic-official",
    "tier": 1,
    "type": "https_index",
    "url": "https://skills.anthropic.com/index.json",
    "trust": 0.95,
    "last_seen": "...",
    "success_rate": 0.98,
    "notes": "official marketplace"
  },
  {
    "id": "github-awesome-claude-skills",
    "tier": 1,
    "type": "git_repo",
    "url": "https://github.com/.../awesome-claude-skills",
    "trust": 0.80,
    "last_seen": "...",
    "success_rate": 0.87
  },
  {
    "id": "mcp-npm-registry",
    "tier": 2,
    "type": "npm_search",
    "query_template": "keyword:mcp-server",
    "trust": 0.70
  },
  {
    "id": "web-generic",
    "tier": 3,
    "type": "web_search",
    "trust": 0.30
  }
]
```

## Trust Adjustment

After each acquisition attempt:

```text
trust_new = 0.9 * trust_old + 0.1 * outcome
           where outcome = 1.0 if Council pass
                        = 0.2 if Council reject non-safety
                        = 0.0 if Safety veto
```

Trust never goes above `tier_max` (Tier 1: 0.98, Tier 2: 0.85, Tier 3: 0.55).

## Adding a Source

Users may add sources via `/ciel-sources add <url> --tier N`. Addition is routed through `council/invocation_scopes/SKILL_INTEGRATION.md` because it expands the capability surface.

## Removal

Sources with trust below `min_trust` (default 0.2) over 30 days are proposed for removal by Ciel's self-improvement loop.
