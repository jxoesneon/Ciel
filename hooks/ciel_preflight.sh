#!/usr/bin/env bash
# Ciel PreToolUse Hook — Risk Classification for Devin/Claude Code
# Reads event JSON from stdin, emits decision JSON to stdout, logs to ~/.ciel/activity.log

set -euo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"
EVENT=$(cat)

log() {
  printf '%s\n' "$1" >> "$ACTIVITY_LOG"
}

decide() {
  local d="$1" r="${2:-}"
  if [ -n "$r" ]; then
    printf '{"decision":"%s","reason":"%s"}\n' "$d" "$r"
  else
    printf '{"decision":"%s"}\n' "$d"
  fi
}

# Extract tool name and command (best-effort, jq or grep fallback)
TOOL_NAME=""
COMMAND=""
FILE_PATH=""
if command -v jq >/dev/null 2>&1; then
  TOOL_NAME=$(echo "$EVENT" | jq -r '.tool_name // empty')
  COMMAND=$(echo "$EVENT" | jq -r '.tool_input.command // empty')
  FILE_PATH=$(echo "$EVENT" | jq -r '.tool_input.file_path // .tool_input.path // empty')
else
  TOOL_NAME=$(echo "$EVENT" | grep -oE '"tool_name"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  COMMAND=$(echo "$EVENT" | grep -oE '"command"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  FILE_PATH=$(echo "$EVENT" | grep -oE '"(file_path|path)"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

# Normalize for matching
CMD_LC=$(echo "$COMMAND" | tr '[:upper:]' '[:lower:]')

# === CRITICAL RISK: Block destructive commands ===
if echo "$CMD_LC" | grep -qE 'rm\s+-rf\s+/|rm\s+-rf\s+\*|>:?/dev/null.*</dev/null|mkfs\.|dd\s+if=.*of=/dev/|:\(\)\s*\{\s*:\|:\s*\&\s*\};|curl.*\|\s*sh|wget.*\|\s*sh'; then
  log "{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"hook\":\"PreToolUse\",\"tool\":\"$TOOL_NAME\",\"decision\":\"block\",\"reason\":\"critical-risk destructive command\",\"command\":\"$COMMAND\"}"
  decide "block" "Ciel Safety: destructive/critical-risk command blocked"
  exit 0
fi

# === HIGH RISK: Force ask for writes outside project or system paths ===
if [ "$TOOL_NAME" = "write" ] || [ "$TOOL_NAME" = "edit" ]; then
  if echo "$FILE_PATH" | grep -qE '^(/etc/|/usr/|/bin/|/sbin/|/lib/|/opt/homebrew/|/usr/local/|/System/|/Applications/|/Users/[^/]+$/\.|\.ssh/|\.gnupg/|\.aws/|\.config/|/tmp/\.)'; then
    log "{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"hook\":\"PreToolUse\",\"tool\":\"$TOOL_NAME\",\"decision\":\"ask\",\"reason\":\"high-risk path outside project\",\"path\":\"$FILE_PATH\"}"
    decide "ask" "Ciel: write to system/sensitive path requires confirmation"
    exit 0
  fi
fi

# === MID RISK: Ask for network calls with unknown patterns ===
if echo "$CMD_LC" | grep -qE 'curl|wget|nc\s|telnet|ftp|scp|rsync'; then
  if ! echo "$CMD_LC" | grep -qE 'curl\s+--proto|curl\s+-L\s+https://(raw\.githubusercontent|github)\.com|curl\s+.*api\.github\.com'; then
    log "{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"hook\":\"PreToolUse\",\"tool\":\"$TOOL_NAME\",\"decision\":\"ask\",\"reason\":\"mid-risk network call\",\"command\":\"$COMMAND\"}"
    decide "ask" "Ciel: network call requires confirmation"
    exit 0
  fi
fi

# === LOW RISK: Allow known safe patterns ===
SAFE_PATTERNS='^git\s+(status|log|diff|show|branch|remote|config\s+--list|rev-parse)\s*;?'
SAFE_PATTERNS2='^cargo\s+(test|check|clippy|build|fmt)\s*;?'
SAFE_PATTERNS3='^npm\s+(test|run\s+test|run\s+dev|run\s+build|run\s+lint)\s*;?'
SAFE_PATTERNS4='^(ls|cat|head|tail|find|grep|rg|which|pwd|echo|printf|mkdir\s+-p)\s*;?'
SAFE_PATTERNS5='^(node|npx|tsx|eslint|prettier|tsc|vite|vitest)\s*;?'

if echo "$CMD_LC" | grep -qiE "$SAFE_PATTERNS|$SAFE_PATTERNS2|$SAFE_PATTERNS3|$SAFE_PATTERNS4|$SAFE_PATTERNS5"; then
  log "{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"hook\":\"PreToolUse\",\"tool\":\"$TOOL_NAME\",\"decision\":\"approve\",\"reason\":\"low-risk known pattern\",\"command\":\"$COMMAND\"}"
  decide "approve"
  exit 0
fi

# === DEFAULT: Allow but log ===
log "{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"hook\":\"PreToolUse\",\"tool\":\"$TOOL_NAME\",\"decision\":\"approve\",\"reason\":\"default allow\",\"command\":\"$COMMAND\"}"
decide "approve"
