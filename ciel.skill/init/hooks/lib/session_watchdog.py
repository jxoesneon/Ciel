#!/usr/bin/env python3
"""Session watchdog — stall detection, resume hints, transcript secret sweep.

Docket council-20260923-conversation-audit, mechanisms M6 + M3c.

M6 (self-resume): hooks cannot restart a stalled host runtime, so the named
resume trigger is (a) SessionStart consuming watchdog hints — the next
session sees what a dead session left unfinished — and (b) an opt-in
headless resume via ``devin -c -p`` driven by the ciel-watchdog systemd
timer (see init/systemd/). Auto-resume is hard-capped: <=1 attempt per
session, <=3 per day, >=30min between attempts, notify-send on fire.

M3c (redact-at-write): no hook owns transcript writes, so this runs an
incremental post-write scan — files changed since the last sweep are
checked for secret patterns; hits are reported by file + category, never
content. In-place redaction lives in scripts/transcript_sanitize.py.

CLI:
    session_watchdog.py --check            # hints + incremental scan (session_start)
    session_watchdog.py --resume [--dry]   # fire headless resume if warranted+capped
"""

import json
import os
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

HOME = Path.home()
CIEL = HOME / ".ciel"
CKPT = CIEL / "checkpoints"
STATE = CKPT / "watchdog_state.json"
HINT = CKPT / "resume_hint.json"
ACTIVITY = CIEL / "activity.log"
TRANSCRIPTS = HOME / ".local" / "share" / "devin" / "cli" / "transcripts"
SUMMARIES = HOME / ".local" / "share" / "devin" / "cli" / "summaries"

MAX_RESUME_PER_SESSION = 1
MAX_RESUME_PER_DAY = 3
MIN_RESUME_INTERVAL = 1800  # 30 min
STALL_AGE_S = 600           # session considered dead after 10 min idle
SCAN_MAX_BYTES = 8 * 1024 * 1024

# API-shaped error signatures only — keyword matching FPs hard on sessions
# that merely *discuss* rate limits (as this one did during the audit).
ERROR_TAIL = re.compile(
    r"rate_limit_error|\"status\"\s*:\s*429|HTTP 429|429 Too Many|"
    r"overloaded_error|insufficient_quota|\"error\"\s*:\s*\{[^}]{0,200}rate",
    re.IGNORECASE,
)


def _load(path: Path, default):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return default


def _save(path: Path, data) -> None:
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data, ensure_ascii=False, indent=1))
        try:
            os.chmod(path, 0o600)
        except OSError:
            pass
    except OSError:
        pass


def _emit_signal(name: str, payload: dict) -> None:
    signals = CIEL / "improvements" / "signals"
    try:
        signals.mkdir(parents=True, exist_ok=True)
        out = signals / f"{name}-{datetime.now(timezone.utc):%Y%m%dT%H%M%SZ}.json"
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n")
    except OSError:
        pass


# ---------------------------------------------------------------- stall find
def _recent_entries(n: int = 400) -> list[dict]:
    try:
        lines = ACTIVITY.read_text(encoding="utf-8").splitlines()[-n:]
    except OSError:
        return []
    out = []
    for line in lines:
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return out


def _session_last_seen() -> dict[str, float]:
    """session_id -> epoch of most recent activity entry."""
    seen: dict[str, float] = {}
    for e in _recent_entries():
        sid = e.get("session_id")
        ts = e.get("ts")
        if not sid or not ts:
            continue
        try:
            t = datetime.fromisoformat(ts.replace("Z", "+00:00")).timestamp()
        except ValueError:
            continue
        seen[sid] = max(seen.get(sid, 0), t)
    return seen


def _transcript_tail_errors(limit: int = 3) -> list[str]:
    """Transcript files whose tail shows error patterns — likely rate-limit stalls."""
    flagged = []
    try:
        files = sorted(TRANSCRIPTS.glob("*.json"), key=lambda p: p.stat().st_mtime)
    except OSError:
        return flagged
    for f in files[-5:]:
        try:
            if time.time() - f.stat().st_mtime > STALL_AGE_S * 6:
                continue
            text = f.read_text(encoding="utf-8", errors="replace")[-200_000:]
        except OSError:
            continue
        if ERROR_TAIL.search(text):
            flagged.append(f.stem)
            if len(flagged) >= limit:
                break
    return flagged


def pending_ledger_by_session() -> dict[str, int]:
    """Unresolved requirement-ledger items grouped by session."""
    import requirements
    counts: dict[str, int] = {}
    for e in requirements.pending_items(None):
        sid = e.get("session") or "unknown"
        counts[sid] = counts.get(sid, 0) + 1
    return counts


def find_stalled(current_session: str | None = None) -> dict:
    """Sessions with pending ledger items whose last activity is stale."""
    last_seen = _session_last_seen()
    pending = pending_ledger_by_session()
    now = time.time()
    stalled = {}
    for sid, count in pending.items():
        if sid == current_session or sid == "unknown":
            continue
        last = last_seen.get(sid, 0)
        if now - last > STALL_AGE_S:
            stalled[sid] = {"pending": count, "idle_s": int(now - last)}
    return {
        "stalled": stalled,
        "error_tails": _transcript_tail_errors(),
    }


# ------------------------------------------------------- transcript sweep (M3c)
def transcript_sweep(state: dict) -> dict:
    """Incremental secret scan over transcripts/summaries — changed files only."""
    import secret_scan
    prev = state.get("transcript_mtimes", {})
    hits = {}
    scanned = 0
    for base, pattern in ((TRANSCRIPTS, "*.json"), (SUMMARIES, "*.md")):
        if not base.is_dir():
            continue
        for f in base.glob(pattern):
            try:
                st = f.stat()
            except OSError:
                continue
            key = str(f)
            if prev.get(key) == st.st_mtime:
                continue
            if st.st_size > SCAN_MAX_BYTES:
                continue
            try:
                text = f.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            scanned += 1
            prev[key] = st.st_mtime
            r = secret_scan.scan(text)
            if r["hits"]:
                hits[f.name] = r["categories"]
            state.setdefault("transcript_hits", {})[key] = r["categories"] if r["hits"] else None
    state["transcript_mtimes"] = prev
    # prune cleared entries; files stay flagged until a re-scan clears them
    state["transcript_hits"] = {
        k: v for k, v in state.get("transcript_hits", {}).items() if v
    }
    return {"scanned": scanned, "hits": hits,
            "known_hits": len(state["transcript_hits"])}


# ------------------------------------------------------------------ resume
def resume_capable(state: dict, session_hint: str) -> tuple[bool, str]:
    now = time.time()
    attempts = state.get("resume_attempts", {})
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    if attempts.get("_day") != today:
        attempts = {"_day": today}
    if attempts.get(session_hint, 0) >= MAX_RESUME_PER_SESSION:
        return False, "session cap reached"
    if sum(v for k, v in attempts.items() if k != "_day") >= MAX_RESUME_PER_DAY:
        return False, "daily cap reached"
    last = state.get("last_resume", 0)
    if now - last < MIN_RESUME_INTERVAL:
        return False, "backoff window"
    return True, "ok"


def do_resume(session_hint: str, reason: str, dry: bool = False) -> dict:
    state = _load(STATE, {})
    ok, why = resume_capable(state, session_hint)
    if not ok:
        return {"fired": False, "reason": f"capped: {why}"}
    if not os.environ.get("CIEL_WATCHDOG_AUTORESUME"):
        return {"fired": False, "reason": "autoresume disabled (CIEL_WATCHDOG_AUTORESUME unset)"}
    prompt = (
        "A previous session stalled with unfinished work. Read "
        "~/.ciel/checkpoints/requirements.jsonl for pending items and "
        "resume them; verify progress before declaring completion."
    )
    if dry:
        return {"fired": False, "reason": "dry-run", "would_run": f"devin -c -p <resume prompt>"}
    try:
        proc = subprocess.run(
            ["devin", "-c", "-p", prompt],
            capture_output=True, text=True, timeout=1800,
        )
        fired = proc.returncode == 0
    except (OSError, subprocess.TimeoutExpired):
        fired = False
    attempts = state.get("resume_attempts", {})
    attempts[session_hint] = attempts.get(session_hint, 0) + 1
    state["resume_attempts"] = attempts
    state["last_resume"] = time.time()
    _save(STATE, state)
    _emit_signal("watchdog_resume", {
        "session": session_hint, "reason": reason, "fired": fired,
    })
    try:
        subprocess.run(
            ["notify-send", "Ciel watchdog", f"Auto-resumed stalled session ({reason})"],
            capture_output=True, timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        pass
    return {"fired": fired, "reason": reason}


# -------------------------------------------------------------------- main
def cmd_check(current_session: str | None = None) -> int:
    state = _load(STATE, {})
    stall = find_stalled(current_session)
    sweep = transcript_sweep(state)
    _save(STATE, state)

    hints: list[str] = []
    for sid, info in stall["stalled"].items():
        hints.append(
            f"session {sid[:8]}… ended with {info['pending']} unresolved "
            f"ledger item(s) (idle {info['idle_s'] // 60}m) — resume via `devin -c` "
            "or reconcile the ledger"
        )
    if stall["error_tails"]:
        hints.append(
            "recent transcript(s) end on rate-limit/error patterns: "
            + ", ".join(stall["error_tails"])
        )
    if sweep["hits"]:
        _emit_signal("transcript_secret_hits", {"files": sweep["hits"]})
    if sweep["known_hits"]:
        detail = ""
        if sweep["hits"]:
            detail = " — new: " + ", ".join(
                f"{n}({'+'.join(c)})" for n, c in list(sweep["hits"].items())[:5]
            )
        hints.append(
            f"{sweep['known_hits']} transcript-store file(s) carry secret "
            "patterns — sanitize via transcript_sanitize.py --redact and "
            f"rotate{detail}"
        )

    if hints:
        _save(HINT, {"ts": datetime.now(timezone.utc).isoformat(), "hints": hints})
        print("; ".join(hints))
    else:
        try:
            HINT.unlink(missing_ok=True)
        except OSError:
            pass
    return 0


def cmd_resume(dry: bool = False) -> int:
    stall = find_stalled()
    if not stall["stalled"] and not stall["error_tails"]:
        print(json.dumps({"fired": False, "reason": "nothing stalled"}))
        return 0
    hint = next(iter(stall["stalled"]), stall["error_tails"][0] if stall["error_tails"] else "unknown")
    reason = f"{len(stall['stalled'])} stalled session(s), {len(stall['error_tails'])} error-tail transcript(s)"
    print(json.dumps(do_resume(hint, reason, dry=dry)))
    return 0


def main() -> int:
    args = sys.argv[1:]
    if "--resume" in args:
        return cmd_resume(dry="--dry" in args)
    session = None
    if "--session" in args:
        i = args.index("--session")
        session = args[i + 1] if i + 1 < len(args) else None
    return cmd_check(session)


if __name__ == "__main__":
    sys.exit(main())
