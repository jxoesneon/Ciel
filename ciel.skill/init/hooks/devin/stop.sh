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

try:
    payload = json.loads(os.environ.get("INPUT_JSON", "{}"))
except json.JSONDecodeError:
    payload = {}
session_id = payload.get("session_id")
entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "devin",
    "event": "Stop",
    "session_id": session_id,
    "prompt_id": payload.get("prompt_id"),
}
try:
    with (Path.home() / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass

# Completion check (council-20260923 M5/M7): if the requirement ledger has
# unresolved items, nudge the agent to reconcile before declaring done.
# Hard cap of 2 nudges per session — the hook must never trap the runtime
# in a re-prompt loop.
try:
    import requirements
    pending = requirements.pending_items(session_id)
except Exception:
    pending = []
if pending:
    state = Path.home() / ".ciel" / "checkpoints" / "stop_nudge.json"
    counts = {}
    try:
        counts = json.loads(state.read_text())
    except (OSError, json.JSONDecodeError):
        pass
    key = session_id or "unknown"
    n = int(counts.get(key, 0))
    if n < 2:
        counts[key] = n + 1
        try:
            state.parent.mkdir(parents=True, exist_ok=True)
            state.write_text(json.dumps(counts))
        except OSError:
            pass
        items = "; ".join(f"{e['id']}: {e.get('text','')}" for e in pending[:8])
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "Stop",
                "additionalContext": (
                    f"Requirement ledger has {len(pending)} unresolved item(s): "
                    f"{items}. Reconcile the ledger (mark done or note why "
                    "deferred) before declaring completion — per the "
                    "completion-evidence contract, 'done' requires the "
                    "verification artifact for the task class."
                ),
            }
        }))
PY
