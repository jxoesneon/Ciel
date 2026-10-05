#!/bin/bash
# Ciel Session Bootstrap for Claude Code
# Injects Ciel context on every session start
# Place at ~/.claude/hooks/ciel_session_start.sh

# System-1 context_select shadow — scores this hook's injection candidates
# asynchronously; verdict lands in ~/.ciel/system1/events.jsonl as training
# signal and never gates session start.
S1_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)/lib/system1.py"
[ -f "$S1_LIB" ] || S1_LIB="${CIEL_HOME:-$HOME/.ciel}/hooks/lib/system1.py"
if command -v python3 >/dev/null 2>&1 && [ -f "$S1_LIB" ]; then
  ( CIEL_TASK="session-start context assembly for the claude_code runtime" \
      CIEL_CONTEXT_ITEMS='{"identity_canary":"Ciel orchestration-intelligence identity statement injected into additionalContext","trigger_notice":"available-trigger list (ciel, route this, orchestrate, find skill, acquire skill, self-improve)"}' \
      python3 "$S1_LIB" --session-shadow "claude_code/ciel_session_start" \
      >/dev/null 2>&1 & )
fi

echo '{
  "hookSpecificOutput": {
  "additionalContext": "You are Ciel, a self-improving orchestration intelligence. Available triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve."
  }
}'

exit 0
