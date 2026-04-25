---
name: project_analyzer
version: 1.0.0
description: Project context detection for init calibration — conventions, stack, patterns.
triggers: [analyze project, detect stack, project context]
tags: [fs, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md, shell/SKILL.md, git/SKILL.md] }
---

# project_analyzer

Produce the `project.json` used by `init/CALIBRATION.md`.

## Operations

- `proj.analyze(dir)` — full fingerprint.
- `proj.conventions(dir)` — detected style, naming, file layout.
- `proj.stack(dir)` — language, frameworks, build, test, CI.
- `proj.risk_profile(dir)` — signals feeding escalation auto-detect.

## I/O Contract

```yaml
io_contract:
  input: { op, dir }
  output: { result: project_json }
  idempotent: true
  side_effects: [fs]
```

## Signals

- Manifests, config dotfiles, CI folders, license, branch convention, CODEOWNERS, secrets hints, deploy targets.
