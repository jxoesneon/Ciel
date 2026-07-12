---
title: Active Context
type: dashboard
tags: [index, dashboard, active, context]
status: active
created: 2026-07-11
---

# Active Context

One-glance dashboard for Ciel. Read this at the start of a session to see what is in flight, where the blockers are, and what to do next.

## Recent Sessions

```dataview
TABLE date, project, tags
FROM "ciel/diary"
SORT file.name DESC
LIMIT 15
```

## Active Projects

```dataview
TABLE status, priority, updated
FROM "ciel/projects"
WHERE type = "project" AND status = "active"
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

## Open Blockers

- None explicitly recorded. Pull blockers from the most recent diary entry into this list.

## Next Actions

- Keep the vault clean: run the frontmatter/link verification after large edits.
- Backfill `ciel/kg/people/` when a person or organization becomes relevant.
- Seed `raw/` with unprocessed source material and link it from `wiki/` pages.

## Quick Links

- [[index]] — full vault catalog
- [[verification-commands]] — per-project test/build commands
- [[ciel/projects.md]] — all projects
- [[ciel/kg/decisions.md]] — all decisions
- [[ciel/kg/concepts.md]] — all concepts
- [[ciel/diary.md]] — all diary entries
