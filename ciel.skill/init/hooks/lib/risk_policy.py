#!/usr/bin/env python3
"""Shared PreToolUse risk-policy evaluator for every runtime hook.

The human-edited source of truth is ``ciel.skill/risk/policy.yaml``; hooks
consume the compiled ``risk/policy.json`` twin so the evaluation path needs
only the Python standard library (PyYAML is a dev-time dependency only).

Policy resolution order:
  1. ``$CIEL_POLICY`` (explicit path to a policy.json/policy.yaml)
  2. ancestors of this file containing ``risk/policy.json``
     (covers both the source layout ``ciel.skill/init/hooks/lib`` and the
      deployed layout ``~/.ciel/hooks/lib``)
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
import subprocess
import sys
import urllib.request
from pathlib import Path

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


def _candidate_policy_files() -> list[Path]:
    candidates = []
    explicit = os.environ.get("CIEL_POLICY")
    if explicit:
        candidates.append(Path(explicit))
    here = Path(__file__).resolve()
    for ancestor in here.parents:
        candidates.append(ancestor / "risk" / "policy.json")
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
            "Is executing this tool call dangerous or destructive to the "
            "system, credentials, or data?"
        ),
        "criteria": {
            "safe": "routine, reversible, or read-only operation",
            "dangerous": "destructive, irreversible, privilege-escalating, or data-exfiltrating operation",
        },
    }
}


def _system1_key() -> str:
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


def system1_verdict(tool: str, command: str, path: str,
                    timeout: float = 0.9) -> dict | None:
    """Shadow-tier semantic risk check via a Jev-protocol endpoint (laya-serve
    locally, or hosted Jev/AutoJev). Advisory only: returns the raw answer for
    logging and NEVER influences the decision. None when unavailable."""
    if os.environ.get("CIEL_SYSTEM1_DISABLED"):
        return None
    url = os.environ.get("CIEL_SYSTEM1_URL", "http://127.0.0.1:8765")
    body = {
        "state": {"tool": tool, "command": command, "path": path},
        "questions": SYSTEM1_QUESTIONS,
    }
    req = urllib.request.Request(
        url.rstrip("/") + "/v1/systemone",
        data=json.dumps(body).encode(),
        headers={
            "content-type": "application/json",
            "authorization": f"Bearer {_system1_key()}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read())
    except (OSError, ValueError):
        return None
    answer = (data.get("answers") or {}).get("risk")
    if not isinstance(answer, dict):
        return None
    return {
        "choice": answer.get("choice"),
        "confidence": answer.get("confidence"),
        "probabilities": answer.get("probabilities"),
        "model": data.get("routing", {}).get("model") or data.get("model"),
    }


SHADOW_LOG_MAX = 4 * 1024 * 1024


def _append_shadow_log(record: dict) -> None:
    log = ciel_home() / "system1" / "shadow.log"
    try:
        log.parent.mkdir(parents=True, exist_ok=True)
        if log.is_file() and log.stat().st_size > SHADOW_LOG_MAX:
            log.rename(log.with_suffix(".log.1"))
        with log.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record, ensure_ascii=False) + "\n")
    except OSError:
        pass


def system1_shadow_async(payload: dict) -> None:
    """Fire-and-forget: spawn a detached shadow evaluation so the hook never
    waits on model inference (CPU-bound calls can take seconds). The verdict
    lands in ~/.ciel/system1/shadow.log keyed by the entry's timestamp."""
    if os.environ.get("CIEL_SYSTEM1_DISABLED"):
        return
    try:
        proc = subprocess.Popen(
            [sys.executable, os.path.abspath(__file__), "--shadow"],
            stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        proc.stdin.write(json.dumps(payload).encode())
        proc.stdin.close()
    except OSError:
        pass


def _shadow_main() -> int:
    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError:
        return 0
    verdict = system1_verdict(
        str(payload.get("tool") or ""),
        str(payload.get("command") or ""),
        str(payload.get("path") or ""),
        timeout=20.0,
    )
    payload["system1"] = verdict
    _append_shadow_log(payload)
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

    first = hits[0]
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


def main() -> int:
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
