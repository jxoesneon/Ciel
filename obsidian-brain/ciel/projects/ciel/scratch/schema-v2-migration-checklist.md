---
title: Schema v2 Migration Checklist
project_note: update
type: task
project: Ciel
status: complete
created: 2026-07-14
tags: [scratch, checklist, migration, schema, ciel]
---

# Schema v2 Migration Checklist

This is the running checklist for the Vault Schema v2 migration.

## Pre-flight

- [x] Create git branch `vault-schema-v2` and tag baseline.
- [x] Confirm no uncommitted vault changes that would conflict.

## External dependencies (must happen before vault renames)

- [x] Patch `_CLAUDE.md` line referencing `ciel/projects/<project>/overview.md`.
- [x] Patch `.codeium/windsurf/memories/global_rules.md` line referencing `ciel/projects/<project>/overview.md`.
- [x] Patch `Ciel/scripts/obsidian/init-ciel-project.mjs`.
- [x] Patch `Ciel/scripts/obsidian/mine-ciel-project.mjs`.
- [x] Patch `Ciel/scripts/obsidian/mine-refresh-ipfs.mjs`.
- [x] Patch `Ciel/scripts/obsidian/write-init-brain-diary.mjs`.
- [x] Patch `Ciel/ciel.skill/memory/backends/obsidian/README.md` overview examples.

## Vault schema backfill

- [x] Add `type:` to every durable note in vault.
- [x] Add `project_note:` sub-type where applicable.
- [x] Update templates to emit `type:` and new naming.

## Rename pass

- [x] Rename `ciel/projects/<project>/overview.md` → `ciel/projects/<project>/<project>.md`.
- [x] Exception: `ciel/projects/.github/overview.md` → `ciel/projects/.github/github.md` with alias `.github`.

## Link rewrite

- [x] Rewrite `[[ciel/projects/<project>/overview|...]]` → `[[ciel/projects/<project>/<project>|...]]` across vault.
- [x] Update `ciel/projects.md` index lines.
- [x] Update `index.md` Dataview blocks / links.
- [x] Update `active.md` Dataview blocks / links.
- [x] Update templates and template index.

## Verification

- [x] Run frontmatter sweep: every durable note has `type`, `title`, `status`, `created`, `tags`.
- [x] Run link sweep: zero broken internal links.
- [x] Confirm zero stale `overview.md` wikilink references.
- [x] Confirm exactly one hub note per project folder.
- [ ] Rebuild Obsidian cache or restart Obsidian.

## Post-migration

- [x] Mark decision `2026-07-14-obsidian-brain-schema-v2` as adopted.
- [x] Write diary entry summarizing the migration.
- [ ] Commit changes to `vault-schema-v2` branch.
