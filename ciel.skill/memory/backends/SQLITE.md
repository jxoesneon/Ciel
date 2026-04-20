# SQLITE — Fallback Backend

Single-file SQLite database as a fallback when MemPalace-rs is unavailable.

## Location

- Global: `~/.ciel/ciel.db`
- Local: `<project>/.ciel/ciel.db`

Per-project file isolation. Partition boundaries enforced at app layer.

## Schema

```sql
CREATE TABLE IF NOT EXISTS entries (
  partition TEXT NOT NULL,
  key       TEXT NOT NULL,
  value     BLOB NOT NULL,
  metadata  TEXT NOT NULL,
  created   INTEGER NOT NULL,
  updated   INTEGER NOT NULL,
  PRIMARY KEY (partition, key)
);
CREATE INDEX IF NOT EXISTS idx_entries_created ON entries(partition, created);

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
  partition, key, content,
  content='entries', content_rowid='rowid'
);

CREATE TABLE IF NOT EXISTS meta (
  key TEXT PRIMARY KEY,
  value TEXT
);
```

## API Mapping

| MemPalace API | SQLite impl |
| --- | --- |
| `put` | `INSERT OR REPLACE INTO entries` |
| `get` | `SELECT value FROM entries WHERE partition=? AND key=?` |
| `query(filter)` | parameterized `SELECT` with `LIKE`/exact match |
| `search(query, k)` | FTS5 match on `entries_fts` |
| `delete` | `DELETE FROM entries` |
| `list(prefix)` | `SELECT key FROM entries WHERE partition=? AND key LIKE prefix + '%'` |
| `compact` | no-op (FTS5 auto-maintains) |
| `snapshot` | file copy with WAL checkpoint |

## Migration

Migration scripts live at `memory/backends/sqlite_migrations/`. Version tracked in `meta.key='schema_version'`.

## Embeddings

SQLite alone doesn't support embeddings. Semantic-search operations are approximated by FTS5 + lexical rank. Ciel logs the degradation.

## Durability

PRAGMA `journal_mode=WAL` + `synchronous=NORMAL`. For critical writes (council decisions), Ciel switches to `FULL` on demand.
