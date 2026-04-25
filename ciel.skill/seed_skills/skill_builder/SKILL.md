---
name: skill_builder
version: 1.0.0
description: Build new .skill ZIP files — directory, SKILL.md, MANIFEST, packaging.
triggers: [new skill, build skill, package skill, zip skill]
tags: [skill, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md, archive_manager/SKILL.md, markdown_processor/SKILL.md] }
---
# skill_builder

Create a new `.skill` bundle from structured input.

## Operations

- `skill.new(spec)` — scaffold directory from `templates/skill.template.md`.
- `skill.assemble(dir)` — build ZIP (`.skill`) with checksum manifest.
- `skill.validate(dir)` — schema + integrity check.
- `skill.frontmatter(parts) -> yaml` — build frontmatter.

## I/O Contract

```yaml
io_contract:
  input: { op, "spec?", "dir?" }
  output: { path_or_bytes, report }
  idempotent: depends
  side_effects: [fs]
```

## Integration

Used by `acquisition/TIER_3_WEB.md` for synthesis outputs, and by `self_improvement/` for composing new registered skills from accumulated patterns.
