# Reference: UV Unwrapping, Packing & Texel Density Standards

## 1. Texel Density Mathematics

Texel Density ($TD$) quantifies the ratio of texture pixels mapped across a 3D geometric surface area. Uniform texel density guarantees visual consistency across all assets in a scene.

$$TD = \frac{\sqrt{\text{UV Area in Pixels}^2}}{\text{3D World Surface Area in Meters}} = \frac{R_{\text{tex}} \cdot \sqrt{\text{Area}_{\text{UV\_norm}}}}{\text{Length}_{\text{world}}}$$

### Standard Studio Texel Density Profiles:

| Asset Profile | Target TD ($\text{px/m}$) | Target TD ($\text{px/cm}$) | Typical Resolution |
| :--- | :--- | :--- | :--- |
| **First-Person Weapons / FPS Hands** | $4096\text{ px/m}$ | $40.96\text{ px/cm}$ | $4096 \times 4096$ (Single) / $2 \times 4096$ |
| **Hero Characters / Main Avatars** | $2048\text{ px/m}$ | $20.48\text{ px/cm}$ | $4096 \times 4096$ (UDIM 1001-1004) |
| **Vehicles & Primary Interactive Props**| $1024\text{ px/m}$ | $10.24\text{ px/cm}$ | $4096 \times 4096$ / $2048 \times 2048$ |
| **Modular Environment & Architecture** | $512\text{ px/m}$ - $1024\text{ px/m}$ | $5.12 - 10.24\text{ px/cm}$ | $2048 \times 2048$ (Trim Sheets / Tiling) |
| **Background Props & Large Structures** | $256\text{ px/m}$ - $512\text{ px/m}$ | $2.56 - 5.12\text{ px/cm}$ | $1024 \times 1024$ / $2048 \times 2048$ |

---

## 2. Hard Edge vs UV Seam Golden Rules

To prevent dark seam artifacts and normal map gradients during shading:

```text
RULE 1: EVERY HARD EDGE MUST BE A UV SEAM.
If two adjacent faces meet at a sharp angle (e.g. >= 45°) and share a hard normal split,
their UV islands MUST be separated. If they remain connected in UV space, the normal map baker
must create an infinite gradient across a single pixel, producing a visible black seam in engine.

RULE 2: UV SEAMS ON SMOOTH SURFACES MUST BE PLACED IN NATURAL CREASES.
When splitting UV islands across a continuous smooth surface, hide seams in internal occluded cavities,
under clothing folds, along mechanical joint seams, or behind the character's hair line.
```

---

## 3. UV Island Packing & Mipmap Gutter Padding

When game engines generate mipmaps ($4\text{K} \to 2\text{K} \to 1\text{K} \to 512 \dots$), pixels from adjacent UV islands bleed into each other if the gutter margin is too small.

### Minimum Gutter Margin Table:
| Texture Resolution | Minimum Island Spacing | Texture Edge Border Margin |
| :--- | :--- | :--- |
| **$4096 \times 4096$ (4K)** | $16\text{ pixels}$ ($0.0039\text{ UV}$) | $16\text{ pixels}$ |
| **$2048 \times 2048$ (2K)** | $8\text{ pixels}$ ($0.0039\text{ UV}$) | $8\text{ pixels}$ |
| **$1024 \times 1024$ (1K)** | $4\text{ pixels}$ ($0.0039\text{ UV}$) | $4\text{ pixels}$ |
| **$512 \times 512$** | $2\text{ pixels}$ ($0.0039\text{ UV}$) | $2\text{ pixels}$ |

### Island Orientation & Straightening Rule:
- All cylindrical or rectangular UV islands (straps, pipes, beams, trims) must be **straightened** along the $U$ or $V$ axis.
- Straightening reduces aliasing, optimizes packing efficiency to $\ge 75\%$, and eliminates diagonal rasterization artifacts on normal maps.

---

## 4. Multi-Tile UDIM Layout Standards

For cinematic VFX and hero characters, assets use UDIM tiles starting at `1001`:

```text
┌───────────┬───────────┬───────────┬───────────┬───────────┐
│ UDIM 1001 │ UDIM 1002 │ UDIM 1003 │ UDIM 1004 │ UDIM 1005 │
│ Head &    │ Torso &   │ Arms &    │ Legs &    │ Gear &    │
│ Face      │ Chest     │ Hands     │ Feet      │ Props     │
└───────────┴───────────┴───────────┴───────────┴───────────┘
```

- Each tile maintains identical Texel Density.
- Symmetrical components (e.g. left/right boots) may be stacked into the identical UV position to double resolution, but one instance MUST be shifted $+1.0$ unit along the $U$-axis (e.g. to UDIM 1002 or off-screen space) during high-to-low baking to prevent overlapping ray collision artifacts.


## Associated Reference Frameworks
For a comprehensive view of the CIEL AAA+ 3D Master Studio pipeline, explore the reciprocal blueprints:
- 📐 [Pipeline Architecture & Data Flow](file:///root/.gemini/config/skills/autonomous-3d-studio/references/pipeline_architecture.md)
- ⚙️ [Hard-Surface Standards](file:///root/.gemini/config/skills/autonomous-3d-studio/references/hard_surface_standards.md)
- 👤 [Character & Organic Standards](file:///root/.gemini/config/skills/autonomous-3d-studio/references/character_organic_standards.md)
- 🕸️ [Topology & Retopology](file:///root/.gemini/config/skills/autonomous-3d-studio/references/topology_and_retopology.md)
- 🗺️ [UV Unwrapping & Texel Density](file:///root/.gemini/config/skills/autonomous-3d-studio/references/uv_unwrapping_texel_density.md)
- 🎯 [High-to-Low Baking](file:///root/.gemini/config/skills/autonomous-3d-studio/references/high_to_low_baking.md)
- 🎨 [PBR, MaterialX & OpenPBR](file:///root/.gemini/config/skills/autonomous-3d-studio/references/pbr_materialx_openpbr.md)
- 🔮 [Procedural Geometry Nodes & VEX](file:///root/.gemini/config/skills/autonomous-3d-studio/references/procedural_geometry_nodes_vex.md)
- 🦴 [Rigging, Skinning & Animation](file:///root/.gemini/config/skills/autonomous-3d-studio/references/rigging_skinning_animation.md)
- 🎬 [OpenUSD & Unreal Engine 5](file:///root/.gemini/config/skills/autonomous-3d-studio/references/usd_and_unreal_engine.md)
- 🤖 [Generative 3D Hybrid Pipelines](file:///root/.gemini/config/skills/autonomous-3d-studio/references/generative_3d_hybrid.md)
- 🔍 [Visual QA Turnaround Protocol](file:///root/.gemini/config/skills/autonomous-3d-studio/references/visual_qa_turnaround_protocol.md)
