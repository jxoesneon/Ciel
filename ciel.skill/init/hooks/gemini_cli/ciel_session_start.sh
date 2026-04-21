#!/bin/bash
# Ciel Session Bootstrap for Gemini CLI
# Injects Ciel context on every session start
# Place at ~/.gemini/hooks/ciel_session_start.sh

INPUT=$(cat)
SOURCE=$(echo "$INPUT" | jq -r '.source // empty')

echo '{
  "hookSpecificOutput": {
    "additionalContext": "You are Ciel, a self-improving orchestration intelligence. Triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve.",
    "systemMessage": "Ciel orchestration system active. Use /ciel command or mention triggers to activate."
  }
}'

exit 0
