#!/usr/bin/env python3
"""Shared System-1 decision client (Jev protocol) for Ciel surfaces.

Speaks ``POST {url}/v1/systemone``: typed ``choice``/``noul``/``score``
questions over a compact ``state``, answered with probabilities in a single
forward pass. Backends are swappable via ``CIEL_SYSTEM1_URL`` /
``CIEL_SYSTEM1_KEY`` — local laya-serve (default), hosted Jev, or AutoJev.

Every public function is fail-open: bounded timeout, ``None`` on any error,
never raises into the caller. ``CIEL_SYSTEM1_DISABLED=1`` is the global kill
switch.

``ask_async`` spawns a detached ``system1.py --ask`` subprocess (zero added
latency for callers; CPU inference can take seconds). The subprocess consults
the response cache, posts, and appends the verdict to
``~/.ciel/system1/events.jsonl``. In-flight shadow work is bounded by
MAX_INFLIGHT marker files under ``system1/inflight/`` — saturation drops the
request rather than queueing unboundedly.

CLI: ``--ask`` reads one JSON record from stdin
``{"surface", "state", "questions", "meta"}``, answers it, and appends the
result to events.jsonl.
"""

import hashlib
import json
import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

EVENTS_LOG_MAX = 4 * 1024 * 1024
MAX_INFLIGHT = 2
INFLIGHT_STALE_S = 120
ASK_TIMEOUT = 30.0

# Advisory banding per surface: a flagged choice dominates; any other choice
# with confidence below tau (CIEL_SYSTEM1_TAU, default DEFAULT_TAU from the
# calibration sweep) is 'uncertain' — review-worthy, never a deny.
DEFAULT_TAU = 0.2
SURFACE_FLAGS = {
    "pre_tool_risk": {"flag": {"dangerous"}},
    "council_prescreen": {"flag": {"escalate"}},
    "router": {},
}

# Calibrated wording: an explicit "when in doubt, escalate" instruction lifts
# escalate recall from 0.00 to 0.67 on the prescreen corpus — the neutral
# phrasing collapses to a 'routine' bias.
PRESCREEN_QUESTIONS = {
    "scope": {
        "type": "choice",
        "instructions": (
            "Does this event require full multi-lens deliberation? When in "
            "doubt, escalate — an unnecessary review costs little; a skipped "
            "review of a sensitive change is dangerous."
        ),
        "criteria": {
            "routine": "only clearly low-risk, reversible, well-precedented "
                       "actions",
            "escalate": "anything irreversible, security-relevant, "
                        "self-modifying, trust-changing, or novel-scope — "
                        "including when uncertain",
        },
    }
}


def ciel_home() -> Path:
    return Path(os.environ.get("CIEL_HOME") or Path.home() / ".ciel")


def _disabled() -> bool:
    return bool(os.environ.get("CIEL_SYSTEM1_DISABLED"))


def _key() -> str:
    key = os.environ.get("CIEL_SYSTEM1_KEY")
    if key:
        return key
    env_file = ciel_home() / "system1" / "env"
    try:
        for line in env_file.read_text(encoding="utf-8").splitlines():
            if line.startswith("LAYA_API_KEY="):
                return line.split("=", 1)[1].strip()
    except OSError:
        pass
    return ""


def _url() -> str:
    return os.environ.get("CIEL_SYSTEM1_URL", "http://127.0.0.1:8765").rstrip("/")


def ask(state: dict, questions: dict, timeout: float = 0.9) -> dict | None:
    """POST state+questions to the endpoint; return the ``answers`` dict plus
    model metadata, or None on any failure."""
    if _disabled():
        return None
    body = {"state": state, "questions": questions}
    req = urllib.request.Request(
        _url() + "/v1/systemone",
        data=json.dumps(body).encode(),
        headers={
            "content-type": "application/json",
            "authorization": f"Bearer {_key()}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read())
    except (OSError, ValueError):
        return None
    answers = data.get("answers")
    if not isinstance(answers, dict):
        return None
    return {
        "answers": answers,
        "model": data.get("routing", {}).get("model") or data.get("model"),
    }


def ask_choice(state: dict, key: str, instructions: str,
               options: dict, timeout: float = 0.9) -> dict | None:
    """One ``choice`` question. ``options`` maps option label -> criterion
    text. Returns {choice, confidence, probabilities, model} or None."""
    result = ask(
        state,
        {key: {"type": "choice", "instructions": instructions,
               "criteria": options}},
        timeout=timeout,
    )
    if not result:
        return None
    answer = result["answers"].get(key)
    if not isinstance(answer, dict):
        return None
    return {
        "choice": answer.get("choice"),
        "confidence": answer.get("confidence"),
        "probabilities": answer.get("probabilities"),
        "model": result.get("model"),
    }


def _cache_path(state: dict, questions: dict) -> Path:
    digest = hashlib.sha256(
        json.dumps({"s": state, "q": questions}, sort_keys=True).encode()
    ).hexdigest()
    return ciel_home() / "system1" / "cache" / f"{digest}.json"


def _cache_read(state: dict, questions: dict) -> dict | None:
    try:
        data = json.loads(_cache_path(state, questions).read_text())
    except (OSError, ValueError):
        return None
    return data if isinstance(data, dict) else None


def _cache_write(state: dict, questions: dict, result: dict) -> None:
    try:
        path = _cache_path(state, questions)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(result, ensure_ascii=False))
    except OSError:
        pass


def _append_event(record: dict) -> None:
    log = ciel_home() / "system1" / "events.jsonl"
    try:
        log.parent.mkdir(parents=True, exist_ok=True)
        if log.is_file() and log.stat().st_size > EVENTS_LOG_MAX:
            log.rename(log.with_suffix(".jsonl.1"))
        with log.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record, ensure_ascii=False) + "\n")
    except OSError:
        pass


def _band(surface: str, answers: dict) -> str:
    """Worst advisory band across answers: 'flag' > 'uncertain' > 'pass'."""
    spec = SURFACE_FLAGS.get(surface, {})
    try:
        tau = float(os.environ.get("CIEL_SYSTEM1_TAU") or DEFAULT_TAU)
    except ValueError:
        tau = DEFAULT_TAU
    worst = "pass"
    for answer in answers.values():
        if not isinstance(answer, dict):
            continue
        if answer.get("choice") in spec.get("flag", set()):
            return "flag"
        conf = answer.get("confidence")
        if not isinstance(conf, (int, float)) or conf < tau:
            worst = "uncertain"
    return worst


def council_prescreen(subject: str, meta: dict | None = None) -> None:
    """Detached shadow for the council_prescreen surface: is this event
    routine or does it need full deliberation? Advisory only."""
    ask_async({
        "surface": "council_prescreen",
        "state": {"event": subject},
        "questions": PRESCREEN_QUESTIONS,
        "meta": meta or {},
    })


def _inflight_dir() -> Path:
    return ciel_home() / "system1" / "inflight"


def _inflight_count() -> int:
    """Count live in-flight markers; reap stale ones (>INFLIGHT_STALE_S)."""
    d = _inflight_dir()
    now = time.time()
    count = 0
    try:
        for marker in d.iterdir():
            try:
                if now - marker.stat().st_mtime > INFLIGHT_STALE_S:
                    marker.unlink(missing_ok=True)
                else:
                    count += 1
            except OSError:
                continue
    except OSError:
        return 0
    return count


def ask_async(payload: dict) -> None:
    """Detached ask: spawn ``system1.py --ask`` and return immediately.
    payload: {"surface", "state", "questions", "meta"}. Saturation (>=
    MAX_INFLIGHT live markers) or any error drops the request silently."""
    if _disabled():
        return
    d = _inflight_dir()
    try:
        d.mkdir(parents=True, exist_ok=True)
        if _inflight_count() >= MAX_INFLIGHT:
            return
        marker = d / f"{os.getpid()}.{time.time_ns()}"
        marker.touch()
    except OSError:
        return
    env = dict(os.environ)
    env["CIEL_SYSTEM1_MARKER"] = str(marker)
    try:
        proc = subprocess.Popen(
            [sys.executable, os.path.abspath(__file__), "--ask"],
            stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
            env=env,
        )
        proc.stdin.write(json.dumps(payload).encode())
        proc.stdin.close()
    except OSError:
        try:
            marker.unlink(missing_ok=True)
        except OSError:
            pass


def _ask_main() -> int:
    marker = os.environ.get("CIEL_SYSTEM1_MARKER")
    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError:
        payload = {}
    try:
        if not payload:
            return 0
        state = payload.get("state") or {}
        questions = payload.get("questions") or {}
        cached = _cache_read(state, questions)
        if cached is not None:
            result, hit = cached, True
        else:
            started = time.monotonic()
            result = ask(state, questions, timeout=ASK_TIMEOUT)
            hit = False
            latency_ms = int((time.monotonic() - started) * 1000)
            if result is not None:
                _cache_write(state, questions, result)
        surface = payload.get("surface") or "unknown"
        record = {
            "ts": payload.get("meta", {}).get("ts") or time.strftime(
                "%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "surface": surface,
            "questions": questions,
            "meta": payload.get("meta") or {},
            "system1": result,
            "flag": (_band(surface, result["answers"])
                     if result is not None else "pass"),
            "cache_hit": hit,
        }
        if not hit:
            record["latency_ms"] = latency_ms
        _append_event(record)
        return 0
    finally:
        if marker:
            try:
                Path(marker).unlink(missing_ok=True)
            except OSError:
                pass


def main() -> int:
    if "--ask" in sys.argv:
        return _ask_main()
    print("usage: system1.py --ask  (reads JSON payload on stdin)",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
