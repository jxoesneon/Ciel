---
name: blender-organic-modeling
description: Organic/character modeling in Blender via blender-mcp — sculpt-first pipeline, quad topology & edge flow, retopology, rigging math that actually works, Blender 5.x animation API changes.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# Blender Organic Modeling (agent-driven)

Pipeline: blockout/base mesh -> sculpt (voxel remesh) -> retopology -> UV/bake -> rig/animate.
When scripting via blender-mcp you mostly skip interactive sculpting; focus on base meshes,
procedural deformation, retopo helpers, and correct rigging.

## Topology laws

- Quads everywhere deformable; tris/ngons only hidden or flat. Ngons break under any curvature.
- Edge flow follows anatomy/muscle direction — loops cross bend areas (elbow/knee/mouth corners) perpendicular-ish, at least 3 loops around articulation zones (eyes, mouth, joints — "three curve principle").
- Poles: 3- and 5-valence poles are normal routing tools; avoid 6+ poles in deforming areas (pinch on subdiv). Place poles OUTSIDE expression/deformation bounds.
- Even quad distribution beats density. Localize detail with all-quad junctions instead of running detail loops around the whole model.
- Voxel remesh is for sculpt prep only — it can never produce deformation-ready topology; automatic retopo (Quadriflow) is likewise NOT rig-ready. Final game/film topology is manual or carefully guided.
- Retopo technique: new mesh over sculpt, Shrinkwrap modifier (snap onto high-poly), Retopology viewport overlay, Poly Build tool, face snapping. Start symmetrical (Mirror), keep a symmetrical copy before shrinkwrapping back asymmetry.
- Detail lives in normal/height bakes, not in retopo density.

## Rigging — the math that actually works

Manual bone parenting was repeatedly wrong across many attempts (pose-vs-rest matrices,
inherited armature rotations). DO NOT hand-compute `matrix_parent_inverse`.

Correct approach:

```python
# position child object at world pose first, select child then armature, then:
bpy.ops.object.mode_set(mode='POSE')
bpy.context.view_layer.objects.active = armature_obj
pbone.select = True          # Blender 5.x: selection is on POSE bones, bone.select removed
bpy.ops.object.parent_set(type='BONE', keep_transform=True)
```

This preserves world transforms exactly where every manual matrix formula drifted.

More rigging rules learned from failures:

- Define bones in WORLD space; do not parent the armature under a rotated root and expect local axes to line up — Y/Z get swapped and fold animations break. If you must rotate, rebuild bones world-space.
- Bones default to QUATERNION rotation mode: keyframing/driving `rotation_euler` silently does NOTHING. Either set `bone.rotation_mode='XYZ'` or animate quaternions.
- IK: targets beyond chain reach cause wild flailing; add pole targets to control elbow/knee direction; often simple FK keyframed rotations are MORE reliable for agent-built rigs than IK setups.
- Instanced/cloned limbs need IDENTICAL bone layouts (share one mesh data across arms/legs, orient via object transform), otherwise vertex groups don't transfer.
- Vertex groups belong to the OBJECT, not mesh data (Blender 5.x) when building programmatically.

## Blender 5.x API changes (things that broke real code)

- Layered actions: `action.fcurves` is GONE. Use `action.layers[0].strips[0].channelbags[0].fcurves`.
- `obj.driver_add(path)` returns a LIST of fcurves for array/vector properties.
- `BLENDER_EEVEE_NEXT` renamed to `BLENDER_EEVEE` (old EEVEE removed; Next is default).
- Compositor accessed via `scene.compositing_node_group` (not `scene.node_tree`); Composite output node handling changed.
- Edit-bone `bone.select` removed — use `pose_bone.select`.
- No EEVEE bloom — compositor/volumetrics instead.

## Animation sanity checklist

1. Set frame range, insert keyframes on the CORRECT channel type (quaternion vs euler).
2. Evaluate: sample `fcurve.evaluate(frame)` AND read actual pose values — drivers/constraints can silently not apply; verify world positions of end effectors at each keyframe.
3. Contact sheets / few keyframe stills beat full render previews (full anim renders time out; render keyframes and assemble).
4. Save incrementally (v0.1.0, v0.2.0...) BEFORE risky rigging passes — broken parenting corrupted saved files once and forced a redo from autosave.

## Base mesh starting points

Humanoid: start from a proven base mesh rather than primitives (meta-rig proportions
are hard to eyeball). Creatures: block out silhouette volumes first, mirror symmetric,
then subdivide. Grid/cylinder primitives + subsurf + proportional edit get surprisingly
far for organic forms before any sculpting.
