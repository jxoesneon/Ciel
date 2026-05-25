#!/usr/bin/env bash
# Ciel PostToolUse Hook — Outcome Scoring & Activity Logging
set -euo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"
EVENT=$(cat)

# Extract fields
tool_name=""
tool_input_cmd=""
success=""
output_len="0"
error=""

if command -v jq >/dev/null 2>&1; then
  tool_name=$(echo "$EVENT" | jq -r '.tool_name // empty')
  tool_input_cmd=$(echo "$EVENT" | jq -r '.tool_input.command // empty')
  success=$(echo "$EVENT" | jq -r '.tool_response.success // "unknown"')
  output_len=$(echo "$EVENT" | jq -r '(.tool_response.output // "" | length)')
  error=$(echo "$EVENT" | jq -r '.tool_response.error // empty')
else
  tool_name=$(echo "$EVENT" | grep -oE '"tool_name"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  success=$(echo "$EVENT" | grep -oE '"success"[[:space:]]*:[[:space:]]*(true|false)' | sed 's/.*:\s*//' | head -n1)
  error=$(echo "$EVENT" | grep -oE '"error"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

# Score outcome
score="neutral"
if [ "$success" = "true" ]; then
  score="success"
elif [ "$success" = "false" ]; then
  score="failure"
fi

# Log structured entry
printf '{"ts":"%s","hook":"PostToolUse","tool":"%s","success":%s,"score":"%s","output_len":%s,"error":"%s","command":"%s"}\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  "$tool_name" \
  "${success:-\"unknown\"}" \
  "$score" \
  "${output_len:-0}" \
  "${error:-}" \
  "${tool_input_cmd:-}" \
  >> "$ACTIVITY_LOG"

# No decision needed for PostToolUse — just exit cleanly
exit 0
