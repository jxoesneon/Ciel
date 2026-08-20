# Reference: Pipeline Architecture & Data Flow Standards

## 1. Directory Structure & Asset Lifecycle

Every AAA 3D asset project is organized into standardized workspace folders to maintain pipeline isolation:

```text
<project_root>/
├── assets/
│   └── <asset_category>/               # e.g., characters, props, vehicles, environment
│       └── <asset_name>/
│           ├── 01_concept/             # Reference images, 2D turnarounds, AI concept prompts
│           ├── 02_highpoly/            # Sculpted / CAD / SubD high-poly source (.blend, .zpr, .obj)
│           ├── 03_lowpoly/             # Clean retopologized mesh (.blend, .fbx, .obj)
│           ├── 04_uv/                  # Unwrapped UVs, UDIM sets, packing layout data
│           ├── 05_bakes/               # Raw 16-bit float bake maps (Normal, AO, Curvature, ID, Pos)
│           ├── 06_textures/            # PBR textures (BaseColor, ORM, Normal, Emissive) in 2K/4K
│           ├── 07_rig/                 # Rigged deformation mesh, armature, skin weights, blendshapes
│           └── 08_export/              # Final engine-ready delivery (.fbx, .gltf, .usd, .usda)
├── materials/                          # Master MaterialX (.mtlx) and shader definitions
└── manifests/                          # Asset manifest and QA compliance audit logs
```

---

## 2. Universal Units, Axes & Coordinate Systems

Mismatch in scale and axis alignment is the leading cause of downstream animation and physics failure. All autonomous agents MUST enforce the following matrix:

| Tool / Engine | Up Axis | Forward Axis | Handedness | Unit Scale (1.0 Unit) |
| :--- | :--- | :--- | :--- | :--- |
| **Blender** | $+Z$ | $+Y$ | Right-Handed | $1.0\text{ m}$ ($100.0\text{ cm}$) |
| **Unreal Engine 5** | $+Z$ | $+X$ | Left-Handed | $1.0\text{ cm}$ ($0.01\text{ m}$) |
| **Maya** | $+Y$ | $+Z$ | Right-Handed | $1.0\text{ cm}$ |
| **Houdini** | $+Y$ | $+Z$ | Right-Handed | $1.0\text{ m}$ |
| **Unity** | $+Y$ | $+Z$ | Left-Handed | $1.0\text{ m}$ |
| **OpenUSD** | $+Y$ (VFX) / $+Z$ (Games) | $+Z$ / $+Y$ | Right-Handed | Configurable (`metersPerUnit`) |

### Blender to Unreal Engine Transformation Matrix

When exporting FBX from Blender to Unreal Engine 5:

- **Forward**: `-Z Forward`
- **Up**: `Y Up`
- **Apply Transform**: `True` (Apply Unit Scale + Apply Transform to prevent root bone scale $= 100$ issue in UE5).

$$\mathbf{M}_{\text{Blender} \to \text{UE5}} = \begin{bmatrix} 100 & 0 & 0 & 0 \\ 0 & 0 & 100 & 0 \\ 0 & 100 & 0 & 0 \\ 0 & 0 & 0 & 1 \end{bmatrix}$$

---

## 3. Standard Production Naming Conventions

All assets, meshes, bones, materials, and textures must strictly follow standardized prefixes:

| Asset Type | Prefix / Pattern | Example |
| :--- | :--- | :--- |
| Static Mesh | `SM_<Category>_<Name>` | `SM_Prop_Generator_01` |
| Skeletal Mesh | `SK_<Category>_<Name>` | `SK_Char_CyberSoldier` |
| High-Poly Source | `HP_<Name>` | `HP_Weapon_Railgun` |
| Low-Poly Target | `LP_<Name>` | `LP_Weapon_Railgun` |
| Cage Mesh | `Cage_<Name>` | `Cage_Weapon_Railgun` |
| Master Material | `M_<Name>` | `M_Metal_Painted_Wear` |
| Material Instance | `MI_<Name>_<Variant>` | `MI_Metal_Painted_Wear_Red` |
| Base Color Texture | `T_<Name>_BC` / `_D` | `T_Generator_01_BC.png` |
| Occlusion-Rough-Metal | `T_<Name>_ORM` | `T_Generator_01_ORM.png` |
| Normal Map | `T_<Name>_N` | `T_Generator_01_N.png` |
| Emissive Map | `T_<Name>_E` | `T_Generator_01_E.png` |
| Convex Collision Hull | `UCX_<MeshName>_<Index>` | `UCX_SM_Prop_Generator_01_01` |
| Box Collision Hull | `UBX_<MeshName>_<Index>` | `UBX_SM_Prop_Generator_01_01` |
| Sphere Collision Hull | `USP_<MeshName>_<Index>` | `USP_SM_Prop_Generator_01_01` |
| Capsule Collision Hull | `UCP_<MeshName>_<Index>` | `UCP_SM_Prop_Generator_01_01` |
| Root Bone | `root` | `root` |
| Deform Bone | `DEF_<BoneName>` | `DEF_UpperArm_L` |
| Control Bone | `CTRL_<BoneName>` | `CTRL_Hand_IK_R` |

---

## 4. Interchange Data Contracts

### 1. FBX (FilmBox 2020+)

- Geometry: Triangulate or Quad-Preserve (engine importer will triangulate at runtime).
- Tangents & Binormals: Export with **MikkTSpace** tangents computed.
- Smoothing Groups: Enabled.
- Animation: Sample rate 60 FPS, keyframe reduction tolerance $\le 0.001$.

### 2. glTF 2.0 / GLB

- PBR Standard: `KHR_materials_pbrSpecularGlossiness` or core `pbrMetallicRoughness`.
- Texture Packing: Roughness in `G`, Metallic in `B`, Occlusion in `R`.
- Embedded buffers for single-file web/AR deployment, external `.bin` and textures for repo asset tracking.

### 3. OpenUSD (`.usd`, `.usda`, `.usdc`, `.usdz`)

- Stage Units: `metersPerUnit = 1.0` or `0.01`.
- Up Axis: `upAxis = "Z"` for Game engines, `"Y"` for VFX DCC pipelines.
- Prims structured under a root `Xform` named `/Root` or `/<AssetName>`.

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
