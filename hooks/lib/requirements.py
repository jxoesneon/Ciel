#!/usr/bin/env python3
"""Session requirement ledger — lightweight checklist on LOCAL_STORE.

Docket council-20260923-conversation-audit, mechanism M7: mid-session asks
must not silently drop before "done". Append-only JSONL events under
``~/.ciel/checkpoints/requirements.jsonl``; ``pending`` is the set of added
items not yet resolved. No bespoke store — reuses the checkpoints domain.

CLI:
    requirements.py add "requirement text" [--session ID]
    requirements.py done <id|substring> [--session ID]
    requirements.py list [--session ID]
    requirements.py pending [--session ID]     # prints count + items

Items are session-scoped when a session id is supplied; ``pending`` with no
session returns all unresolved items.
"""

import argparse
import json
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

LEDGER = Path.home() / ".ciel" / "checkpoints" / "requirements.jsonl"


def _events() -> list[dict]:
    try:
        out = []
        for line in LEDGER.read_text(encoding="utf-8").splitlines():
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        return out
    except OSError:
        return []


def _append(event: dict) -> None:
    LEDGER.parent.mkdir(parents=True, exist_ok=True)
    event["ts"] = datetime.now(timezone.utc).isoformat()
    with LEDGER.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(event, ensure_ascii=False) + "\n")


def pending_items(session: str | None = None) -> list[dict]:
    added: dict[str, dict] = {}
    done: set[str] = set()
    for e in _events():
        if session and e.get("session") not in (None, session):
            continue
        if e.get("op") == "add":
            added[e["id"]] = e
        elif e.get("op") == "done":
            done.add(e["id"])
    return [e for rid, e in added.items() if rid not in done]


def _resolve_id(needle: str, session: str | None) -> str | None:
    items = pending_items(session)
    for e in items:
        if e["id"] == needle:
            return e["id"]
    matches = [e["id"] for e in items if needle.lower() in e.get("text", "").lower()]
    return matches[0] if len(matches) == 1 else None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("op", choices=["add", "done", "list", "pending"])
    ap.add_argument("arg", nargs="?", default="")
    ap.add_argument("--session", default=None)
    args = ap.parse_args()

    if args.op == "add":
        if not args.arg:
            sys.exit("add requires requirement text")
        rid = f"req-{int(time.time() * 1000) % 10**8}"
        _append({"op": "add", "id": rid, "text": args.arg, "session": args.session})
        print(rid)
        return 0

    if args.op == "done":
        rid = _resolve_id(args.arg, args.session)
        if not rid:
            sys.exit(f"no pending item matching {args.arg!r}")
        _append({"op": "done", "id": rid, "session": args.session})
        print(f"resolved {rid}")
        return 0

    items = pending_items(args.session)
    if args.op == "pending":
        print(len(items))
        for e in items:
            print(f"  {e['id']}: {e.get('text', '')}")
    else:
        for e in items:
            print(f"{e['id']} [{e.get('session') or '-'}] {e.get('text', '')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
