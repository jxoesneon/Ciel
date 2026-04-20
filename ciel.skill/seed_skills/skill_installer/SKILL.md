---
name: skill_installer
version: 1.0.0
description: Install .skill files into the global registry — unzip, validate, register.
triggers: [install skill, register skill, add skill]
tags: [skill, scope:both, runtime:any, risk:mid]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [archive_manager/SKILL.md, filesystem/SKILL.md, skill_builder/SKILL.md] }
---

# skill_installer

Install a `.skill` bundle into `~/.ciel/skills/`.

## Operations

- `install(path_or_bytes)` — unzip → validate schema → checksum → register.
- `install.dry_run(path)` — what would happen without applying.
- `install.uninstall(id)` — remove from registry; move to `~/.ciel/.attic/`.
- `install.list()` — installed skills with versions.

## I/O Contract

```yaml
io_contract:
  input: { op, path_or_id, "dry_run?" }
  output: { result, changes }
  idempotent: for_same_version
  side_effects: [fs, state_mutation]
```

## Safety

Installs always flow through `council/invocation_scopes/SKILL_INTEGRATION.md`. Bypass is forbidden by Constitution.
