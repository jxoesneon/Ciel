#!/usr/bin/env bash
# Godot Skill — PreToolUse Hook for exec tool
# Kills stale Godot test/runtime instances BEFORE launching a new one.
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

# Only act if the command involves Godot binary
if ! echo "$COMMAND" | grep -qiE 'godot|Godot\.app'; then
  exit 0
fi

# --- Kill stale Godot test/runtime instances ---
# DO NOT kill:
#   - The editor (process has --editor flag)
#   - The godot-mcp node server (node process, not Godot binary)
#   - This hook script itself or other hook scripts
#   - Any bash/sh wrapper running a hook script

SELF_PID=$$
KILLED_COUNT=0

# Find Godot binary processes that are NOT the editor and NOT hook scripts
# Match the actual Godot binary, not scripts with "godot" in their path
while IFS= read -r line; do
  PID=$(echo "$line" | awk '{print $2}')
  [ -z "$PID" ] && continue
  [ "$PID" = "$SELF_PID" ] && continue

  CMDLINE=$(ps -p "$PID" -o args= 2>/dev/null || true)
  # Skip if not a Godot binary (must contain Godot.app or godot binary path)
  if ! echo "$CMDLINE" | grep -qiE 'Godot\.app/Contents/MacOS/Godot|/bin/godot'; then
    continue
  fi
  # Skip the editor
  if echo "$CMDLINE" | grep -qi '\-\-editor'; then
    continue
  fi
  # Kill it
  kill "$PID" 2>/dev/null || true
  KILLED_COUNT=$((KILLED_COUNT + 1))
done < <(ps aux | grep -iE 'Godot\.app/Contents/MacOS/Godot|/bin/godot' | grep -v 'grep')

# Wait briefly for graceful termination
if [ "$KILLED_COUNT" -gt 0 ]; then
  sleep 0.5
  # Force kill any that didn't die
  while IFS= read -r line; do
    PID=$(echo "$line" | awk '{print $2}')
    [ -z "$PID" ] && continue
    [ "$PID" = "$SELF_PID" ] && continue
    CMDLINE=$(ps -p "$PID" -o args= 2>/dev/null || true)
    if echo "$CMDLINE" | grep -qiE 'Godot\.app/Contents/MacOS/Godot|/bin/godot' && ! echo "$CMDLINE" | grep -qi '\-\-editor'; then
      kill -9 "$PID" 2>/dev/null || true
    fi
  done < <(ps aux | grep -iE 'Godot\.app/Contents/MacOS/Godot|/bin/godot' | grep -v 'grep')

  printf '{"ts":"%s","hook":"PreToolUse","skill":"godot-exec","action":"killed_stale","count":%d}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$KILLED_COUNT" >> "$ACTIVITY_LOG" 2>/dev/null || true
fi

exit 0
