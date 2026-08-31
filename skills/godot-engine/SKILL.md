---
name: godot-engine
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Godot 4.x engine architecture, node systems, signals, scene management, and AAA rendering patterns. Advisory and design guidance only.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems", "godot", "gamedev", "gdscript", "rendering", "physics"]
triggers:
  - pattern: "(godot|gdscript|gamedev|scene[- ]tree|node[- ]architecture|forward\\+|jolt|gdextension)"
    confidence: 0.9
  - pattern: "(multiplayer[- ]sync|navigation[- ]mesh|collision[- ]layer|autoload|signal[- ]bus)"
    confidence: 0.85
source: { tier: 1, origin: self-synthesized }
side_effects: []
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Godot 4.x Engine Expertise

Advisory framework for Godot 4.x enterprise game development. Provides architectural patterns, rendering strategy, scripting conventions, and physics guidance. Design-only — no execution; Ciel's runtime gates handle any build/run operations.

## Core Architecture & Node Hierarchy

- **Decoupled scenes**: Self-contained scenes; avoid `get_node("../../../")`. Inject references from high-level managers via `_ready()`.
- **Data-Oriented escape hatch**: For 1000s of entities, bypass SceneTree via Godot Servers (`RenderingServer`, `PhysicsServer3D`) using RIDs.
- **Composition > inheritance**: Single-purpose node components (`HealthComponent`, `HitboxComponent`). Rule: "Call Down, Signal Up".
- **Minimize Autoloads**: Over-reliance creates God objects. Store persistent state in `.tres` Resource files (reference-counted, serializable).
- **Event Bus**: Dedicated Autoload holding only signals to decouple systems (Observer pattern).

## Rendering (Forward+ Pipeline)

- **GI selection**: SDFGI for open worlds (+ SSIL micro-detail); VoxelGI for indoor (bake to `.res`); LightmapGI for static (zero runtime cost).
- **Volumetric fog**: Lower froxel buffer resolution; use localized `FogVolume` nodes over global flooding; Alpha Scissor on semi-transparent objects.
- **Materials**: Standardize on ORM-packed textures; enable Physical Light Units; TAA for specular aliasing.
- **Compute**: Keep results on GPU in `StorageBuffers` (avoid `buffer_get_data` readbacks).
- **Draw calls**: `MultiMeshInstance3D` for repeated objects (foliage, crowds); manual Visibility Ranges for LOD swaps.
- **Occlusion**: Enable Raster Occlusion Culling; use simplified baked blocker meshes as `OccluderInstance3D`, never render meshes.

## Scripting (GDScript 2.0)

- **Static typing**: Enforce `:=` and `-> Type`; elevate `Unsafe Property Access` warnings to `Error`.
- **Lambdas**: Avoid defining lambdas inside hot paths (`_process`) — they allocate memory.
- **FSM**: `StateMachine` parent orchestrating child `State` nodes; transitions listen to state signals, keeping states isolated.

## Networking & Synchronization

- **High-level sync**: `MultiplayerSynchronizer` for continuous state, `MultiplayerSpawner` for dynamic instancing (handles late-joiners).
- **RPC security**: Never trust client. Server validates via `multiplayer.get_remote_sender_id()`. `reliable` for critical events, `unreliable` for frequent updates.
- **Reconciliation**: Godot lacks built-in. Implement tick-based simulation: clients buffer input, predict, rollback on server mismatch.

## Physics & Gameplay

- **Godot Jolt**: Mandatory for AAA 3D; drop-in replacement resolving jitter/ghost collisions, highly multi-threaded.
- **Physics interpolation**: Enable (Godot 4.3+) to decouple physics ticks from render FPS.
- **Shapes**: `CapsuleShape3D` for humanoids; never trimesh/concave for dynamic bodies.
- **Raycast scaling**: Query `PhysicsDirectSpaceState3D.intersect_ray()` directly; time-slice instead of hundreds of `RayCast3D` nodes.
- **NavMesh**: Chunk world into smaller `NavigationRegion3D`; bake async on simplified invisible collision geometry. Throttle `NavigationAgent3D` updates; flow fields for RTS scale.
- **Animation**: Separate gameplay FSM from `AnimationTree` presentation; Advance Expressions read state vars directly.

## Memory & GDExtension

- **Object pooling**: Pre-instantiate into inactive `Array` stack; pop/reset/push to avoid runtime stutters.
- **GDExtension (C++/Rust)**: Migrate hot paths to native; avoid API boundary crossings in tight loops; batch with `PackedFloat32Array`/`PackedVector3Array`.

## Camera Depth Ratio

- Directional shadow culler inverts projection in float32; underflow if `far/near >= 10,000,000:1`.
- **Golden rule**: keep `<= 1,000,000:1`. Orbital: `far=2,000,000 / near=2.0`. Surface: `far=50,000 / near=0.5`.

## Anti-Patterns

- **Deep node paths**: `get_node("../../../Sibling")` — brittle, couples scenes. Inject references instead.
- **Autoload God objects**: Singletons holding logic + state + signals. Split into Event Bus + Resource state.
- **Trimesh dynamic bodies**: `ConcavePolygonShape3D` on moving bodies — unstable. Use primitives.
- **Lambda in `_process`**: Allocates per frame. Hoist or use named methods.
- **CPU readback in compute**: `buffer_get_data` stalls the pipeline. Keep data GPU-resident.
