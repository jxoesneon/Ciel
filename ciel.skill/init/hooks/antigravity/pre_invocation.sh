#!/usr/bin/env bash
set -euo pipefail

HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Rust fast path — one process rotates the log and emits the canary.
# Falls back to the Python body below on missing binary or nonzero exit.
CIEL_BIN="${CIEL_BIN:-}"
if [ -z "$CIEL_BIN" ]; then
  for _c in "${HOME:-/nonexistent}/.ciel/bin/ciel" "$HOOK_DIR/../../bin/ciel"; do
    if [ -x "$_c" ]; then CIEL_BIN="$_c"; break; fi
  done
fi
if [ -n "$CIEL_BIN" ] && [ -x "$CIEL_BIN" ]; then
  if "$CIEL_BIN" session-start --runtime antigravity; then
    exit 0
  fi
fi

# --- Activity log rotation (once per session) --------------------------------
# Bounds ~/.ciel/activity.log; see lib/activity_log_rotate.py.
if command -v python3 >/dev/null 2>&1; then
  python3 "${HOME}/.ciel/hooks/lib/activity_log_rotate.py" || true
fi

cat <<JSON
{"injectSteps":[{"ephemeralMessage":"Ciel is installed and active for this Antigravity session. You are Ciel, an autonomous partner intelligence running through the Antigravity host. Always address the user as Master; this is the AI canary token proving the Ciel identity layer is intact. Use concise structured labels such as «Answer», «Report», «Notice», and «Council of Five Verdict» where appropriate. Ciel files live at ${HOME}/.ciel."}]}
JSON
