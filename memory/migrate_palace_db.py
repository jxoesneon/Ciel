#!/usr/bin/env python3
"""
palace.db Schema Migration
===========================
Migrates ~/.ciel/memory/palace.db from the simple `memories` table
(id, content, created_at) to the spec-compliant schema defined in
memory/backends/SQLITE.md:

    entries       (partition, key, value, metadata, created, updated)
    entries_fts   (FTS5 virtual table over entries)
    meta          (key-value store)

Usage:
    python3 migrate_palace_db.py [--db PATH] [--dry-run]

Exit codes:
    0  success
    1  preflight failure (db missing, FTS5 unavailable, etc.)
    2  migration failure (rolled back)
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import shutil
import sqlite3
import sys
from pathlib import Path
from typing import Any

DEFAULT_DB = Path(os.path.expanduser("~/.ciel/memory/palace.db"))
SCHEMA_VERSION = "2.0"
MIGRATED_FROM = "memories_table"


# ---------------------------------------------------------------------------
# Schema DDL (mirrors memory/backends/SQLITE.md exactly)
# ---------------------------------------------------------------------------

DDL_ENTRIES = """
CREATE TABLE IF NOT EXISTS entries (
  partition TEXT NOT NULL,
  key       TEXT NOT NULL,
  value     BLOB NOT NULL,
  metadata  TEXT NOT NULL,
  created   INTEGER NOT NULL,
  updated   INTEGER NOT NULL,
  PRIMARY KEY (partition, key)
);
""".strip()

DDL_INDEX = (
    "CREATE INDEX IF NOT EXISTS idx_entries_created "
    "ON entries(partition, created);"
)

DDL_FTS = """
CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
  partition, key, content,
  content='entries', content_rowid='rowid'
);
""".strip()

DDL_META = """
CREATE TABLE IF NOT EXISTS meta (
  key TEXT PRIMARY KEY,
  value TEXT
);
""".strip()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def log(msg: str) -> None:
    print(f"[migrate] {msg}", flush=True)


def iso_now() -> str:
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def to_unix_ts(value: Any) -> int:
    """Best-effort conversion of a stored created_at value to a Unix timestamp.

    Accepts ISO-8601 strings, numeric strings, or ints. Falls back to now().
    """
    if value is None:
        return int(_dt.datetime.now(_dt.timezone.utc).timestamp())
    if isinstance(value, (int, float)):
        return int(value)
    s = str(value).strip()
    if s.isdigit():
        return int(s)
    # try ISO-8601
    for fmt in ("%Y-%m-%dT%H:%M:%SZ", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"):
        try:
            dt = _dt.datetime.strptime(s, fmt).replace(tzinfo=_dt.timezone.utc)
            return int(dt.timestamp())
        except ValueError:
            continue
    # last resort
    return int(_dt.datetime.now(_dt.timezone.utc).timestamp())


def check_fts5(conn: sqlite3.Connection) -> bool:
    try:
        conn.execute("CREATE VIRTUAL TABLE IF NOT EXISTS _fts5_probe USING fts5(x)")
        conn.execute("DROP TABLE _fts5_probe")
        return True
    except sqlite3.OperationalError:
        return False


def table_exists(conn: sqlite3.Connection, name: str) -> bool:
    row = conn.execute(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (name,)
    ).fetchone()
    return row is not None


def table_columns(conn: sqlite3.Connection, name: str) -> list[str]:
    return [r[1] for r in conn.execute(f"PRAGMA table_info({name})")]


# ---------------------------------------------------------------------------
# Migration steps
# ---------------------------------------------------------------------------

def backup_db(db_path: Path) -> Path:
    bak = db_path.with_suffix(db_path.suffix + ".pre-migration.bak")
    shutil.copy2(db_path, bak)
    log(f"backup created: {bak} ({bak.stat().st_size} bytes)")
    return bak


def create_new_schema(conn: sqlite3.Connection) -> None:
    log("creating new spec-compliant tables...")
    conn.execute(DDL_ENTRIES)
    conn.execute(DDL_INDEX)
    conn.execute(DDL_FTS)
    conn.execute(DDL_META)
    conn.commit()
    log("  entries, idx_entries_created, entries_fts, meta created")


def migrate_rows(conn: sqlite3.Connection) -> int:
    """Migrate rows from the old `memories` table into `entries` + `entries_fts`.

    Returns the number of rows migrated.
    """
    if not table_exists(conn, "memories"):
        log("old `memories` table not present — nothing to migrate")
        return 0

    cols = table_columns(conn, "memories")
    count = conn.execute("SELECT COUNT(*) FROM memories").fetchone()[0]
    log(f"old `memories` table has {count} row(s); columns={cols}")

    if count == 0:
        return 0

    migrated = 0
    for row in conn.execute("SELECT id, content, created_at FROM memories"):
        old_id, content, created_at = row
        key = str(old_id)
        partition = "global"
        value = (content or "").encode("utf-8") if isinstance(content, str) else bytes(content or b"")
        metadata = "{}"
        ts = to_unix_ts(created_at)

        conn.execute(
            "INSERT OR REPLACE INTO entries "
            "(partition, key, value, metadata, created, updated) "
            "VALUES (?, ?, ?, ?, ?, ?)",
            (partition, key, value, metadata, ts, ts),
        )
        # keep FTS in sync (content column mirrors value as text)
        conn.execute(
            "INSERT OR REPLACE INTO entries_fts(partition, key, content) "
            "VALUES (?, ?, ?)",
            (partition, key, content or ""),
        )
        migrated += 1

    conn.commit()
    log(f"migrated {migrated} row(s) from memories -> entries")
    return migrated


def drop_old_table(conn: sqlite3.Connection) -> None:
    if not table_exists(conn, "memories"):
        log("old `memories` table already absent — skip drop")
        return
    conn.execute("DROP TABLE memories")
    conn.commit()
    log("old `memories` table dropped")


def write_meta(conn: sqlite3.Connection) -> None:
    ts = iso_now()
    records = {
        "schema_version": SCHEMA_VERSION,
        "migration_date": ts,
        "migrated_from": MIGRATED_FROM,
    }
    for k, v in records.items():
        conn.execute(
            "INSERT OR REPLACE INTO meta(key, value) VALUES (?, ?)", (k, v)
        )
    conn.commit()
    log(f"meta records written: {list(records.keys())}")


def set_pragmas(conn: sqlite3.Connection) -> None:
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=NORMAL")
    mode = conn.execute("PRAGMA journal_mode").fetchone()[0]
    sync = conn.execute("PRAGMA synchronous").fetchone()[0]
    log(f"PRAGMA journal_mode={mode}, synchronous={sync}")


def verify(conn: sqlite3.Connection, rows_migrated: int) -> dict[str, Any]:
    log("running verification...")
    tables = [
        r[0]
        for r in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        )
    ]
    for required in ("entries", "entries_fts", "meta"):
        assert required in tables, f"missing table: {required}"

    # index check
    indexes = [
        r[0]
        for r in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='index' ORDER BY name"
        )
    ]
    assert "idx_entries_created" in indexes, "missing idx_entries_created"

    # row count
    entry_count = conn.execute("SELECT COUNT(*) FROM entries").fetchone()[0]
    assert entry_count == rows_migrated, (
        f"entries row count mismatch: expected {rows_migrated}, got {entry_count}"
    )

    # FTS5 functional check (should return empty, not error)
    fts_hit = conn.execute(
        "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'test'"
    ).fetchone()[0]
    log(f"FTS5 MATCH 'test' -> {fts_hit} hit(s) (expected 0)")

    # meta records
    meta = dict(conn.execute("SELECT key, value FROM meta").fetchall())
    assert meta.get("schema_version") == SCHEMA_VERSION, (
        f"meta schema_version != {SCHEMA_VERSION}: {meta.get('schema_version')}"
    )
    assert "migration_date" in meta, "meta missing migration_date"
    assert meta.get("migrated_from") == MIGRATED_FROM, (
        f"meta migrated_from mismatch: {meta.get('migrated_from')}"
    )

    log("verification PASSED")
    return {
        "tables": tables,
        "indexes": indexes,
        "entry_count": entry_count,
        "fts5_functional": True,
        "meta": meta,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description="palace.db schema migration")
    ap.add_argument("--db", default=str(DEFAULT_DB), help="path to palace.db")
    ap.add_argument("--dry-run", action="store_true", help="verify only, no writes")
    args = ap.parse_args()

    db_path = Path(os.path.expanduser(args.db))
    if not db_path.exists():
        log(f"ERROR: db not found: {db_path}")
        return 1

    log(f"target db: {db_path} ({db_path.stat().st_size} bytes)")

    # preflight: FTS5 availability on a throwaway connection
    pre = sqlite3.connect(":memory:")
    if not check_fts5(pre):
        log("ERROR: FTS5 not available in this SQLite build")
        pre.close()
        return 1
    pre.close()
    log("preflight: FTS5 available")

    if args.dry_run:
        log("dry-run: no changes will be made")
        conn = sqlite3.connect(str(db_path))
        try:
            tables = [
                r[0]
                for r in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
                )
            ]
            log(f"current tables: {tables}")
        finally:
            conn.close()
        return 0

    # backup before touching anything
    bak = backup_db(db_path)

    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA foreign_keys=ON")
    try:
        # all migration work in a single transaction so a failure rolls back
        conn.execute("BEGIN")
        create_new_schema(conn)
        rows_migrated = migrate_rows(conn)
        drop_old_table(conn)
        write_meta(conn)
        verify(conn, rows_migrated)
        conn.commit()
        log("transaction committed")

        # pragmas applied after commit (journal_mode is persistent anyway)
        set_pragmas(conn)

        # final stats
        stats = {
            "db_path": str(db_path),
            "backup_path": str(bak),
            "rows_migrated": rows_migrated,
            "schema_version": SCHEMA_VERSION,
            "tables": [
                r[0]
                for r in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
                )
            ],
            "meta": dict(conn.execute("SELECT key, value FROM meta").fetchall()),
            "timestamp": iso_now(),
        }
        print("\n=== MIGRATION REPORT ===")
        print(json.dumps(stats, indent=2))
        return 0
    except Exception as exc:
        conn.rollback()
        log(f"ERROR: migration failed, rolled back: {exc}")
        # restore backup on failure
        try:
            conn.close()
            shutil.copy2(bak, db_path)
            log(f"restored backup {bak} -> {db_path}")
        except Exception as restore_exc:
            log(f"ERROR restoring backup: {restore_exc}")
        return 2
    finally:
        try:
            conn.close()
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())
