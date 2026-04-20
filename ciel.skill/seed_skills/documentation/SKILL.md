---
name: documentation
version: 1.0.0
description: README, API docs, inline comments, changelog generation.
triggers: [readme, document, docs, changelog, api docs]
tags: [docs, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md, code_analysis/SKILL.md] }
---

# documentation

Generate and maintain project documentation.

## Operations

- `doc.readme(project)` — generate/update README.
- `doc.api(source_path, format)` — generate API docs from source (TSDoc, rustdoc, Sphinx, etc.).
- `doc.changelog(since_ref?, style=keep-a-changelog)`.
- `doc.inline(file, policy)` — add/update docstrings matching project style.

## I/O Contract

```yaml
io_contract:
  input: { op, target, "format?", "policy?" }
  output: { files_written: [paths], diff }
  idempotent: partial
  side_effects: [fs]
```

## Style

Respects `configuration/local/rules.config.md.documentation.*` and detected conventions.
