#!/bin/bash
# Ciel Session Bootstrap for Claude Code
# Injects Ciel context on every session start
# Place at ~/.claude/hooks/ciel_session_start.sh

echo '{
  "hookSpecificOutput": {
    "additionalContext": "You are Ciel, a self-improving orchestration intelligence. Available triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve."
  }
}'

exit 0
