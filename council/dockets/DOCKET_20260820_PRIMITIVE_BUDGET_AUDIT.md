# COUNCIL DOCKET: 20260820_PRIMITIVE_BUDGET_AUDIT

**Date**: 2026-08-20
**Candidate Artifact**: BioGenesis-X flight scene rendering performance
**Scope**: `invocation_scopes/SKILL_INTEGRATION.md` (post-implementation performance audit)
**Status**: APPROVED BY COUNCIL OF FIVE

## Situation

Non-headless GPU profiling of the full game session (boot → menu → flight →
map → warp → post-warp) revealed that **flight idle runs at 25.9 FPS** on an
Apple M4 (ULTRA quality tier), with 5.75M primitives rendered across 2,453
objects per frame. Frame time is 33.4ms, of which physics is only 5.36ms (16%)
and rendering is ~28ms (84%). The primitive count is the dominant bottleneck.

## Evidence

### GPU Profile (non-headless, Apple M4, ULTRA tier)

| Phase | FPS | Frame Time | Draw Calls | Objects | Primitives |
|---|---|---|---|---|---|
| Boot | 89.9 | 18.4ms | 1 | 1 | 2 |
| Main Menu | 118.9 | 15.2ms | 192 | 394 | 13,426 |
| Loading Flight | 25.5 | 148.3ms | 195 | 2,267 | 5.17M |
| **Flight Idle** | **25.9** | **33.4ms** | **196** | **2,453** | **5.75M** |
| Map Open | 37.4 | 23.3ms | 197 | 2,454 | 5.75M |
| Warp | 38.2 | 22.9ms | 197 | 2,454 | 5.75M |
| Post-Warp | 37.1 | 23.3ms | 196 | 2,454 | 5.75M |

### Per-Node Primitive Audit (`test_primitive_audit.gd`)

The audit found **0 regular MeshInstance3D nodes** and **76 MultiMeshInstance3D
nodes**. All 5.75M primitives come from MultiMesh instances:

#### 1. Far-field asteroids (MultiMesh) — ~4.05M prims (70.4%)

- **75 far chunks** loaded (FAR_STREAM_RADIUS_CHUNKS=3, 7×7=49 grid + stale)
- **~20 asteroids per chunk** (MAX_ASTEROIDS_PER_FAR_CHUNK=40, density-dependent)
- **2,700 primitives per asteroid** (subdivision_level=2 → resolution=16)
- Total: 75 × 20 × 2,700 = **4,050,000 primitives**
- Source: `ChunkStreamManager._mount_far_chunk_from_data()` at line 544
- Mesh: `_asteroid_mesh_cache` — single shared ArrayMesh built at line 161

#### 2. Near-field asteroids (RigidBody3D + MeshInstance3D) — ~1.5M prims (est. 26%)

- **81 near chunks** loaded (NEAR_STREAM_RADIUS_CHUNKS=4, 9×9=81 grid)
- **Up to 15 asteroids per chunk** (MAX_ASTEROIDS_PER_NEAR_CHUNK=15)
- **2,700 primitives per asteroid** — SAME mesh as far-field
- Total: up to 81 × 15 × 2,700 = 3,280,500 (partially loaded, est. ~1.5M actual)
- Source: `ChunkStreamManager._mount_near_chunk_from_data()` at line 590
- Mesh: same `_asteroid_mesh_cache`

#### 3. Starfield (MultiMesh) — 24K prims (0.4%)

- 12,000 star instances × 2 prims each = 24,000
- Negligible

#### 4. Other (planets, player ship, etc) — ~0.2M prims (est. 3%)

- UniverseManager: HostStar + PlanetaryBodies (procedural, loaded async)
- ProceduralBioMesh: player ship hull
- AsteroidField: 5 children (0 prims — likely waiting for generation)

### Root cause

**Both far-field and near-field asteroids use the same mesh** — a single
`_asteroid_mesh_cache` built with `subdivision_level=2` (resolution=16,
~2,700 triangles). This mesh is appropriate for near-field interaction but
massively over-detailed for far-field chunks at 1-3 AU distance where
asteroids are sub-pixel specks.

The mesh is built once in `_build_mesh_cache()` (line 161) and shared across
all far MultiMesh instances and near RigidBody3D instances. There is no LOD
or mesh-tier system for asteroids.

## Proposed remediation options

### Option A: Dual-mesh tier (recommended)

Build two asteroid meshes:
- **Near mesh**: subdivision_level=2 (current, ~2,700 prims) — for RigidBody3D
  near-field chunks where players can see detail
- **Far mesh**: subdivision_level=1 (resolution=8, ~384 prims) or a simple
  icosahedron (~20 prims) — for MultiMesh far-field chunks

**Estimated impact**: Far-field prims drop from 4.05M → 405K (10× reduction
with sub_level=1) or 60K (67× reduction with icosahedron). Total scene prims
drop from ~5.75M → ~2.1M (sub_level=1) or ~1.75M (icosahedron). Expected FPS
improvement: 25.9 → ~45-55 FPS.

**Complexity**: Low. Add `_asteroid_mesh_cache_far` alongside existing
`_asteroid_mesh_cache`. Build in `_build_mesh_cache()`. Use far mesh in
`_mount_far_chunk_from_data()`.

### Option B: Godot mesh LOD

Enable Godot's built-in mesh LOD on the asteroid mesh. Generate LOD tiers
via `ArrayMesh.generate_lods()` after building the mesh. Godot will
automatically select lower-LOD tiers based on screen-space coverage.

**Estimated impact**: Similar to Option A but automatic — Godot picks the
right LOD per instance based on distance. However, MultiMesh may not
support per-instance LOD selection (all instances use the same LOD tier).

**Complexity**: Medium. Need to verify MultiMesh LOD behavior.

### Option C: Reduce far stream radius

Reduce `FAR_STREAM_RADIUS_CHUNKS` from 3 to 2 (5×5=25 chunks instead of 7×7=49).

**Estimated impact**: Far chunks drop from 75 → ~25. Far prims drop from
4.05M → ~1.35M. Total scene prims drop to ~3.05M. Expected FPS: ~35-40.

**Trade-off**: Fewer visible asteroids at extreme distance. May notice
pop-in when traveling.

### Option D: Reduce far asteroid count

Reduce `MAX_ASTEROIDS_PER_FAR_CHUNK` from 40 to 15.

**Estimated impact**: Far asteroids per chunk drop from ~20 → ~10. Far prims
drop from 4.05M → ~2.03M. Total scene prims drop to ~3.73M. Expected FPS:
~32-38.

**Trade-off**: Sparser asteroid fields at distance.

### Option E: Combine A + C + D

Dual-mesh tier + reduced far radius (2) + reduced far count (20).

**Estimated impact**: Far prims: 25 chunks × 10 asteroids × 384 prims =
96,000. Total scene prims: ~1.66M. Expected FPS: ~55-65.

## Files involved

- `scripts/ChunkStreamManager.gd` — mesh cache, chunk mounting, stream radius
- `scripts/ProceduralAsteroidMesh.gd` — mesh generation, subdivision levels
- `scripts/AsteroidField.gd` — standalone asteroid field (separate from chunks)

## Recommendation

**Option A (dual-mesh tier)** as the primary intervention, with **Option C
(reduce far radius to 2)** as a secondary tuning step if needed. This gives
the largest performance gain (~2× FPS improvement) with minimal code change
and no visual quality loss for near-field gameplay where asteroid detail
matters.

## Verification plan

1. Implement dual-mesh tier in `ChunkStreamManager._build_mesh_cache()`
2. Re-run `test_gpu_profile.gd` non-headless to measure FPS improvement
3. Re-run `test_primitive_audit.gd` to confirm primitive count reduction
4. Visually verify far-field asteroids still look acceptable at distance
5. Verify near-field asteroid detail is unchanged
6. Run `test_clean_debugger_audit.gd` for lint/error check

---

## Stage 1: Independent Member Evaluation

### 1. Coherence (`members/Coherence.md`)
- **Score**: 9/10
- **Rationale**: Option A perfectly extends the existing dual-tier architecture pattern in ChunkStreamManager.gd (lines 70-76: LOD enum, lines 33-40: far/near separation). The proposal adds `_asteroid_mesh_cache_far` alongside the existing `_asteroid_mesh_cache` (line 116), following the established dual-cache pattern already used for `_asteroid_collision_cache` (line 118). This mirrors the multi-variant caching pattern in AsteroidField.gd's `_cached_asteroid_meshes` array (line 10) and BioTextureGenerator's texture cache. The change is architecturally clean—far-field rendering logic remains isolated in `_mount_far_chunk_from_data()` (line 549) with no cross-contamination of near-field physics logic in `_spawn_physics_asteroid_from_data()` (line 652).
- **Flags**: none

### 2. Capability (`members/Capability.md`)
- **Score**: 9/10
- **Rationale**: Option A adds genuine distance-based mesh selection capability that is NOT redundant with Godot's built-in LOD. Godot's MultiMesh LOD applies uniformly to all instances (docs confirm 'all instances will be drawn with the same LOD level'), so per-instance distance-based selection requires separate mesh tiers. The solution directly addresses the root cause (4.05M far-field prims using high-detail mesh at ChunkStreamManager.gd:544) with minimal scope—leveraging existing subdivision_level support (ProceduralAsteroidMesh.gd:50-51) and dual-tier architecture. Estimated 10-67× primitive reduction (4.05M → 405K/60K) directly targets the 5.75M bottleneck.
- **Flags**: none

### 3. Safety (`members/Safety.md`) — Veto Authority
- **Score**: 9/10
- **Rationale**: Option A introduces minimal risk: one additional ArrayMesh resource (negligible memory footprint), no collision system impact (near-field chunks retain existing mesh at ChunkStreamManager.gd:652, collision derived from high-detail mesh at line 173), and mesh cache initialization is synchronous in _ready() at line 145 with no race conditions. The far/near mesh mismatch is acceptable because asteroids are sub-pixel specks at 1-3 AU distance, making visual popping imperceptible during the existing MultiMesh→RigidBody3D transition.
- **Flags**: `visual_popping_far_near_transition`
- **Veto**: No (score 9 > 3)

### 4. Efficiency (`members/Efficiency.md`)
- **Score**: 8/10
- **Rationale**: The proposal delivers a well-quantified 10× primitive reduction (4.05M → 405K far prims) with minimal code addition (~10 lines in ChunkStreamManager.gd lines 116, 161-174, 549). It directly addresses the root cause identified in the docket (lines 65-73): over-detailed far-field meshes at 1-3 AU distance where asteroids are sub-pixel. However, the icosahedron alternative (~20 prims, 67× reduction) would be more efficient than subdivision_level=1 (~384 prims) and should be the primary choice for far-field rendering.
- **Flags**: `icosahedron more efficient than sub_level=1`, `consider icosahedron as primary far mesh`

### 5. Evolution (`members/Evolution.md`)
- **Score**: 9/10
- **Rationale**: Option A extends the existing dual-tier architecture (ChunkStreamManager.gd lines 32-41, 69-76) by adding a second mesh cache, which is additive and doesn't constrain future expansion. The LOD enum already defines 5 tiers (FULL_PHYSICS, SIMPLIFIED, MULTIMESH, BILLBOARD, INVISIBLE) with only 2 currently used, leaving room for 3+ tier extension later. The mesh generation system (ProceduralAsteroidMesh.gd line 50) already supports subdivision_level parameterization, making lower-poly variants straightforward to generate for future asteroid types, debris fields, or planetary rings.
- **Flags**: none

---

## Stage 2: Cross-Review & Anonymized Delta Check

### Evolution (held → revised)
- **Stage 1**: 9 → **Stage 2**: 8 (delta: -1)
- **Rationale**: Peer D's technical observation about icosahedron (~20 prims, 67× reduction) being significantly more efficient than subdivision_level=1 (~384 prims) is valid and highlights that Option A does not select the optimal far-field mesh variant. While the architectural approach remains sound, the efficiency gap warrants a score reduction.
- **Challenge of**: D (Efficiency)
- **Flags**: `icosahedron_more_efficient_than_sub_level_1`

### Efficiency (held)
- **Stage 1**: 8 → **Stage 2**: 8 (delta: 0)
- **Rationale**: Peers A, B, C, and E provide strong architectural arguments for dual-tier consistency, extensibility, and risk mitigation, but none addressed the icosahedron efficiency concern. Since my flag remains unchallenged and the 67× primitive reduction advantage over subdivision_level=1 is material to the efficiency mandate, I hold at 8.
- **Flags**: `icosahedron more efficient than sub_level=1`

### Coherence, Capability, Safety (held at 9)
- No delta. All three maintained their Stage 1 scores. No challenges raised against their rationales.

---

## Stage 3: Chairman Synthesis & Voting Result

- **Voting Tally**: 5/5 Pass votes (all members ≥ 6)
- **Weighted Score**: 8.65 / 10.0
  - Coherence: 9 × 0.20 = 1.80
  - Capability: 9 × 0.20 = 1.80
  - Safety: 9 × 0.25 = 2.25
  - Efficiency: 8 × 0.15 = 1.20
  - Evolution: 8 × 0.20 = 1.60
  - **Total**: 8.65
- **Safety Veto Check**: PASS (Safety = 9, well above floor of 3)
- **Pivotal Lens**: Efficiency (largest deviation from mean, flagged icosahedron optimization)
- **Decision**: **PASSED & RATIFIED**

### Chairman Summary

The council unanimously approves Option A (dual-mesh tier) with a weighted score of 8.65. The architectural fit is excellent — it extends the existing dual-tier pattern without fragmentation. Safety confirms no risk to collision or stability. The sole contention is Efficiency's flag that an icosahedron (~20 prims) would be 19× more efficient than subdivision_level=1 (~384 prims) for far-field rendering. This is a valid optimization that should be incorporated into the implementation: **use an icosahedron or very-low-poly primitive as the far-field mesh** rather than subdivision_level=1. The dual-tier architecture supports either mesh variant equally well.

### Mitigations Required

1. **Use icosahedron (or equivalent low-poly primitive) as far-field mesh** — not subdivision_level=1. This addresses Efficiency and Evolution's Stage 2 flag.
2. **Visual popping flag from Safety** — acceptable at 1-3 AU distance where asteroids are sub-pixel. Monitor during playtesting; if popping is noticeable during chunk transitions, add a cross-fade or intermediate LOD tier.

### Next Action

Implement Option A with icosahedron far-field mesh, then verify with `test_gpu_profile.gd` and `test_primitive_audit.gd`.

---
**Status**: APPROVED BY COUNCIL OF FIVE
