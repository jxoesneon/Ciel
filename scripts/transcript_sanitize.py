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
    (HOME / ".gemini" / "antigravity" / "conversations", "*.db"),
    (HOME / ".gemini" / "antigravity-ide" / "conversations", "*.db"),
    (HOME / ".gemini" / "antigravity-cli" / "conversations", "*.db"),
]

PLACEHOLDER = "[REDACTED:{category}]"


def iter_files():
    for base, pattern in STORES:
        if not base.is_dir():
            continue
        for f in sorted(base.glob(pattern)):
            yield f


def scan_all() -> dict:
    hits = {}
    for f in iter_files():
        try:
            if f.stat().st_size > 64 * 1024 * 1024:
                continue
            raw = f.read_bytes()
            text = raw.decode("utf-8", errors="replace")
        except OSError:
            continue
        r = secret_scan.scan(text)
        if r["hits"]:
            hits[str(f)] = r["categories"]
    return hits


def redact_file(path: Path, dry: bool = False) -> dict:
    try:
        raw = path.read_bytes()
        text = raw.decode("utf-8", errors="surrogateescape")
    except OSError as e:
        return {"file": str(path), "changed": False, "error": str(e)}

    replacements = 0
    categories = set()

    def _sub(rx_name, rx, s):
        nonlocal replacements
        def repl(m):
            nonlocal replacements
            replacements += 1
            categories.add(rx_name)
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
        path.write_bytes(text.encode("utf-8", errors="surrogateescape"))
        os.chmod(path, 0o600)
    except OSError as e:
        return {"file": str(path), "changed": False, "error": str(e)}
    return {"file": str(path), "changed": True, "replacements": replacements,
            "categories": sorted(categories)}


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
    args = ap.parse_args()

    if args.redact:
        results = []
        for f in iter_files():
            r = redact_file(f, dry=args.dry)
            if r.get("changed") or r.get("error"):
                results.append(r)
        tightened = tighten_store_perms()
        print(json.dumps({
            "mode": "dry" if args.dry else "redact",
            "files_changed": len(results),
            "perms_tightened": tightened,
            "results": results,
        }, indent=1, ensure_ascii=False))
        return 0

    hits = scan_all()
    print(json.dumps({
        "mode": "scan",
        "files_with_hits": len(hits),
        "hits": {Path(k).name: v for k, v in hits.items()},
    }, indent=1, ensure_ascii=False))
    return 0 if not hits else 1


if __name__ == "__main__":
    sys.exit(main())
