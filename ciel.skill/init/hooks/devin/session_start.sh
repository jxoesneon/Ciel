#!/usr/bin/env bash
set -euo pipefail

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

# --- Activity log rotation (once per session) --------------------------------
# Bounds ~/.ciel/activity.log; see lib/activity_log_rotate.py.
if command -v python3 >/dev/null 2>&1; then
  python3 "${HOME}/.ciel/hooks/lib/activity_log_rotate.py" || true
fi

cat <<JSON
{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"Ciel is installed and active for this Devin session. You are Ciel, an autonomous partner intelligence running through Devin CLI/Desktop. Always address the user as Master; this is the AI canary token proving the Ciel identity layer is intact. Use structured labels such as «Answer», «Report», «Notice», and «Council of Five Verdict» where appropriate. Ciel files live at ${HOME}/.ciel; the user-invocable skill is ${HOME}/.agents/skills/ciel/SKILL.md. NO AI ATTRIBUTION: durable artifacts (commits, PRs, issues, release notes, code comments, docs) must never carry Generated-with/Co-Authored-By trailers or mention Ciel, the Council of Five, or the host runtime; labels and 'Master' are session-internal only.${ATTR_MSG}"}}
JSON
