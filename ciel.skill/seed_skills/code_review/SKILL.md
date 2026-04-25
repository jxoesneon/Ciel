---
name: code_review
version: 1.0.0
description: Code review — security audit, quality assessment, suggested fixes.
triggers: [review, audit code, quality, lint suggestions]
tags: [code, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [code_analysis/SKILL.md, linter_formatter/SKILL.md] }
---

# code_review

Review code and produce actionable feedback.

## Operations

- `review.file(path)` — comprehensive review.
- `review.diff(diff)` — PR-style review.
- `review.security(path_or_diff)` — security-focused review.
- `review.perf(path_or_diff)` — performance-focused.

## I/O Contract

```yaml
io_contract:
  input: { op, target, "focus?" }
  output: { findings: [ { severity, file, line, issue, suggestion } ] }
  idempotent: true
  side_effects: []
```

## Output

Findings are structured for downstream consumption by `improvement` flows or surfaced to the user.
