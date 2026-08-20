---
name: autonomous-3d-studio
version: 2.0.0
format: skill/1.0
description: CIEL's autonomous AAA+ studio-grade 3D modeling, sculpting, retopology, UV unwrapping, PBR/MaterialX texturing, baking, procedural generation, USD composition, and engine integration engine.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "3d", "blender", "unreal-engine", "usd", "pbr", "materialx", "retopology", "baking", "hard-surface", "character", "geometry-nodes", "substance", "mcp", "aaa-quality", "domain:graphics", "harmonized"]
triggers:
  - pattern: "(3d|model|mesh|sculpt|retopo|retopology|uv|bake|baking|pbr|materialx|openpbr|blender|bpy|maya|houdini|unreal|nanite|gltf|fbx|usd|obj|substance)"
    confidence: 0.95
  - pattern: "(hard[- ]surface|organic|character|anatomy|facs|rigging|skinning|geometry[- ]nodes|vex|texel[- ]density|subd|trellis|hunyuan3d|rodin)"
    confidence: 0.90
source: { tier: 1, origin: harmonized }
dependencies:
  skills: ["ciel-quality-and-verification", "ciel-artifact-management", "research-ops", "continuous-learning-v2"]
  mcp: []
  system: ["python3", "blender"]
---

# Autonomous 3D Studio — AAA+ Production Engine

`autonomous-3d-studio` is CIEL's comprehensive, deepest-in-class autonomous 3D pipeline orchestration skill. It empowers agents to execute end-to-end 3D asset creation at AAA+ game studio and cinematic film VFX standards.

The skill provides live Blender MCP viewport bridging, deterministic headless execution harnesses, strict geometric QA gates, automated multi-angle visual inspection, mathematical UV and texel density optimization, 16/32-bit high-to-low baking, procedural geometry node construction, MaterialX/OpenPBR shader graphs, Substance SAT integration, QuadriFlow retopology, seamless Unreal Engine 5 / USD integration, and closed-loop self-repair loops.

---

## 1. The 7-Stage AAA Production Pipeline

Every 3D asset created under this skill strictly adheres to the 7-Stage End-to-End Pipeline:

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               7-STAGE AAA 3D PRODUCTION PIPELINE                                 │
├─────────────────┬─────────────────┬─────────────────┬──────────────────┬─────────────────────────┤
│ STAGE 1         │ STAGE 2         │ STAGE 3         │ STAGE 4          │ STAGE 5                 │
│ Concept & High- │ Topology &      │ UV Unwrapping & │ Cage Prep & High-│ PBR & MaterialX         │
│ Poly Sculpting  │ Retopology      │ Packing         │ to-Low Baking    │ Authoring               │
├─────────────────┼─────────────────┼─────────────────┼──────────────────┼─────────────────────────┤
│ • SubD / CAD    │ • Quad flow     │ • Seam math     │ • Averaged cage  │ • Metal/Rough PBR       │
│ • Voxel / Dyntopo│ • Pole control  │ • Texel density │ • 16-bit Normal  │ • OpenPBR / MatX        │
│ • AI-3D cleanup │ • Deform loops  │ • 0% overlap    │ • AO & Curvature │ • Channel packing       │
│ • Micro-bevels  │ • Nanite/LODs   │ • UDIM tiles    │ • ID & Bent Norm │ • Smart layers          │
└────────┬────────┴────────┬────────┴────────┬────────┴────────┬─────────┴────────────┬────────────┘
         │                 │                 │                 │                      │
         ▼                 ▼                 ▼                 ▼                      ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ STAGE 6: Rigging, Skinning & Blendshapes  │ STAGE 7: Engine Integration & Scene Assembly         │
├───────────────────────────────────────────┼──────────────────────────────────────────────────────┤
│ • Bone roll alignment & IK/FK chains      │ • Unreal Engine 5 Nanite, Lumen & LODs               │
│ • Dual-Quaternion / Heat-diffuse weights │ • OpenUSD (UsdGeomMesh, UsdShade) Stage Assembly     │
│ • FACS 52 Facial Blendshape deltas        │ • UCX/UBX Chaos Collision Hulls & Physics Setup      │
└───────────────────────────────────────────┴──────────────────────────────────────────────────────┘
```

---

## 2. Autonomous Execution Harnesses & Tool Suites

The skill ships with 13 battle-tested automation scripts located in `scripts/`:

| Script | Purpose | Command Line Usage |
| :--- | :--- | :--- |
| [`scripts/blender_mcp_server.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/blender_mcp_server.py) | Live Blender MCP & TCP socket RPC bridge for interactive real-time viewport and scene control | `python3 scripts/blender_mcp_server.py --summary` |
| [`scripts/autonomous_refinement_loop.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/autonomous_refinement_loop.py) | Master closed-loop self-repair engine orchestrating multi-pass auto-convergence | `python3 scripts/autonomous_refinement_loop.py --mesh asset.obj --profile aaa_game` |
| [`scripts/generative_3d_adapter.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/generative_3d_adapter.py) | AI 3D foundation model adapter (TRELLIS.2 / Hunyuan3D / Rodin) with delighting | `python3 scripts/generative_3d_adapter.py --input raw.obj --out clean.obj` |
| [`scripts/substance_sat_baker.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/substance_sat_baker.py) | Adobe Substance Automation Toolkit (SAT) headless GPU baker bridge | `python3 scripts/substance_sat_baker.py --high high.obj --low low.obj` |
| [`scripts/retopology_quadriflow.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/retopology_quadriflow.py) | Curvature-guided quad retopology and automated 5-tier LOD hierarchy generator | `python3 scripts/retopology_quadriflow.py --mesh mesh.obj --outdir lods/` |
| [`scripts/blender_pipeline_executor.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/blender_pipeline_executor.py) | Headless Blender (`bpy`) batch runner with process-group isolation | `python3 scripts/blender_pipeline_executor.py --mode generate --config asset.json` |
| [`scripts/geometry_qa_validator.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/geometry_qa_validator.py) | Strict geometry audit with programmatic `--fix` self-healing and instinct telemetry | `python3 scripts/geometry_qa_validator.py --input mesh.obj --fix --out clean.obj` |
| [`scripts/turnaround_qa_renderer.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/turnaround_qa_renderer.py) | Renders 8-angle Beauty, Clay, Wireframe, and Normal turnarounds with HTML viewer | `python3 scripts/turnaround_qa_renderer.py --mesh mesh.obj --outdir renders/` |
| [`scripts/uv_texel_analyzer.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/uv_texel_analyzer.py) | Automated UV unwrapping, packing efficiency calculation, and Texel Density audit | `python3 scripts/uv_texel_analyzer.py --mesh mesh.obj --target-td 20.48 --res 4096` |
| [`scripts/high_to_low_baker.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/high_to_low_baker.py) | Automated high-to-low mesh transfer: cage creation, normal, AO, curvature, and ID baking | `python3 scripts/high_to_low_baker.py --high high.obj --low low.obj --out maps/` |
| [`scripts/procedural_kit_generator.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/procedural_kit_generator.py) | Procedural modular kit builder with boolean cutters, chamfers, and floater panels | `python3 scripts/procedural_kit_generator.py --type modular_panel --out kit/` |
| [`scripts/usd_materialx_bridge.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/usd_materialx_bridge.py) | OpenUSD stage composition and MaterialX / OpenPBR network generation | `python3 scripts/usd_materialx_bridge.py --usd stage.usda --matx shader.mtlx` |
| [`scripts/unreal_engine_bridge.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/unreal_engine_bridge.py) | Unreal Engine 5 remote script for automated import, Nanite, LODs, and UCX collision | `python3 scripts/unreal_engine_bridge.py --asset asset.fbx --target /Game/Assets` |
| [`scripts/collision_hull_generator.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/collision_hull_generator.py) | Generates UE5 Chaos UBX/UCX collision primitives via V-HACD | `python3 scripts/collision_hull_generator.py --input mesh.obj` |
| [`scripts/vertex_normal_transfer.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/vertex_normal_transfer.py) | Face-area weighted CAD/NURBS normal transfer to low-poly | `python3 scripts/vertex_normal_transfer.py --source high.obj --target low.obj` |
| [`scripts/facs_blendshape_mirror.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/facs_blendshape_mirror.py) | Bilateral symmetry mirroring for ARKit/FACS 52 blendshapes | `python3 scripts/facs_blendshape_mirror.py --base face.obj --shape smile_L.obj` |
| [`scripts/udim_pack_analyzer.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/udim_pack_analyzer.py) | Multi-tile UDIM distribution and boundary overlap check | `python3 scripts/udim_pack_analyzer.py --mesh asset.obj` |
| [`scripts/usd_variant_manager.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/usd_variant_manager.py) | OpenUSD UsdVariantSets (LODs, Material States) composer | `python3 scripts/usd_variant_manager.py --stage out.usda` |
| [`scripts/distill_3d_instincts.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/distill_3d_instincts.py) | Consolidates QA telemetry into workspace spatial rules | `python3 scripts/distill_3d_instincts.py` |

---

## 3. Strict AAA Studio Hard Quality Gates

Before any asset is marked complete or promoted to integration, it MUST pass the **AAA Studio Hard Gate Protocol**:

1. **Geometry Topology Gate**:
   - Non-manifold edges: **0** (strictly prohibited).
   - Loose vertices / isolated edges: **0**.
   - Degenerate faces (zero area / collinear edges): **0**.
   - N-gons (>4 vertices): **0** on deforming meshes; max 0.1% on planar hard-surface only if coplanar (<0.01 deg variance).
   - Star poles (valence $\ge 6$): **0** in deforming regions; strict quad flow along anatomical deformation lines.
   - Scale & Rotation transforms: Fully normalized ($Scale = (1.0, 1.0, 1.0)$, $Rot = (0, 0, 0)$).
2. **UV & Texel Density Gate**:
   - UV Overlap: **0%** (except intentional symmetrical mirror stacks).
   - Flipped / Inverted UV faces: **0%**.
   - Texel Density Variance: **$\le \pm 5\%$** across all asset islands belonging to the same material ID.
   - Texture Gutter / Padding: Minimum **16 px** for 4K, **8 px** for 2K, **4 px** for 1K textures.
   - UV Boundary Hard Edges: 100% correlation — all UV seams MUST be marked as hard edges / smoothing group splits to prevent normal shading splits.
3. **Texture & Normal Map Gate**:
   - Bit Depth: Strictly **16-bit float** (PNG/TIFF/EXR) for Normal and Displacement maps to eliminate 8-bit staircase banding.
   - Normal Orientation: Explicitly validated (DirectX $-Y$ vs OpenGL $+Y$) matching the target engine.
   - Color Space Encoding:
     - **sRGB**: Base Color / Albedo, Subsurface Color, Transmittance.
     - **Linear / Raw**: Normal, Metallic, Roughness, Ambient Occlusion, Height/Displacement, Specular, Thickness.
4. **Visual Multimodal QA Gate**:
   - 8-angle turnaround inspection (Clay, Wireframe-over-Clay, Normal Map, Roughness, Beauty).
   - No shading pinching, normal polarity inversions, visible ray cage clipping, or texture stretching.

---

## 4. Specialized Technical Reference Blueprints

For in-depth procedural steps, mathematical formulations, and domain-specific rules, consult the modular reference documents:

- 📐 [Pipeline Architecture & Data Flow](file://~/.gemini/config/skills/autonomous-3d-studio/references/pipeline_architecture.md)
- ⚙️ [Hard-Surface Modeling & CAD/SubD Workflows](file://~/.gemini/config/skills/autonomous-3d-studio/references/hard_surface_standards.md)
- 👤 [Character, Anatomy & Organic Standards](file://~/.gemini/config/skills/autonomous-3d-studio/references/character_organic_standards.md)
- 🕸️ [Topology, Retopology & Quad Flow](file://~/.gemini/config/skills/autonomous-3d-studio/references/topology_and_retopology.md)
- 🗺️ [UV Unwrapping, Packing & Texel Density](file://~/.gemini/config/skills/autonomous-3d-studio/references/uv_unwrapping_texel_density.md)
- 🎯 [Cage Math & High-to-Low Baking](file://~/.gemini/config/skills/autonomous-3d-studio/references/high_to_low_baking.md)
- 🎨 [PBR, MaterialX & OpenPBR Standards](file://~/.gemini/config/skills/autonomous-3d-studio/references/pbr_materialx_openpbr.md)
- 🔮 [Procedural Geometry Nodes & Houdini VEX](file://~/.gemini/config/skills/autonomous-3d-studio/references/procedural_geometry_nodes_vex.md)
- 🦴 [Rigging, Skinning & Facial FACS Blendshapes](file://~/.gemini/config/skills/autonomous-3d-studio/references/rigging_skinning_animation.md)
- 🎬 [OpenUSD Stages & Unreal Engine 5 Nanite](file://~/.gemini/config/skills/autonomous-3d-studio/references/usd_and_unreal_engine.md)
- 🤖 [AI-Generative 3D Foundation Model Pipelines](file://~/.gemini/config/skills/autonomous-3d-studio/references/generative_3d_hybrid.md)
- 🔍 [Visual QA Turnaround & Audit Protocol](file://~/.gemini/config/skills/autonomous-3d-studio/references/visual_qa_turnaround_protocol.md)

---

## 5. Standard Assets & Configuration Presets

Pre-configured production profiles located in `assets/`:
- [`assets/blender_mcp_addon.py`](file://~/.gemini/config/skills/autonomous-3d-studio/assets/blender_mcp_addon.py): Live TCP socket bridge addon for Blender 3.0+/4.0+.
- [`assets/dcc_export_presets.json`](file://~/.gemini/config/skills/autonomous-3d-studio/assets/dcc_export_presets.json): Standard interchange export settings (Blender $\to$ Unreal/Unity/Maya/USD).
- [`assets/texel_density_presets.json`](file://~/.gemini/config/skills/autonomous-3d-studio/assets/texel_density_presets.json): Industry texel density calibration matrix.
- [`assets/studio_qa_checklist.json`](file://~/.gemini/config/skills/autonomous-3d-studio/assets/studio_qa_checklist.json): Machine-readable QA evaluation criteria.

---

## 6. Verification Loop & Closed-Loop Convergence

When an agent encounters a QA violation during execution:
1. **Automated Iterative Refinement**:
   - Run `scripts/autonomous_refinement_loop.py --mesh asset.obj --profile <profile>` to automatically execute multi-pass self-healing.
2. **Programmatic Self-Repair**:
   - Run `scripts/geometry_qa_validator.py --input mesh.obj --fix --out clean.obj` to dissolve degenerate zero-area faces and prune loose unreferenced vertices.
3. **Telemetry & Instinct Learning**:
   - All topology defects and audit outcomes automatically append to `~/.ciel/instincts/3d_studio_observations.jsonl` for continuous pattern learning across CIEL sessions.
4. **Visual Turnaround Review**:
   - Multi-pass turntable HTML review sheet allows immediate multimodal inspection of Clay, Wireframe, Normal, and Beauty passes.
