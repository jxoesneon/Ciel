---
title: Diary
type: index
tags: [index, diary]
status: active
created: 2026-07-11
---

# Diary

Chronological audit trail of sessions, decisions, and reflections.

## Recent Entries

```dataview
TABLE title, date, project, tags
FROM "ciel/diary"
SORT file.name DESC
LIMIT 20
```

## By Project

```dataview
TABLE title, date, project
FROM "ciel/diary"
WHERE project
GROUP BY project
```
