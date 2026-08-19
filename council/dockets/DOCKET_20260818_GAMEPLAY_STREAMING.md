# COUNCIL DOCKET: 20260818_GAMEPLAY_STREAMING

**Date**: 2026-08-18
**Candidate Artifact**: Gameplay Element Streaming Architecture for BioGenesis-X
**Scope**: Architecture Council (SELF_MODIFICATION equivalent — new core system)

---

## Stage 1: Independent Member Evaluation

### 1. Coherence (Score: 8)
- **Rationale**: Approach C maintains all existing patterns — signal decoupling, sibling lookup, preload const, @tool preservation, floating origin group compatibility.
- **Flags**: chunk_grid_conflict, asteroid_field_tool_preservation, floating_origin_group_compatibility
- **Recommends**: C

### 2. Capability (Score: 7)
- **Rationale**: Approach A is the only option that genuinely expands capability. Game already has chunk streaming for terrain. SystemNoiseField API lacks bulk region sampling.
- **Flags**: existing_streaming_infrastructure, wave_speed_mismatch, noise_api_limitation
- **Recommends**: A

### 3. Safety (Score: 2 — VETO)
- **Rationale**: Approach A has critical floating origin interaction risk, physics stability risk with kinematic RigidBody3D, determinism broken (unseeded randf()), signal leak risk.
- **Flags**: floating_origin_mismatch, physics_kinematic_instability, non_deterministic_spawns, signal_leak_risk, editor_tool_breakage
- **Recommends**: C
- **Veto**: YES

### 4. Efficiency (Score: 8)
- **Rationale**: A's chunking efficient (1.25 chunk load/sec at Wave speed). Jolt handles 1000+ bodies. MultiMesh for far-field 10-100x faster.
- **Flags**: multiMesh_opportunity, noise_sampling_cheap, chunk_load_rate_safe
- **Recommends**: A

### 5. Evolution (Score: 7)
- **Rationale**: A supports growth trajectory: multiplayer, modding, seamless space-to-surface. Safety's concerns are fixable implementation issues.
- **Flags**: floating_origin_group_fix_required, determinism_seeding_required, save_load_infrastructure_needed
- **Recommends**: A

---

## Chairman's Mitigated Approach A'

To address Safety's veto, Chairman proposed Approach A with mandatory mitigations:
1. Floating origin fix: All streamed elements join 'celestial_bodies' group
2. Physics stability fix: Far-field = MultiMesh only, near-field 0.01 AU = RigidBody3D at rest
3. Determinism fix: Seeded RNG per chunk from system_seed ^ chunk_coords
4. Signal leak fix: Explicit disconnect before queue_free(), connection registry
5. @tool preservation: AsteroidField.gd NOT rewritten, new ChunkStreamManager.gd
6. SystemNoiseField API extension: Add sample_channel_region() for bulk queries

---

## Stage 2: Cross-Review & Anonymized Delta Check

### 1. Coherence (Final: 7, Delta: -1)
- Held. Flagged architectural duplication with PlanetTerrainGenerator and unverified bulk sampling API dependency.

### 2. Capability (Final: 8, Delta: +1)
- Moved up. Challenged Safety's veto: concerns are implementation details, not architectural flaws. Seeded RNG pattern exists in SystemNoiseField. Signal cleanup pattern exists in DescentAudioController.

### 3. Safety (Final: 7, Delta: +5 — VETO LIFTED)
- Mitigations address all 5 risk vectors. ChunkStreamManager doesn't exist yet — specifications not code. Recommends implementation verification before production.

### 4. Efficiency (Final: 8, Delta: 0)
- Held. Mitigations preserve performance profile. MultiMesh far-field and at-rest near-field maintain efficiency.

### 5. Evolution (Final: 8, Delta: +1)
- Moved up. Chunk streaming for entities is architecturally distinct from terrain GPU compute chunks. Specification-first is appropriate for growth.

---

## Stage 3: Chairman Synthesis & Voting Result

- **Voting Tally**: 5/5 Pass votes
- **Weighted Score**: 7.55 / 10.0
- **Safety Veto Check**: PASS (7, veto lifted)
- **Pivotal Lens**: Safety (largest delta: +5, veto to pass)
- **Decision**: PASSED & RATIFIED

### Mandatory Mitigations (must be implemented)
1. All streamed gameplay elements MUST be added to 'celestial_bodies' group
2. Far-field chunks (1 AU) MUST use MultiMesh only — zero RigidBody3D
3. Near-field chunks (0.01 AU) spawn RigidBody3D at rest (freeze=true)
4. Each chunk MUST use seeded RNG: system_seed ^ hash(chunk_x, chunk_z)
5. Despawn protocol MUST: disconnect signals → remove from groups → queue_free()
6. AsteroidField.gd MUST NOT be modified — new ChunkStreamManager.gd handles streaming
7. SystemNoiseField MUST be extended with sample_channel_region() for bulk queries
8. Implementation MUST be verified via parse check + runtime test before production

---

**Status**: APPROVED BY COUNCIL OF FIVE
