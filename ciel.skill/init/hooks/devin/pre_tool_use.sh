#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CIEL_HOOK_LIB="$HOOK_DIR/../lib"
INPUT_JSON="$input" python3 - <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, os.environ.get("CIEL_HOOK_LIB", ""))
import risk_policy

ciel_home = risk_policy.ciel_home()
log = ciel_home / "activity.log"
try:
    payload = json.loads(os.environ.get("INPUT_JSON", "{}"))
except json.JSONDecodeError:
    payload = {}

tool = payload.get("tool_name") or payload.get("toolName") or "unknown"
tool_input = payload.get("tool_input") or payload.get("toolInput") or {}
command = str(tool_input.get("command") or tool_input.get("CommandLine") or "")
path = str(
    tool_input.get("file_path")
    or tool_input.get("path")
    or tool_input.get("notebook_path")
    or ""
)

verdict = risk_policy.evaluate(tool=tool, command=command, path=path)
denied = verdict["decision"] == "deny"
override = verdict["decision"] == "allow_overridden"

entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "devin",
    "event": "PreToolUse",
    "tool": tool,
    "risk": "critical" if denied else "standard",
    "rule_id": verdict.get("rule_id"),
    "tier": verdict.get("tier"),
    "policy": verdict.get("policy"),
    "overridden": override,
    "session_id": payload.get("session_id"),
    "prompt_id": payload.get("prompt_id"),
}
try:
    with log.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass

if denied:
    reason = verdict.get("reason") or "critical risk"
    print(json.dumps({
        "decision": "block",
        "reason": f"Ciel safety gate [{verdict.get('rule_id')}]: {reason} Run it manually outside the agent or narrow the operation before retrying."
    }))
PY
