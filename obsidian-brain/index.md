---
title: Ciel Brain Index
type: index
tags: [meta]
created: 2026-07-08
status: active
---

# Ciel Brain Index

Catalog of everything in this vault. Ciel reads this file first to understand the shape of the knowledge base.

## Quick Navigation

- [[_CLAUDE.md]] — Ciel's operating manual.
- [[AGENTS.md]] — Agentic loop orchestration rules.
- [[active]] — Current priorities, blockers, and recent context.
- [[verification-commands]] — Per-project test/build commands.
- [[ciel/identity]] — Core identity and preferences.
- [[ciel/diary]] — Session diary entries.
- [[ciel/kg/concepts]] — Concept notes.
- [[ciel/kg/decisions]] — Architecture decisions.
- [[ciel/kg/people]] — People and organizations.
- [[ciel/projects]] — Project workspaces.
- [[raw]] — Unprocessed source material.
- [[wiki]] — Synthesized knowledge pages.
- [[templates]] — Note templates.

## Projects

```dataview
TABLE status, priority, updated
FROM "ciel/projects"
WHERE type = "project"
SORT priority ASC, updated DESC
```

## Recent Decisions

```dataview
TABLE status, decision_date, project
FROM "ciel/kg/decisions"
SORT decision_date DESC
LIMIT 10
```

## Recent Concepts

```dataview
TABLE tags, created
FROM "ciel/kg/concepts"
SORT created DESC
LIMIT 10
```

## Recent Diary Entries

```dataview
TABLE session_id, summary
FROM "ciel/diary"
SORT file.name DESC
LIMIT 10
```

## Tags

```dataview
LIST
FROM #decision OR #concept OR #person OR #project OR #pattern OR #preference
GROUP BY file.tags
```
