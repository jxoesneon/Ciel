#!/usr/bin/env bash
# godot_safe_run.sh — Safely launches a Godot command with process lifecycle management.
#
# Usage:
#   godot_safe_run.sh --headless --script res://scripts/test_aaa_audio_suite.gd
#   godot_safe_run.sh --path . --debug res://scenes/space_flight.tscn --timeout 15
#
# This script:
#   1. Kills any stale Godot test/runtime instances (preserves the editor and MCP server)
#   2. Launches the requested Godot command
#   3. Waits for completion or timeout
#   4. Ensures the Godot process is killed after completion
#   5. Reports exit status

set -euo pipefail

GODOT_BIN="${GODOT_BIN:-/opt/homebrew/Caskroom/godot/4.7.1/Godot.app/Contents/MacOS/Godot}"
TIMEOUT=30
GODOT_ARGS=()

# Parse args — extract --timeout, pass rest to Godot
while [[ $# -gt 0 ]]; do
  case "$1" in
    --timeout)
      TIMEOUT="$2"
      shift 2
      ;;
    *)
      GODOT_ARGS+=("$1")
      shift
      ;;
  esac
done

# --- Step 1: Kill stale Godot instances (preserve editor + MCP) ---
echo "[godot_safe_run] Cleaning up stale Godot instances..."
STALE_PIDS=$(ps aux | grep -i '[G]odot' | grep -v 'grep' | grep -v '\-\-editor' | awk '{print $2}')
for PID in $STALE_PIDS; do
  CMDLINE=$(ps -p "$PID" -o args= 2>/dev/null || true)
  if echo "$CMDLINE" | grep -qi 'godot' && ! echo "$CMDLINE" | grep -qi '\-\-editor'; then
    kill "$PID" 2>/dev/null || true
  fi
done
sleep 0.5
# Force kill stragglers
for PID in $STALE_PIDS; do
  CMDLINE=$(ps -p "$PID" -o args= 2>/dev/null || true)
  if echo "$CMDLINE" | grep -qi 'godot' && ! echo "$CMDLINE" | grep -qi '\-\-editor'; then
    kill -9 "$PID" 2>/dev/null || true
  fi
done

# --- Step 2: Launch Godot ---
echo "[godot_safe_run] Launching: $GODOT_BIN ${GODOT_ARGS[*]}"
"$GODOT_BIN" "${GODOT_ARGS[@]}" &
GODOT_PID=$!

# --- Step 3: Wait for completion or timeout ---
EXIT_CODE=0
if [ "$TIMEOUT" -gt 0 ]; then
  # Wait up to TIMEOUT seconds
  ELAPSED=0
  while kill -0 "$GODOT_PID" 2>/dev/null; do
    sleep 1
    ELAPSED=$((ELAPSED + 1))
    if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
      echo "[godot_safe_run] Timeout reached (${TIMEOUT}s), killing Godot process..."
      kill "$GODOT_PID" 2>/dev/null || true
      sleep 0.5
      kill -9 "$GODOT_PID" 2>/dev/null || true
      EXIT_CODE=124  # timeout exit code
      break
    fi
  done
  if [ "$EXIT_CODE" -eq 0 ]; then
    wait "$GODOT_PID" || EXIT_CODE=$?
  fi
else
  # No timeout — wait indefinitely
  wait "$GODOT_PID" || EXIT_CODE=$?
fi

# --- Step 4: Ensure process is dead ---
if kill -0 "$GODOT_PID" 2>/dev/null; then
  kill -9 "$GODOT_PID" 2>/dev/null || true
fi

echo "[godot_safe_run] Done (exit code: $EXIT_CODE)"
exit $EXIT_CODE
