---
title: Obsidian Brain Cleanup and Memory Conventions
type: decision
project: ciel
tags: [decision, adr, ciel, obsidian-brain, memory, conventions]
status: adopted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Decision: Obsidian Brain Cleanup and Memory Conventions

## Status

Adopted

## Context

A comprehensive audit of the Ciel Obsidian brain found structural gaps, missing frontmatter on many durable notes, broken or case-mismatched internal links, and missing index/scaffolding files. The vault is the primary durable memory for Ciel, so consistency directly affects retrieval quality and agentic-loop correctness.

## Decision

1. **Frontmatter is mandatory for every durable note.** Every `.md` file under `ciel/`, project notes, and root system notes must include `title`, `tags`, `created`, and `status`.
2. **Index files are the canonical entry points.** Create and maintain `ciel/kg/concepts.md`, `ciel/kg/decisions.md`, `ciel/kg/people.md`, `ciel/diary.md`, `templates.md`, `raw.md`, and `wiki.md`.
3. **Folder names use the exact casing of the underlying project.** Project folder paths in wikilinks must match the filesystem (e.g., `ciel/projects/ciel/ciel`, not `Ciel`). Obsidian on Windows is case-insensitive, but cross-platform sync and automated tooling require a single canonical spelling.
4. **The agentic-loop controller lives in the parent Ciel repository.** References in `README.md` and `AGENTS.md` now point to `C:/Users/josee/Ciel/scripts/obsidian/agentic-loop.mjs`.
5. **Missing artifact links are converted to plain-text paths.** When a diary or update references a plan or task file that was never persisted, the link is replaced by an inline path so the note remains honest and unbroken.
6. **A goal-note template is added** under `templates/goal-note.md` to support the AGENTS.md goal-decomposition step.

## Consequences

- **Positive**: Automated frontmatter and link checks now pass across the vault.
- **Positive**: New notes have consistent scaffolding.
- **Positive**: Cross-platform path handling is robust.
- **Trade-off**: Older diary entries received inferred frontmatter based on filename and first heading; any errors can be corrected in future sessions.

## Verification

- `node` frontmatter sweep: 229 files, 0 missing frontmatter, 0 duplicate frontmatter blocks.
- `node` link sweep: 0 unresolved internal links (excluding intentional `wikilinks` and `note name` examples in `_CLAUDE.md`).

## Related

- [[ciel/projects/Ciel/goals/2026-07-11-obsidian-brain-introspection]]
- [[ciel/diary/2026-07-11-obsidian-brain-introspection-cleanup]]
- [[ciel/kg/decisions/obsidian-brain-migration-audit]]
- [[AGENTS.md]]
- [[_CLAUDE.md]]
