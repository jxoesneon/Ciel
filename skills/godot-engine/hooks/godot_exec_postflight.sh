#!/usr/bin/env bash
# Godot Skill — PostToolUse Hook for exec tool
# Kills any Godot test/runtime instances left running after an exec command.
# Preserves: the editor (--editor), the MCP node server, hook scripts themselves.

set -uo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"
EVENT=$(cat)

# Extract tool name and command
TOOL_NAME=""
COMMAND=""
if command -v jq >/dev/null 2>&1; then
  TOOL_NAME=$(echo "$EVENT" | jq -r '.tool_name // empty')
  COMMAND=$(echo "$EVENT" | jq -r '.tool_input.command // empty')
else
  TOOL_NAME=$(echo "$EVENT" | grep -oE '"tool_name"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  COMMAND=$(echo "$EVENT" | grep -oE '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

# Only act on exec tool
if [ "$TOOL_NAME" != "exec" ]; then
  exit 0
fi

# Only act if the command involved Godot binary
if ! echo "$COMMAND" | grep -qiE 'godot|Godot\.app'; then
  exit 0
fi

# --- Kill any Godot binary instances still running (not editor, not hooks) ---
SELF_PID=$$
KILLED_COUNT=0

while IFS= read -r line; do
  PID=$(echo "$line" | awk '{print $2}')
  [ -z "$PID" ] && continue
  [ "$PID" = "$SELF_PID" ] && continue

  CMDLINE=$(ps -p "$PID" -o args= 2>/dev/null || true)
  if ! echo "$CMDLINE" | grep -qiE 'Godot\.app/Contents/MacOS/Godot|/bin/godot'; then
    continue
  fi
  if echo "$CMDLINE" | grep -qi '\-\-editor'; then
    continue
  fi
  kill "$PID" 2>/dev/null || true
  KILLED_COUNT=$((KILLED_COUNT + 1))
done < <(ps aux | grep -iE 'Godot\.app/Contents/MacOS/Godot|/bin/godot' | grep -v 'grep')

if [ "$KILLED_COUNT" -gt 0 ]; then
  sleep 0.5
  # Force kill stragglers
  while IFS= read -r line; do
    PID=$(echo "$line" | awk '{print $2}')
    [ -z "$PID" ] && continue
    [ "$PID" = "$SELF_PID" ] && continue
    CMDLINE=$(ps -p "$PID" -o args= 2>/dev/null || true)
    if echo "$CMDLINE" | grep -qiE 'Godot\.app/Contents/MacOS/Godot|/bin/godot' && ! echo "$CMDLINE" | grep -qi '\-\-editor'; then
      kill -9 "$PID" 2>/dev/null || true
    fi
  done < <(ps aux | grep -iE 'Godot\.app/Contents/MacOS/Godot|/bin/godot' | grep -v 'grep')

  printf '{"ts":"%s","hook":"PostToolUse","skill":"godot-exec","action":"cleanup","count":%d}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$KILLED_COUNT" >> "$ACTIVITY_LOG" 2>/dev/null || true
fi

exit 0
