---
name: code_generation
version: 1.0.0
description: Generate code across languages from spec — implementations, stubs, transformations.
triggers: [generate code, scaffold, implement, write function, write class]
tags: [code, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md, code_analysis/SKILL.md] }
---

# code_generation

Generate code from a specification.

## Operations

- `gen.function(spec, lang, style)` — a function with tests if requested.
- `gen.module(spec, target_path)` — a module.
- `gen.scaffold(framework, name)` — framework-native scaffold.
- `gen.transform(file, transformation)` — AST-level edit (rename, extract, inline).

## I/O Contract

```yaml
io_contract:
  input: { op, spec, target_path?, lang?, style? }
  output: { files_written: [paths], diff }
  idempotent: false
  side_effects: [fs]
---

## Style Adherence

Project style rules from `configuration/local/rules.config.md`. When generating tests, respects project testing conventions (pytest vs jest etc).
