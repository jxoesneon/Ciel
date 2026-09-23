#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
INPUT_JSON="$input" python3 - <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

try:
    payload = json.loads(os.environ.get("INPUT_JSON", "{}"))
except json.JSONDecodeError:
    payload = {}
tool_call = payload.get("toolCall") or {}
entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "antigravity",
    "event": "PostToolUse",
    "tool": tool_call.get("name") or payload.get("toolName") or "unknown",
    "error": payload.get("error"),
    "conversationId": payload.get("conversationId"),
}
try:
    with (Path.home() / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass
print("{}")
PY
