---
title: Vault Schema v2 — Unique project hubs and typed frontmatter
type: decision
project: ciel
decision_date: 2026-07-14
tags: [decision, adr, ciel, obsidian-brain, memory, schema]
status: adopted
created: "2026-07-14T00:00:00Z"
---

# Decision: Vault Schema v2 — Unique project hubs and typed frontmatter

## Status

Proposed — awaiting user approval after Council of Five review.

## Context

The Ciel Obsidian brain has grown to 243 markdown files. A recurring pain point is that every project hub is named `overview.md`, producing ~60 identical graph nodes in Obsidian and forcing path-prefixed wikilinks such as `[[ciel/projects/IPFS/IPFS|IPFS]]`. This makes the graph view, backlink pane, and AI retrieval ambiguous.

The user asked the Council of Five to research optimal AI-memory schemas for Obsidian and propose a final schema that fits Ciel's project portfolio (Flutter/Dart, Rust, Python, TypeScript, Godot, TeX, MCP servers, P2P media tools) and aligns with current best practices and standards up to system date.

## Council of Five review

| Lens | Score | Verdict | Key point |
|------|-------|---------|-----------|
| Coherence | 5 | proceed | Unique filenames + controlled `type:` taxonomy; shallow ACE-informed tree. |
| Capability | 4 | proceed | Typed frontmatter enables Dataview/MCP queries; reserve atomic facts/events for later growth. |
| Safety | 3 | revise | External scripts/rules still hardcode `overview.md`; use git branch, Obsidian-aware renames or manual link rewrite, and verification. |
| Efficiency | 4 | proceed | Minimal change: rename hubs, add four-field type contract, Dataview indexes. Defer enterprise atomic-fact schemas until 500+ files. |
| Evolution | 5 | proceed | Plain markdown + YAML frontmatter is portable; leave an extensibility path toward JSON Schema/atomic facts. |

**Synthesized verdict:** proceed with a pragmatic v2 schema now, while documenting the extension path toward a future v3 (atomic facts, events, generated views, schema validation).

## Decision

1. **Project hubs are named after the project.**
   - `ciel/projects/<project>/overview.md` → `ciel/projects/<project>/<project>.md`.
   - Example: `ciel/projects/IPFS/IPFS.md`, `ciel/projects/X-Seed/X-Seed.md`.
   - Exception: `ciel/projects/.github/` stays as-is because `.github.md` would be dot-prefixed and potentially hidden in Obsidian. Its hub becomes `ciel/projects/.github/github.md` with alias `.github`.

2. **Every durable note has a controlled `type:` field.**
   - Required values: `project`, `project-note`, `concept`, `decision`, `diary`, `person`, `org`, `index`, `dashboard`, `template`, `system`, `goal`, `task`, `raw`, `wiki`.
   - Optional `project_note:` sub-type for project children: `hub`, `goal`, `knowledgebase`, `update`, `subsystem`.

3. **Minimal required frontmatter contract.**
   - `type`
   - `title`
   - `status` (`active`, `draft`, `review`, `archived`, `backlog`, `complete`)
   - `created`
   - `tags`

4. **Folder structure stays shallow.**
   - `ciel/projects/<project>/` — project hubs and project-specific notes.
   - `ciel/kg/{concepts,decisions,people}/` — durable, cross-project knowledge.
   - `ciel/diary/` — session logs.
   - `ciel/raw/` — unprocessed captures.
   - `ciel/wiki/` — synthesized pages.
   - `templates/` — note templates.
   - Root dashboards: `index.md`, `active.md`, `verification-commands.md`.

5. **Indexes use Dataview where possible, but remain static-MOC friendly.**
   - Root `index.md` and folder indexes are hand-curated MOCs.
   - `active.md` uses Dataview for recent sessions/projects/decisions.
   - Future generated rollups (v3) will be serialized to static markdown, not live Dataview/JS, to stay readable by Claude Code, Devin, Cursor, etc.

6. **Templates emit the new schema.**
   - `templates/project-hub.md` (renamed from `project-overview.md`) produces a project hub with `type: project`.
   - All templates include `type`.

7. **External references must be updated before the rename.**
   - `C:/Users/josee/Ciel/obsidian-brain/_CLAUDE.md`
   - `C:/Users/josee/.codeium/windsurf/memories/global_rules.md`
   - `C:/Users/josee/Ciel/scripts/obsidian/init-ciel-project.mjs`
   - `C:/Users/josee/Ciel/scripts/obsidian/mine-ciel-project.mjs`
   - `C:/Users/josee/Ciel/scripts/obsidian/mine-refresh-ipfs.mjs`
   - `C:/Users/josee/Ciel/scripts/obsidian/write-init-brain-diary.mjs`
   - Ciel backend README examples referencing `overview.md`.

## Consequences

- **Positive:** Every project becomes a unique, linkable node in the graph. `type:` lets Dataview and agents query only hubs, only decisions, only active concepts, etc.
- **Positive:** Frontmatter becomes a portable schema across AI tools (Claude Code, Devin, Cursor).
- **Positive:** Templates enforce the schema, reducing drift.
- **Trade-off:** ~60 file renames and ~160 link updates require a one-time scripted migration and verification.
- **Trade-off:** External Ciel repo scripts must be patched in the same operation or they will recreate stale `overview.md` files.
- **Risk:** Bulk filesystem renames do not trigger Obsidian's link-update logic; migration must rewrite wikilinks explicitly and then run a link checker.

## Migration plan

1. Freeze vault state in git on a dedicated branch; push a tagged baseline.
2. Patch external scripts/rules to emit/read `<project>.md`.
3. Backfill `type:` frontmatter on all 243 durable notes via script.
4. Rename each `overview.md` to `<project>.md` (or `github.md` for `.github`).
5. Rewrite all `[[ciel/projects/<project>/overview|...]]` links to `[[ciel/projects/<project>/<project>|...]]`.
6. Update `ciel/projects.md`, `index.md`, `active.md`, and templates.
7. Run frontmatter + broken-link verification; confirm zero `overview.md` references.
8. Rebuild Obsidian cache / reindex.
9. Commit as a single isolated change.

## Future extension path (v3)

When the vault passes ~500 durable notes or cross-agent writes become common:
- Add `ciel/kg/facts/<entity>/<predicate>.md` atomic typed facts and `ciel/kg/events/YYYY-MM-DD/<slug>.md` episodic records.
- Add `ciel/kg/schema/*.schema.yaml` and a linter.
- Add generated `_views/` serialized to static markdown.
- Add agent inbox `ciel/kg/_inbox/<agent-id>/ops/`.
- Evaluate Smart Connections, QMD, or obsidian-semantic-memory for vector/hybrid retrieval.

## Related

- [[ciel/kg/decisions/2026-07-11-obsidian-brain-cleanup-conventions]]
- [[ciel/kg/concepts/council-subagent-invocation]]
- [[ciel/projects/Ciel/goals/2026-07-11-obsidian-brain-introspection]]
- [[active]]
- [[verification-commands]]
