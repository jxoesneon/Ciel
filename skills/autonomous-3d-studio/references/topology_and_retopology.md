# Reference: Topology, Retopology & Quad Flow Standards

## 1. Topological Primitives & Valence Control

In AAA 3D asset engineering, mesh topology directly governs deformation, subdivision behavior, light shading, and GPU memory bandwidth.

### Vertex Valence Classification
The valence $V(v)$ of vertex $v$ is the number of edges incident to it:

- **Regular Vertex (Quad Grid)**: $V(v) = 4$. Perfect surface continuity under subdivision.
- **N-Pole (Corner / Boundary)**: $V(v) = 3$. Used to turn edge loops or terminate outer corners.
- **E-Pole (Extrusion / Junction)**: $V(v) = 5$. Used to redirect edge flow and bifurcate loops.
- **Star Pole (Valence $\ge 6$)**: $V(v) \ge 6$. **STRICTLY FORBIDDEN** in AAA production. Causes severe shading pinch and vertex pinching during skeletal deformation.

```text
VALENCE CONTROL RULES:
1. E-Poles (5) and N-Poles (3) must ALWAYS be placed on planar or low-curvature areas.
2. NEVER place an E-Pole or N-Pole directly on an anatomical crease line, joint pivot, or facial deformation loop.
3. NEVER allow two 5-poles or two 3-poles to share an adjacent edge (creates localized tension).
```

---

## 2. QuadriFlow Curvature-Aligned Retopology

QuadriFlow constructs quad meshes whose edge loops follow the **Principal Curvature Frame Fields** ($K_{\min}, K_{\max}$), ensuring optimal deformation under animation.

$$\min_{\vec{u}, \vec{v}} \int_{\mathcal{M}} \left( \|\nabla \vec{u} - \mathbf{K}_{\min}\|^2 + \|\nabla \vec{v} - \mathbf{K}_{\max}\|^2 \right) dA$$

Implemented in [`scripts/retopology_quadriflow.py`](file:///root/.gemini/config/skills/autonomous-3d-studio/scripts/retopology_quadriflow.py).

---

## 3. Quad Reduction & Loop Redirection Topologies

When reducing edge loop density from high-detail zones (e.g. hands, face) to low-detail zones (e.g. torso, arms), the reduction MUST be performed using 100% pure quad configurations.

### 4-to-2 Quad Reduction Pattern
```text
INPUT (4 LOOPS):
 │   │   │   │
 ├───┼───┼───┤
 │   ├───┤   │  <-- Center quad inserted horizontally
 ├───┴───┴───┤
 │     │     │
OUTPUT (2 LOOPS)
```

### 3-to-1 Quad Reduction Pattern
```text
INPUT (3 LOOPS):
 │   │   │
 ├───┼───┤
 │ ┌─┴─┐ │  <-- Diamond center quad redirecting outer loops
 ├─┴───┴─┤
 │       │
OUTPUT (1 LOOP)
```

---

## 4. Nanite Dense Meshes vs Traditional LOD Budgets

### Unreal Engine 5 Nanite Mesh Standards:
- **Polycount**: Up to $1\text{M} - 10\text{M}$ triangles allowed directly in engine.
- **Geometry**: Direct high-poly SubD or CAD tessellation.
- **Rules**:
  - Meshes must be **100% manifold watertight** solids (Nanite cluster culling requires closed silhouettes).
  - Eliminate micro-triangles smaller than $0.01\text{ mm}$ to prevent cluster boundary artifacts.

### Traditional 5-Tier Game LOD Polycount & Screen-Size Hierarchy:
| Level | Polycount Ratio | Target Screen Size | Output File Pattern | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **LOD 0** | $100\%$ ($20\text{k}-80\text{k}$ tris) | $> 0.60$ | `<Asset>_LOD0.obj` | Close-up Hero view / Cutscenes |
| **LOD 1** | $50\%$ ($10\text{k}-40\text{k}$ tris) | $0.35 - 0.60$ | `<Asset>_LOD1.obj` | Medium gameplay distance |
| **LOD 2** | $25\%$ ($5\text{k}-20\text{k}$ tris) | $0.15 - 0.35$ | `<Asset>_LOD2.obj` | Distant gameplay view |
| **LOD 3** | $12.5\%$ ($2.5\text{k}-10\text{k}$ tris) | $0.05 - 0.15$ | `<Asset>_LOD3.obj` | Far horizon view |
| **LOD 4** | $6.25\%$ ($500-2.5\text{k}$ tris) | $< 0.05$ | `<Asset>_LOD4.obj` | Ultra-distant billboard / silhouette |


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
