#!/usr/bin/env bash
# Godot Skill — PostToolUse Hook
# Validates Godot-specific anti-patterns after write/edit operations on Godot files.
# Reports violations as warnings via additionalContext (non-blocking).

set -euo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"
EVENT=$(cat)

# Extract fields
TOOL_NAME=""
FILE_PATH=""
FILE_CONTENT=""
if command -v jq >/dev/null 2>&1; then
  TOOL_NAME=$(echo "$EVENT" | jq -r '.tool_name // empty')
  FILE_PATH=$(echo "$EVENT" | jq -r '.tool_input.file_path // .tool_input.path // empty')
  # For write tool, content is in tool_input.content; for edit, new_string
  FILE_CONTENT=$(echo "$EVENT" | jq -r '.tool_input.content // .tool_input.new_string // empty')
else
  TOOL_NAME=$(echo "$EVENT" | grep -oE '"tool_name"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  FILE_PATH=$(echo "$EVENT" | grep -oE '"(file_path|path)"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

# Only validate write/edit on Godot files
if [ "$TOOL_NAME" != "write" ] && [ "$TOOL_NAME" != "edit" ]; then
  exit 0
fi

case "$FILE_PATH" in
  *.gd|.tscn|.tres|.gdshader|.gdshaderinc|*/project.godot) ;;
  *) exit 0 ;;
esac

# If we don't have content from the event, try reading the file
if [ -z "$FILE_CONTENT" ] && [ -f "$FILE_PATH" ]; then
  FILE_CONTENT=$(cat "$FILE_PATH" 2>/dev/null || true)
fi

if [ -z "$FILE_CONTENT" ]; then
  exit 0
fi

WARNINGS=""

# === GDScript anti-pattern checks ===
case "$FILE_PATH" in
  *.gd)
    # Check for deep relative get_node paths (../../../)
    DEEP_PATHS=$(echo "$FILE_CONTENT" | grep -cE 'get_node\("[^"]*\.\./\.\./' || true)
    if [ "$DEEP_PATHS" -gt 0 ]; then
      WARNINGS="${WARNINGS}- [DI VIOLATION] Found ${DEEP_PATHS} deep relative get_node() path(s) (../../../). Use Dependency Injection instead — pass references via _ready().\n"
    fi

    # Check for untyped var declarations (var x = without type)
    # This is a heuristic — looks for "var name = " without ": Type" or ":="
    UNTYPED_VARS=$(echo "$FILE_CONTENT" | grep -cE '^\s*var\s+[a-z_]+\s*=\s*[^=:]' || true)
    if [ "$UNTYPED_VARS" -gt 5 ]; then
      WARNINGS="${WARNINGS}- [TYPING] Found ${UNTYPED_VARS} potentially untyped var declarations. Use := for inference or : Type for explicit typing.\n"
    fi

    # Check for lambda in _process or _physics_process
    LAMBDA_IN_PROCESS=$(echo "$FILE_CONTENT" | grep -A5 -E 'func _process|func _physics_process' | grep -cE 'func\s*\(' || true)
    if [ "$LAMBDA_IN_PROCESS" -gt 0 ]; then
      WARNINGS="${WARNINGS}- [PERF] Lambda(s) detected inside _process/_physics_process — these allocate memory each frame. Hoist them outside.\n"
    fi

    # Check for queue_free in hot paths (should pool instead)
    QFREE_IN_PROCESS=$(echo "$FILE_CONTENT" | grep -cE 'queue_free\(\)' || true)
    if [ "$QFREE_IN_PROCESS" -gt 3 ]; then
      WARNINGS="${WARNINGS}- [POOLING] ${QFREE_IN_PROCESS} queue_free() calls — consider object pooling for frequently spawned/destroyed objects.\n"
    fi

    # Check for @tool placement (must be before extends)
    if echo "$FILE_CONTENT" | grep -qE '^extends.*' && echo "$FILE_CONTENT" | grep -qE '^@tool'; then
      TOOL_LINE=$(echo "$FILE_CONTENT" | grep -n '@tool' | head -1 | cut -d: -f1)
      EXTENDS_LINE=$(echo "$FILE_CONTENT" | grep -n '^extends' | head -1 | cut -d: -f1)
      if [ -n "$TOOL_LINE" ] && [ -n "$EXTENDS_LINE" ] && [ "$TOOL_LINE" -gt "$EXTENDS_LINE" ]; then
        WARNINGS="${WARNINGS}- [SYNTAX] @tool annotation must be before extends (currently at line ${TOOL_LINE}, extends at line ${EXTENDS_LINE}).\n"
      fi
    fi

    # Check for class_name conflict with autoload (common Godot 4 error)
    if echo "$FILE_CONTENT" | grep -qE '^class_name\s+\w+' && echo "$FILE_CONTENT" | grep -qE '^@tool'; then
      WARNINGS="${WARNINGS}- [AUTOLOAD] class_name present in @tool script — if this is an autoload, class_name will conflict. Comment it out.\n"
    fi
    ;;
esac

# === Shader checks ===
case "$FILE_PATH" in
  *.gdshader|*.gdshaderinc)
    # Check for buffer_get_data in hot shader paths (CPU readback penalty)
    if echo "$FILE_CONTENT" | grep -q 'buffer_get_data'; then
      WARNINGS="${WARNINGS}- [GPU PERF] buffer_get_data detected — this causes costly CPU readback. Keep compute results on GPU in StorageBuffers.\n"
    fi
    ;;
esac

# === project.godot checks ===
case "$FILE_PATH" in
  */project.godot)
    # Check if Jolt is configured
    if ! echo "$FILE_CONTENT" | grep -qi 'jolt'; then
      WARNINGS="${WARNINGS}- [PHYSICS] Godot Jolt not detected in project.godot — mandatory for AAA 3D. Install JoltPhysics3D GDExtension.\n"
    fi
    # Check for TAA
    if ! echo "$FILE_CONTENT" | grep -qi 'taa'; then
      WARNINGS="${WARNINGS}- [RENDERING] TAA not enabled — recommended for PBR specular stability. Set rendering/anti_aliasing/quality/use_taa=true.\n"
    fi
    ;;
esac

# Emit warnings if any
if [ -n "$WARNINGS" ]; then
  # Clean up for JSON
  CLEAN_WARNINGS=$(echo "$WARNINGS" | sed 's/"/\\"/g' | python3 -c 'import sys; print(sys.stdin.read().replace("\n", "\\n").strip())')
  printf '{"hookSpecificOutput":{"additionalContext":"GODOT AAA+ POST-EDIT VALIDATION:\\n%s"}}\n' "$CLEAN_WARNINGS"

  printf '{"ts":"%s","hook":"PostToolUse","skill":"godot","tool":"%s","file":"%s","warnings":"%s"}\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$TOOL_NAME" "$FILE_PATH" "$(echo "$WARNINGS" | tr '\n' ';')" >> "$ACTIVITY_LOG"
fi

exit 0
