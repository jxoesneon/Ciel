# Reference: Cage Math & High-to-Low Texture Baking Standards

## 1. Cage Mesh Mathematics & Projection Mechanics

Baking transfers fine geometric details from a high-poly source mesh ($M_{\text{high}}$) onto a low-poly real-time target mesh ($M_{\text{low}}$) by ray-casting from an expanded cage envelope ($M_{\text{cage}}$).

```text
       M_cage  (Outer projection envelope)
      /
 ────o──────────────────────────────────────── Ray Origin
     │  \ 
     │   \  Ray Vector (Inward along averaged normal)
     │    \
 ────o─────o────────────────────────────────── M_high (Surface intersection)
     │      \
 ────o───────o──────────────────────────────── M_low  (Target UV pixel)
```

### Cage Vertex Position Formula:
For each vertex $v_i \in M_{\text{low}}$, the cage vertex position $v_{\text{cage}, i}$ is calculated using the **Averaged Vertex Normal** $\vec{N}_{\text{avg}}(v_i)$:

$$v_{\text{cage}, i} = v_i + \vec{N}_{\text{avg}}(v_i) \cdot d_{\text{push}}$$

Where:
- $\vec{N}_{\text{avg}}(v_i) = \frac{\sum_{f \in \text{Faces}(v_i)} \text{Area}(f) \cdot \vec{N}(f)}{\left\| \sum_{f \in \text{Faces}(v_i)} \text{Area}(f) \cdot \vec{N}(f) \right\|}$
- $d_{\text{push}}$ is the minimum distance required to completely envelop all high-poly peaks without intersecting neighboring geometry.

---

## 2. Tangent Space Normal Maps & MikkTSpace

To ensure tangent space normal maps render identically across all DCC tools and game engines, all bakes MUST use the **MikkTSpace standard**.

### Coordinate System Inversion Matrix:
- **OpenGL ($+Y$)**: Blender, Maya, Unity, Godot, Substance 3D Painter.
  $$\vec{N}_{\text{OpenGL}} = (R, G, B) = (T_x, +T_y, T_z)$$
- **DirectX ($-Y$)**: Unreal Engine 4/5, 3ds Max.
  $$\vec{N}_{\text{DirectX}} = (R, 1.0 - G, B) = (T_x, -T_y, T_z)$$

### 16-Bit Bit-Depth Mandate:
- Standard 8-bit normal maps have only 256 discrete values per channel ($\frac{2.0}{256} \approx 0.0078\text{ normal step}$). On smooth curved surfaces, this introduces visible stair-step banding.
- All raw normal map bakes MUST be output as **16-bit per channel PNG or OpenEXR** ($65,536$ discrete values per channel).

---

## 3. Substance Automation Toolkit (SAT) Headless Baking

For enterprise-scale studio pipelines, the skill orchestrates Adobe Substance Automation Toolkit (SAT) command-line tools via [`scripts/substance_sat_baker.py`](file://~/.gemini/config/skills/autonomous-3d-studio/scripts/substance_sat_baker.py):

| SAT Command | Map Type | Ray Optimization |
| :--- | :--- | :--- |
| `sbsbaker normal-from-mesh` | Tangent Space Normal | MikkTSpace, 16-bit PNG/EXR, Skew Cage |
| `sbsbaker ambient-occlusion` | Ambient Occlusion (AO) | Cosine distribution, 256 rays/pixel |
| `sbsbaker curvature` | Curvature (Laplacian) | High/Low frequency multi-scale details |
| `sbsbaker world-space-normals` | World Normal | Directional weather & snow masks |
| `sbsbaker position` | Normalized Position | XYZ gradient bounding box $[0, 1]$ |
| `sbsbaker thickness` | Subsurface Thickness | Inverted ray penetration depth |
| `sbsbaker color-from-mesh` | Material ID / Color | High-poly vertex color ID masking |

---

## 4. Essential Auxiliary Bake Maps & Formulation

| Map Type | Description & Formula | Value Range & Color Space | Usage in PBR |
| :--- | :--- | :--- | :--- |
| **Normal Map** | Encodes surface angle offsets via tangent space vectors. | $[0, 1] \implies [-1, 1]$ (Linear) | Micro-surface light reflection & bumps |
| **Ambient Occlusion (AO)** | Measures hemispherical sky occlusion: $AO(p) = \frac{1}{\pi} \int_{\Omega} V(p, \omega)(\vec{n}\cdot\omega)d\omega$ | $[0, 1]$ (Linear / Raw) | Crease shadowing, indirect bounce damping |
| **Curvature Map** | Discrete Laplace-Beltrami operator: $\Delta_S p = 2 H \vec{n}$ ($0.5 = \text{flat}$, $>0.5 = \text{convex}$, $<0.5 = \text{concave}$) | $[0, 1]$ (Linear / Raw) | Procedural edge wear & cavity dirt masks |
| **World Space Normal** | Normalized normal vectors in world coordinates $(X, Y, Z)$. | $[0, 1] \implies [-1, 1]$ (Linear) | Directional weather, snow accumulation, top-down dust |
| **Position Map** | Normalized mesh coordinates: $P_{\text{norm}} = \frac{P - P_{\min}}{P_{\max} - P_{\min}}$ | $[0, 1]$ RGB (Linear / Raw) | Height gradients, water level line masks |
| **ID Map** | Color-coded masking texture derived from high-poly vertex colors or material groups. | Clean RGB values (sRGB) | Multi-material automated masking |
| **Thickness Map** | Inverted ray transmission depth through thin geometry. | $[0, 1]$ (Linear / Raw) | Subsurface Scattering (SSS) & ear/skin translucency |


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
