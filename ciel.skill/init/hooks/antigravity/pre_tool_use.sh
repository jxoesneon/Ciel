#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CIEL_HOOK_LIB="$HOOK_DIR/../lib"

# Rust fast path — single process for evaluate+log+shadow. Any failure
# (missing binary, nonzero exit) falls through to the Python body below.
CIEL_BIN="${CIEL_BIN:-}"
if [ -z "$CIEL_BIN" ]; then
  for _c in "${HOME:-/nonexistent}/.ciel/bin/ciel" "${HOME:-/nonexistent}/.cargo/bin/ciel" "$HOOK_DIR/../../bin/ciel"; do
    if [ -x "$_c" ]; then CIEL_BIN="$_c"; break; fi
  done
fi
if [ -n "$CIEL_BIN" ] && [ -x "$CIEL_BIN" ]; then
  if printf '%s' "$input" | "$CIEL_BIN" pretool --runtime antigravity; then
    exit 0
  fi
fi

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

# Shadow tier: detached semantic check — zero added latency, verdict lands in
# ~/.ciel/system1/shadow.log keyed by this entry's ts. Advisory only.
risk_policy.system1_shadow_async({
    "ts": entry["ts"], "runtime": "antigravity", "tool": tool,
    "command": command, "path": path, "regex_decision": verdict["decision"],
    "rule_id": verdict.get("rule_id"),
})

if denied:
    reason = verdict.get("reason") or "critical risk"
    print(json.dumps({"decision": "deny", "reason": f"Ciel safety gate [{verdict.get('rule_id')}]: {reason}"}))
elif override:
    print(json.dumps({"decision": "allow", "reason": f"Ciel safety gate [{verdict.get('rule_id')}]: permitted by local allow_privileged override."}))
else:
    print(json.dumps({"decision": "allow", "reason": "Ciel pre-flight check passed."}))
PY
