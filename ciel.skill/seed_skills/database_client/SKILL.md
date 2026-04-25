---
name: database_client
version: 1.0.0
description: DB query execution — SQL / NoSQL, schema inspection, migration awareness.
triggers: [sql, query, database, db, select, migration]
tags: [data, scope:both, runtime:any, risk:mid]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [secrets_manager/SKILL.md, api_client/SKILL.md] }
---
# database_client

Run queries against databases.

## Operations

- `db.connect(dsn_ref)` — DSN retrieved via `secrets_manager`.
- `db.query(conn, sql, params?)` — returns rows + metadata.
- `db.execute(conn, sql, params?)` — write queries (at least mid-risk).
- `db.schema(conn, object?)` — introspect.
- `db.migrations.status(dir)` — checked against detected migration tool.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { rows_or_meta }
  idempotent: for_SELECT
  side_effects: [network, "state_mutation?"]
```

## Safety

- Destructive SQL (`DROP`, `TRUNCATE`, `DELETE` without WHERE on prod) → critical.
- `db.execute` on production connection → Council + user.
- All queries logged (with parameter values redacted if they match secret patterns).
