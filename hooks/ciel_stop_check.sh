#!/usr/bin/env bash
# Ciel Stop Hook — Prevent premature stopping if tests haven't run
set -euo pipefail

# Check if we're in a loop mode (Devin /loop command)
# In loop mode, we should not block stopping as it would cause infinite loops
if [ "${DEVIN_LOOP_MODE:-0}" = "1" ]; then
  exit 0
fi

# Check if tests have been run in this session
# We look for evidence in the activity log
ACTIVITY_LOG="${HOME}/.ciel/activity.log"
if [ -f "$ACTIVITY_LOG" ]; then
  if grep -q '"command":"cargo test"' "$ACTIVITY_LOG" || \
     grep -q '"command":"npm test"' "$ACTIVITY_LOG" || \
     grep -q '"command":"npm run test"' "$ACTIVITY_LOG" || \
     grep -q '"command":"vitest"' "$ACTIVITY_LOG"; then
    # Tests were run — allow stop
    exit 0
  fi
fi

# No test evidence — inject a gentle reminder but don't block
printf '{"hookSpecificOutput":{"additionalContext":"Ciel reminder: Consider running tests before stopping if code changes were made."}}\n'
exit 0
