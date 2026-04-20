---
name: markdown_processor
version: 1.0.0
description: Markdown read / write / render / transform — front-matter, link extraction, anchor blocks.
triggers: [markdown, md, frontmatter, anchor, extract links]
tags: [docs, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md] }
---

# markdown_processor

Parse + manipulate markdown.

## Operations

- `md.parse(path)` — AST + frontmatter.
- `md.render(ast)` — back to text preserving style.
- `md.frontmatter.read(path)` / `md.frontmatter.update(path, patch)`.
- `md.links(path)` — extract link targets + resolve relative paths.
- `md.anchor.upsert(path, anchor_id, content)` — write between `<!-- ANCHOR:start --> / :end -->`.
- `md.anchor.read(path, anchor_id)`.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { result }
  idempotent: partial
  side_effects: [fs]
```

## Anchors

Used by `adapters/*/CONTEXT_FILES.md` for safe injection into user-maintained markdown without clobbering hand-edits.
