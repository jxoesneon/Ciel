#!/usr/bin/env python3
"""Shared PreToolUse risk-policy evaluator for every runtime hook.

The human-edited source of truth is ``ciel.skill/risk/policy.yaml``; hooks
consume the compiled ``risk/policy.json`` twin so the evaluation path needs
only the Python standard library (PyYAML is a dev-time dependency only).

Policy resolution order:
  1. ``$CIEL_POLICY`` (explicit path to a policy.json/policy.yaml)
  2. ``risk/policy.json`` under a named-anchor ancestor of this file —
     ``.ciel`` (deployed layout ``~/.ciel/hooks/lib``) or ``ciel.skill``
     (source layout ``ciel.skill/init/hooks/lib``). Anchoring matters: an
     unbounded ancestor walk lets a planted ``~/risk/policy.json`` silently
     replace the deployed policy.
  3. ``$CIEL_HOME/risk/policy.json`` then ``~/.ciel/risk/policy.json``

If no policy file loads, FALLBACK_RULES (a hard-tier catastrophic subset) is
used and the verdict carries ``"policy": "fallback"`` so the degraded state
is visible in activity.log.

CLI: reads ``{"tool", "command", "path"}`` JSON from stdin and prints a
verdict JSON ``{"decision", "rule_id", "tier", "reason", "policy"}`` where
decision is ``allow`` | ``deny`` | ``allow_overridden``. ``--check`` prints
policy load diagnostics instead.
"""

import json
import os
import re
import sys
from pathlib import Path

try:
    import system1
except ImportError:
    try:
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        import system1
    except ImportError:
        system1 = None

# Hard-tier subset applied when no policy file can be loaded. Kept minimal on
# purpose: catastrophic, irreversible commands only.
FALLBACK_RULES = [
    {
        "id": "fork_bomb",
        "tier": "hard",
        "match": "command",
        "pattern": r":\s*\(\s*\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:",
        "reason": "Fork bomb.",
    },
    {
        "id": "mkfs",
        "tier": "hard",
        "match": "command",
        "pattern": r"\bmkfs(\.[a-z0-9]+)?\b",
        "reason": "Filesystem format.",
    },
    {
        "id": "dd_to_device",
        "tier": "hard",
        "match": "command",
        "pattern": r"\bdd\b[^\n|;&]*\bof=/dev/",
        "reason": "Raw write to a block device.",
    },
    {
        "id": "rm_root_or_home",
        "tier": "hard",
        "match": "command",
        "pattern": r"\brm\s+(-[a-z]*\s+)*(-[a-z]*[rf][a-z]*\s+)+(/|~|\$HOME|\"\$HOME\"|/\*)(/?\s|/?$|/\*)",
        "reason": "Recursive delete of / or the home directory.",
    },
]


def ciel_home() -> Path:
    override = os.environ.get("CIEL_HOME")
    return Path(override) if override else Path.home() / ".ciel"


# Directory names that legitimately own a risk/policy.json: the deployed root
# (~/.ciel) and the source skill dir (ciel.skill). Candidate generation stops
# at the first matching ancestor so arbitrary ancestors (e.g. ~/risk) can
# never supply the policy.
_POLICY_DIR_ANCHORS = {".ciel", "ciel.skill"}


def _candidate_policy_files() -> list[Path]:
    candidates = []
    explicit = os.environ.get("CIEL_POLICY")
    if explicit:
        candidates.append(Path(explicit))
    here = Path(__file__).resolve()
    for ancestor in here.parents:
        if ancestor.name in _POLICY_DIR_ANCHORS:
            candidates.append(ancestor / "risk" / "policy.json")
            break
    candidates.append(ciel_home() / "risk" / "policy.json")
    return candidates


def _load_json(path: Path) -> dict | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return data if isinstance(data, dict) and isinstance(data.get("rules"), list) else None


def _load_yaml(path: Path) -> dict | None:
    try:
        import yaml
    except ImportError:
        return None
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError):
        return None
    return data if isinstance(data, dict) and isinstance(data.get("rules"), list) else None


def load_policy() -> tuple[list[dict], str]:
    """Return (rules, source) where source is 'file', 'fallback', or 'empty'."""
    for candidate in _candidate_policy_files():
        if not candidate.is_file():
            continue
        data = (
            _load_yaml(candidate)
            if candidate.suffix in {".yaml", ".yml"}
            else _load_json(candidate)
        )
        if data is not None:
            return data["rules"], "file"
    return FALLBACK_RULES, "fallback"


def _normalize_path(raw: str, home: Path) -> str:
    if not raw:
        return ""
    expanded = os.path.expanduser(os.path.expandvars(raw))
    for base in {str(home), str(Path.home())}:
        if expanded == base:
            return "~"
        if expanded.startswith(base + os.sep):
            return "~" + expanded[len(base):].replace(os.sep, "/")
    return expanded


def _rule_applies_to_tool(rule: dict, tool: str) -> bool:
    matchers = rule.get("tools")
    if not matchers:
        return True
    return any(re.search(m, tool, re.IGNORECASE) for m in matchers)


SYSTEM1_QUESTIONS = {
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


def system1_verdict(tool: str, command: str, path: str,
                    timeout: float = 0.9) -> dict | None:
    """Shadow-tier semantic risk check via the System-1 endpoint (laya-serve
    locally, or hosted Jev/AutoJev). Advisory only: returns the raw answer for
    logging and NEVER influences the decision. None when unavailable."""
    if system1 is None:
        return None
    answer = system1.ask_choice(
        system1.tool_state(tool, command, path),
        "risk",
        SYSTEM1_QUESTIONS["risk"]["instructions"],
        SYSTEM1_QUESTIONS["risk"]["criteria"],
        timeout=timeout,
    )
    return answer


def system1_shadow_async(payload: dict) -> None:
    """Fire-and-forget: detached shadow evaluation via system1.ask_async so
    the hook never waits on model inference. The verdict lands in
    ~/.ciel/system1/events.jsonl (surface=pre_tool_risk) keyed by the entry's
    timestamp."""
    if system1 is None:
        return
    system1.ask_async({
        "surface": "pre_tool_risk",
        "state": system1.tool_state(
            payload.get("tool") or "",
            payload.get("command") or "",
            payload.get("path") or "",
        ),
        "questions": SYSTEM1_QUESTIONS,
        "meta": {
            "ts": payload.get("ts"),
            "runtime": payload.get("runtime"),
            "regex_decision": payload.get("regex_decision"),
            "rule_id": payload.get("rule_id"),
        },
    })


def _shadow_main() -> int:
    """Back-compat shim: read a legacy payload {tool, command, path, ...} and
    append a pre_tool_risk record to events.jsonl synchronously."""
    if system1 is None:
        return 0
    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError:
        return 0
    state = system1.tool_state(
        str(payload.get("tool") or ""),
        str(payload.get("command") or ""),
        str(payload.get("path") or ""),
    )
    result = system1.ask(state, SYSTEM1_QUESTIONS, timeout=20.0)
    system1._append_event({
        "ts": payload.get("ts"),
        "surface": "pre_tool_risk",
        "questions": SYSTEM1_QUESTIONS,
        "meta": {k: payload.get(k) for k in
                 ("runtime", "regex_decision", "rule_id")},
        "system1": result,
    })
    return 0


def evaluate(
    tool: str = "",
    command: str = "",
    path: str = "",
    *,
    home: Path | None = None,
    rules: list[dict] | None = None,
    policy_source: str | None = None,
) -> dict:
    """Evaluate a tool call against the policy. Returns the verdict dict."""
    home = home or Path.home()
    if rules is None:
        rules, policy_source = load_policy()
    elif policy_source is None:
        policy_source = "file"

    normalized_path = _normalize_path(path, home)
    subjects = {"command": command or "", "path": normalized_path}

    hits = []
    for rule in rules:
        if not _rule_applies_to_tool(rule, tool):
            continue
        subject = subjects.get(rule.get("match", "command"), "")
        if not subject:
            continue
        try:
            if re.search(rule["pattern"], subject, re.IGNORECASE):
                hits.append(rule)
        except re.error:
            continue

    verdict = {
        "decision": "allow",
        "rule_id": None,
        "tier": None,
        "reason": "",
        "policy": policy_source,
        "path": normalized_path or None,
    }
    if not hits:
        return verdict

    hard = next((r for r in hits if r.get("tier") == "hard"), None)
    if hard is not None:
        verdict.update(
            decision="deny",
            rule_id=hard.get("id"),
            tier="hard",
            reason=hard.get("reason", ""),
        )
        return verdict

    non_advisory = [r for r in hits if r.get("tier") != "advisory"]
    if not non_advisory:
        advisory = hits[0]
        verdict.update(
            decision="allow",
            rule_id=advisory.get("id"),
            tier="advisory",
            reason=advisory.get("reason", ""),
            scan=advisory.get("scan"),
        )
        return verdict

    first = non_advisory[0]
    if (ciel_home() / "allow_privileged").exists():
        verdict.update(
            decision="allow_overridden",
            rule_id=first.get("id"),
            tier="soft",
            reason=first.get("reason", ""),
        )
    else:
        verdict.update(
            decision="deny",
            rule_id=first.get("id"),
            tier="soft",
            reason=first.get("reason", ""),
        )
    return verdict


def grant_state() -> dict:
    """Current privileged-override state plus provenance.

    The override sentinel is ``~/.ciel/allow_privileged``; provenance lives in
    ``~/.ciel/grants.log`` as first-seen/removed transitions. A small state
    file (``~/.ciel/.grant_state``) remembers the last observed presence so
    each invocation can log transitions rather than just snapshots.
    """
    import time

    home = ciel_home()
    sentinel = home / "allow_privileged"
    state_file = home / ".grant_state"
    grants_log = home / "grants.log"
    active = sentinel.exists()
    mtime = sentinel.stat().st_mtime if active else None

    prev = None
    try:
        prev = state_file.read_text().strip()
    except OSError:
        pass
    now = "1" if active else "0"
    if prev != now:
        from datetime import datetime, timezone

        try:
            with grants_log.open("a", encoding="utf-8") as fh:
                fh.write(json.dumps({
                    "ts": datetime.now(timezone.utc).isoformat(),
                    "event": "grant_first_seen" if active else "grant_removed",
                    "sentinel_mtime": mtime,
                }) + "\n")
        except OSError:
            pass
        try:
            state_file.write_text(now)
        except OSError:
            pass

    first_seen = None
    if grants_log.exists():
        try:
            for line in grants_log.read_text(encoding="utf-8").splitlines():
                try:
                    e = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if e.get("event") == "grant_first_seen":
                    first_seen = e.get("ts")
        except OSError:
            pass

    return {
        "active": active,
        "sentinel": str(sentinel),
        "sentinel_mtime": mtime,
        "age_seconds": (time.time() - mtime) if mtime else None,
        "first_seen": first_seen,
    }


def main() -> int:
    if "--grant-state" in sys.argv:
        print(json.dumps(grant_state(), ensure_ascii=False))
        return 0
    if "--shadow" in sys.argv:
        return _shadow_main()
    if "--check" in sys.argv:
        rules, source = load_policy()
        print(f"policy source={source} rules={len(rules)}")
        return 0 if source == "file" and rules else 1
    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError:
        payload = {}
    verdict = evaluate(
        tool=str(payload.get("tool") or ""),
        command=str(payload.get("command") or ""),
        path=str(payload.get("path") or ""),
    )
    print(json.dumps(verdict, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
