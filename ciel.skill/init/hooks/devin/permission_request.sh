#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CIEL_HOOK_LIB="$HOOK_DIR/../lib"

# Rust fast path — `ciel permission-request` appends grant-aware entry.
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
  if printf '%s' "$input" | "$CIEL_BIN" permission-request --runtime devin; then
    exit 0
  fi
fi
# Parity-soak telemetry (DOCKET_20261004_RUST_MIGRATION_AUDIT): every descent
# into the Python body is counted; 14 days silent + drill = twin deletion.
(printf '%s\n' "{\"ts\":\"$(date -u +%FT%TZ)\",\"hook\":\"devin/permission_request\",\"reason\":\"$_fb_reason\"}" >> "${CIEL_HOME:-$HOME/.ciel}/fallback_events.jsonl") 2>/dev/null || true
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
grant_active = None
try:
    import risk_policy
    grant_active = bool(risk_policy.grant_state().get("active"))
except Exception:
    pass
entry = {
    "ts": datetime.now(timezone.utc).isoformat(),
    "runtime": "devin",
    "event": "PermissionRequest",
    "tool": payload.get("tool_name") or payload.get("toolName") or "unknown",
    "grant_active": grant_active,
    "session_id": payload.get("session_id"),
    "prompt_id": payload.get("prompt_id"),
}
try:
    with (Path.home() / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
except OSError:
    pass
PY
