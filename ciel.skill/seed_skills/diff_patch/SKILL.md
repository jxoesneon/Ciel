---
name: diff_patch
version: 1.0.0
description: Unified diff generation, patch application, conflict detection.
triggers: [diff, patch, apply patch, hunk]
tags: [code, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md, shell/SKILL.md] }
---
# diff_patch

Unified diffs + patch application.

## Operations

- `diff.create(a_path, b_path)` / `diff.from_strings(a, b)`.
- `patch.apply(path, patch, strict=true)`.
- `patch.conflict(path, patch)` — detects without applying.
- `patch.three_way(ours, theirs, base)` — textual 3-way merge.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { result, "conflicts?" }
  idempotent: for_same_inputs
  side_effects: [fs]
```

## Use

`self_improvement/` uses this to construct and apply mutation diffs.
