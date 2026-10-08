#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Rust fast path — `ciel session-end` appends the activity entry in-process.
CIEL_BIN="${CIEL_BIN:-}"
if [ -z "$CIEL_BIN" ]; then
  for _c in "${HOME:-/nonexistent}/.ciel/bin/ciel" "${HOME:-/nonexistent}/.cargo/bin/ciel" "$HOOK_DIR/../../bin/ciel"; do
    if [ -x "$_c" ]; then
      CIEL_BIN="$_c"
      break
    fi
  done
fi
_fb_reason="binary-absent"
if [ -n "$CIEL_BIN" ] && [ -x "$CIEL_BIN" ]; then
  _fb_reason="binary-failed"
  if printf '%s' "$input" | "$CIEL_BIN" session-end --runtime devin; then
    exit 0
  fi
fi
# Parity-soak telemetry (DOCKET_20261004_RUST_MIGRATION_AUDIT): every descent
# into the Python body is counted; 14 days silent + drill = twin deletion.
(printf '%s\n' "{\"ts\":\"$(date -u +%FT%TZ)\",\"hook\":\"devin/session_end\",\"reason\":\"$_fb_reason\"}" >>"${CIEL_HOME:-$HOME/.ciel}/fallback_events.jsonl") 2>/dev/null || true

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
    "event": "SessionEnd",
    "reason": payload.get("reason"),
    "session_id": payload.get("session_id"),
}
try:
    with (Path.home() / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass
PY
