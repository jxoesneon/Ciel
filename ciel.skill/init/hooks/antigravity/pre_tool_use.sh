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

try:
    payload = json.loads(os.environ.get("INPUT_JSON", "{}"))
except json.JSONDecodeError:
    payload = {}

tool_call = payload.get("toolCall") or {}
tool = tool_call.get("name") or payload.get("toolName") or "unknown"
args = tool_call.get("args") or payload.get("toolInput") or {}
command = str(args.get("CommandLine") or args.get("command") or "")
path = str(args.get("path") or args.get("file_path") or args.get("filePath") or "")

verdict = risk_policy.evaluate(tool=tool, command=command, path=path)
denied = verdict["decision"] == "deny"
override = verdict["decision"] == "allow_overridden"

entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "antigravity",
    "event": "PreToolUse",
    "tool": tool,
    "risk": "critical" if denied else "standard",
    "rule_id": verdict.get("rule_id"),
    "tier": verdict.get("tier"),
    "policy": verdict.get("policy"),
    "overridden": override,
    "conversationId": payload.get("conversationId"),
}
try:
    with (risk_policy.ciel_home() / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass

if denied:
    reason = verdict.get("reason") or "critical risk"
    print(json.dumps({"decision": "deny", "reason": f"Ciel safety gate [{verdict.get('rule_id')}]: {reason}"}))
elif override:
    print(json.dumps({"decision": "allow", "reason": f"Ciel safety gate [{verdict.get('rule_id')}]: permitted by local allow_privileged override."}))
else:
    print(json.dumps({"decision": "allow", "reason": "Ciel pre-flight check passed."}))
PY
