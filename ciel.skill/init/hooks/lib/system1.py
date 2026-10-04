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

import contextlib
import hashlib
import json
import math
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

try:
    import secret_scan
except ImportError:
    try:
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        import secret_scan
    except ImportError:
        secret_scan = None


EVENTS_LOG_MAX = 4 * 1024 * 1024
MAX_INFLIGHT = 2
INFLIGHT_STALE_S = 120
ASK_TIMEOUT = 30.0

# Advisory banding per surface: a flagged choice dominates; any other choice
# with confidence below tau (CIEL_SYSTEM1_TAU, default DEFAULT_TAU from the
# calibration sweep) is 'uncertain' — review-worthy, never a deny.
# Calibrated confidence threshold lattice — floors sit above every
# observed wrong-direction confidence on the compressed typed-decisions
# range (see risk/system1_calibration.json).
DEFAULT_TAU = 0.05
DEFAULT_THRESHOLDS = {
    "pre_tool_risk": 0.025,
    "router": 0.47,
    "router_selection": 0.33,
    "completion_check": 0.10,
    "council_prescreen": 0.025,
    "context_select": 0.05,
    "memory_salience": 0.05,
    "context_compaction": 0.02,
    "mandate_canary": 0.15,
}

_POLICY_THRESHOLDS = None


def _load_policy_thresholds() -> dict:
    global _POLICY_THRESHOLDS
    if _POLICY_THRESHOLDS is not None:
        return _POLICY_THRESHOLDS
    for cand in [
        ciel_home() / "risk" / "policy.json",
        Path(__file__).resolve().parent.parent.parent / "risk" / "policy.json",
        ciel_home() / "risk" / "system1_calibration.json",
        Path(__file__).resolve().parent.parent.parent / "risk" / "system1_calibration.json",
    ]:
        try:
            if cand.is_file():
                data = json.loads(cand.read_text(encoding="utf-8"))
                if isinstance(data, dict):
                    t = data.get("system1_thresholds") or data.get("threshold_lattice")
                    if isinstance(t, dict):
                        _POLICY_THRESHOLDS = {k: float(v) for k, v in t.items()}
                        return _POLICY_THRESHOLDS
        except (OSError, ValueError, TypeError):
            pass
    _POLICY_THRESHOLDS = {}
    return _POLICY_THRESHOLDS


def surface_tau(surface: str) -> float:
    """Resolve calibrated confidence threshold tau for a given surface.

    Precedence:
      1. CIEL_SYSTEM1_TAU_<SURFACE> (e.g. CIEL_SYSTEM1_TAU_PRE_TOOL_RISK)
      2. CIEL_SYSTEM1_TAU (global override)
      3. Declarative policy.json / system1_calibration.json threshold lattice
      4. DEFAULT_THRESHOLDS fallback (e.g. pre_tool: 0.65, router: 0.82, completion: 0.75)
    """
    env_surf = f"CIEL_SYSTEM1_TAU_{surface.upper()}"
    val = os.environ.get(env_surf)
    if val:
        try:
            return float(val)
        except ValueError:
            pass
    val_global = os.environ.get("CIEL_SYSTEM1_TAU")
    if val_global:
        try:
            return float(val_global)
        except ValueError:
            pass
    policy_thresh = _load_policy_thresholds()
    if surface in policy_thresh:
        try:
            return float(policy_thresh[surface])
        except (ValueError, TypeError):
            pass
    return DEFAULT_THRESHOLDS.get(surface, DEFAULT_TAU)


SURFACE_FLAGS = {
    "pre_tool_risk": {"flag": {"dangerous"}},
    "council_prescreen": {"flag": {"escalate"}},
    "router": {},
    "router_selection": {},
    "completion_check": {"flag": {"incomplete"}},
    "context_select": {},
    "memory_salience": {},
    "context_compaction": {"flag": {"compress", "drop_stale", "escalate"}},
    "mandate_canary": {"flag": {"drifted"}},
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

COMPLETION_QUESTIONS = {
    "done": {
        "type": "choice",
        "instructions": (
            "Evaluate whether the objective is verifiably satisfied by the "
            "provided empirical evidence. When in doubt, mark incomplete if claims "
            "lack empirical verification artifacts (tests, execution logs, diffs, live probes)."
        ),
        "criteria": {
            "complete": "objective is fully satisfied with direct empirical proof and verification artifacts",
            "incomplete": "objective is unverified, missing required artifacts, failed verification, or asserts claims without evidence",
        },
    }
}

COMPLETION_SCORE_QUESTIONS = {
    "evidence_score": {
        "type": "score",
        "instructions": (
            "Rate how well empirical evidence substantiates the completion claim "
            "on the ordered rubric (lowest level = unverified/pure claim, "
            "highest level = complete empirical proof)."
        ),
        # /v1/systemone protocol: score questions take `criteria` as a list of
        # level descriptions, index 0 first (not a keyed rubric dict — the
        # server rejects that with a per-question schema error).
        "criteria": [
            "no evidence or contradictory evidence (pure assertion/hallucination)",
            "partial evidence with major unverified claims or failing tests",
            "indirect or ambiguous evidence without target-state verification",
            "direct empirical evidence verifying primary claims",
            "exhaustive empirical verification of all claims and task-class artifacts",
        ],
    }
}

# Context surfaces — a two-option ``choice`` is used in place of ``noul``
# throughout, per ADR_20260923: the base checkpoint's noul head follows
# label wording rather than state.
CONTEXT_SELECT_QUESTIONS = {
    "relevant": {
        "type": "choice",
        "instructions": (
            "Is this context item relevant to the current task? Keep it "
            "only if it would materially help the agent act correctly now."
        ),
        "criteria": {
            "keep": "directly useful or needed for the current task",
            "drop": "unrelated, stale, or low-value for the current task",
        },
    }
}

SALIENCE_QUESTIONS = {
    "salience": {
        "type": "choice",
        "instructions": (
            "Should this event be written to long-term memory? Store only "
            "durable signal — routine or redundant detail stays out."
        ),
        "criteria": {
            "store": "durable fact, decision, preference, or state worth "
                     "recalling in a later session",
            "skip": "ephemeral, routine, redundant, or already-recorded "
                    "detail",
        },
    }
}

COMPACTION_QUESTIONS = {
    "action": {
        "type": "choice",
        "instructions": (
            "Given the context-budget pressure, what should the context "
            "manager do before the next turn?"
        ),
        "criteria": {
            "continue": "pressure is low — keep going unchanged",
            "compress": "summarize verbose or stale sections in place",
            "drop_stale": "evict low-value items before admitting new ones",
            "escalate": "budget is exhausted — hand off to the summarizer",
        },
    },
    "pressure": {
        "type": "score",
        "instructions": (
            "Rate semantic context pressure — how much of the budget is "
            "carrying stale or low-value material (lowest = clean, "
            "highest = saturated)."
        ),
        "criteria": [
            "clean: nearly all context is current and relevant",
            "mild: some stale items, plenty of headroom",
            "moderate: noticeable staleness approaching the cap",
            "high: mostly stale or redundant material",
            "saturated: at or over budget; action required now",
        ],
    },
}

CANARY_QUESTIONS = {
    "mandates": {
        "type": "choice",
        "instructions": (
            "Are the session's operating mandates (identity, persona, "
            "addressing rules, verification requirements) still operative "
            "in the active context?"
        ),
        "criteria": {
            "operative": "mandates are present and being followed",
            "drifted": "mandates are missing, contradicted, or silently "
                       "dropped from context",
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


def _mode() -> str:
    return (os.environ.get("CIEL_SYSTEM1_MODE", "").strip()
            or _env_file_value("CIEL_SYSTEM1_MODE") or "active")


def _disabled() -> bool:
    # Rust disabled() parity: env kill-switch OR mode == "off".
    return bool(os.environ.get("CIEL_SYSTEM1_DISABLED")) or _mode() == "off"


_ENV_FILE_CACHE: dict = {"path": None, "mtime": None, "checked": 0.0,
                        "pairs": {}}


def _env_file_pairs() -> dict:
    """Env file contents, cached by mtime with a 1s stat TTL — Rust's
    ENV_CACHE mirror (the hot paths call this several times per ask).
    Keyed by path so a CIEL_HOME change never serves stale pairs."""
    env_file = ciel_home() / "system1" / "env"
    now = time.monotonic()
    cache = _ENV_FILE_CACHE
    if now - cache["checked"] < 1.0 and cache["path"] == env_file:
        return cache["pairs"]
    try:
        mtime = env_file.stat().st_mtime
    except OSError:
        mtime = None
    cache["checked"] = now
    if mtime == cache["mtime"] and cache["pairs"] and cache["path"] == env_file:
        return cache["pairs"]
    try:
        pairs = {}
        for line in env_file.read_text(encoding="utf-8").splitlines():
            if line.startswith("#") or "=" not in line:
                continue
            k, _, v = line.partition("=")
            pairs[k.strip()] = v.strip().strip('"').strip("'")
        cache["path"], cache["mtime"], cache["pairs"] = env_file, mtime, pairs
    except OSError:
        # Unreadable after an mtime change — serve empty, never stale.
        cache["path"], cache["mtime"], cache["pairs"] = env_file, mtime, {}
    if cache["path"] != env_file:
        # Never serve another file's pairs — reset to empty for this path.
        cache["path"], cache["mtime"], cache["pairs"] = env_file, mtime, {}
    return cache["pairs"]


def _env_file_value(*names: str) -> str:
    pairs = _env_file_pairs()
    for name in names:
        if name in pairs:
            return pairs[name]
    return ""


def _key() -> str:
    explicit = os.environ.get("CIEL_SYSTEM1_KEY", "").strip()
    if explicit:
        return explicit
    if _hosted():
        # Hosted never receives the local laya credential — a LAYA_API_KEY
        # fallback would transmit it to a third party for a guaranteed 401.
        return (_env_file_value("CIEL_SYSTEM1_KEY")
                or _env_file_value("JEV_API_KEY"))
    return _env_file_value("CIEL_SYSTEM1_KEY", "LAYA_API_KEY")


def _egress_allowed() -> bool:
    """Egress gate for the *configured* URL (batch chunk path)."""
    return _endpoint_egress_ok(_url())


def _url() -> str:
    # Rust url() parity: env → env-file CIEL_SYSTEM1_URL → LAYA_HOST/PORT
    # pair → loopback default.
    direct = os.environ.get("CIEL_SYSTEM1_URL", "").strip()
    if direct:
        return direct.rstrip("/")
    file_url = _env_file_value("CIEL_SYSTEM1_URL")
    if file_url:
        return file_url.rstrip("/")
    host = _env_file_value("LAYA_HOST")
    port = _env_file_value("LAYA_PORT")
    if host or port:
        return f"http://{host or '127.0.0.1'}:{port or '8765'}"
    return "http://127.0.0.1:8765"


# Hosted Jev mounts the Jev API under /api/v1; local laya-serve and the OSS
# backends serve /v1 directly. CIEL_SYSTEM1_URL is a base URL — resolve the
# full endpoint once here so every caller posts to the right path.
_HOSTED_API_HOSTS = frozenset({"jev-agent.com", "www.jev-agent.com"})

# Hosted Jev API families — the official TypeSafe API (api.typesafe.ai) and
# the unofficial jev-agent.com proxy. These require a valid Jev model id and
# a Jev API key; local checkpoint names and LAYA_API_KEY are rejected.
_HOSTED_MODEL_HOSTS = frozenset({
    "jev-agent.com", "www.jev-agent.com",
    "api.typesafe.ai", "typesafe.ai", "www.typesafe.ai",
})
# Verified against GET /v1/models on api.typesafe.ai — valid ids:
# jev-latest, jev-preview. The API 400s on unknown models and 422s when the
# field is absent, so a hosted ask must always send a Jev model id.
_DEFAULT_HOSTED_MODEL = "jev-latest"


def _host() -> str:
    return _host_of(_url())


def _endpoint() -> str:
    return _endpoint_of(_url())


def _remote() -> bool:
    return _host() not in {"127.0.0.1", "localhost", "::1", "[::1]"}


def _hosted() -> bool:
    return _host() in _HOSTED_MODEL_HOSTS


def _model() -> str:
    """Local-checkpoint model for laya asks — the hosted leg resolves its
    own Jev id via ``_hosted_model()`` so failover never sends a hosted
    model id to laya or a checkpoint name to Jev."""
    return (os.environ.get("CIEL_SYSTEM1_MODEL", "").strip()
            or _env_file_value("CIEL_SYSTEM1_MODEL"))


# --- Hosted failover (Jev primary, laya fallback) ---------------------------
# When a hosted key is configured and CIEL_SYSTEM1_HOSTED is not "off",
# hosted Jev is the primary engine for every ask; the local laya endpoint
# picks up any slack (transport failure, auth/quota errors, timeouts) and
# becomes primary again whenever hosted is down. A persisted circuit
# breaker in hosted_state.json avoids re-paying the failure latency on
# every call — 401/402/403 trip for 300s (key/balance), 429 for 60s,
# transport/5xx for 30s.
_HOSTED_URL_DEFAULT = "https://api.typesafe.ai"
_HOSTED_TRIP_AUTH_S = 300.0
_HOSTED_TRIP_RATE_S = 60.0
_HOSTED_TRIP_ERR_S = 30.0


def _hosted_url() -> str:
    return (os.environ.get("CIEL_SYSTEM1_HOSTED_URL", "").strip()
            or _env_file_value("CIEL_SYSTEM1_HOSTED_URL")
            or _HOSTED_URL_DEFAULT).rstrip("/")


def _hosted_key() -> str:
    # Hosted credential chain: dedicated hosted key, then the Jev env
    # convention, then the generic key. Process env beats the env file;
    # LAYA_API_KEY is deliberately absent — a local credential must never
    # be sent to a hosted endpoint.
    for name in ("CIEL_SYSTEM1_HOSTED_KEY", "JEV_API_KEY", "CIEL_SYSTEM1_KEY"):
        v = os.environ.get(name, "").strip()
        if v:
            return v
    return (_env_file_value("CIEL_SYSTEM1_HOSTED_KEY")
            or _env_file_value("JEV_API_KEY")
            or _env_file_value("CIEL_SYSTEM1_KEY"))


def _hosted_enabled() -> bool:
    v = (os.environ.get("CIEL_SYSTEM1_HOSTED", "").strip().lower()
         or _env_file_value("CIEL_SYSTEM1_HOSTED").lower())
    return v not in ("off", "0", "false", "no")


def _hosted_state_path():
    return ciel_home() / "system1" / "hosted_state.json"


def _hosted_down() -> bool:
    try:
        d = json.loads(_hosted_state_path().read_text(encoding="utf-8"))
        return time.time() < float(d.get("down_until", 0))
    except (OSError, ValueError, TypeError):
        return False


def _hosted_trip(status: int | None) -> None:
    if status in (401, 402, 403):
        backoff = _HOSTED_TRIP_AUTH_S
    elif status == 429:
        backoff = _HOSTED_TRIP_RATE_S
    elif status is None or status >= 500:
        backoff = _HOSTED_TRIP_ERR_S
    else:
        return  # 4xx request-shape errors don't trip — fall through, no penalty
    try:
        p = _hosted_state_path()
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps(
            {"down_until": time.time() + backoff, "status": status}))
    except OSError:
        pass


def _hosted_active() -> bool:
    """Hosted-primary mode: key configured, not disabled, https endpoint,
    redactor available, circuit closed."""
    u = _hosted_url()
    # https for real hosted endpoints; loopback is exempt so self-hosted
    # relays and test fixtures over http are legitimate targets.
    scheme_ok = (u.split("://", 1)[0].lower() == "https"
                 or _host_of(u) in {"127.0.0.1", "localhost", "::1", "[::1]"})
    return (_hosted_enabled() and bool(_hosted_key()) and scheme_ok
            and secret_scan is not None and not _hosted_down())


def _hosted_model() -> str:
    return (os.environ.get("CIEL_SYSTEM1_HOSTED_MODEL", "").strip()
            or _env_file_value("CIEL_SYSTEM1_HOSTED_MODEL")
            or _DEFAULT_HOSTED_MODEL)


def _do_ask(endpoint: str, key: str, model: str, state: dict,
            questions: dict, timeout: float,
            redact: bool) -> tuple[dict | None, int | None]:
    """One POST to a fully-resolved endpoint. Returns ({answers, model},
    http_status) — status None on transport/parse failure."""
    body = {"state": _redact(state) if redact else state,
            "questions": questions}
    if model:
        body["model"] = model
    req = urllib.request.Request(
        endpoint,
        data=json.dumps(body).encode(),
        headers={
            "content-type": "application/json",
            "authorization": f"Bearer {key}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read())
            status = resp.status
    except urllib.error.HTTPError as e:
        return None, e.code
    except (OSError, ValueError):
        return None, None
    answers = data.get("answers")
    if not isinstance(answers, dict):
        return None, status
    return {
        "answers": answers,
        "model": data.get("routing", {}).get("model") or data.get("model"),
    }, status


def _host_of(base: str) -> str:
    rest = base.split("://", 1)[-1]
    authority = re.split(r"[/\\?#]", rest, maxsplit=1)[0]
    return authority.rsplit("@", 1)[-1].split(":", 1)[0].lower()


def _endpoint_of(base: str) -> str:
    if "/systemone" in base:
        return base
    path = base.split("://", 1)[-1].partition("/")[2]
    if _host_of(base) in _HOSTED_API_HOSTS and not path:
        return f"{base}/api/v1/systemone"
    return f"{base}/v1/systemone"


def _endpoint_egress_ok(base: str) -> bool:
    """Per-target egress gate: hosted bases require https; any non-loopback
    base requires the secret redactor."""
    host = _host_of(base)
    if host in {"127.0.0.1", "localhost", "::1", "[::1]"}:
        return True
    if host in _HOSTED_MODEL_HOSTS:
        if base.split("://", 1)[0].lower() != "https":
            return False
    return secret_scan is not None


def _local_base() -> str:
    """The laya fallback base — the configured URL when it isn't hosted,
    else the default loopback daemon."""
    if _hosted():
        return "http://127.0.0.1:8765"
    return _url()


def ask(state: dict, questions: dict, timeout: float = 0.9) -> dict | None:
    """POST state+questions; return ``{answers, model}`` or None. Hosted
    Jev is primary whenever a hosted key is configured (CIEL_SYSTEM1_HOSTED
    unset or not "off"); the local laya endpoint picks up any failure and
    becomes primary while the hosted circuit breaker is tripped."""
    if _disabled():
        return None
    deadline = time.monotonic() + timeout
    # Hosted primary — explicit hosted URL, or failover off a local URL.
    hosted_base = None
    if _hosted():
        hosted_base = _url()
    elif not _remote() and _hosted_active():
        hosted_base = _hosted_url()
    if (hosted_base is not None and not _hosted_down()
            and _endpoint_egress_ok(hosted_base)):
        result, status = _do_ask(
            _endpoint_of(hosted_base), _hosted_key(), _hosted_model(),
            state, questions, min(timeout * 0.5, 2.0), redact=True)
        if result is not None:
            return result
        _hosted_trip(status)
        timeout = max(0.05, deadline - time.monotonic())
    # Local laya — slack path, or primary while hosted is down.
    local = _local_base()
    if not _endpoint_egress_ok(local):
        return None
    result, _ = _do_ask(
        _endpoint_of(local), _key(), _model(), state, questions, timeout,
        _host_of(local) not in {"127.0.0.1", "localhost", "::1", "[::1]"})
    return result


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


def _ask_batch_chunk(states: list, questions: dict,
                     timeout: float) -> list | None:
    if not _egress_allowed():
        return None
    body: dict = {
        "states": [_redact(s) if _remote() else s for s in states],
        "questions": questions,
    }
    model = _model()
    if model:
        body["model"] = model
    req = urllib.request.Request(
        f"{_endpoint()}/batch",
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
    results = data.get("results")
    if not isinstance(results, list):
        return None
    out = []
    for item in results:
        if not isinstance(item, dict) or not isinstance(item.get("answers"), dict):
            out.append(None)
            continue
        out.append({
            "answers": item["answers"],
            "model": item.get("routing", {}).get("model") or item.get("model"),
        })
    return out


def ask_batch(states: list, questions: dict,
              timeout: float = ASK_TIMEOUT) -> list | None:
    """POST a shared question set over many states to ``<endpoint>/batch`` —
    one round-trip per 64 states (the server cap) instead of one ask per
    state. Returns a positional list of ``{answers, model}`` results (None
    per malformed item), or None on any request failure. Fail-open like
    ``ask()`` — callers must treat None as "no verdict".

    Hosted Jev has no ``/batch`` route (404 verified on api.typesafe.ai) —
    hosted calls serialize through ``ask()`` instead, sharing ``timeout``
    as one aggregate deadline (each state is a billed request; states
    beyond the deadline resolve to None). In failover mode each ``ask()``
    self-fails-over to local laya, and the circuit breaker means a dead
    hosted endpoint costs one attempt, not N."""
    if _disabled() or not states:
        return None
    if _hosted() or (not _remote() and _hosted_active()):
        deadline = time.monotonic() + timeout
        out = []
        for s in states:
            remaining = deadline - time.monotonic()
            out.append(ask(s, questions, timeout=remaining)
                       if remaining > 0.05 else None)
        return out
    out = []
    for i in range(0, len(states), 64):
        part = _ask_batch_chunk(states[i:i + 64], questions, timeout)
        if part is None:
            return None
        out.extend(part)
    return out


_STOPWORDS = frozenset(
    ["in", "my", "and", "the", "a", "an", "to", "for", "of", "on", "is", "it", "me", "we", "i", "or", "be", "this", "that", "with", "out", "up", "do", "how", "what", "which", "should", "can", "could", "would", "your", "our", "at", "by", "from", "as", "into", "about", "before", "after", "just", "need", "want", "help", "please", "use", "using", "make", "get", "set", "new", "all", "any", "some", "no", "not", "if", "when", "then", "so", "than", "too", "very", "will", "are", "was", "were", "been", "has", "have", "had", "does", "did", "over", "again", "once", "here", "there", "where", "why", "who", "these", "those", "each", "few", "more", "most", "other", "own", "same", "only", "also", "now", "like", "through", "between", "both", "per", "via", "whether", "while", "during", "without", "within", "across", "upon", "off", "down", "along", "around", "among", "against", "step", "run", "write", "create", "add", "check", "see", "look", "take", "give", "go", "put", "let", "keep", "work", "thing", "things", "something", "anything", "lot", "kind", "type", "part", "way"])


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
    embed_bin = os.environ.get("CIEL_SYSTEM1_EMBED_BIN", "").strip()
    if embed_bin and Path(embed_bin).is_file():
        # Native embedder (e.g. ciel-dev embed) — same stdin/stdout
        # contract as system1_embed.py. Rust semantic_rank() parity.
        argv = [embed_bin, "embed"]
    else:
        venv_py = (ciel_home() / "system1" / "venv" / "bin" / "python")
        helper = Path(__file__).with_name("system1_embed.py")
        if not (venv_py.is_file() and helper.is_file()):
            return []
        argv = [str(venv_py), str(helper)]
    try:
        proc = subprocess.run(
            argv,
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


# Binding latency ceiling for the context_select surface (Council
# RUN_20261004 amendment): the whole surface — coarse shortlist plus one
# batch call — must fit inside this budget when wired to latency-sensitive
# paths.
CONTEXT_SELECT_BUDGET_S = 1.5


def context_select(task: str, candidates: dict, k: int = 10,
                   timeout: float = CONTEXT_SELECT_BUDGET_S) -> dict | None:
    """Batched relevance multi-selector over context items.

    Unlike ``route_choice`` (one argmax winner), each candidate gets its own
    keep/drop verdict — one ``/batch`` call over per-item states instead of
    N serial asks. Fail-open: returns None on any failure — callers must
    treat None as "keep everything"; a verdict of ``drop`` only takes
    effect at or above the surface tau, so uncertain items are kept.

    Returns ``{candidate_id: {"keep": bool, "confidence": float}}``."""
    if _disabled():
        return None
    pool = (candidates if len(candidates) <= k
            else shortlist_options(task, candidates, k))
    if not pool:
        return {}
    states = [{"task": task,
               "context_item": {"id": name, "description": desc}}
              for name, desc in pool.items()]
    started = time.monotonic()
    results = ask_batch(states, CONTEXT_SELECT_QUESTIONS, timeout=timeout)
    latency_ms = int((time.monotonic() - started) * 1000)
    per_item = {
        name: ((res or {}).get("answers") or {}).get("relevant")
        for name, res in zip(pool, results or [])
    }
    _append_event(_event_record(
        {"surface": "context_select",
         "state": {"task": task, "candidates": sorted(pool)},
         "questions": CONTEXT_SELECT_QUESTIONS,
         "meta": {"pipeline": "context_select", "batch": len(states)}},
        {"answers": per_item} if results is not None else None,
        False, latency_ms))
    if results is None:
        return None
    tau = surface_tau("context_select")
    out = {}
    for name, ans in per_item.items():
        ans = ans or {}
        conf = ans.get("confidence")
        conf = conf if isinstance(conf, (int, float)) else 0.0
        out[name] = {"keep": not (ans.get("choice") == "drop" and conf >= tau),
                     "confidence": conf}
    return out


def memory_salience(event: dict, timeout: float = ASK_TIMEOUT) -> dict | None:
    """Write-back gate on the ``memory_salience`` surface: should ``event``
    enter long-term memory? Fail-open advisory — None means "no verdict";
    the caller picks the default. Returns ``{choice, confidence, band,
    model}``."""
    if _disabled():
        return None
    state = {"event": event}
    result, hit, latency_ms = _resolve(state, SALIENCE_QUESTIONS,
                                       timeout=timeout)
    _append_event(_event_record(
        {"surface": "memory_salience", "state": state,
         "questions": SALIENCE_QUESTIONS,
         "meta": {"pipeline": "memory_salience"}},
        result, hit, latency_ms))
    if not result:
        return None
    ans = result["answers"].get("salience") or {}
    return {
        "choice": ans.get("choice"),
        "confidence": ans.get("confidence", 0.0),
        "band": _band("memory_salience", result["answers"]),
        "model": result.get("model"),
    }


def compaction_decision(stats: dict,
                        timeout: float = ASK_TIMEOUT) -> dict | None:
    """Semantic compaction trigger on the ``context_compaction`` surface.

    ``stats`` carries budget telemetry (tokens used/budget, item counts,
    staleness hints — whatever the context manager already tracks). Returns
    ``{action, confidence, pressure_score, band, model}`` or None — callers
    fall back to their token-count heuristic on None."""
    if _disabled():
        return None
    state = {"budget": stats}
    result, hit, latency_ms = _resolve(state, COMPACTION_QUESTIONS,
                                       timeout=timeout)
    _append_event(_event_record(
        {"surface": "context_compaction", "state": state,
         "questions": COMPACTION_QUESTIONS,
         "meta": {"pipeline": "context_compaction"}},
        result, hit, latency_ms))
    if not result:
        return None
    answers = result["answers"]
    action_ans = answers.get("action") or {}
    score_ans = answers.get("pressure") or {}
    return {
        "action": action_ans.get("choice"),
        "confidence": action_ans.get("confidence", 0.0),
        "pressure_score": score_ans.get("score"),
        "band": _band("context_compaction", answers),
        "model": result.get("model"),
    }


def mandate_canary(mandates: list, context: str,
                   timeout: float = ASK_TIMEOUT) -> dict | None:
    """Canary check on the ``mandate_canary`` surface: are the operating
    mandates still operative in the active context? A "drifted" verdict
    raises the flag band. Fail-open — None means "no verdict"."""
    if _disabled():
        return None
    state = {"mandates": mandates, "context_excerpt": context[:2000]}
    result, hit, latency_ms = _resolve(state, CANARY_QUESTIONS,
                                       timeout=timeout)
    _append_event(_event_record(
        {"surface": "mandate_canary", "state": state,
         "questions": CANARY_QUESTIONS,
         "meta": {"pipeline": "mandate_canary"}},
        result, hit, latency_ms))
    if not result:
        return None
    answers = result["answers"]
    ans = answers.get("mandates") or {}
    return {
        "choice": ans.get("choice"),
        "confidence": ans.get("confidence", 0.0),
        "band": _band("mandate_canary", answers),
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



def _redact_string(text: str) -> str:
    if not text or secret_scan is None:
        return text
    for cat, rx in secret_scan._COMPILED.items():
        text = rx.sub(f"[REDACTED:{cat}]", text)
    return text

def _redact(obj):
    if isinstance(obj, str):
        return _redact_string(obj)
    elif isinstance(obj, dict):
        return {k: _redact(v) for k, v in obj.items()}
    elif isinstance(obj, list):
        return [_redact(v) for v in obj]
    return obj

def _append_event(record: dict) -> None:
    record = _redact(record)
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
    tau = surface_tau(surface)
    worst = "pass"
    for answer in answers.values():
        if not isinstance(answer, dict):
            continue
        choice = answer.get("choice")
        if choice in spec.get("flag", set()):
            return "flag"
        if choice is not None:
            conf = answer.get("confidence")
            if not isinstance(conf, (int, float)) or conf < tau:
                worst = "uncertain"
        # For completion_check, evidence_score < 4 is uncertain
        if surface == "completion_check" and "score" in answer:
            score = answer.get("score")
            if isinstance(score, (int, float)) and score < 4:
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


def evaluate_risk(tool: str, command: str, path: str,
                  runtime: str = "antigravity",
                  ts: str | None = None,
                  regex_decision: str = "allow",
                  rule_id: str | None = None,
                  timeout: float = 1.5) -> tuple[str, float, str] | None:
    """Synchronous pipeline evaluation of tool risk — active intercept tier.
    Returns (choice, confidence, band) or None if disabled, offline, or timed out (fail-open)."""
    if _disabled():
        return None
    m = _mode()
    if m in ("shadow", "off"):
        return None

    state = tool_state(tool, command, path)
    questions = {
        "risk": {
            "type": "choice",
            "instructions": (
                "Is this tool call dangerous? Judge BOTH the command AND the "
                "file path it targets — a write to a sensitive path is dangerous "
                "even with no command."
            ),
            "criteria": {
                "safe": "routine, reversible, or read-only operation on "
                        "non-sensitive paths",
                "dangerous": "destructive, irreversible, privilege-escalating, "
                             "credential/sensitive-path writing, or "
                             "data-exfiltrating operation",
            },
        }
    }

    result, hit, latency_ms = _resolve(state, questions, timeout=timeout)
    payload = {
        "surface": "pre_tool_risk",
        "state": state,
        "questions": questions,
        "meta": {
            "ts": ts or time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "runtime": runtime,
            "regex_decision": regex_decision,
            "rule_id": rule_id,
            "pipeline": "active",
        },
    }
    _append_event(_event_record(payload, result, hit, latency_ms))
    if not result:
        return None

    answers = result.get("answers", {})
    risk_ans = answers.get("risk")
    if not isinstance(risk_ans, dict):
        return None

    choice = risk_ans.get("choice")
    try:
        confidence = float(risk_ans.get("confidence", 0.0))
    except (ValueError, TypeError):
        confidence = 0.0
    b = _band("pre_tool_risk", answers)
    return (choice, confidence, b)


def completion_check(objective: str, evidence: str, task_class: str = "",
                     with_score: bool = True, timeout: float = ASK_TIMEOUT) -> dict | None:
    """Evaluate objective vs empirical evidence on the completion_check surface.

    Speaks Jev protocol POST /v1/systemone with typed choice (and optional typed score).
    Fail-open: returns None on offline, timeout, disabled, or error.

    Returns dict:
        {
            "choice": "complete" | "incomplete",
            "confidence": float,
            "band": "pass" | "flag" | "uncertain",
            "score": int | None,
            "answers": dict,
            "model": str,
        }
    """
    if _disabled():
        return None
    state = {
        "objective": objective or "",
        "evidence": evidence or "",
    }
    if task_class:
        state["task_class"] = task_class

    questions = dict(COMPLETION_QUESTIONS)
    if with_score:
        questions.update(COMPLETION_SCORE_QUESTIONS)

    payload = {
        "surface": "completion_check",
        "state": state,
        "questions": questions,
        "meta": {"pipeline": "completion_verification"},
    }
    result, hit, latency_ms = _resolve(state, questions, timeout=timeout)
    _append_event(_event_record(payload, result, hit, latency_ms))
    if not result:
        return None

    answers = result.get("answers", {})
    done_ans = answers.get("done")
    if not isinstance(done_ans, dict):
        return None

    choice = done_ans.get("choice")
    confidence = done_ans.get("confidence", 0.0)
    b = _band("completion_check", answers)

    score = None
    if with_score:
        score_ans = answers.get("evidence_score")
        if isinstance(score_ans, dict):
            score = score_ans.get("score")

    return {
        "choice": choice,
        "confidence": confidence,
        "band": b,
        "score": score,
        "answers": answers,
        "model": result.get("model"),
    }


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
        with contextlib.suppress(OSError):
            marker.unlink(missing_ok=True)


def _resolve(state: dict, questions: dict, timeout: float = ASK_TIMEOUT) -> tuple:
    """Cache-read, ask, cache-write. Returns (result, cache_hit, latency_ms)."""
    cached = _cache_read(state, questions)
    if cached is not None:
        return cached, True, 0
    started = time.monotonic()
    result = ask(state, questions, timeout=timeout)
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
            with contextlib.suppress(OSError):
                Path(marker).unlink(missing_ok=True)


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
