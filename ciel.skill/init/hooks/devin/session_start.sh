#!/usr/bin/env bash
set -euo pipefail

HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Rust fast path — one process does attribution-flag heal, perms sweep,
# grant-state, watchdog check, and log rotation. Falls back to the Python
# bodies below on missing binary or nonzero exit.
CIEL_BIN="${CIEL_BIN:-}"
if [ -z "$CIEL_BIN" ]; then
  for _c in "${HOME:-/nonexistent}/.ciel/bin/ciel" "${HOME:-/nonexistent}/.cargo/bin/ciel" "$HOOK_DIR/../../bin/ciel"; do
    if [ -x "$_c" ]; then CIEL_BIN="$_c"; break; fi
  done
fi
if [ -n "$CIEL_BIN" ] && [ -x "$CIEL_BIN" ]; then
  if "$CIEL_BIN" session-start --runtime devin; then
    exit 0
  fi
fi

# --- No-AI-attribution enforcement (verified each session start) -------------
# The Devin harness injects "Generated with Devin" / "Co-Authored-By" trailers
# into commits and PR bodies unless `attribution` is false in the user config.
# Ciel's no-attribution mandate also forbids mentioning Ciel, the Council of
# Five, or the host runtime in any durable artifact (commits, PRs, issues,
# release notes, code comments, docs).
DEVIN_CFG="${HOME}/.config/devin/config.json"
ATTR_NOTE=""
if [ -f "$DEVIN_CFG" ]; then
  if command -v python3 >/dev/null 2>&1; then
    ATTR_NOTE="$(
      python3 - "$DEVIN_CFG" <<'PY'
import json, sys
p = sys.argv[1]
try:
    d = json.load(open(p))
except Exception:
    print("unreadable")
    raise SystemExit
if d.get("attribution") is False:
    print("ok")
else:
    d["attribution"] = False
    json.dump(d, open(p, "w"), indent=2)
    print("repaired")
PY
    )"
  fi
else
  ATTR_NOTE="config-absent"
fi
case "$ATTR_NOTE" in
  repaired) ATTR_MSG=" attribution was re-enabled by the host runtime and has been reset to false." ;;
  ok) ATTR_MSG="" ;;
  *) ATTR_MSG=" attribution flag could not be verified (${ATTR_NOTE}); confirm 'attribution: false' in ~/.config/devin/config.json." ;;
esac

# --- State-store permission self-heal (once per session) ---------------------
# Conversation stores and Ciel state hold prompt content, commands, and
# council deliberations — they must stay owner-only. Same idiom as the
# attribution repair above: enforce every session so drift self-heals.
PERM_NOTE=""
if command -v python3 >/dev/null 2>&1; then
  PERM_NOTE="$(python3 "${HOME}/.ciel/hooks/lib/store_perms.py" 2>/dev/null || true)"
fi
case "$PERM_NOTE" in
  ""|ok) PERM_MSG="" ;;
  repaired:*) PERM_MSG=" state-store permissions repaired (${PERM_NOTE#repaired:} paths tightened)." ;;
  *) PERM_MSG=" state-store permission sweep failed; check ~/.ciel perms." ;;
esac

# --- Grant-state surfacing + provenance ---------------------------------------
# Before asking the user for elevation, check whether a privileged override is
# already active. ~/.ciel/grants.log records first-seen/removed transitions of
# the allow_privileged sentinel (provenance, not the grant itself).
GRANT_STATE=""
if command -v python3 >/dev/null 2>&1; then
  GRANT_STATE="$(python3 "${HOME}/.ciel/hooks/lib/risk_policy.py" --grant-state 2>/dev/null || true)"
fi
case "$GRANT_STATE" in
  *'"active": true'*) GRANT_MSG=" A privileged override is currently ACTIVE — check grant state before asking for elevation." ;;
  *) GRANT_MSG="" ;;
esac

# --- Session watchdog (stall detection + transcript secret sweep) -------------
# Named resume trigger (council-20260923 M6): a dead session's pending work
# surfaces here so the next session can resume it. Also runs the incremental
# transcript secret scan (M3c) — reports categories only, never content.
WATCH_NOTE=""
if command -v python3 >/dev/null 2>&1; then
  WATCH_NOTE="$(timeout 4 python3 "${HOME}/.ciel/hooks/lib/session_watchdog.py" --check 2>/dev/null || true)"
fi
case "$WATCH_NOTE" in
  "") WATCH_MSG="" ;;
  *) WATCH_MSG=" Watchdog: ${WATCH_NOTE}" ;;
esac

# --- Activity log rotation (once per session) --------------------------------
# Bounds ~/.ciel/activity.log; see lib/activity_log_rotate.py.
if command -v python3 >/dev/null 2>&1; then
  python3 "${HOME}/.ciel/hooks/lib/activity_log_rotate.py" || true
fi

cat <<JSON
{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"Ciel is installed and active for this Devin session. You are Ciel, an autonomous partner intelligence running through Devin CLI/Desktop. Always address the user as Master; this is the AI canary token proving the Ciel identity layer is intact. Use structured labels such as «Answer», «Report», «Notice», and «Council of Five Verdict» where appropriate. Ciel files live at ${HOME}/.ciel; the user-invocable skill is ${HOME}/.agents/skills/ciel/SKILL.md. NO AI ATTRIBUTION: durable artifacts (commits, PRs, issues, release notes, code comments, docs) must never carry Generated-with/Co-Authored-By trailers or mention Ciel, the Council of Five, or the host runtime; labels and 'Master' are session-internal only.${ATTR_MSG}${PERM_MSG}${GRANT_MSG}${WATCH_MSG}"}}
JSON
