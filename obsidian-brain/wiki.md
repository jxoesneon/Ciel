---
title: Wiki
type: index
tags: [index, wiki]
status: active
created: 2026-07-11
---

# Wiki

Synthesized knowledge pages live in the `wiki/` folder. These are AI-readable distillations of raw sources, project research, and cross-cutting concepts.

## Conventions

- One idea per page; link back to the [[raw]] source.
- Use YAML frontmatter with `title`, `tags`, `created`, and `status`.
- Update wiki pages when source material or decisions change.

## Contents

```dataview
TABLE tags, created
FROM "wiki"
SORT file.name ASC
```
