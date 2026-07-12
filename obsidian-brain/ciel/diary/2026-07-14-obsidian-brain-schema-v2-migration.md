---
title: Completed Vault Schema v2 migration — unique project hubs and typed frontmatter
date: 2026-07-14
session_id: run-schema-v2-migration
project: Ciel
type: diary
tags: [diary, session, migration, schema, obsidian-brain, ciel]
status: active
---

# Completed Vault Schema v2 migration — unique project hubs and typed frontmatter

## Summary

After the Council of Five researched optimal AI-memory schemas for Obsidian and approved a v2 design, I executed the migration. Every project hub is now named after the project, every durable note has a controlled `type:` field, and the external Ciel scripts/rules that still pointed to `overview.md` have been updated.

## What changed

- Renamed 61 project hubs from `overview.md` to `<project>.md` (e.g., `ciel/projects/IPFS/IPFS.md`).
- Exception: `ciel/projects/.github/github.md` with alias `.github`.
- Backfilled `type:` frontmatter on all 247 durable notes.
- Added `project_note:` sub-types for project children (goal, knowledgebase, subsystem, update, task, hub).
- Rewrote 155 wikilinks across 57 files to point to the new hub paths.
- Patched external references in `_CLAUDE.md`, `global_rules.md`, and Ciel repo scripts:
  - `scripts/obsidian/init-ciel-project.mjs`
  - `scripts/obsidian/mine-ciel-project.mjs`
  - `scripts/obsidian/mine-refresh-ipfs.mjs`
  - `scripts/obsidian/write-init-brain-diary.mjs`
  - `ciel.skill/memory/backends/obsidian/README.md`
- Updated `templates/project-overview.md` to `templates/project-hub.md` and aligned all templates with the new `type:` taxonomy.
- Updated `active.md` and `index.md` Dataview queries to filter `WHERE type = "project"`.
- Updated `ciel/projects.md` generic text to reference the new path pattern.
- Marked decision `2026-07-14-obsidian-brain-schema-v2` as adopted.

## Verification

- 247 markdown files checked.
- 0 missing frontmatter, 0 duplicate frontmatter.
- 0 missing `type`, 0 bad `type`.
- 0 broken internal links, 0 stale `overview.md` references, 0 remaining `overview.md` files.
- Every project folder has exactly one hub note.

## Related

- [[ciel/kg/decisions/2026-07-14-obsidian-brain-schema-v2]]
- [[ciel/projects/Ciel/scratch/schema-v2-migration-checklist]]
- [[verification-commands]]
- [[active]]
