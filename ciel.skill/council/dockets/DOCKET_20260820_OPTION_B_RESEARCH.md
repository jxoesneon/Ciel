# COUNCIL RESEARCH REPORT: Option B — Godot Built-in Mesh LOD with MultiMesh

**Date**: 2026-08-20
**Researcher**: Chairman (inline research, subagent quota exhausted)
**Subject**: Can Godot 4.7's built-in mesh LOD replace or complement the manual dual-mesh tier (Option A)?

---

## Executive Summary

**Godot's built-in mesh LOD cannot replace Option A for far-field asteroid rendering.**
MultiMesh LOD selects a single LOD level for ALL instances based on the closest point
of the MultiMeshInstance3D's AABB to the camera. This means all instances — whether 0.5 AU
or 3 AU away — render at the same LOD level. Per-instance LOD is not supported and is an
open feature request (godot-proposals #10669, unresolved since 2023).

However, **Option B can complement Option A** in three specific ways:
1. Adding `generate_lods()` to the near-field mesh for automatic LOD when near chunks are distant
2. Using Visibility Ranges (HLOD) to hide far-field MultiMesh chunks beyond a distance threshold
3. Splitting far-field chunks into smaller MultiMesh nodes for per-chunk LOD selection

---

## Per-Question Findings

### 1. Per-instance LOD selection in MultiMesh

**Finding: NOT SUPPORTED. All instances use the same LOD level.**

From the official Godot 4.7 docs (`mesh_lod.html`):
> "For LOD selection, the point of the node's AABB that is the closest to the camera is
> used as a basis... this means that all instances will be drawn with the same LOD level
> at a given time."

From GitHub issue #76436 (godotengine/godot):
> "For performance reasons, LOD selection is performed once for the entire MultiMesh.
> This means all meshes within a MultiMeshInstance3D will use the same LOD level when
> drawing. If you want better LOD selection, don't use MultiMesh or create smaller lumps
> of concentrated MultiMeshes instead."

From godot-proposals #10669 (open, unresolved):
> "It does not support LOD switching on individual instance mesh, the lod only happens
> on all instances at same time. This is useless if you are using a big patch
> MultiMeshInstance3D. We need per instance LOD."

**Implication**: Our far-field chunks span 1 AU each. An asteroid at the near edge of a
chunk and one at the far edge are ~1.5×10⁸ km apart but share the same LOD level. Godot's
built-in LOD cannot differentiate them.

### 2. Screen-space LOD with MultiMesh — how the LOD level is chosen

**Finding: Based on closest AABB point to camera, not average or per-instance.**

From Godot 4.7 docs:
> "For LOD selection, the point of the node's AABB that is the closest to the camera is
> used as a basis."

From PR #92290 (merged in Godot 4.4):
> "The new code automatically handles being inside the AABB (distance will always be 0
> inside the AABB) and gradually increases from 0 at the surface of the AABB."

**Implication**: The LOD level is determined by the CLOSEST instance to the camera. If any
instance in the MultiMesh is close, ALL instances render at high detail. This is the worst
case for our chunk system — a chunk that spans from 0.5 AU to 1.5 AU will always use the
highest LOD level because its AABB touches the near edge.

### 3. Manual LOD via generate_lods() on ArrayMesh

**Finding: Works, but same per-instance limitation applies to MultiMesh.**

`ArrayMesh.add_surface_from_arrays()` accepts a `lods` Dictionary parameter:
> "Each entry in the dictionary represents an LOD level of the surface, where the value
> is the Mesh.ARRAY_INDEX array to use for the LOD level and the key is roughly
> proportional to the distance at which the LOD starts being used."

From forum discussions, the LOD key values are not well-documented and require
trial-and-error tuning. The keys are "roughly proportional to distance" but the exact
relationship depends on screen-space metrics, not raw world distance.

**Implication**: We COULD call `generate_lods()` on our asteroid mesh to create automatic
LOD tiers. However, for MultiMesh, all instances would still use the same LOD level
(the one selected by the closest AABB point). This is useful for NEAR-field MeshInstance3D
asteroids (individual RigidBody3D nodes) but NOT for far-field MultiMesh.

### 4. MultiMeshInstance3D LOD control properties

**Finding: `lod_bias` exists on GeometryInstance3D (parent class), `mesh_lod_threshold`
on Viewport.**

- `GeometryInstance3D.lod_bias` — per-node multiplier that makes LOD transitions happen
  sooner or later. Setting this high on far-field MultiMesh chunks would force lower LOD
  tiers, but ALL instances in the chunk would use that tier.
- `Viewport.mesh_lod_threshold` — global threshold (default 1.0 pixel). Higher values make
  all LOD transitions happen sooner across the entire viewport.

**Implication**: We could set `lod_bias` high on far-field MultiMeshInstance3D nodes to
force them to use the lowest available LOD tier. This would work IF the mesh has LOD tiers
generated via `generate_lods()`. Combined with Option A, this could provide a middle ground:
- Far mesh = icosahedron (20 prims, always)
- Near mesh with `generate_lods()` + high `lod_bias` = automatic degradation for distant
  near-field chunks

### 5. Impostor sprites / billboards for far-field

**Finding: Supported via Visibility Ranges (HLOD) — replace MultiMesh with Sprite3D at distance.**

From Godot 4.7 docs (`visibility_ranges.html`):
> "Visibility ranges can be used with any node that inherits from GeometryInstance3D...
> this makes it possible to use different node types as part of a LOD system. For example,
> you could display a MeshInstance3D representing a tree when up close, and replace it with
> a Sprite3D impostor in the distance."

From `optimizing_3d_performance.html`:
> "An alternative is to render not just one tree, but a number of trees together as a group.
> This can be especially effective if you can see an area but cannot physically approach it."

**Implication**: At extreme distances (3+ AU), asteroids could be replaced with a single
billboard sprite per chunk — a pre-rendered texture of an asteroid field. This would reduce
far-field rendering from 20 prims × 20 instances = 400 prims per chunk to 2 prims (one quad)
per chunk. However, our icosahedron approach is already extremely cheap (22,560 total prims),
so the marginal gain is minimal.

### 6. Performance comparison: Option A vs Option B

| Metric | Option A (dual-mesh) | Option B (Godot LOD) | Option A+B (combined) |
|---|---|---|---|
| Far-field prims | 22,560 (icosahedron) | ~405K (sub_level=1, lowest auto-LOD) | 22,560 (icosahedron) |
| Per-instance LOD | Yes (separate mesh) | No (all same level) | Yes for far, auto for near |
| Draw calls | 57 (one per chunk) | 57 (same) | 57 (same) |
| CPU overhead | Negligible (mesh swap) | Low (LOD selection per frame) | Low |
| Memory | +1 ArrayMesh (~1KB) | +LOD tiers in mesh (~50KB) | +1KB + 50KB |
| Complexity | ~10 lines code | generate_lods() + tuning | ~15 lines code |

### 7. Godot 4.7 specific changes

**Finding: PR #92290 (AABB surface distance) was merged for 4.4. No 4.7-specific LOD changes.**

The LOD selection logic was improved in Godot 4.4 to use distance to AABB surface (PR #92290),
fixing the camera-angle-dependent LOD jumping issue (#76436, #95948). Godot 4.7 inherits this
fix but adds no new MultiMesh LOD features.

### 8. Best practices from other Godot projects

**Finding: Split MultiMesh into smaller patches for per-patch LOD.**

From godot-proposals #10669:
> "If you have instances in a MultiMesh that are far away from each other, they should be
> placed in a separate MultiMeshInstance3D node."

From the Godot docs:
> "Doing so will also improve rendering performance, as frustum and occlusion culling will
> be able to cull individual nodes (while they can't cull individual instances in a MultiMesh)."

From a forum user with 190K foliage instances:
> "I'm using 1700 MultiMeshInstance3Ds... it will be stuck for a few seconds every time
> you reorder nodes."

**Implication**: Our chunk system already follows this best practice — each far chunk is a
separate MultiMeshInstance3D. The per-chunk AABB means Godot's LOD selection operates per-chunk,
not per-asteroid. With `generate_lods()` on the mesh, distant chunks would get lower LOD tiers
automatically. But this requires the mesh to have LOD tiers, which our icosahedron doesn't need.

---

## Recommendation

### Option B alone: NOT VIABLE as a replacement for Option A

Godot's MultiMesh LOD cannot provide per-instance distance-based mesh selection. All instances
in a MultiMesh share the same LOD level, selected by the closest AABB point. For our 1 AU chunks,
this means the entire chunk renders at the detail level of its closest asteroid — defeating the
purpose of LOD for far-field rendering.

### Option B as a complement to Option A: VIABLE for near-field optimization

**Proposal B-1: Add `generate_lods()` to the near-field mesh.**
The near-field mesh (subdivision_level=2, ~2,700 prims) could benefit from automatic LOD tiers.
When a near-field chunk is at the edge of the near stream radius (0.04 AU), its asteroids are
far enough that a lower LOD tier would be invisible. This would reduce near-field primitive
count for distant near-chunks without changing the close-range experience.

**Proposal B-2: Use `lod_bias` on distant near-field chunks.**
Set `lod_bias` higher on near-field chunks that are far from the ship. This forces Godot to
use lower LOD tiers for those chunks' individual MeshInstance3D asteroids.

**Proposal B-3: Visibility Ranges for far-field chunk hiding.**
Set `visibility_range_end` on far-field MultiMeshInstance3D nodes to hide chunks beyond
a certain distance. This complements the stream radius by providing a smooth fade-out
instead of a hard unload.

### Priority

1. **Option A (DONE)**: Dual-mesh tier with icosahedron far-field — 99.4% far-field reduction
2. **Proposal B-1 (NEXT)**: `generate_lods()` on near-field mesh — estimated 30-50% near-field
   reduction for distant near-chunks
3. **Proposal B-3 (FUTURE)**: Visibility Ranges for smooth far-field fade — quality improvement
4. **Proposal B-2 (OPTIONAL)**: Per-chunk `lod_bias` tuning — marginal gain

---

## Implementation Sketch for Proposal B-1

```gdscript
# In ChunkStreamManager._build_mesh_cache(), after building near mesh:
if _asteroid_mesh_cache:
    # Generate automatic LOD tiers for the near-field mesh.
    # This creates lower-poly variants that Godot selects based on screen-space coverage.
    # For near-field RigidBody3D asteroids (individual MeshInstance3D nodes), Godot will
    # automatically use lower LOD tiers for distant asteroids within the near stream radius.
    _asteroid_mesh_cache.generate_lods(40.0, 60.0, [])
    print("[ChunkStreamManager] Near mesh LOD tiers: %d" % _asteroid_mesh_cache.get_lod_count())
```

**Estimated impact**: Near-field chunks at the edge of the stream radius (0.04 AU = 6M km)
would use the lowest auto-generated LOD tier (~300-500 prims instead of 2,700). With 81 near
chunks, ~40 of which are at the edge, this could reduce near-field prims from ~1.5M to ~800K.

---

## Citations

1. Godot 4.7 Mesh LOD docs: https://docs.godotengine.org/en/4.7/tutorials/3d/mesh_lod.html
2. Godot 4.7 Visibility Ranges docs: https://docs.godotengine.org/en/stable/tutorials/3d/visibility_ranges.html
3. godot-proposals #10669: https://github.com/godotengine/godot-proposals/issues/10669
4. Godot issue #76436: https://github.com/godotengine/godot/issues/76436
5. Godot issue #95948: https://github.com/godotengine/godot/issues/95948
6. Godot PR #92290: https://github.com/godotengine/godot/pull/92290
7. Godot 4.7 Optimizing 3D Performance: https://docs.godotengine.org/en/4.7/tutorials/performance/optimizing_3d_performance.html
8. Godot ArrayMesh docs: https://docs.godotengine.org/en/stable/classes/class_arraymesh.html
9. Godot MultiMesh performance: https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html

---

## Council Verdict — Chairman Inline Synthesis

**Note**: Subagent quota was exhausted during Stage 1 dispatch. Per the `council_runner`
skill's Runtime Adaptation clause ("Generic: sequential inline"), the Chairman performed
inline synthesis across all five lenses as a fallback.

### Scoring

#### Proposal B-1: `generate_lods()` on near-field mesh

| Lens | Score | Weight | Weighted |
|---|---|---|---|
| Coherence | 9 | 0.20 | 1.80 |
| Capability | 8 | 0.20 | 1.60 |
| Safety | 9 | 0.25 | 2.25 |
| Efficiency | 9 | 0.15 | 1.35 |
| Evolution | 8 | 0.20 | 1.60 |
| **Total** | | | **8.60** |

**Verdict: PASSED (5/5 passing, weighted 8.60 ≥ 6.5)**

- Coherence (9): Fits the existing `_build_mesh_cache()` pattern — one extra API call.
- Capability (8): Genuine expansion — adds automatic LOD for individual near-field asteroids.
- Safety (9): `generate_lods()` is a standard Godot API with no side effects.
- Efficiency (9): ~3 lines for estimated 30-50% near-field reduction — excellent ROI.
- Evolution (8): Opens door for future LOD tuning without constraining anything.

#### Proposal B-2: Per-chunk `lod_bias` tuning

| Lens | Score | Weight | Weighted |
|---|---|---|---|
| Coherence | 7 | 0.20 | 1.40 |
| Capability | 5 | 0.20 | 1.00 |
| Safety | 9 | 0.25 | 2.25 |
| Efficiency | 5 | 0.15 | 0.75 |
| Evolution | 6 | 0.20 | 1.20 |
| **Total** | | | **6.60** |

**Verdict: PASSED (3/5 passing, weighted 6.60 ≥ 6.5)** — but marginal, lowest priority.

- Coherence (7): Fits chunk system but adds per-chunk state management complexity.
- Capability (5): Marginal — `lod_bias` is just a multiplier on existing LOD selection.
- Safety (9): No risk — standard GeometryInstance3D property.
- Efficiency (5): Marginal gain for added complexity — hard-coded bias values are fragile.
- Evolution (6): Closes some doors — per-chunk bias tuning doesn't scale well.

#### Proposal B-3: Visibility Ranges for far-field fade

| Lens | Score | Weight | Weighted |
|---|---|---|---|
| Coherence | 8 | 0.20 | 1.60 |
| Capability | 7 | 0.20 | 1.40 |
| Safety | 9 | 0.25 | 2.25 |
| Efficiency | 7 | 0.15 | 1.05 |
| Evolution | 8 | 0.20 | 1.60 |
| **Total** | | | **7.90** |

**Verdict: PASSED (5/5 passing, weighted 7.90 ≥ 6.5)**

- Coherence (8): Replaces hard chunk unload with smooth fade — fits streaming pattern.
- Capability (7): Quality improvement — smooth transitions eliminate pop-in.
- Safety (9): No risk — `visibility_range` is a standard GeometryInstance3D property.
- Efficiency (7): Saves rendering chunks about to unload, but marginal vs icosahedron.
- Evolution (8): Opens door for HLOD system (billboard impostors at extreme range).

### Priority Order

1. **B-1 (PASS, 8.60)** — Implement first. Highest ROI: ~3 lines for 30-50% near-field reduction.
2. **B-3 (PASS, 7.90)** — Implement second. Quality improvement for far-field transitions.
3. **B-2 (PASS, 6.60)** — Optional/deferred. Marginal gain, adds tuning complexity.

### Chairman Summary

All three proposals pass, but with clear priority stratification. B-1 is the highest-value
next step — it directly addresses the remaining near-field bottleneck (~1.5M prims) with
minimal code and no risk. B-3 is a quality improvement that complements the existing chunk
streaming. B-2 is marginal and should be deferred unless B-1 alone is insufficient.

**Recommended next action**: Implement B-1 (`generate_lods()` on near-field mesh), re-run
the GPU profile and primitive audit, then evaluate whether B-3 is needed for visual quality.

---

## Post-Implementation Report: B-1 REVERTED — Net-Negative Performance Impact

**Date**: 2026-08-20 (same day, post-implementation)
**Verdict**: B-1 implemented, measured, and **reverted** — performance decreased.

### Implementation

`generate_lods()` was called on the near-field mesh via `ImporterMesh` intermediate:
- `ImporterMesh.from_mesh(_asteroid_mesh_cache)`
- `imesh.generate_lods(40.0, 60.0, [])`
- `_asteroid_mesh_cache = imesh.get_mesh(_asteroid_mesh_cache)`

### LOD tiers successfully generated

```
LOD 0: 1350 triangles (size=0.44)  — original detail
LOD 1:  674 triangles (size=0.88)  — 50% reduction
LOD 2:  336 triangles (size=1.88)  — 75% reduction
LOD 3:  168 triangles (size=4.44)  — 87% reduction
LOD 4:  108 triangles (size=11.60) — 92% reduction
```

The meshoptimizer library successfully generated 5 LOD tiers with progressive decimation.

### Measured performance impact (non-headless, Apple M4, ULTRA)

| Metric | Option A only | B-1 implemented | Delta |
|---|---|---|---|
| Flight idle FPS | 31.2 | 29.7 | **-1.5 (-4.8%)** |
| Map open FPS | 46.0 | 44.2 | -1.8 (-3.9%) |
| Warp FPS | 46.8 | 46.0 | -0.8 (-1.7%) |
| Post-warp FPS | 46.6 | 46.0 | -0.6 (-1.3%) |
| Primitives | 5,718,276 | 5,680,192 | -38,084 (-0.7%) |
| Draw calls | 191.6 | 186.2 | -5.4 (-2.8%) |

### Root cause of negative impact

1. **Only 8 near chunks loaded** (not 81 as estimated). The near stream radius of 4 chunks
   produces a 9×9 grid in theory, but only 8 chunks were active during profiling. The total
   near-field primitive count is ~324K, not ~1.5M as estimated.

2. **CPU LOD selection overhead**. Each near-field asteroid is an individual MeshInstance3D
   on a RigidBody3D. Godot performs per-frame LOD selection for each node. With ~120 near-field
   asteroids, the CPU cost of LOD selection outweighs the GPU savings from lower primitive counts.

3. **Most near-field asteroids are close enough to use LOD 0**. The near stream radius is only
   0.04 AU (6M km), but the ship starts at the center of the near chunk grid. Most asteroids
   are within 1-2 chunks of the ship, close enough that Godot selects LOD 0 or LOD 1.

4. **The 38K primitive reduction (12% of near-field)** came from the few asteroids at the edge
   of the near stream radius that qualified for LOD 2-4. This GPU saving was smaller than the
   CPU cost of per-frame LOD selection across all ~120 nodes.

### Council self-correction

The council's estimated 30-50% near-field reduction was incorrect because:
- The estimate assumed 81 near chunks; only 8 were active
- The estimate did not account for CPU LOD selection overhead
- The estimate assumed most near-field asteroids would use lower LOD tiers; in practice,
  most are close enough to the ship to use LOD 0

### Decision

**B-1 reverted.** The near-field mesh remains at subdivision_level=2 without LOD tiers.
The code change has been removed from `ChunkStreamManager.gd`.

### Revised next steps

With B-1 rejected and B-2 deferred (marginal), the remaining viable optimization is:
- **B-3 (Visibility Ranges for far-field fade)**: Quality improvement, not a primitive reduction
- **Option C (reduce far stream radius from 3 to 2)**: Reduces far chunk count from ~57 to ~25
- **Option D (reduce near stream radius from 4 to 3)**: Reduces near chunk count
- **Near-field mesh simplification**: Reduce subdivision_level from 2 to 1 (~384 prims instead
  of ~2,700) — but this affects visual quality for close-range asteroids

The current performance (31.2 FPS flight idle, 46+ FPS in map/warp) may be acceptable for
the current development stage. Further optimization should be deferred until the full gameplay
loop is in place and we can profile with real combat, debris, and VFX load.

---
**Status**: B-1 REVERTED — Council self-correction recorded
