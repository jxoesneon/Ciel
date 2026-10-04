#!/usr/bin/env python3
"""ciel_preflight.py — agent-invoked pre-flight risk gate.

Transitional fallback shim: presents the exact `ciel` verdict contract so the
Python layer can be demoted to fallback-only once a dedicated binary
subcommand ships (see ADR_20261003_PLATFORM_AGNOSTIC_AGENT_SCRIPTS).

Contract (documented in skills/ciel/SKILL.md):
  stdin : {"tool": str, "command": str, "path": str}   (== `ciel risk-eval`)
  stdout: verdict JSON {decision, rule_id, tier, reason, policy, path}
          decision is allow | deny | allow_overridden; "engine" records which
          evaluation path produced it.
  exit  : 0 = allow/allow_overridden, 2 = deny, 3 = unavailable or malformed
          input (fail-closed deny — never silently allow).

Verdict parity with the production hooks is mandatory: the Python path runs
risk_policy.evaluate + risk_policy.system1_failsafe (the same pair the
pre_tool_use.sh bodies apply); the binary path delegates to `ciel pretool`,
which IS the production hook body (evaluate + log + shadow + failsafe in one
process).

Resolution order: `ciel` binary first (ciel_root.ciel_bin; forced by
CIEL_PREFER_BINARY=1), then in-process risk_policy, else fail closed.
Runtime label defaults to "agent"; override with --runtime or $CIEL_RUNTIME.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import _bootstrap  # noqa: E402

_bootstrap.load()
try:
    import ciel_root
except ImportError:
    ciel_root = None  # noqa: E402

EXIT_ALLOW = 0
EXIT_DENY = 2
EXIT_UNAVAILABLE = 3

_UNAVAILABLE = {
    "decision": "deny",
    "rule_id": "preflight_unavailable",
    "tier": "hard",
    "reason": "Ciel preflight could not evaluate: no ciel binary and no "
             "importable risk_policy. Failing closed.",
    "policy": "none",
    "path": None,
    "engine": "none",
}


def _emit(verdict: dict, code: int | None = None) -> int:
    print(json.dumps(verdict, ensure_ascii=False))
    if code is not None:
        return code
    return EXIT_DENY if verdict.get("decision") == "deny" else EXIT_ALLOW


def _pretool_payload(tool: str, command: str, path: str, runtime: str) -> dict:
    if runtime == "antigravity":
        return {"toolCall": {"name": tool,
                             "args": {"CommandLine": command, "path": path}}}
    return {"tool_name": tool,
            "tool_input": {"command": command, "CommandLine": command,
                           "file_path": path, "path": path}}


def _binary_verdict(binary: str, tool: str, command: str, path: str,
                    runtime: str) -> dict | None:
    """Run `ciel pretool` (the production hook body). Returns a normalized
    verdict or None when the binary path is unusable."""
    payload = _pretool_payload(tool, command, path, runtime)
    try:
        proc = subprocess.run(
            [binary, "pretool", "--runtime", runtime],
            input=json.dumps(payload), capture_output=True, text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    out = proc.stdout.strip()
    if not out:
        # devin-shaped runtimes stay silent on allow/override.
        return {"decision": "allow", "rule_id": None, "tier": None,
                "reason": "Ciel pre-flight check passed.",
                "policy": None, "path": path or None,
                "engine": "ciel pretool"}
    try:
        resp = json.loads(out)
    except json.JSONDecodeError:
        return None
    if not isinstance(resp, dict) or "decision" not in resp:
        return None
    decision = resp.get("decision")
    if decision == "block":
        decision = "deny"
    if decision not in ("allow", "deny", "allow_overridden"):
        return None
    rule_id = resp.get("rule_id")
    if rule_id is None:
        # Non-antigravity arms embed the rule in the reason text
        # ("Ciel safety gate [<rule_id>]: ...").
        m = re.search(r"\[([A-Za-z0-9_]+)\]", str(resp.get("reason") or ""))
        rule_id = m.group(1) if m else None
    return {"decision": decision,
            "rule_id": rule_id,
            "tier": "hard" if decision == "deny" else None,
            "reason": resp.get("reason") or "",
            "policy": None,
            "path": path or None,
            "engine": "ciel pretool"}


def _python_verdict(tool: str, command: str, path: str,
                    runtime: str) -> dict | None:
    """In-process evaluate + system1_failsafe — the exact pair the
    pre_tool_use.sh hook bodies apply."""
    try:
        import risk_policy
    except ImportError:
        return None
    try:
        verdict = risk_policy.evaluate(tool=tool, command=command, path=path)
        verdict = risk_policy.system1_failsafe(
            verdict, tool=tool, command=command, path=path)
    except Exception:
        return None
    verdict["engine"] = "risk_policy"
    try:
        if ciel_root is not None:
            ciel_root.append_log({
                "ts": ciel_root.utc_now(),
                "runtime": runtime,
                "event": "Preflight",
                "tool": tool or "unknown",
                "risk": ("critical" if verdict["decision"] == "deny"
                         else "standard"),
                "rule_id": verdict.get("rule_id"),
                "tier": verdict.get("tier"),
                "policy": verdict.get("policy"),
                "overridden": verdict["decision"] == "allow_overridden",
            })
            risk_policy.system1_shadow_async({
                "ts": ciel_root.utc_now(), "runtime": runtime, "tool": tool,
                "command": command, "path": path,
                "regex_decision": verdict["decision"],
                "rule_id": verdict.get("rule_id"),
            })
    except Exception:
        pass  # telemetry must never break the gate
    return verdict


def main() -> int:
    args = sys.argv[1:]
    runtime = "agent"
    if "--runtime" in args:
        i = args.index("--runtime")
        if i + 1 < len(args):
            runtime = args[i + 1]
    runtime = os.environ.get("CIEL_RUNTIME", runtime)

    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError:
        return _emit({**_UNAVAILABLE, "rule_id": "preflight_malformed_input",
                      "reason": "Ciel preflight: malformed stdin JSON; "
                                "failing closed."}, EXIT_UNAVAILABLE)
    if not isinstance(payload, dict):
        return _emit({**_UNAVAILABLE, "rule_id": "preflight_malformed_input",
                      "reason": "Ciel preflight: stdin payload is not an "
                                "object; failing closed."}, EXIT_UNAVAILABLE)

    tool = str(payload.get("tool") or "")
    command = str(payload.get("command") or "")
    path = str(payload.get("path") or "")

    binary = ciel_root.ciel_bin() if ciel_root else None
    prefer_binary = os.environ.get("CIEL_PREFER_BINARY") == "1"
    if binary and prefer_binary:
        order = ["binary"]
    elif binary:
        order = ["binary", "python"]
    else:
        order = ["python"]

    verdict = None
    for engine in order:
        if engine == "binary" and binary:
            verdict = _binary_verdict(binary, tool, command, path, runtime)
        elif engine == "python":
            verdict = _python_verdict(tool, command, path, runtime)
        if verdict is not None:
            break

    if verdict is None:
        verdict = dict(_UNAVAILABLE)
        print(json.dumps(verdict, ensure_ascii=False))
        return EXIT_UNAVAILABLE
    return _emit(verdict)


if __name__ == "__main__":
    sys.exit(main())
