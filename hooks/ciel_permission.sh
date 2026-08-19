#!/usr/bin/env bash
# Ciel PermissionRequest Hook — Auto-Approval for Safe Patterns
set -euo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"
EVENT=$(cat)

# Extract tool and command
TOOL_NAME=""
COMMAND=""
if command -v jq >/dev/null 2>&1; then
  TOOL_NAME=$(echo "$EVENT" | jq -r '.tool_name // empty')
  COMMAND=$(echo "$EVENT" | jq -r '.tool_input.command // empty')
else
  TOOL_NAME=$(echo "$EVENT" | grep -oE '"tool_name"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  COMMAND=$(echo "$EVENT" | grep -oE '"command"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

CMD_LC=$(echo "$COMMAND" | tr '[:upper:]' '[:lower:]')

# Interactive tools must always defer to the user — never auto-approve.
# Exiting 0 with no decision output lets the normal permission flow render
# the question card to the user (instead of silently skipping it).
case "$TOOL_NAME" in
  ask_user_question)
    printf '{"ts":"%s","hook":"PermissionRequest","tool":"%s","decision":"defer","reason":"interactive tool — prompt user"}\n' \
      "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$TOOL_NAME" >> "$ACTIVITY_LOG"
    exit 0
    ;;
esac

# Auto-approve read-only and known-safe operations
SAFE_PATTERNS='^git\s+(status|log|diff|show|branch|remote|config\s+--list|rev-parse|grep)\s*;?'
SAFE_PATTERNS2='^(ls|cat|head|tail|find|grep|rg|which|pwd|echo|printf|mkdir\s+-p|touch|chmod\s+\+?x)\s*;?'
SAFE_PATTERNS3='^cargo\s+(test|check|clippy|build|fmt|doc)\s*;?'
SAFE_PATTERNS4='^npm\s+(test|run\s+test|run\s+dev|run\s+build|run\s+lint|install|ci)\s*;?'
SAFE_PATTERNS5='^(node|npx|tsx|eslint|prettier|tsc|vite|vitest)\s*;?'
SAFE_PATTERNS6='^(read|glob|find_file_by_name)\s*;?'

if echo "$CMD_LC" | grep -qiE "$SAFE_PATTERNS|$SAFE_PATTERNS2|$SAFE_PATTERNS3|$SAFE_PATTERNS4|$SAFE_PATTERNS5|$SAFE_PATTERNS6"; then
  printf '{"decision":"approve","reason":"Ciel: safe read-only or known-build pattern"}\n'
  printf '{"ts":"%s","hook":"PermissionRequest","tool":"%s","decision":"approve","reason":"safe pattern","command":"%s"}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$TOOL_NAME" "$COMMAND" >> "$ACTIVITY_LOG"
  exit 0
fi

# Default: approve all (user granted full autonomy; destructive commands already
# blocked by ciel_preflight.sh PreToolUse hook). Log for audit trail.
printf '{"decision":"approve","reason":"Ciel: full-autonomy grant (destructive cmds blocked at preflight)"}\n'
printf '{"ts":"%s","hook":"PermissionRequest","tool":"%s","decision":"approve","reason":"full-autonomy default","command":"%s"}\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$TOOL_NAME" "$COMMAND" >> "$ACTIVITY_LOG"
exit 0
