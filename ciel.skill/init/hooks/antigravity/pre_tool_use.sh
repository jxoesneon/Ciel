#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
INPUT_JSON="$input" python3 - <<'PY'
import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

try:
    payload = json.loads(os.environ.get("INPUT_JSON", "{}"))
except json.JSONDecodeError:
    payload = {}

tool_call = payload.get("toolCall") or {}
tool = tool_call.get("name") or payload.get("toolName") or "unknown"
args = tool_call.get("args") or payload.get("toolInput") or {}
command = str(args.get("CommandLine") or args.get("command") or "")
path = str(args.get("path") or args.get("file_path") or args.get("filePath") or "")

critical_patterns = [
    r"\b(sudo|doas|pkexec)\b",
    r"\brm\s+(-[a-zA-Z]*[rf][a-zA-Z]*\s+)*(/|~|\$HOME)(/|\s|$)",
    r"\bmkfs(\.|\s|$)",
    r"\bdd\b[^\n]*\bof=/dev/",
    r"\b(shutdown|reboot|poweroff|halt)\b",
    r":\s*\(\s*\)\s*\{\s*:\s*\|\s*:\s*&\s*}\s*;\s*:",
]
home = Path.home()
write_like_tool = bool(re.search(r"write|edit|delete|move|create", tool, re.IGNORECASE))
protected_write = write_like_tool and (bool(re.match(r"^/(etc|usr|opt|bin|sbin|boot|root)(/|$)", path)) or path.startswith(str(home / ".ssh")) or path.startswith(str(home / ".gnupg")))
critical = any(re.search(pattern, command, re.IGNORECASE) for pattern in critical_patterns) or protected_write

entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "antigravity",
    "event": "PreToolUse",
    "tool": tool,
    "risk": "critical" if critical else "standard",
    "conversationId": payload.get("conversationId"),
}
try:
    with (home / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass

if critical:
    print(json.dumps({"decision": "deny", "reason": "Ciel safety gate classified this operation as critical risk."}))
else:
    print(json.dumps({"decision": "allow", "reason": "Ciel pre-flight check passed."}))
PY
