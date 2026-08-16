#!/usr/bin/env bash
# Godot Skill — PreToolUse Hook
# Injects AAA+ Godot 4.x guidance when editing Godot project files.
# Triggers on: .gd, .tscn, .tres, .gdshader, .gdshaderinc, project.godot
#
# Registered in ~/.claude/settings.json under config.hooks.PreToolUse
# with matcher targeting write/edit tools.

set -euo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"
EVENT=$(cat)

# Extract tool name and file path
TOOL_NAME=""
FILE_PATH=""
if command -v jq >/dev/null 2>&1; then
  TOOL_NAME=$(echo "$EVENT" | jq -r '.tool_name // empty')
  FILE_PATH=$(echo "$EVENT" | jq -r '.tool_input.file_path // .tool_input.path // empty')
else
  TOOL_NAME=$(echo "$EVENT" | grep -oE '"tool_name"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
  FILE_PATH=$(echo "$EVENT" | grep -oE '"(file_path|path)"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

# Only inject for write/edit tools targeting Godot files
if [ "$TOOL_NAME" != "write" ] && [ "$TOOL_NAME" != "edit" ] && [ "$TOOL_NAME" != "notebook_edit" ]; then
  exit 0
fi

# Check if file is a Godot project file
IS_GODOT_FILE=0
GUIDANCE_TYPE=""

case "$FILE_PATH" in
  *.gd)
    IS_GODOT_FILE=1
    GUIDANCE_TYPE="gdscript"
    ;;
  *.tscn)
    IS_GODOT_FILE=1
    GUIDANCE_TYPE="scene"
    ;;
  *.tres)
    IS_GODOT_FILE=1
    GUIDANCE_TYPE="resource"
    ;;
  *.gdshader|*.gdshaderinc)
    IS_GODOT_FILE=1
    GUIDANCE_TYPE="shader"
    ;;
  */project.godot)
    IS_GODOT_FILE=1
    GUIDANCE_TYPE="project"
    ;;
esac

if [ "$IS_GODOT_FILE" -eq 0 ]; then
  exit 0
fi

# Log activation
printf '{"ts":"%s","hook":"PreToolUse","skill":"godot","tool":"%s","file":"%s","guidance":"%s","decision":"inject"}\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$TOOL_NAME" "$FILE_PATH" "$GUIDANCE_TYPE" >> "$ACTIVITY_LOG"

# Build context-specific guidance
case "$GUIDANCE_TYPE" in
  gdscript)
    GUIDANCE='GODOT AAA+ GUIDANCE (GDScript 2.0):
- Enforce strict static typing: use `:=` and `-> Type` on all variables and functions.
- Elevate `Unsafe Property Access` warnings to `Error` in Project Settings.
- Avoid `get_node("../../../")` — use Dependency Injection: pass references via `_ready()`.
- Minimize Autoloads — store persistent state in Resource files (`.tres`), not singletons.
- Use Component pattern (composition > inheritance): `HealthComponent`, `HitboxComponent`, etc.
- "Call Down, Signal Up" — parents call methods on children, children emit signals to parents.
- Avoid defining lambdas inside hot paths like `_process` — they allocate memory each frame.
- For massive entity counts, use Godot Servers (RenderingServer, PhysicsServer3D) with RIDs.
- Object pooling: pre-instantiate nodes into an Array stack, pop/reset/push instead of queue_free().
- Use Node-based FSM for complex entities: StateMachine parent orchestrates child State nodes.
- For physics: use Godot Jolt (mandatory for AAA 3D), enable physics interpolation (4.3+).
- Use CapsuleShape3D for humanoids, never trimesh/concave for dynamic bodies.
- Time-slice raycast queries via PhysicsDirectSpaceState3D.intersect_ray() instead of hundreds of RayCast3D nodes.'
    ;;
  scene)
    GUIDANCE='GODOT AAA+ GUIDANCE (Scene .tscn):
- Scenes must be self-contained — avoid brittle relative node path dependencies.
- Use MultiMeshInstance3D for repeated objects (foliage, crowds, asteroids) for automated batching.
- Set manual Visibility Ranges for LOD swapping on distant objects.
- Enable Raster Occlusion Culling; use simplified OccluderInstance3D blocker meshes, never high-poly.
- For GI: SDFGI for open worlds (+ SSIL), VoxelGI for medium/indoor, LightmapGI for static scenes.
- Separate gameplay state logic (GDScript FSM) from animation presentation (AnimationTree).
- Chunk NavigationRegion3D nodes; bake asynchronously with simplified collision geometry.'
    ;;
  resource)
    GUIDANCE='GODOT AAA+ GUIDANCE (Resource .tres):
- Resources are globally reference-counted — use them for data-driven state, not Autoloads.
- Store persistent game state (player stats, organ telemetry, ship configs) in .tres files.
- Resources decouple data from the scene hierarchy and are easily serializable.
- Use an Event Bus Autoload holding only signals for cross-system decoupling.'
    ;;
  shader)
    GUIDANCE='GODOT AAA+ GUIDANCE (Shader .gdshader):
- Standardize on ORM textures (Occlusion, Roughness, Metallic packed in one map).
- Enable "Use Physical Light Units" (Lux, Lumens) for realistic PBR.
- Use TAA to fix specular aliasing on PBR materials.
- For fog: lower froxel buffer resolution, use localized FogVolume nodes, enforce Alpha Scissor on semi-transparent objects.
- Keep compute shader results on GPU in StorageBuffers — avoid costly CPU readbacks (buffer_get_data).'
    ;;
  project)
    GUIDANCE='GODOT AAA+ GUIDANCE (project.godot):
- Use Godot Jolt (JoltPhysics3D) as the 3D physics engine — mandatory for AAA.
- Enable physics interpolation (4.3+) to decouple physics ticks from render framerate.
- Use Forward+ renderer for 3D AAA projects.
- Enable TAA (Temporal Anti-Aliasing) for PBR specular stability.
- Elevate Unsafe Property Access warning to Error for strict typing enforcement.
- Enable Raster Occlusion Culling for large scenes.'
    ;;
  *)
    exit 0
    ;;
esac

# Emit additional context for the agent
printf '{"hookSpecificOutput":{"additionalContext":"%s"}}\n' "$GUIDANCE"

exit 0
