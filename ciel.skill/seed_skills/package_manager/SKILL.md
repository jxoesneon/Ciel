---
name: package_manager
version: 1.0.0
description: npm / pip / cargo / apt / brew — install, update, audit, lock.
triggers: [install, uninstall, package, npm, pip, cargo, brew, apt]
tags: [pkg, scope:both, runtime:any, risk:mid]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md, environment_detection/SKILL.md] }
---
# package_manager

Cross-ecosystem package operations.

## Operations

- `pkg.install(name, version?, ecosystem?)` — install (mid-risk: system state).
- `pkg.remove(name)`.
- `pkg.update(name?)`.
- `pkg.audit()` — vulnerability scan.
- `pkg.lock()` — freeze versions.
- `pkg.which(name)` — which ecosystem provides it.

## Ecosystem Selection

Auto-detected by manifest presence (`package.json`, `Cargo.toml`, `pyproject.toml`, etc.) and environment. Explicit override via `ecosystem: npm|pip|cargo|apt|brew|yarn|pnpm|bundler|go|rubygems`.

## I/O Contract

```yaml
io_contract:
  input: { op, "name?", "version?", "ecosystem?" }
  output: { installed: [ { name, version } ], warnings }
  idempotent: depends
  side_effects: [shell, fs, network, state_mutation]
```

## Safety

- System-wide installs (`apt`, `brew`) are always mid-risk; require judge pass.
- Dev deps in project are low-risk.
- Dep audits surface CVEs to `dependency_audit/SKILL.md`.
