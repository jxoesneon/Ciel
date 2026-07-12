---
title: Raw Sources
type: index
tags: [index, raw]
status: active
created: 2026-07-11
---

# Raw Sources

Unprocessed source material lives in the `raw/` folder: clips, transcripts, logs, user paste-ins, and mined web pages.

## Conventions

- Drop source material into `raw/<project>/` or `raw/unsorted/`.
- Use `defuddle` or similar tools to extract clean markdown before long-term storage.
- Cite raw sources from synthesized [[wiki]] pages and decision records.

## Contents

```dataview
TABLE file.ctime AS "Added"
FROM "raw"
SORT file.name ASC
```
