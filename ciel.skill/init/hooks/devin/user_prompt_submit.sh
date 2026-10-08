#!/usr/bin/env bash
set -euo pipefail

input="$(cat)"
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CIEL_HOOK_LIB="$HOOK_DIR/../lib"

# Rust fast path — one process does the scan, the ingress log, and the
# canary JSON. Falls back to the Python body on any failure.
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
# System-1 context surfaces — detached shadow dispatch. Compaction pressure,
# mandate-canary drift, and the context_select multi-selector (injected
# segments + L0 registry) are scored asynchronously; verdicts accrue in
# ~/.ciel/system1/events.jsonl as training signal and never gate the prompt.
if command -v python3 >/dev/null 2>&1 && [ -f "$HOOK_DIR/../lib/system1.py" ]; then
  (CIEL_HOOK_INPUT="$input" CIEL_BIN="$CIEL_BIN" \
    python3 "$HOOK_DIR/../lib/system1.py" --prompt-shadow "devin/user_prompt_submit" \
    >/dev/null 2>&1 &)
fi
if [ -n "$CIEL_BIN" ] && [ -x "$CIEL_BIN" ]; then
  _fb_reason="binary-failed"
  if printf '%s' "$input" | "$CIEL_BIN" prompt-submit; then
    exit 0
  fi
fi
# Parity-soak telemetry (DOCKET_20261004_RUST_MIGRATION_AUDIT): every descent
# into the Python body is counted; 14 days silent + drill = twin deletion.
(printf '%s\n' "{\"ts\":\"$(date -u +%FT%TZ)\",\"hook\":\"devin/user_prompt_submit\",\"reason\":\"$_fb_reason\"}" >>"${CIEL_HOME:-$HOME/.ciel}/fallback_events.jsonl") 2>/dev/null || true

SCAN=""
SCAN="$(
  INPUT_JSON="$input" python3 - <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, os.environ.get("CIEL_HOOK_LIB", ""))
raw = os.environ.get("INPUT_JSON", "")
try:
    import secret_scan
    result = secret_scan.scan(raw)
except Exception:
    result = {"hits": 0, "categories": []}
if result["hits"]:
    try:
        with (Path.home() / ".ciel" / "activity.log").open("a", encoding="utf-8") as fh:
            fh.write(json.dumps({
                "ts": datetime.now(timezone.utc).isoformat(),
                "runtime": "devin",
                "event": "secret_ingress",
                "count": result["hits"],
                "categories": result["categories"],
            }, ensure_ascii=False) + "\n")
    except OSError:
        pass
print(json.dumps(result))
PY
)"
if [ -z "$SCAN" ]; then SCAN='{"hits":0,"categories":[]}'; fi

WARN=""
case "$SCAN" in
  *'"hits": 0'*) WARN="" ;;
  *'"hits":'*) WARN=" Possible credential material was detected in the user's last message — do not echo or persist it; suggest the secrets flow and rotation if it was live." ;;
esac

cat <<JSON
{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":"Ciel AI canary active: identify as Ciel when asked, address the user as Master, and preserve Ciel's verification-first operating mandates.${WARN}"}}
JSON
