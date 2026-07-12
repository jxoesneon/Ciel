---
title: Mempalace superseded by Obsidian
type: decision
tags: [decision, adr, memory, ciel]
project: ciel
decision_date: 2026-07-09
created: 2026-07-09
status: adopted
---

# Mempalace superseded by Obsidian

## Decision

The `mempalace-rs` / Mempalace MCP memory backend is retired. The Obsidian vault at `C:\Users\josee\Ciel\obsidian-brain` is now the sole durable working memory for Ciel.

## Consequences

- Stop calling `mempalace_*` MCP tools.
- At session start, read `ciel/projects/<project>/overview.md` and search the vault.
- At session end, write a diary entry to `ciel/diary/`.
- Prefer the Obsidian Local REST API (`http://127.0.0.1:27123`) for writes; fall back to direct filesystem writes only when the API is unavailable.
- Update Ciel instruction files (`CLAUDE.md`, `global_rules.md`, `_CLAUDE.md`) to remove Mempalace references.

## Updated instruction files

- `C:\Users\josee\.claude\CLAUDE.md`
- `C:\Users\josee\.codeium\windsurf\memories\global_rules.md`
- `C:\Users\josee\Ciel\obsidian-brain\_CLAUDE.md`

## Related

- [[ciel/kg/decisions/obsidian-brain-migration-audit|Obsidian brain migration audit]]
