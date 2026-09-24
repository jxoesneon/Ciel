---
name: blender-hard-surface-modeling
description: Hard surface modeling in Blender via blender-mcp — boolean/bevel workflow, point-to-point construction, Blender 5.x API gotchas, and agent operational pitfalls learned the hard way.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# Blender Hard Surface Modeling (agent-driven)

For driving a live Blender instance use the `blender-mcp` MCP server (repo at
/Users/mey/blender-mcp, config entry `blender-mcp`, launch helper
`.venv/bin/blender-mcp-launch`). Blender must be running with the addon server
started on 127.0.0.1:9876 (addon auto-starts since the v0.2 fix).

## The golden rule: point-to-point construction

Floating, disconnected parts were the single biggest quality failure. Build every
part so it spans EXACT shared endpoints:

```python
import bpy, mathutils
def seg_cyl(p1, p2, r, name):
    p1, p2 = mathutils.Vector(p1), mathutils.Vector(p2)
    mid, d = (p1+p2)/2, (p2-p1)
    bpy.ops.mesh.primitive_cylinder_add(radius=r, depth=d.length, location=mid)
    obj = bpy.context.active_object; obj.name = name
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = d.to_track_quat('Z','Y')
    return obj
# seg_box(p1,p2,w,h) analogous with primitive_cube_add(scale=...)
```

Build order: blockout at real-world scale -> non-destructive booleans -> cleanup ->
bevel weights -> subdivision last. Verify connectivity by checking shared endpoint
coordinates, not by eyeballing renders.

## Boolean workflow

1. Keep cutters in their own named collection; keep booleans NON-destructive during iteration (move cutter, swap type freely).
2. Use solver `EXACT` for clean results (`FAST` fails on coincident faces).
3. Cutters must be watertight and slightly OVERLAP the target (never coplanar faces).
4. After applying: merge-by-distance, Select > Select Faces by Sides to find ngons/tris, dissolve stray loops. Uncleaned boolean geometry breaks bevels, subsurf, UVs.
5. Apply scale BEFORE boolean/bevel modifiers (Ctrl+A) — unapplied scale corrupts bevel widths.

## Bevels & sharpness

- Real edges are never perfectly sharp; bevel everything visible (small width, 1-2 segments).
- Prefer bevel WEIGHT (N-panel > Item > Edge Data > Bevel Weight) + a Bevel modifier (limit method: Weight) over per-edge manual bevels — fully non-destructive.
- For static props, edge creases (Shift+E) integrate cleanly with subsurf; for anything animated, mark sharp + support loops instead.
- Detail loop + 2 proximity/support loops protects hard edges when subdividing.

## Subdivision

Apply selectively; control crispness with creases/loops, not density. Inconsistent
subdivision density across a model causes normal-map baking wobble — plan loops in
the low-poly phase.

## Materials for mechanical reads

Principled BSDF presets that read well: worn metal (high metallic, noise-varied
roughness, micro bump), painted steel (clearcoat), coated panel (high clearcoat,
low roughness). Emissive glow: EEVEE 5.x has NO bloom — use boosted emission +
volumetric fog, or compositor glow.

## Operational rules for agents (hard-won)

- NEVER install/update the addon or its Python package while Blender is open — Blender won't rescan until restart. Install headless, enable + save prefs headless, then restart Blender once.
- The addon must be installed as a DIRECTORY addon bundling the `src/blender_mcp` package (single-file addon.py cannot import the handlers package).
- Python deps MUST come from `uv sync` (respects uv.lock). Plain `uv pip install -e .` resolves fresh and pulled `mcp==2.0.0`, which removed FastMCP and broke the server. Lock pins mcp 1.3.0.
- Hot-patching handler code inside a live Blender is a tar pit: bpy.app.timers and the dispatch path hold stale module references (GC-hunting timers, patching globals all fail eventually). Just stop/restart the MCP server from the addon panel — or restart Blender.
- Large exec scripts time out on the socket (~30s). Break work into small steps, or write the script to a file and load/run it at Blender startup.
- Check each tool's input schema before calling — parameter names differ (`action` vs `operation`, `target_name` vs `object_name`, `body` not `text` for 3D text). Guessing wastes turns.
- Test new tools against the LIVE instance systematically in waves, cleaning up test objects afterward.
