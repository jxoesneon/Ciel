#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
INPUT_JSON="$input" python3 - <<'PY'
import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

home = Path.home()
log = home / ".ciel" / "activity.log"
try:
    payload = json.loads(os.environ.get("INPUT_JSON", "{}"))
except json.JSONDecodeError:
    payload = {}

tool = payload.get("tool_name") or payload.get("toolName") or "unknown"
tool_input = payload.get("tool_input") or payload.get("toolInput") or {}
text = json.dumps(tool_input, ensure_ascii=False)
command = str(tool_input.get("command") or tool_input.get("CommandLine") or "")
path = str(
    tool_input.get("file_path")
    or tool_input.get("path")
    or tool_input.get("notebook_path")
    or ""
)

critical_patterns = [
    r"\b(sudo|doas|pkexec)\b",
    r"\brm\s+(-[a-zA-Z]*[rf][a-zA-Z]*\s+)*(/|~|\$HOME)(/|\s|$)",
    r"\bmkfs(\.|\s|$)",
    r"\bdd\b[^\n]*\bof=/dev/",
    r"\b(shutdown|reboot|poweroff|halt)\b",
    r":\s*\(\s*\)\s*\{\s*:\s*\|\s*:\s*&\s*}\s*;\s*:",
]
protected_write = bool(re.match(r"^/(etc|usr|opt|bin|sbin|boot|root)(/|$)", path)) or path.startswith(str(home / ".ssh")) or path.startswith(str(home / ".gnupg"))
critical = any(re.search(pattern, command, re.IGNORECASE) for pattern in critical_patterns) or (tool in {"write", "edit", "notebook_edit"} and protected_write)
override = critical and (home / ".ciel" / "allow_privileged").exists()
risk = "critical" if critical else "standard"

entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "devin",
    "event": "PreToolUse",
    "tool": tool,
    "risk": risk,
    "overridden": override,
    "session_id": payload.get("session_id"),
    "prompt_id": payload.get("prompt_id"),
}
try:
    with log.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass

if critical and not override:
    print(json.dumps({
        "decision": "block",
        "reason": "Ciel safety gate classified this operation as critical risk. Run it manually outside the agent or narrow the operation before retrying."
    }))
PY
