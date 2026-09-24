# COUNCIL DOCKET: 20260820_SPATIAL_MISMATCH_AUDIT

**Date**: 2026-08-20
**Candidate Artifact**: BioGenesis-X flight scene — chunk streaming vs camera visibility
**Scope**: Root-cause analysis of why 5.59M primitives are rendered when the player sees only a few asteroids
**Predecessor**: DOCKET_20260820_PRIMITIVE_BUDGET_AUDIT_2.md (Option K — +26.3% FPS)

## Situation

After Options A and K, the flight scene still renders **5.59M primitives** at **39.4 FPS**.
The user reports seeing only a few asteroids on screen. A visibility diagnostic was run
to determine where the 2,424 rendered objects actually are relative to the camera.

## Evidence

### Visibility diagnostic (GPU profile, FLIGHT_IDLE entry, after Option K)

```
Camera: Camera3D at (0.0, -43.11, 14.22)
Camera far: 5,000,000 m (5,000 km)
Camera fov: 80.0
RENDER_TOTAL_OBJECTS: 2,424
RENDER_TOTAL_PRIMITIVES: 5,591,287
RENDER_TOTAL_DRAW_CALLS: 172

Ship pos: (0.0, -48.22, 2.63) (dist from origin: 48.3 m)

MeshInstance3D: 26 within far clip, 391 beyond far clip
RigidBody3D: 0 within far clip, 189 beyond far clip
Closest non-ship object: 11 m (the host star — ship spawns at origin near star)
Farthest within-far object: 343 m
```

**Only 26 of 417 MeshInstance3D nodes are within the 5,000 km far clip.**
**Zero of 189 RigidBody3D asteroids are within the far clip.**
**2,424 objects are being rendered despite most being beyond the camera's view.**

### Spatial scale mismatch

| Scale | Value | In km |
|---|---|---|
| Camera far clip | 5,000,000 m | 5,000 km |
| Near chunk size | 0.01 AU = 1,495,978,707 m | 1,495,979 km |
| Near stream radius | 3 chunks = 0.03 AU | 4,487,936 km |
| Far chunk size | 1.0 AU = 149,597,870,700 m | 149,597,871 km |
| Far stream radius | 3 chunks = 3.0 AU | 448,793,612 km |
| Asteroid mesh radius | 6 m | 0.006 km |
| Closest near chunk center | 0.5 × 1.5M km = 748M m | 747,989 km |
| Closest near asteroid (edge of chunk 0) | ~75,000,000 m | 75,000 km |

The closest possible near-field asteroid is **75,000 km away** — **15× beyond the camera's
5,000 km far clip**. All far-field asteroids are at **150+ million km** — **30,000× beyond
the far clip**. The camera physically cannot see any ChunkStreamManager asteroid.

### What IS visible (within 5,000 km)

The 26 objects within the far clip are:
- **Host star**: StarSphere (4,224 prims) + corona shell (4,224 prims) + ~8 small MIs (32 prims each) — at ~45m from camera (ship spawns at origin, star is at origin)
- **PlayerShip**: 4 MeshInstance3D (bio hull, shield, cockpit, fill light) — at 0-14m
- **Small nearby objects**: ~14 MeshInstance3D at 100-343m (8 prims each — HUD markers, scanner elements, NoiseMapDebugOverlay)

**Total visible unique primitives: ~56,000** (star + ship + HUD).
**GPU reports 5,591,287** — **99% of rendered primitives are invisible to the player.**

### Why frustum culling isn't working

From Godot 4.7 documentation research:

1. **`RENDER_TOTAL_PRIMITIVES_IN_FRAME` excludes culled objects** — the 2,424 objects
   are genuinely being rendered, not culled. Source:
   https://docs.godotengine.org/en/4.7/classes/class_performance.html

2. **The metric includes shadow and depth prepass** — "Due to the depth prepass and shadow
   passes, the number of primitives is always higher than the actual number of vertices in
   the scene (typically double or triple the original vertex count)."
   - 1.86M unique × 3 passes (color + depth + shadow) ≈ 5.59M — matches exactly.

3. **Float32 precision at large distances** — Godot's large world coordinates doc warns:
   "Around 1 million units from the origin, large amounts of snapping and popping occur."
   Our near chunks are at 748 million meters. At that distance, float32 precision is ~64m.
   AABB calculations for frustum culling may be corrupted.
   Source: https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html

4. **DirectionalLight3D shadow pass** — The sun light (`shadow_enabled = true`) renders
   the scene from the light's perspective. The shadow frustum may encompass objects beyond
   the camera's far clip. No `directional_shadow_max_distance` is set, so it defaults to
   the camera's far clip (5,000 km) — but the shadow PSSM splits may still render distant
   objects.
   Source: https://docs.godotengine.org/en/4.4/tutorials/3d/lights_and_shadows.html

5. **MultiMesh AABB** — Each far chunk MultiMesh has an AABB spanning the entire chunk
   (150 million km). If the AABB is corrupted by float precision, Godot may consider the
   entire MultiMesh "visible" and render all instances.

6. **Occlusion culling** — `use_occlusion_culling=true` is set in project.godot, but there
   are no OccluderInstance3D nodes in the scene. Occlusion culling requires baked occluder
   geometry to function. With no occluders, it does nothing.

### What about the floating origin system?

FlightController.gd has a floating origin system (`_apply_floating_origin()`) that shifts
the world when the ship is >50km from origin. However:
- The ship spawns at origin (48m from origin)
- The floating origin threshold is 50km
- The ship would need to fly 50km before the origin shifts
- Even after shifting, the near chunks are at 748 million meters — shifting by 50km
  doesn't meaningfully reduce the coordinate magnitude
- The floating origin addresses physics precision, not rendering culling

### Are galaxy-level objects rendering?

**Not directly.** No GalaxyMap or star system indicators are rendering during flight.
POIIndicatorManager hides its markers during flight (visible=false). The galaxy map is
only instantiated when explicitly opened.

However, **planet objects** are in the scene at real AU orbital distances
(149.6 billion meters for 1 AU) with only 120m visual radius. They are beyond the far
clip and should be culled — but given that frustum culling isn't working for asteroids,
they may also be rendering. The visibility diagnostic found 43 MeshInstance3D nodes in
UniverseManager (56K prims) — these include the star, corona, and potentially planets
that should be culled.

### The AsteroidField (now disabled) was the visible one

The old AsteroidField (asteroid_count=0 after Option K) spawned 60 RigidBody3D asteroids
within **400m** of the ship — well within the 5,000 km view distance. These were the
asteroids the player actually saw. ChunkStreamManager asteroids at 75,000+ km were never
visible. By disabling AsteroidField to save primitives, we removed the visible asteroids
while the invisible ones continue rendering.

## Root cause

The ChunkStreamManager was designed around **astronomical distances** (0.01 AU near chunks,
1.0 AU far chunks) for a game with Newtonian physics at real solar-system scale. But the
**camera far clip is only 5,000 km**, and the **asteroid mesh radius is only 6m**. At
75,000 km, a 6m asteroid subtends 0.003 pixels — physically impossible to see.

The chunk system is loading, spawning, and rendering hundreds of asteroids that are:
1. Beyond the camera's far clip (invisible)
2. Beyond float32 precision for reliable frustum culling
3. Too small to see even if they were within view distance
4. Rendered 3× (color + depth + shadow) for no visual benefit

## Proposed remediation options

### Precision and conversion research

The user asked whether we could use "scientific notation or something to fully utilize
Godot's capabilities while keeping full accuracy." This maps to three real techniques:

**1. Godot's double precision build (`precision=double`)**

Godot 4 can be compiled with `precision=double` which makes all `Vector3` types use
64-bit doubles on the CPU. For GPU rendering, Godot uses "emulated double precision"
(PR #66178) — it splits each double into two floats (a high-order and low-order part,
essentially mantissa + exponent in scientific notation terms) and reconstructs the
precision on the GPU. This works on Metal (Apple Silicon) since it doesn't use actual
GPU doubles.

Source: https://godotengine.org/article/emulating-double-precision-gpu-render-large-worlds/

With the double build:
- All AABB calculations use double precision → frustum culling works at AU distances
- All physics use double precision → no jitter at any distance
- GPU rendering uses emulated double → no vertex snapping at large coordinates
- No floating origin needed (though still recommended for extreme distances)

Limitations (from PR #66178):
- Does NOT work with `skip_vertex_transform` or `world_vertex_coords` shader modes
- World-space shader calculations still limited to float32
- MultiMesh emulated double precision has reduced accuracy beyond ~1,000 km
- No official precompiled double builds — must build from source
- ~20% CPU performance penalty, ~2× memory for vectors
- Requires scons + Python + Xcode (user has Xcode, needs scons)

**2. Floating origin every frame (Outer Wilds approach)**

Outer Wilds keeps the player at origin by applying an opposite force to every physics
body every frame. The cost is trivial — "it doesn't actually really do anything to
performance because we're already doing that. We're already applying forces to every
object." (Alex Beachum, Outer Wilds director).

Source: https://gamedev.stackexchange.com/questions/200945/moving-player-inside-of-moving-spaceship

For BioGenesis-X, the existing `_apply_floating_origin()` in FlightController.gd already
shifts all `celestial_bodies` group nodes. The change is to run it every frame instead
of at a 50km threshold. Cost: ~600 Vector3 additions per frame = ~36,000 ops/sec at 60fps
= negligible (the user's intuition that "conversions should be fairly inexpensive" is
correct).

With per-frame floating origin:
- All render coordinates stay within a few km of origin → frustum culling works
- No custom Godot build needed
- Full astronomical accuracy preserved in GDScript (floats are already 64-bit double)
- The "true" position is `render_position + origin_offset` (stored as double in GDScript)

**3. Split-position / chunk-local coordinates (manual scientific notation)**

Store each object's position as two values:
- `chunk_origin`: double-precision Vector3 (the coarse position, e.g. 1.5e11 m)
- `local_offset`: float32 Vector3 (the fine position within the chunk, e.g. 0-1500 m)

The render position is `local_offset` (always small). The true position is
`chunk_origin + local_offset` (computed in double when needed). This is essentially
what the chunk system already does — chunk nodes are positioned at `center_m` and
asteroids are positioned relative to the chunk node. The issue is that `center_m` is
stored as float32 in `Node3D.position`, losing precision.

This approach is what Babylon.js calls "floating origin mode" and what Unity HDRP calls
"camera-relative rendering" — the engine offsets all matrices by the camera position
before sending to the GPU.

### MultiMesh culling limitation

Critical finding from Godot docs: MultiMesh does NOT support per-instance frustum
culling. "The only drawback is that there is no screen or frustum culling possible for
individual instances. This means that millions of objects will be always or never drawn,
depending on the visibility of the whole MultiMesh."

Source: https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html

Our 69 far-field MultiMesh nodes each span an entire 1-AU chunk (150M km). If the
MultiMesh node's AABB intersects the frustum (which it does at large coordinates due to
float precision corruption), ALL instances in that MultiMesh are rendered. With 49 far
chunks × 40 asteroids × 20 prims = 39,200 prims — small individually, but the AABB
issue means none are culled.

Godot staff recommendation: "If you have instances in a MultiMesh that are far away from
each other, they should be placed in a separate MultiMeshInstance3D node. Doing so will
also improve rendering performance, as frustum and occlusion culling will be able to
cull individual nodes."

### Option L: Redesign chunk sizes around visibility

Replace astronomical chunk sizes with visibility-based sizes:

- **Near chunks**: 10 km × 10 km (was 1.5M km) — asteroids within 30 km of ship
- **Far chunks**: 100 km × 100 km (was 150M km) — asteroids within 300 km
- **Stream radius**: 3 chunks for both (30 km near, 300 km far)
- **Asteroid density**: tuned per chunk for visual density
- **Far clip**: keep 5,000 km — far-field asteroids at 300 km are well within view

**Estimated impact**: Only ~50-100 asteroids within view distance. ~100K-200K unique
primitives instead of 1.86M. With shadow/depth: 300K-600K rendered (vs 5.59M).
FPS: 39.4 → likely 60+ (capped by vsync).
**Complexity**: Medium — change 4 constants + retune density. Physics and gameplay
need to work at km-scale instead of AU-scale.
**Trade-off**: Loses real astronomical scale. The "wave-warp drive" for interplanetary
travel would need to handle the transition between scaled-down and real-scale space.
May conflict with the existing Newtonian flight model and planet landing system.

### Option M: Add visibility-range culling to existing chunks

Keep astronomical chunk sizes but add `GeometryInstance3D.visibility_range_*` properties
to cull asteroids beyond a screen-size threshold:

- Set `visibility_range_begin` = 0 (always visible from 0m)
- Set `visibility_range_end` = 5,000,000 (cull at camera far clip)
- Set `visibility_range_fade_mode` = FADE_DISTANCE (smooth fade-out)

**Estimated impact**: Godot would cull asteroids beyond 5,000 km. But this requires
per-node settings on 600+ RigidBody3D asteroids and 70 MultiMesh nodes. And it may not
work correctly at float32 precision limits.
**Complexity**: Low per-node, but high node count. Need to set visibility_range on
every spawned asteroid.
**Trade-off**: Doesn't fix the root cause — chunks still load at AU distances, wasting
memory and thread time on invisible objects. Frustum culling may still fail at large
coordinates.

### Option N: Floating origin every frame (recommended — no custom build)

Enhance the existing floating origin system to shift the world every frame, keeping
the ship always at (0,0,0). This keeps all render coordinates within a few km of
origin, where float32 precision is sufficient for frustum culling.

**Implementation**:
- Change `_FLOATING_ORIGIN_THRESHOLD_M` from 50,000 to 0 (shift every frame)
- Or better: remove the threshold check entirely and always shift
- The existing `_floating_origin_bodies` cache already handles `celestial_bodies` group
- Need to also shift ChunkStreamManager children (they're in `celestial_bodies` group)
- Store the cumulative origin offset as a double-precision GDScript variable
- True position = `node.global_position + origin_offset` (when needed for game logic)

**Estimated impact**: Frustum culling would work correctly, culling all 391 + 189
objects beyond the far clip. Rendered primitives would drop from 5.59M to ~168K
(56K visible × 3 passes). FPS: 39.4 → likely 60+ (vsync capped).
**Conversion cost**: ~600 Vector3 additions per frame = negligible (confirmed by
Outer Wilds: "it doesn't actually really do anything to performance").
**Complexity**: Low — the system already exists, just needs to run every frame.
Need to ensure ChunkStreamManager chunk loading uses the origin offset for distance
calculations.
**Trade-off**: Doesn't address the fact that near chunks are at 75,000 km — even with
perfect frustum culling, we're loading 664 asteroids that are all beyond the far clip.
CPU and memory waste on invisible objects. But this is a separate issue from rendering.

### Option O: Screen-size-based culling (the "human vision" approach)

Implement a custom culling system that culls objects based on their projected screen size:

- For each asteroid: `screen_size = mesh_radius / distance × screen_height_pixels`
- Cull if `screen_size < 1.0` (sub-pixel)
- A 6m asteroid at 5,000 km = 0.002 pixels → culled
- A 6m asteroid at 500m = 20 pixels → visible

**Estimated impact**: Only asteroids within ~5 km of the ship would render (6m / 5000m ×
1080px ≈ 1.3 pixels). This is the "human vision based" approach the user described.
**Complexity**: Medium-high — custom culling logic in `_process`, checking each asteroid
against camera distance. Or use Godot's visibility_range with screen-size mode.
**Trade-off**: Most physically accurate. But doesn't fix the chunk loading waste —
chunks still load at AU distances.

### Option P: Combine N + O (floating origin + screen-size cull)

Fix floating origin to keep coordinates small (enables frustum culling) AND add
screen-size-based culling for the final precision.

**Estimated impact**: Frustum culling handles most objects. Screen-size culling handles
edge cases. Rendered primitives: ~168K (only visible objects × 3 passes).
**Complexity**: High — two systems to implement and maintain.
**Trade-off**: Most thorough but most work.

### Option Q: Reduce camera far clip + shadow distance

Set camera far to 100 km (was 5,000 km) and `directional_shadow_max_distance` to 50 km.
This would force frustum culling to cull everything beyond 100 km.

**Estimated impact**: Frustum culling would cull all 391 + 189 objects beyond 100 km.
But this may break the star rendering (star is at origin, ship is at origin — star should
still be visible). And planets at AU distances would disappear (may be desired).
**Complexity**: Very low — two property changes.
**Trade-off**: Reduces view distance to 100 km. May not be enough for the "vast space"
feeling. Doesn't fix chunk loading waste.

### Option R: Godot double precision build (full accuracy, custom build)

Build Godot 4.7.1 from source with `precision=double`. This makes all Vector3 types
use 64-bit doubles on the CPU, and Godot's built-in emulated double precision handles
GPU rendering.

**Implementation**:
- Install scons: `pip3 install scons`
- Clone Godot: `git clone https://github.com/godotengine/godot.git -b 4.7-stable`
- Build: `scons platform=macos arch=arm64 precision=double -j8`
- Use the double-precision binary for development and exports
- All existing code works unchanged (Vector3 is just higher precision)

**Estimated impact**:
- Frustum culling works at AU distances (AABBs computed in double precision)
- No vertex jitter at any distance (emulated double on GPU)
- Physics precision is perfect at any distance
- MultiMesh emulated double has reduced accuracy beyond ~1,000 km (PR #66178 note)
- CPU performance: ~20% penalty for physics, small for rendering
- Memory: ~2× for all Vector3 types
**Complexity**: Medium — one-time build setup. Need to maintain custom binary.
No code changes needed, but need to ensure all addons are compatible (JoltPhysics3D
should be — it's a physics addon that benefits from double precision).
**Trade-off**: Full astronomical accuracy preserved everywhere. No floating origin
needed. But requires building from source and maintaining a custom binary. MultiMesh
still has limitations beyond ~1,000 km. And it doesn't fix the fact that we're loading
600+ invisible asteroids (frustum culling would cull them, but CPU/memory waste remains).

### Option S: Hybrid — floating origin every frame + double-precision GDScript math

Run floating origin every frame (Option N) AND store true positions as separate
double-precision GDScript variables for game logic accuracy.

**Implementation**:
- Floating origin shifts all Node3D positions every frame (render coordinates stay small)
- Each celestial body stores `true_position: Vector3` (GDScript float = 64-bit double)
- `true_position` is updated by `true_position += velocity * delta` (double precision)
- `node.global_position = true_position - camera_true_position` (cast to float32)
- Game logic (orbits, distances, physics) uses `true_position` (full accuracy)
- Rendering uses `node.global_position` (small, precise, frustum-cullable)

**Estimated impact**: Same as Option N for rendering (5.59M → ~168K prims). Plus full
astronomical accuracy in game logic. Best of both worlds.
**Conversion cost**: Same as Option N (~600 Vector3 additions) plus ~600 double-precision
updates per frame. Still negligible.
**Complexity**: Medium — need to add `true_position` tracking to celestial bodies and
update the floating origin system. The chunk system already uses `center_m` (double in
GDScript) for chunk positioning — just need to keep it in double and subtract camera.
**Trade-off**: Two position systems (render vs true) add complexity. But this is the
standard approach used by Outer Wilds, Elite Dangerous, and Star Citizen. The "scientific
notation" the user described — large coordinates split into a render offset and a true
position.

### Option T: Combine S + L (hybrid precision + visibility-based chunks)

Use the hybrid floating origin (Option S) for precision AND redesign chunk sizes
(Option L) for visibility. This is the complete solution.

**Implementation**:
- Floating origin every frame with double-precision true positions (Option S)
- Near chunks: 10 km × 10 km (Option L) — asteroids within 30 km of ship
- Far chunks: 100 km × 100 km — asteroids within 300 km
- Frustum culling works (small coordinates)
- Screen-size culling handles sub-pixel objects
- Only ~50-100 asteroids loaded (not 664)
- Full astronomical accuracy for game logic (orbits, distances)

**Estimated impact**: ~50-100 asteroids, ~100K-200K unique prims, ~300K-600K rendered
(with shadow/depth). FPS: 60+ (vsync capped). Full accuracy. No invisible objects
loaded or rendered.
**Complexity**: High — combines two systems. But each system is independently valuable.
**Trade-off**: Most complete solution. Highest complexity. The chunk redesign changes
gameplay scale (km instead of AU), which needs careful integration with the wave-warp
drive and planet landing systems.

## Research sources

- Godot 4.7 Performance monitors: https://docs.godotengine.org/en/4.7/classes/class_performance.html
- Godot large world coordinates: https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html
- Godot rendering at long distances (tracker): https://github.com/godotengine/godot/issues/98655
- Emulating double precision on GPU: https://godotengine.org/article/emulating-double-precision-gpu-render-large-worlds/
- Godot PR #66178 — emulated double precision for rendering: https://github.com/godotengine/godot/pull/66178
- Godot issue #58516 — double precision build still jitters: https://github.com/godotengine/godot/issues/58516
- Godot 3D lights and shadows: https://docs.godotengine.org/en/4.4/tutorials/3d/lights_and_shadows.html
- Godot DirectionalLight3D: https://docs.godotengine.org/en/stable/classes/class_directionallight3d.html
- Godot optimizing 3D performance: https://docs.godotengine.org/en/stable/tutorials/performance/optimizing_3d_performance.html
- Godot visibility ranges: https://docs.godotengine.org/en/stable/tutorials/3d/visibility_ranges.html
- Godot frustum culling + extra_cull_margin: https://docs.godotengine.org/en/stable/tutorials/shaders/advanced_postprocessing.html
- Godot MultiMesh — no per-instance culling: https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html
- Godot MultiMesh per-instance culling proposal: https://github.com/godotengine/godot-proposals/issues/10669
- Godot mesh LOD and MultiMesh: https://docs.godotengine.org/en/stable/tutorials/3d/mesh_lod.html
- Outer Wilds floating origin (Alex Beachum): https://gamedev.stackexchange.com/questions/200945/moving-player-inside-of-moving-spaceship
- Unity HDRP camera-relative rendering: https://docs.unity3d.com/Packages/hdrp/manual/Camera-Relative-Rendering.html
- Babylon.js floating origin: https://doc.babylonjs.com/features/featuresDeepDive/scene/large_world
- Unreal Engine LWC + camera-relative: https://dev.epicgames.com/community/learning/tutorials/DdzL/unreal-engine-fortnite-efficient-materials-for-large-worlds
- Gaia Sky precision at AU scale: https://tonisagrista.com/blog/2021/whats-new-gaiasky-31x/
- Building Godot with precision=double on macOS: https://forum.longplay.games/t/building-godot-4-for-large-world-sizes-on-mac/398
- Godot double precision builds proposal: https://github.com/godotengine/godot-proposals/issues/8843

## Files involved

- `scripts/ChunkStreamManager.gd` — chunk sizes, stream radii, asteroid spawning
- `scripts/UniverseManager.gd` — host star, planets, sun light with shadows
- `scripts/FlightController.gd` — floating origin system, camera setup
- `scripts/AsteroidField.gd` — the (now disabled) visible asteroid field
- `scenes/space_flight.tscn` — camera far/near, fov settings
- `project.godot` — occlusion culling setting

## Status

**DRAFT — not yet presented to council.**

Updated with precision/conversion research (Options R, S, T) per user request to explore
"scientific notation" approaches that fully utilize Godot's capabilities while keeping
full astronomical accuracy.

Key finding: the user's intuition is correct — conversions are inexpensive. The floating
origin shift (Option N/S) costs ~600 Vector3 additions per frame (negligible, confirmed by
Outer Wilds). The double precision build (Option R) uses Godot's built-in emulated double
precision which is essentially scientific notation (split float = mantissa + exponent) on
the GPU. Both approaches preserve full astronomical accuracy.

Awaiting user instruction to convene the Architecture Council for scoring and verdict.
