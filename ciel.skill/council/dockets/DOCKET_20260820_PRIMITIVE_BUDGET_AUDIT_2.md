# COUNCIL DOCKET: 20260820_PRIMITIVE_BUDGET_AUDIT_2

**Date**: 2026-08-20
**Candidate Artifact**: BioGenesis-X flight scene — remaining primitive budget after Option A
**Scope**: `invocation_scopes/SKILL_INTEGRATION.md` (post-implementation audit, round 2)
**Predecessor**: DOCKET_20260820_PRIMITIVE_BUDGET_AUDIT.md (Option A — icosahedron far-field)

## Situation

After implementing Option A (icosahedron far-field, 99.4% far-field reduction) and
reverting B-1 (generate_lods on near-field, net-negative), the flight scene still renders
**5.72M primitives per frame at 30.2 FPS** on Apple M4 ULTRA. A full per-node breakdown
revealed the actual sources of the remaining primitives.

## Evidence

### Full per-node primitive breakdown (GPU profile, FLIGHT_IDLE entry)

| System | MeshInstance3D | RigidBody3D | MultiMesh | Total prims | % |
|---|---|---|---|---|---|
| ChunkStreamManager | 967 / 2.48M | 756 / 2.04M | 69 / 27.7K | 4.52M | 79% |
| AsteroidField | 68 / 381K | 60 / 381K | 0 | 762K | 13% |
| UniverseManager | 43 / 56K | 0 | 0 | 56K | 1% |
| PlayerShip | 4 / 56K | 0 | 0 | 56K | 1% |
| Starfield | 0 | 0 | 1 / 24K | 24K | 0.4% |
| **Total** | **1,087 / 2.97M** | **816 / 2.42M** | **70 / 51.7K** | **5.44M** | **95%** |

### ChunkStreamManager breakdown (4.52M prims, 79%)

- **756 RigidBody3D near-field asteroids**: 81 near chunks (9×9 grid, NEAR_STREAM_RADIUS_CHUNKS=4)
  × ~10 asteroids × 2,700 prims = 2.04M prims
- **967 standalone MeshInstance3D nodes**: 2.48M prims. These include:
  - AnomalyBeacons: ~30 × 4,224 prims = 127K prims
  - Near-field asteroid MeshInstance3D children (inside RigidBody3D, counted separately)
  - Other chunk geometry (hazard markers, enemy drones, etc.)
- **69 MultiMesh far-field chunks**: 57 chunks × ~20 instances × 20 prims = 22.6K prims (Option A working)

### AsteroidField breakdown (762K prims, 13%)

- **60 RigidBody3D asteroids**: 60 × 2,700 prims = 162K prims (with MeshInstance3D children)
- **68 standalone MeshInstance3D nodes**: 381K prims (visual effects, dust, etc.)
- This is a SEPARATE system from ChunkStreamManager — both spawn RigidBody3D asteroids
  with the same 2,700-prim mesh. They overlap spatially and functionally.

### Bug fix applied during investigation

- **Duplicate chunk loading**: `_pending_chunks` Dictionary added to track chunks dispatched
  to worker threads but not yet mounted. Previously, `_update_near_chunks()` re-queued the
  same chunk every frame until it appeared in `_active_near_chunks`, causing duplicate
  thread dispatches and duplicate RigidBody3D nodes.

## Proposed remediation options

### Option F: Disable AsteroidField (recommended, highest ROI)

AsteroidField spawns 60 RigidBody3D asteroids + 68 MeshInstance3D nodes (762K prims)
that are redundant with ChunkStreamManager's near-field system. Both use the same mesh
and overlap spatially.

**Estimated impact**: -762K prims (13% reduction). FPS: 30.2 → ~33-35.
**Complexity**: Low — set `asteroid_count = 0` or remove AsteroidField node from scene.
**Trade-off**: Lose AsteroidField's visual variety (dust, cosmic space dust, target drones).
May need to migrate non-asteroid features to ChunkStreamManager.

### Option G: Reduce near stream radius (4 → 3)

Reduce `NEAR_STREAM_RADIUS_CHUNKS` from 4 to 3. Near chunk grid drops from 9×9=81 to 7×7=49.

**Estimated impact**: -32 near chunks × ~10 asteroids × 2,700 prims = -864K prims (15%).
FPS: 30.2 → ~34-36.
**Complexity**: One constant change.
**Trade-off**: Fewer physics-enabled asteroids near the ship. Edge of near field is 0.03 AU
instead of 0.04 AU. May notice pop-in when far-field MultiMesh replaces RigidBody3D closer
to the ship.

### Option H: Reduce AnomalyBeacon mesh complexity

AnomalyBeacons use a 4,224-prim mesh. ~30 beacons = 127K prims. Replace with a simpler
beacon mesh (cylinder + small sphere, ~100 prims).

**Estimated impact**: -117K prims (2% reduction). Marginal FPS gain.
**Complexity**: Low — replace beacon mesh.
**Trade-off**: Less detailed beacon visuals. Beacons are small distant objects — likely
imperceptible.

### Option I: Reduce near asteroid count per chunk (15 → 8)

Reduce `MAX_ASTEROIDS_PER_NEAR_CHUNK` from 15 to 8.

**Estimated impact**: -81 chunks × ~5 asteroids × 2,700 prims = -1.09M prims (19%).
FPS: 30.2 → ~37-40.
**Complexity**: One constant change.
**Trade-off**: Sparser asteroid fields in near field. Half the asteroids per chunk.

### Option J: Use icosahedron for near-field too

Use the icosahedron far-field mesh for near-field RigidBody3D asteroids as well.

**Estimated impact**: -2.04M prims → -30K prims (near-field). Total: 5.72M → ~3.7M.
FPS: 30.2 → ~45-50.
**Complexity**: One line change (use `_asteroid_mesh_cache_far` for near chunks too).
**Trade-off**: Near-field asteroids lose all surface detail. Close-range asteroids will
look like flat-shaded icosahedrons. Visual quality regression for the core gameplay area.

### Option K: Combine F + G + I (recommended combination)

Disable AsteroidField + reduce near radius to 3 + reduce near count to 8.

**Estimated impact**:
- AsteroidField: -762K
- Near radius: -864K
- Near count: -540K (49 chunks × ~4 fewer × 2,700)
- Total: -2.17M prims (38% reduction). FPS: 30.2 → ~42-48.
**Trade-off**: Sparser asteroids, no AsteroidField extras, slightly smaller near field.

## Files involved

- `scripts/AsteroidField.gd` — standalone asteroid field system
- `scripts/ChunkStreamManager.gd` — chunk streaming (near radius, asteroid count)
- `scenes/space_flight.tscn` — scene hierarchy (AsteroidField node)

## Recommendation

**Option K (combine F + G + I)** as the primary intervention. This gives ~38% primitive
reduction with minimal code change and preserves near-field visual quality (still using
the 2,700-prim mesh for close-range asteroids). Option J (icosahedron for near) is the
nuclear option if more aggressive reduction is needed later.

## Verification plan

1. Implement approved options
2. Re-run `test_gpu_profile.gd` non-headless to measure FPS improvement
3. Re-run `test_primitive_audit.gd` to confirm primitive count reduction
4. Visually verify asteroid density is still acceptable
5. Run `test_clean_debugger_audit.gd` for lint/error check

---

## Council Verdict — Chairman Inline Synthesis

**Note**: Subagent quota exhausted. Per `council_runner` fallback ("Generic: sequential
inline"), Chairman performed inline synthesis across all five lenses.

### Scoring Summary

| Option | Description | Weighted | Passing | Verdict |
|---|---|---|---|---|
| **K** | Combine F+G+I | **8.30** | 5/5 | **PASS** |
| F | Disable AsteroidField | 7.95 | 5/5 | PASS |
| I | Reduce near count 15→8 | 7.80 | 5/5 | PASS |
| G | Reduce near radius 4→3 | 7.65 | 5/5 | PASS |
| J | Icosahedron for near-field | 6.75 | 3/5 | PASS (marginal) |
| H | Reduce AnomalyBeacon mesh | 6.45 | 2/5 | DEADLOCK |

### Per-option rationale

**Option K (PASS, 8.30)** — Highest score. Combines three independently-tunable
reductions for 38% total primitive removal. All three components (F, G, I) individually
pass with 5/5. The combination preserves near-field visual quality (still 2,700-prim
mesh for close asteroids) while cutting redundant systems and excess density.

**Option F (PASS, 7.95)** — AsteroidField is functionally redundant with
ChunkStreamManager. Both spawn RigidBody3D asteroids with the same mesh. Safety flagged
at 8 (not 9) because gameplay dependencies on AsteroidField's dust/drones need verification.

**Option G (PASS, 7.65)** — Simple constant change. Far-field MultiMesh covers the
gap when near chunks unload. Minor pop-in risk at the near/far boundary.

**Option I (PASS, 7.80)** — Halving near asteroid density is the single highest-impact
constant change (1.09M prims). Density is easily tunable later.

**Option J (PASS, 6.75, marginal)** — Massive efficiency gain (10/10) but Capability (4)
and Evolution (4) flag the visual quality regression for core gameplay. Approved as a
fallback if K is insufficient, but not recommended for implementation now.

**Option H (DEADLOCK, 6.45)** — Only 2/5 passing. AnomalyBeacons are 127K prims (2%),
not worth the effort. Efficiency scored 4 — the ROI is too low.

### Chairman Summary

Option K is the clear winner with a weighted score of 8.30. It combines three safe,
independently-tunable changes that together remove 2.17M primitives (38% reduction)
without degrading near-field visual quality. The expected FPS improvement is 30.2 → ~42-48.

Option J (icosahedron for near-field) is approved but deferred as a nuclear option —
it would push FPS to ~50 but at unacceptable visual quality cost for the core gameplay
area where players see asteroids up close.

### Mitigations Required

1. **Verify AsteroidField dependencies** — Before disabling, check if any gameplay
   scripts reference AsteroidField nodes (dust, drones, signals). Migrate needed
   features to ChunkStreamManager if required.
2. **Monitor pop-in at near/far boundary** — With radius 3, the boundary is closer.
   If pop-in is noticeable, add a smooth transition or intermediate LOD tier.

### Next Action

Implement Option K (F + G + I): disable AsteroidField, reduce NEAR_STREAM_RADIUS_CHUNKS
to 3, reduce MAX_ASTEROIDS_PER_NEAR_CHUNK to 8. Then verify with GPU profile and audit.

---
**Status**: APPROVED BY COUNCIL (Chairman inline synthesis, subagent quota fallback)
