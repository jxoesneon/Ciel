#!/usr/bin/env python3
"""Transcript secret sanitizer — post-write scan + in-place redact (M3c).

Docket council-20260923-conversation-audit: no hook owns transcript writes,
so host-owned conversation stores get an advisory post-write scan (run
incrementally by session_watchdog at session_start) plus this manual
sanitizer for actual remediation.

Modes:
    transcript_sanitize.py --scan           report files + categories (never content)
    transcript_sanitize.py --redact         replace matches with [REDACTED:<category>]
                                            in place; writes <file>.bak first
    transcript_sanitize.py --redact --dry   show which files would change

Stores scanned: devin transcripts + summaries, antigravity conversation DBs
(binary-safe: redaction only applied to decodable text spans). Permissions
are re-asserted to 0600/0700 after any write.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path

HOME = Path.home()
sys.path.insert(0, str(HOME / ".ciel" / "hooks" / "lib"))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "ciel.skill" / "init" / "hooks" / "lib"))
import secret_scan  # noqa: E402

STORES = [
    (HOME / ".local" / "share" / "devin" / "cli" / "transcripts", "*.json"),
    (HOME / ".local" / "share" / "devin" / "cli" / "summaries", "*.md"),
    (HOME / ".local" / "share" / "devin" / "cli" / "logs", "*.log"),
    (HOME / ".local" / "share" / "devin" / "cli" / "logs", "*.log.gz"),
    (HOME / ".gemini" / "antigravity" / "conversations", "*.db"),
    (HOME / ".gemini" / "antigravity-ide" / "conversations", "*.db"),
    (HOME / ".gemini" / "antigravity-cli" / "conversations", "*.db"),
    # Ciel's own state stores persist full command text (system1 events/RLCD
    # pairs, activity/grant ledgers, requirement checkpoints) — a secret that
    # passed through a tool call can land here too.
    (HOME / ".ciel" / "system1", "*.jsonl"),
    (HOME / ".ciel" / "checkpoints", "*.jsonl"),
    (HOME / ".ciel", "activity.log"),
    (HOME / ".ciel", "grants.log"),
]

# sessions.db is handled separately — it is a live SQLite WAL database; raw
# byte redaction would corrupt it, so it gets SQL-level redaction instead.
SESSIONS_DB = Path(os.environ.get(
    "CIEL_SESSIONS_DB",
    str(HOME / ".local" / "share" / "devin" / "cli" / "sessions.db")))
# (table, column, row-key, like-terms, deep_only). message_nodes is the
# multi-GB message table — its LIKE evaluation alone costs minutes, so the
# advisory scan skips it unless --deep; the deferred redact always covers it.
SESSIONS_TABLES = [
    ("prompt_history", "content", "id", "broad", False),
    ("message_nodes", "chat_message", "row_id", "big", True),
    ("sessions", "metadata", "id", "broad", False),
    ("sessions", "cogs_json", "id", "broad", False),
    ("sessions", "title", "id", "broad", False),
]
# Two-tier LIKE prefilter (patterns are SQL LIKE, '\_' = literal underscore,
# '_' runs approximate the regex length requirements so common prefixes stay
# selective). STRICT is cheap enough to verify row-by-row in a scan; BROAD
# adds keyword needles for the redact pass, where exhaustive coverage of the
# assignment-shaped categories matters more than speed.
PREFILTER_STRICT = [
    r"%ghp\_%", r"%gho\_%", r"%ghu\_%", r"%ghs\_%", r"%ghr\_%",
    r"%github\_pat\_%", r"%AKIA%", r"%PRIVATE KEY%",
    r"%xox%", r"%AIza%", r"%eyJ%.%.%", r"%npm\_%",
    r"%cio_________________________%",          # cio + >=25 (crates_token)
    r"%sk\_%____________________%",              # sk_ + >=20
    r"%pk\_%____________________%",              # pk_ + >=20
    r"%key\_%____________________%",             # key_ + >=20
    r"%api\_%____________________%",             # api_ + >=20
    r"%tok\_%____________________%",             # tok_ + >=20
    r"%sk-____________________%",                # sk- + >=20
]
# Assignment-needle keywords (password_assignment / secret_assignment /
# generic_secret_kv categories). Bare "token"/"secret" are excluded — they
# explode on ordinary prose at multi-GB scale.
PREFILTER_ASSIGNMENT = [
    r"%password%", r"%passwd%", r"%passphrase%",
    r"%api\_key%", r"%api-key%", r"%access\_token%", r"%access-token%",
    r"%auth\_token%", r"%auth-token%", r"%secret\_key%", r"%secret-key%",
    r"%client\_secret%", r"%client-secret%",
    r"%token is%", r"%secret is%", r"%token=%", r"%secret=%",
    r"%token:%", r"%secret:%",
]
PREFILTER_BROAD = PREFILTER_STRICT + PREFILTER_ASSIGNMENT + [
    r"%secret%", r"%token%",
]

PLACEHOLDER = "[REDACTED:{category}]"
# Binary stores (SQLite/protobuf blobs) must keep byte length identical —
# a longer placeholder corrupts length-delimited wire fields.
BINARY_SUFFIXES = {".db", ".sqlite", ".sqlite3"}


def _length_preserving(match_text: str, category: str) -> str:
    n = len(match_text)
    tag = f"[REDACTED:{category[:10]}]"
    if n >= len(tag):
        return tag + "*" * (n - len(tag))
    return "*" * n


def iter_files():
    for base, pattern in STORES:
        if not base.is_dir():
            continue
        for f in sorted(base.glob(pattern)):
            yield f


def _read_text(path: Path) -> str | None:
    try:
        if path.suffix == ".gz":
            import gzip
            return gzip.decompress(path.read_bytes()).decode("utf-8", errors="replace")
        return path.read_bytes().decode("utf-8", errors="replace")
    except OSError:
        return None


def scan_sessions_db(deep: bool = False) -> dict:
    """Read-only scan of sessions.db via LIKE prefilter + regex verify.

    deep=True also sweeps the multi-GB message_nodes table (minutes)."""
    import sqlite3
    if not SESSIONS_DB.is_file():
        return {}
    try:
        db = sqlite3.connect(f"file:{SESSIONS_DB}?mode=ro", uri=True, timeout=10)
    except sqlite3.OperationalError:
        return {"sessions.db": ["<locked>"]}
    cats = set()
    try:
        cur = db.cursor()
        for table, col, _key, tier, deep_only in SESSIONS_TABLES:
            if deep_only and not deep:
                continue
            try:
                rows = cur.execute(
                    f"SELECT {col} FROM {table} WHERE {_prefilter_where(col, tier)}"
                )
            except sqlite3.OperationalError:
                continue
            for (val,) in rows:
                if isinstance(val, bytes):
                    val = val.decode("utf-8", errors="replace")
                if val:
                    cats.update(secret_scan.scan(val)["categories"])
    finally:
        db.close()
    return {"sessions.db": sorted(cats)} if cats else {}


def scan_all(deep: bool = False) -> dict:
    hits = {}
    for f in iter_files():
        try:
            if f.stat().st_size > 64 * 1024 * 1024:
                continue
        except OSError:
            continue
        text = _read_text(f)
        if text is None:
            continue
        r = secret_scan.scan(text)
        if r["hits"]:
            hits[str(f)] = r["categories"]
    hits.update(scan_sessions_db(deep=deep))
    return hits


def redact_file(path: Path, dry: bool = False) -> dict:
    gz = path.suffix == ".gz"
    try:
        raw = path.read_bytes()
        if gz:
            import gzip
            text = gzip.decompress(raw).decode("utf-8", errors="surrogateescape")
        else:
            text = raw.decode("utf-8", errors="surrogateescape")
    except OSError as e:
        return {"file": str(path), "changed": False, "error": str(e)}

    binary = path.suffix.lower() in BINARY_SUFFIXES
    replacements = 0
    categories = set()

    def _sub(rx_name, rx, s):
        nonlocal replacements
        def repl(m):
            nonlocal replacements
            replacements += 1
            categories.add(rx_name)
            if binary:
                return _length_preserving(m.group(0), rx_name)
            return PLACEHOLDER.format(category=rx_name)
        return rx.sub(repl, s)

    for name, rx in secret_scan._COMPILED.items():
        text = _sub(name, rx, text)

    if not replacements:
        return {"file": str(path), "changed": False, "replacements": 0}
    if dry:
        return {"file": str(path), "changed": True, "dry": True,
                "replacements": replacements, "categories": sorted(categories)}

    try:
        backup = path.with_suffix(path.suffix + ".bak")
        backup.write_bytes(raw)
        os.chmod(backup, 0o600)
        payload = text.encode("utf-8", errors="surrogateescape")
        if gz:
            import gzip
            payload = gzip.compress(payload)
        path.write_bytes(payload)
        os.chmod(path, 0o600)
    except OSError as e:
        return {"file": str(path), "changed": False, "error": str(e)}
    return {"file": str(path), "changed": True, "replacements": replacements,
            "categories": sorted(categories)}


_TERMS = {"strict": PREFILTER_STRICT, "broad": PREFILTER_BROAD,
          "big": PREFILTER_STRICT + PREFILTER_ASSIGNMENT}


def _literal_needle(term: str) -> str:
    """Longest leading literal in a LIKE pattern — used as a byte-level
    instr() needle so BLOB values (invisible to LIKE, which also stops at
    embedded NULs) are still found."""
    out = []
    i, n = 0, len(term)
    while i < n:
        ch = term[i]
        if ch == "\\" and i + 1 < n:
            out.append(term[i + 1])
            i += 2
            continue
        if ch in "%_":
            if out:
                break
            i += 1
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def _prefilter_where(col: str, tier: str = "broad") -> str:
    terms = _TERMS[tier]
    likes = [f"CAST({col} AS TEXT) LIKE '{p}' ESCAPE '\\'" for p in terms]
    needles = {_literal_needle(p) for p in terms}
    instrs = [
        f"instr({col}, X'{n.encode().hex()}') > 0"
        for n in sorted(needles) if n
    ]
    return " OR ".join(likes + instrs)


def redact_sessions_db(dry: bool = False, retries: int = 6, wait: float = 5.0) -> dict:
    """SQL-level redact inside the live sessions.db WAL database.

    The DB is write-locked whenever a devin session is live, so this retries
    on SQLITE_BUSY and reports ``locked: true`` when it cannot get the write
    lock — rerun it when no devin session is active (e.g. via the
    ciel-watchdog timer, which fires between sessions).
    """
    import sqlite3
    import time

    if not SESSIONS_DB.is_file():
        return {"file": str(SESSIONS_DB), "changed": False, "reason": "absent"}

    db = None
    for attempt in range(retries):
        try:
            db = sqlite3.connect(
                f"file:{SESSIONS_DB}?mode=rw", uri=True, timeout=wait
            )
            db.execute("BEGIN IMMEDIATE")
            break
        except sqlite3.OperationalError:
            if db:
                db.close()
            db = None
            if attempt < retries - 1:
                time.sleep(wait)
    if db is None:
        return {"file": str(SESSIONS_DB), "changed": False,
                "locked": True,
                "reason": "database is locked — rerun when no devin session is active"}

    results = {"file": str(SESSIONS_DB), "changed": False, "tables": {}}
    try:
        cur = db.cursor()
        upd = db.cursor()
        for table, col, key, tier, _deep_only in SESSIONS_TABLES:
            try:
                rows = cur.execute(
                    f"SELECT {key}, {col} FROM {table} "
                    f"WHERE {_prefilter_where(col, tier)}"
                )
            except sqlite3.OperationalError:
                continue
            n = 0
            pending_updates = []
            for keyv, val in rows:
                is_bytes = isinstance(val, bytes)
                work = (val.decode("utf-8", errors="surrogateescape")
                        if is_bytes else val)
                new = work
                for name, rx in secret_scan._COMPILED.items():
                    if is_bytes:
                        # protobuf-in-BLOB: keep byte length identical
                        new = rx.sub(
                            lambda m: _length_preserving(m.group(0), name), new)
                    else:
                        new = rx.sub(PLACEHOLDER.format(category=name), new)
                if new != work:
                    n += 1
                    if not dry:
                        out = (new.encode("utf-8", errors="surrogateescape")
                               if is_bytes else new)
                        pending_updates.append((out, keyv))
            # apply after the read cursor finishes — updating a table while
            # its SELECT is iterating can skip rows
            for out, keyv in pending_updates:
                upd.execute(
                    f"UPDATE {table} SET {col}=? WHERE {key}=?", (out, keyv))
            if n:
                results["tables"][table] = n
                results["changed"] = True
        if dry:
            db.rollback()
        else:
            db.commit()
            try:
                cur.execute("PRAGMA wal_checkpoint(TRUNCATE)")
            except sqlite3.OperationalError:
                results["checkpoint"] = "busy"
    finally:
        db.close()
    return results


def tighten_store_perms() -> int:
    n = 0
    for base, _ in STORES:
        if not base.is_dir():
            continue
        try:
            if base.stat().st_mode & 0o077:
                os.chmod(base, 0o700)
                n += 1
        except OSError:
            pass
    return n


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--scan", action="store_true", help="report hits only")
    ap.add_argument("--redact", action="store_true", help="redact in place (+.bak)")
    ap.add_argument("--dry", action="store_true", help="with --redact: report only")
    ap.add_argument("--deep", action="store_true",
                    help="with --scan: also sweep the multi-GB message table")
    args = ap.parse_args()

    if args.redact:
        results = []
        for f in iter_files():
            r = redact_file(f, dry=args.dry)
            if r.get("changed") or r.get("error"):
                results.append(r)
        sdb = redact_sessions_db(dry=args.dry, retries=1 if args.dry else 6)
        if sdb.get("locked"):
            try:
                import session_watchdog
                st = session_watchdog._load(session_watchdog.STATE, {})
                st["sessions_db_sanitize_pending"] = True
                session_watchdog._save(session_watchdog.STATE, st)
                sdb["deferred"] = "flagged for next SessionStart"
            except Exception:
                pass
        tightened = tighten_store_perms()
        print(json.dumps({
            "mode": "dry" if args.dry else "redact",
            "files_changed": len(results),
            "perms_tightened": tightened,
            "sessions_db": sdb,
            "results": results,
        }, indent=1, ensure_ascii=False))
        return 0

    hits = scan_all(deep=args.deep)
    print(json.dumps({
        "mode": "scan",
        "files_with_hits": len(hits),
        "hits": {Path(k).name: v for k, v in hits.items()},
    }, indent=1, ensure_ascii=False))
    return 0 if not hits else 1


if __name__ == "__main__":
    sys.exit(main())
