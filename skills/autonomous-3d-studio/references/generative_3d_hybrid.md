# Reference: AI-Generative 3D Foundation Model Pipelines

## 1. 2025/2026 Foundation Model Architectures

Next-generation generative 3D models operate across diverse geometric representations:

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                         3D FOUNDATION MODEL ARCHITECTURE COMPARISON                              │
├──────────────────────────┬─────────────────────────────┬─────────────────────────────────────────┤
│ MODEL                    │ CORE REPRESENTATION         │ MESH & TEXTURE EXTRACTION MECHANISM     │
├──────────────────────────┼─────────────────────────────┼─────────────────────────────────────────┤
│ Microsoft TRELLIS.2      │ O-Voxel (Sparse Voxel)      │ Instant, optimization-free bidirectional│
│                          │                             │ conversion into PBR meshes & Splats.    │
├──────────────────────────┼─────────────────────────────┼─────────────────────────────────────────┤
│ Tencent Hunyuan3D-2.0    │ Decoupled DiT + Paint       │ ShapeVAE SDF extraction -> Multi-view   │
│                          │                             │ 4K texture baking & delighting.         │
├──────────────────────────┼─────────────────────────────┼─────────────────────────────────────────┤
│ Hyper3D Rodin Gen-2.5    │ Production Smart Low Poly   │ Controllable polycount (4k-50k) with    │
│                          │                             │ native quad-dominant topology loops.    │
└──────────────────────────┴─────────────────────────────┴─────────────────────────────────────────┘
```

---

## 2. Raw Generative 3D Mesh Output Defects

Raw outputs from 3D foundation models represent uncurated isosurfaces with severe production defects:
- **Chaotic Non-Uniform Triangulation**: Sliver triangles, zero-area faces.
- **Nested Floating Geometry**: Internal occluded shells that waste GPU draw calls.
- **Baked Environmental Lighting**: Directional shadows and specular highlights baked into diffuse albedos.
- **Non-Manifold Geometry**: T-junctions, bow-tie vertices, hole punctures.

---

## 3. The 6-Step Autonomous AI-to-AAA Cleanup Protocol

To elevate raw AI-generated 3D geometry into AAA+ studio-ready assets, the agent executes this deterministic 6-step transformation pipeline via [`scripts/generative_3d_adapter.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/generative_3d_adapter.py):

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                            AI-TO-AAA RECONSTRUCTION PIPELINE                                     │
├─────────────────┬──────────────────┬──────────────────┬─────────────────┬────────────────────────┤
│ STEP 1          │ STEP 2           │ STEP 3           │ STEP 4          │ STEP 5 & 6             │
│ Internal Hull   │ Voxel Regularize │ Laplacian Smooth │ Quad Retopo     │ UV & High-to-Low       │
│ Extraction      │ & Watertight Sol │ & Feature Sharp  │ (QuadriFlow)    │ PBR Re-Projection      │
├─────────────────┼──────────────────┼──────────────────┼─────────────────┼────────────────────────┤
│ • Split shells  │ • Voxel grid     │ • Curvature flow │ • Pure quads    │ • Unpack & Pack UVs    │
│ • Delete hidden │ • Volume to mesh │ • Preserve edges │ • Flow alignment│ • Bake 16-bit Normal   │
│ • Keep max comp │ • Seal punctures │ • High-poly base │ • LOD targets   │ • PBR Albedo Delight   │
└─────────────────┴──────────────────┴──────────────────┴─────────────────┴────────────────────────┘
```

### Step 1: Internal Component Pruning
Compute connected components of the mesh $G = (V, E, F)$. Retain only the principal outer hull:

$$\text{Shell}_{\text{primary}} = \arg\max_{C_i \in \text{Components}} \text{SurfaceArea}(C_i)$$

Discard all floating sub-meshes $C_k$ where $\text{SurfaceArea}(C_k) < 0.05 \times \text{SurfaceArea}(\text{Shell}_{\text{primary}})$.

### Step 2: Voxel Remeshing for Watertight Manifold Topology
Convert the mesh to an OpenVDB Signed Distance Field (SDF) and extract an isotropic manifold isosurface:

$$VoxelSize = \frac{\max(L_x, L_y, L_z)}{512}$$

### Step 3: Laplacian Curvature Smoothing
Apply constrained Laplacian smoothing to eliminate high-frequency point-cloud noise while pinning hard boundary features:

$$v_i^{(t+1)} = v_i^{(t)} + \lambda \sum_{j \in \mathcal{N}(i)} \frac{1}{|\mathcal{N}(i)|} \left( v_j^{(t)} - v_i^{(t)} \right)$$

### Step 4: Quad Retopology (QuadriFlow / ZRemesh)
Generate a clean quad mesh aligned with principal curvature lines ($K_{\min}, K_{\max}$) using [`scripts/retopology_quadriflow.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/retopology_quadriflow.py).

### Step 5 & 6: UV Packing & Photometric Delighting
- Generate conformal UV islands with standardized Texel Density.
- Delighting: Remove baked directional shadows from the AI diffuse texture using high-pass frequency filtering or ambient occlusion division:
  $$I_{\text{delit}} = \frac{I_{\text{raw}}}{\max(AO_{\text{baked}}, 0.15)}$$


## Associated Reference Frameworks
For a comprehensive view of the CIEL AAA+ 3D Master Studio pipeline, explore the reciprocal blueprints:
- 📐 [Pipeline Architecture & Data Flow](file://~/.gemini/config/skills/autonomous-3d-studio/references/pipeline_architecture.md)
- ⚙️ [Hard-Surface Standards](file://~/.gemini/config/skills/autonomous-3d-studio/references/hard_surface_standards.md)
- 👤 [Character & Organic Standards](file://~/.gemini/config/skills/autonomous-3d-studio/references/character_organic_standards.md)
- 🕸️ [Topology & Retopology](file://~/.gemini/config/skills/autonomous-3d-studio/references/topology_and_retopology.md)
- 🗺️ [UV Unwrapping & Texel Density](file://~/.gemini/config/skills/autonomous-3d-studio/references/uv_unwrapping_texel_density.md)
- 🎯 [High-to-Low Baking](file://~/.gemini/config/skills/autonomous-3d-studio/references/high_to_low_baking.md)
- 🎨 [PBR, MaterialX & OpenPBR](file://~/.gemini/config/skills/autonomous-3d-studio/references/pbr_materialx_openpbr.md)
- 🔮 [Procedural Geometry Nodes & VEX](file://~/.gemini/config/skills/autonomous-3d-studio/references/procedural_geometry_nodes_vex.md)
- 🦴 [Rigging, Skinning & Animation](file://~/.gemini/config/skills/autonomous-3d-studio/references/rigging_skinning_animation.md)
- 🎬 [OpenUSD & Unreal Engine 5](file://~/.gemini/config/skills/autonomous-3d-studio/references/usd_and_unreal_engine.md)
- 🤖 [Generative 3D Hybrid Pipelines](file://~/.gemini/config/skills/autonomous-3d-studio/references/generative_3d_hybrid.md)
- 🔍 [Visual QA Turnaround Protocol](file://~/.gemini/config/skills/autonomous-3d-studio/references/visual_qa_turnaround_protocol.md)
