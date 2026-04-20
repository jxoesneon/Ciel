# Template — new .skill

Use `seed_skills/skill_builder/SKILL.md` to scaffold from this template.

## Directory

```text
{{id}}/
├── SKILL.md
├── MANIFEST.md          # optional; for multi-file skills
├── CHANGELOG.md         # optional
└── extras/              # optional sub-files, prompts, etc.
```

## SKILL.md Body

```markdown
---
name: {{name}}
version: {{version|0.1.0}}
description: {{one-sentence capability description}}
triggers: {{[triggers]}}
tags: {{[tags]}}
runtime_compatibility:
  claude_code: {{true|false|partial}}
  gemini_cli: {{true|false|partial}}
  generic: {{true|false|partial}}
license: {{SPDX}}
source:
  tier: {{0|1|2|3}}
  origin: {{seed|url|mcp_server_id}}
dependencies:
  skills: {{[ids]|[]}}
  mcp: {{[server_ids]|[]}}
  system: {{[pkgs]|[]}}
---

# {{name}}

{{one-paragraph summary}}

## Operations

- `{{op.name}}({{args}})` — {{description}}

## I/O Contract

```yaml
io_contract:
  input: { {{...}} }
  output: { {{...}} }
  idempotent: {{true|false|depends}}
  side_effects: {{[shell|fs|network|state_mutation|external_api]}}
```

## Safety

{{any mid+ risk notes, or "Low risk; no additional gating."}}

## Integration

{{how this skill connects to others in the registry}}

```text
