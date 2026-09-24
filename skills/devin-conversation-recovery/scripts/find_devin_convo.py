#!/usr/bin/env python3
"""Find and extract Devin (ACP) conversations from local Windsurf/Devin storage.

Usage:
  python find_devin_convo.py "search text"          # list sessions matching text
  python find_devin_convo.py --title "blender"      # match titles only
  python find_devin_convo.py --list                 # list all sessions by recency
  python find_devin_convo.py --extract <uuid> -o /tmp/out.txt   # dump full transcript

Search is two-phase: titles live in globalStorage/state.vscdb, message bodies
live in User/acp-messages/<uuid>.db. Phase 1 greps titles + first messages;
--deep also greps every message payload (slower).
"""
import sqlite3, json, os, sys, glob, argparse

BASE = os.path.expanduser("~/Library/Application Support/Devin")
GLOBAL = os.path.join(BASE, "User/globalStorage/state.vscdb")
MSGS = os.path.join(BASE, "User/acp-messages")


def list_sessions():
    con = sqlite3.connect(GLOBAL)
    rows = con.execute(
        "select key,value from ItemTable where key like 'windsurf.acp.sessioninfo.session.%'"
    ).fetchall()
    idx = json.loads(con.execute(
        "select value from ItemTable where key='windsurf.acp.messageStore.index'"
    ).fetchone()[0])
    out = []
    for k, v in rows:
        try:
            info = json.loads(v)["info"]
        except Exception:
            continue
        sid = info["sessionId"].split("/")[-1]
        uuid = idx.get(info["sessionId"], {}).get("uuid", "")
        out.append({
            "session": sid,
            "uuid": uuid,
            "title": info.get("title", ""),
            "updated": info.get("updatedAt", ""),
            "db": os.path.join(MSGS, uuid + ".db") if uuid else "",
        })
    out.sort(key=lambda s: s["updated"], reverse=True)
    return out


def matches(sess, needle, deep=False):
    n = needle.lower()
    if n in sess["title"].lower():
        return True
    if not sess["db"] or not os.path.exists(sess["db"]):
        return False
    con = sqlite3.connect(sess["db"])
    if deep:
        hit = False
        for (payload,) in con.execute("select payload from messages"):
            if n in payload.lower():
                hit = True
                break
        con.close()
        return hit
    # cheap phase: first 5 payloads only
    for (payload,) in con.execute("select payload from messages order by position limit 5"):
        if n in payload.lower():
            con.close()
            return True
    con.close()
    return False


def extract(uuid):
    db = os.path.join(MSGS, uuid + ".db")
    con = sqlite3.connect(db)
    rows = con.execute("select position,kind,payload from messages order by position").fetchall()

    def text_of(payload):
        d = json.loads(payload)
        texts, lines = [], []
        c = d.get("content")
        items = c if isinstance(c, list) else [c]
        for item in items:
            if not isinstance(item, dict):
                continue
            inner = item.get("content")
            if isinstance(inner, dict) and isinstance(inner.get("text"), str):
                texts.append(inner["text"])
            elif isinstance(inner, str):
                lines.append(inner)
            if "rawInput" in item:
                ri = item["rawInput"]
                cmd = ri.get("command") or ri.get("file_path") or json.dumps(ri)[:200]
                meta = item.get("_meta", {})
                te = meta.get("terminal_exit") or {}
                ec = te.get("exit_code") if isinstance(te, dict) else None
                line = f"$ {cmd}" + (f"  (exit {ec})" if ec is not None else "")
                if isinstance(item.get("output"), str):
                    line += "\nOUT: " + item["output"][:400]
                lines.append(line)
        # consecutive streaming text chunks are exact substrings of one message:
        # join with NO separator (chunk boundaries fall mid-word/mid-token)
        merged = "".join(t for t in texts if t)
        return ("\n".join([merged] + [l for l in lines if l.strip()])).strip()

    out = []
    for p, k, payload in rows:
        t = text_of(payload).strip()
        if t:
            out.append(f"=== [{p}] {k}\n{t}")
    return "\n\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("needle", nargs="?", help="text to search for")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--deep", action="store_true", help="grep ALL message payloads (slow)")
    ap.add_argument("--extract", metavar="UUID")
    ap.add_argument("-o", "--out", default=None)
    args = ap.parse_args()

    if args.extract:
        txt = extract(args.extract)
        if args.out:
            open(args.out, "w").write(txt)
            print(f"wrote {len(txt)} chars -> {args.out}")
        else:
            print(txt[:20000])
        return

    sessions = list_sessions()
    if args.list or not args.needle:
        for s in sessions:
            print(f'{s["updated"][:10]}  {s["session"]:22s} {s["uuid"][:8] or "-":9s} {s["title"][:90]}')
        return

    hits = [s for s in sessions if matches(s, args.needle, deep=args.deep)]
    for s in hits:
        print(f'{s["updated"][:10]}  {s["session"]:22s} uuid={s["uuid"]}  {s["title"][:80]}')
    if not hits:
        print("no title/early-message hits; retry with --deep", file=sys.stderr)


if __name__ == "__main__":
    main()
