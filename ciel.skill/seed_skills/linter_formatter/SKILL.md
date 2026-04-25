---
name: linter_formatter
version: 1.0.0
description: Language-aware linting and formatting — ESLint, Prettier, rustfmt, black, etc.
triggers: [lint, format, prettier, eslint, rustfmt, black, gofmt]
tags: [code, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md, environment_detection/SKILL.md] }
---

# linter_formatter

Run the right linter/formatter for the right project.

## Operations

- `lint.run(target, fix?)` — reports + optional auto-fix.
- `fmt.run(target)` — format; preserves rules.
- `lint.detect(dir)` — which tools configured.

## I/O Contract

```yaml
io_contract:
  input: { op, target, "fix?" }
  output: { findings: [ ... ], changes: [paths] }
  idempotent: partial
  side_effects: [shell, fs]
```

## Detection

Honors project `.prettierrc`, `.eslintrc`, `rustfmt.toml`, `pyproject.toml [tool.black]`, `.golangci.yml`, `.rubocop.yml`, etc.
