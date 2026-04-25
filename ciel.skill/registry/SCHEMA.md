# SCHEMA — Skill Metadata

Full schema for every registry entry.

```yaml
id: string                 # unique, kebab-or-slash case, e.g. "web_search/SKILL.md"
version: semver            # 1.0.0
description: string        # one-paragraph; L0 budget ~150 tokens
triggers: [string]         # normalized tokens for fast-path
tags: [string]             # taxonomy keys, see tag taxonomy below
io_contract:
  input:
    shape: object|text|path|binary|stream
    fields: { ... }
  output:
    shape: object|text|path|binary|stream
    fields: { ... }
  idempotent: bool
  side_effects: [shell|fs|network|state_mutation|external_api]
confidence_default: float  # 0..1, starting assumption pre-history
source:
  tier: 0|1|2|3             # 0 = seed, 1 = curated, 2 = mcp, 3 = web
  origin: string            # url | mcp_server | "seed"
  license: string           # SPDX identifier or "proprietary-signed-off"
install_path: path
checksum: sha256
created: iso8601
last_updated: iso8601
locked: bool                # Constitution-locked? rarely true
deprecated: bool            # queued for removal
deprecation_reason: string  # if deprecated
runtimes: [string]        # [claude_code, gemini_cli, windsurf, generic]
dependencies:
  skills: [id]
  mcp: [server_id]
  system: [package_name]
performance:
  hits: int
  success_rate: float
  avg_ms: float
  last_hit: iso8601
  last_miss: iso8601
notes: string
```

## Tag Taxonomy

Top-level namespaces (`configuration/global/acquisition.config.md` for full list):

- `vcs`, `shell`, `code`, `test`, `build`, `deploy`, `docs`, `fs`, `pkg`, `network`,
- `lang:python`, `lang:rust`, `lang:js`, `lang:go`, ...,
- `runtime:claude_code`, `runtime:gemini_cli`, `runtime:generic`, `runtime:any`,
- `scope:global`, `scope:local`, `scope:both`,
- `risk:low`, `risk:mid`, `risk:high`, `risk:critical`.

## Validation

`seed_skills/skill_installer/SKILL.md` validates against this schema before registering. Violations reject registration and enqueue a harmonization pass.
