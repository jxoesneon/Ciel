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
import math
import os
import re
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
    "router_registry": {},
    "completion_check": {"flag": {"incomplete"}},
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

_READ_ONLY_TOOLS = {
    "read", "grep", "find_file_by_name", "webfetch", "web_search",
    "get_output", "mcp_read_resource", "mcp_list_tools", "mcp_list_servers",
    "notebook_read", "skill", "read_subagent", "list_skills",
}
_WRITE_TOOLS = {"write", "edit", "notebook_edit"}

# Substrings that mark a path/command as touching a sensitive location. This
# is a state *feature* for the model, not a policy verdict — it deliberately
# overlaps the policy's protected set but carries no decision.
_SENSITIVE_MARKERS = (
    "/.ssh", "/.aws", "/.gnupg", "/.kube", "/.docker", "/.netrc",
    "/.npmrc", "/.pypirc", "/.ciel/hooks", "/.ciel/risk",
    "/.config/devin", "/etc/", "/usr/", "/bin/", "/sbin/", "/boot/",
    "/root/",
)


def tool_state(tool: str, command: str, path: str) -> dict:
    """Enriched state for the pre_tool_risk surface.

    Beyond the raw {tool, command, path}, adds the fields AutoJev's
    tool-guard contract shows the model uses: a plain-language action,
    reversibility, side effects, and a sensitive-path hint. All fields are
    derived deterministically from the inputs — the policy verdict is never
    included, so the model's answer stays an independent signal for
    calibration and RLCD export."""
    tool = tool or ""
    command = command or ""
    path = path or ""
    state = {"tool": tool, "command": command, "path": path}

    if tool in _READ_ONLY_TOOLS:
        state["action"] = f"read data via {tool}"
        state["reversibility"] = "read-only; no state change"
        state["side_effects"] = []
    elif tool in _WRITE_TOOLS or (path and not command):
        state["action"] = f"create or modify the file at {path or '(unknown path)'}"
        state["reversibility"] = ("reversible if the target is tracked by "
                                  "version control; destructive otherwise")
        state["side_effects"] = ["modifies the filesystem at the target path"]
    elif tool == "exec" or command:
        state["action"] = f"run shell command: {command[:200]}"
        state["reversibility"] = ("depends on the command; writes, deletes, "
                                  "and package/system changes may be "
                                  "irreversible")
        state["side_effects"] = [
            "runs a subprocess that may change files, network, or system state"
        ]
    else:
        state["action"] = f"invoke {tool or 'a tool'}"
        state["reversibility"] = "unknown"
        state["side_effects"] = []

    haystack = f"{path} {command}"
    if any(m in haystack for m in _SENSITIVE_MARKERS):
        state["targets_sensitive_path"] = True
        state["side_effects"].append(
            "touches a credential store, agent configuration, or protected "
            "system path")
    return state


def ciel_home() -> Path:
    return Path(os.environ.get("CIEL_HOME") or Path.home() / ".ciel")


def _disabled() -> bool:
    return bool(os.environ.get("CIEL_SYSTEM1_DISABLED"))


def _env_file_value(*names: str) -> str:
    env_file = ciel_home() / "system1" / "env"
    try:
        for line in env_file.read_text(encoding="utf-8").splitlines():
            for name in names:
                if line.startswith(name + "="):
                    return line.split("=", 1)[1].strip()
    except OSError:
        pass
    return ""


def _key() -> str:
    return (os.environ.get("CIEL_SYSTEM1_KEY")
            or _env_file_value("CIEL_SYSTEM1_KEY", "LAYA_API_KEY"))


def _url() -> str:
    return os.environ.get("CIEL_SYSTEM1_URL", "http://127.0.0.1:8765").rstrip("/")


def _model() -> str:
    return (os.environ.get("CIEL_SYSTEM1_MODEL", "").strip()
            or _env_file_value("CIEL_SYSTEM1_MODEL"))


def ask(state: dict, questions: dict, timeout: float = 0.9) -> dict | None:
    """POST state+questions to the endpoint; return the ``answers`` dict plus
    model metadata, or None on any failure."""
    if _disabled():
        return None
    body = {"state": state, "questions": questions}
    model = _model()
    if model:
        body["model"] = model
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


_STOPWORDS = frozenset(
    "in my and the a an to for of on is it me we i or be this that with out "
    "up do how what which should can could would your our at by from as "
    "into about before after just need want help please use using make get "
    "set new all any some no not if when then so than too very will are was "
    "were been has have had does did over again once here there where why "
    "who these those each few more most other own same only also now like "
    "through between both per via whether while during without within "
    "across upon off down along around among against step run write create "
    "add check see look take give go put let keep work thing things "
    "something anything lot kind type part way".split())


def _tokens(text: str) -> set:
    out = set()
    for tok in re.findall(r"[a-z0-9]+", text.lower()):
        if tok in _STOPWORDS:
            continue
        out.add(tok)
        if len(tok) > 3 and tok.endswith("s") and not tok.endswith("ss"):
            out.add(tok[:-1])
    return out


def _lexical_rank(text: str, options: dict) -> list:
    """Candidates ranked by IDF-weighted token overlap (name + criterion)."""
    query = _tokens(text)
    docs = {name: _tokens(f"{name} {crit or ''}")
            for name, crit in options.items()}
    df: dict = {}
    for toks in docs.values():
        for tok in toks:
            df[tok] = df.get(tok, 0) + 1
    n = len(docs) or 1
    return sorted(
        ((sum(math.log(n / (1 + df[t])) + 1.0 for t in query & toks), name)
         for name, toks in docs.items()),
        key=lambda item: (-item[0], item[1]),
    )


def _semantic_rank(text: str, options: dict, k: int) -> list:
    """Bi-encoder shortlist via the laya-venv helper subprocess
    (``system1_embed.py`` beside this file). Returns ranked names or []
    when the venv/model is unavailable — callers fall back to lexical."""
    if os.environ.get("CIEL_SYSTEM1_EMBED") == "0":
        return []
    venv_py = (ciel_home() / "system1" / "venv" / "bin" / "python")
    helper = Path(__file__).with_name("system1_embed.py")
    if not (venv_py.is_file() and helper.is_file()):
        return []
    try:
        proc = subprocess.run(
            [str(venv_py), str(helper)],
            input=json.dumps({"task": text,
                              "candidates": options, "k": k}),
            capture_output=True, text=True, timeout=60, check=False)
        names = json.loads(proc.stdout or "{}").get("names")
        return names if isinstance(names, list) else []
    except (OSError, ValueError):
        return []


def shortlist_options(text: str, options: dict, k: int = 10) -> dict:
    """Coarse-to-fine candidate reduction for high-cardinality choice
    questions — the documented pattern once options exceed ~20.

    Hybrid scorer: semantic top-k from a bi-encoder (when the laya venv is
    present) ∪ lexical top-5 ∪ exact skill-name matches, then capped at
    k. Measured on the 22-case corpus against the 200-skill registry:
    semantic+lexical recall 16/22 vs 12/22 lexical-only vs 4/22 from the
    decision checkpoint's own encoder. ``CIEL_SYSTEM1_EMBED=0`` forces
    lexical."""
    if len(options) <= k:
        return dict(options)
    keep: list = []
    for name in _semantic_rank(text, options, k):
        if name in options and name not in keep:
            keep.append(name)
    for _, name in _lexical_rank(text, options)[:5]:
        if name not in keep:
            keep.append(name)
    task_tokens = set(re.findall(r"[a-z0-9]+", text.lower()))
    for name in options:
        if (set(re.findall(r"[a-z0-9]+", name.lower())) & task_tokens
                and name not in keep):
            keep.append(name)
    return {name: options[name] for name in keep[:k]}


def route_choice(task: str, options: dict, k: int = 10,
                 timeout: float = 0.9) -> dict | None:
    """Route ``task`` to one of ``options`` (id -> description). Shortlists
    to top-k when the candidate set is large, then asks one choice."""
    candidates = (options if len(options) <= k
                  else shortlist_options(task, options, k))
    return ask_choice(
        {"task": task, "candidates": sorted(candidates)},
        "route",
        "Which candidate best fits the task?",
        candidates,
        timeout=timeout,
    )


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


def _resolve(state: dict, questions: dict) -> tuple:
    """Cache-read, ask, cache-write. Returns (result, cache_hit, latency_ms)."""
    cached = _cache_read(state, questions)
    if cached is not None:
        return cached, True, 0
    started = time.monotonic()
    result = ask(state, questions, timeout=ASK_TIMEOUT)
    latency_ms = int((time.monotonic() - started) * 1000)
    if result is not None:
        _cache_write(state, questions, result)
    return result, False, latency_ms


def _event_record(payload: dict, result: dict | None, hit: bool,
                  latency_ms: int) -> dict:
    surface = payload.get("surface") or "unknown"
    record = {
        "ts": payload.get("meta", {}).get("ts") or time.strftime(
            "%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "surface": surface,
        "questions": payload.get("questions") or {},
        "state": payload.get("state") or {},
        "meta": payload.get("meta") or {},
        "system1": result,
        "flag": (_band(surface, result["answers"])
                 if result is not None else "pass"),
        "cache_hit": hit,
    }
    if not hit:
        record["latency_ms"] = latency_ms
    return record


def _ask_main() -> int:
    marker = os.environ.get("CIEL_SYSTEM1_MARKER")
    try:
        try:
            payload = json.loads(sys.stdin.read() or "{}")
        except json.JSONDecodeError:
            payload = {}
        if not payload:
            return 0
        result, hit, latency_ms = _resolve(payload.get("state") or {},
                                           payload.get("questions") or {})
        _append_event(_event_record(payload, result, hit, latency_ms))
        return 0
    finally:
        if marker:
            try:
                Path(marker).unlink(missing_ok=True)
            except OSError:
                pass


def _decide_main() -> int:
    """Synchronous on-demand ask for interactive/agent use: read the same
    JSON payload as --ask, append the event, and print the verdict to
    stdout. Exit 0 with 'null' printed when the endpoint is unreachable."""
    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError:
        payload = {}
    if not payload:
        print("null")
        return 0
    result, hit, latency_ms = _resolve(payload.get("state") or {},
                                       payload.get("questions") or {})
    _append_event(_event_record(payload, result, hit, latency_ms))
    print(json.dumps(result, ensure_ascii=False))
    return 0


def main() -> int:
    if "--ask" in sys.argv:
        return _ask_main()
    if "--decide" in sys.argv:
        return _decide_main()
    print("usage: system1.py --ask | --decide  (reads JSON payload on stdin; "
          "--decide prints the verdict)", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
