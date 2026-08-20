# Autonomous 3D Studio — Manifest

## Metadata

- **Skill Name**: `autonomous-3d-studio`
- **Version**: `2.0.0`
- **Format**: `skill/1.0`
- **Tier**: `1` (Production Grade)
- **Engine**: CIEL AAA+ 3D Master Studio

## Components

### Core Skill Entrypoint

- `SKILL.md`: Master orchestration, lifecycle contract, hard QA gates, closed-loop refinement engine, live MCP bridge, and quick-start execution harness.

### Reference Manuals (`references/`)

1. `references/pipeline_architecture.md`: 7-Stage production data pipeline, scene units, axes transformations, directory schemas.
2. `references/hard_surface_standards.md`: CAD/NURBS conversion, SubD control loops, bevel modifiers, trim sheets, floaters.
3. `references/character_organic_standards.md`: Facial topology loops, joint deformation mechanics, 52 FACS blendshapes, grooming.
4. `references/topology_and_retopology.md`: Quad flow math, QuadriFlow curvature fields, pole valence limits (E/N poles), 5-tier LOD hierarchy.
5. `references/uv_unwrapping_texel_density.md`: Texel density calibration, seam heuristics, UDIM layouts, packing optimization.
6. `references/high_to_low_baking.md`: Cage generation math, Substance Automation Toolkit (SAT) CLI patterns, 16/32-bit normal maps, AO, Curvature, ID, Thickness maps.
7. `references/pbr_materialx_openpbr.md`: PBR physics, MaterialX node networks, OpenPBR, Subsurface scattering, channel packing (ORM/RMA).
8. `references/procedural_geometry_nodes_vex.md`: Blender Geometry Nodes patterns, Houdini VEX wrangles, procedural modular kits.
9. `references/rigging_skinning_animation.md`: Armature hierarchies, bone roll alignment, dual-quaternion skinning, weight smoothing.
10. `references/usd_and_unreal_engine.md`: OpenUSD stage assembly, UsdGeomMesh, Unreal Engine 5 Nanite, Lumen, Chaos collision hulls.
11. `references/generative_3d_hybrid.md`: 3D foundation models (Microsoft TRELLIS.2 O-Voxels, Tencent Hunyuan3D-2.0 DiT/Paint, Hyper3D Rodin Gen-2.5).
12. `references/visual_qa_turnaround_protocol.md`: 8-point turntable visual inspection protocol (Beauty, Clay, Wireframe, Normal, Roughness).

### Automation Scripts (`scripts/`)

1. `scripts/blender_mcp_server.py`: Live Blender MCP and socket RPC bridge for interactive real-time viewport and scene control.
2. `scripts/generative_3d_adapter.py`: AI-generative 3D foundation model bridge with disconnected shell pruning and photometric delighting.
3. `scripts/substance_sat_baker.py`: Adobe Substance Automation Toolkit (SAT) headless baker bridge for high-speed multi-threaded GPU baking.
4. `scripts/retopology_quadriflow.py`: Curvature-guided quad retopology and automated 5-tier LOD hierarchy generator with screen-size thresholds.
5. `scripts/autonomous_refinement_loop.py`: Closed-loop convergence engine that orchestrates multi-pass auto-repair until all studio hard gates pass.
6. `scripts/blender_pipeline_executor.py`: Headless Blender batch pipeline execution engine with process group session management.
7. `scripts/geometry_qa_validator.py`: Strict geometry audit script with `--fix` auto-remediation and instinct observation telemetry.
8. `scripts/turnaround_qa_renderer.py`: Multi-angle turnaround renderer with HTML/DOM sanitization and interactive inspection viewer.
9. `scripts/uv_texel_analyzer.py`: Automated UV unwrap, pack efficiency calculator, and Texel Density validator.
10. `scripts/high_to_low_baker.py`: Hardened cage calculation and high-to-low texture baker (16-bit Normal, AO, Curvature, ID).
11. `scripts/procedural_kit_generator.py`: Procedural modular kit builder with boolean cutters and floater panels.
12. `scripts/usd_materialx_bridge.py`: OpenUSD stage builder and MaterialX PBR graph generator.
13. `scripts/unreal_engine_bridge.py`: Unreal Engine 5 Python Remote Execution bridge with injection-safe parameter encoding.
14. `scripts/geometry_core.py`: C-grade parsing, spatial hashing, DSU trees, and OOM-safe mesh streaming constraints (500k vertex ceiling).
15. `scripts/collision_hull_generator.py`: V-HACD compound convex hulls for UE5 Chaos collision generation.
16. `scripts/vertex_normal_transfer.py`: Face-area weighted CAD/NURBS normal transfer.
17. `scripts/facs_blendshape_mirror.py`: Bilateral symmetry mirroring for ARKit/FACS 52 blendshapes.
18. `scripts/udim_pack_analyzer.py`: Multi-tile UDIM distribution and boundary mapping analyzer.
19. `scripts/usd_variant_manager.py`: OpenUSD UsdVariantSets (LODs, Material States) procedural composer.
20. `scripts/distill_3d_instincts.py`: Autonomous instinct consolidation engine distilling `3d_studio_observations.jsonl`.

### Assets & Presets (`assets/`)

1. `assets/blender_mcp_addon.py`: Installable Blender Add-on for live TCP socket agent bridging.
2. `assets/dcc_export_presets.json`: Standard interchange export settings (Blender $\to$ Unreal/Unity/Maya/USD).
3. `assets/texel_density_presets.json`: Industry texel density calibration matrix for game/film profiles.
4. `assets/studio_qa_checklist.json`: Machine-readable QA evaluation criteria and gate thresholds.
