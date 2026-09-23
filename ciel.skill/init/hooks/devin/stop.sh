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
entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "devin",
    "event": "Stop",
    "session_id": payload.get("session_id"),
    "prompt_id": payload.get("prompt_id"),
}
try:
    with (Path.home() / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass
PY
