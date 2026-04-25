---
name: code_analysis
version: 1.0.0
description: Codebase reading, dependency mapping, architecture tracing, symbol lookup.
triggers: [analyze, understand, trace, symbol, architecture, code map]
tags: [code, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md, shell/SKILL.md] }
---

# code_analysis

Read and understand code.

## Operations

- `code.map(dir)` — build a high-level map (languages, modules, entry points).
- `code.symbol(name, path?)` — definitions + references.
- `code.deps(path)` — import graph for a file.
- `code.trace(start_symbol)` — call chain.
- `code.architecture(dir)` — summarize top-level architecture.

## I/O Contract

```yaml
io_contract:
  input: { op, path_or_symbol, "scope?" }
  output: { result: structured_map }
  idempotent: true
  side_effects: [fs]
```

## Strategy

Prefer existing LSPs / language-aware tools (tree-sitter, ripgrep, `cargo tree`, `npm ls`, `pydeps`). Falls back to regex/grep heuristics.
