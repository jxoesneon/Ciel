# Reference: Hard-Surface Modeling & CAD/SubD Standards

## 1. Core Hard-Surface Modeling Paradigms

AAA studio hard-surface modeling utilizes three distinct production workflows depending on asset type, silhouette requirements, and polycount budget:

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 HARD-SURFACE WORKFLOW TAXONOMY                                  │
├──────────────────────────────┬──────────────────────────────┬───────────────────────────────────┤
│ PARADIGM A: SubD Modeling    │ PARADIGM B: Weighted Normals │ PARADIGM C: Boolean-Dynamesh      │
│ (Cinematics / High-Poly)     │ (Mid-Poly / Real-Time Props) │ (Complex Mechanical / Weapons)    │
├──────────────────────────────┼──────────────────────────────┼───────────────────────────────────┤
│ • Strict Quad Topology       │ • Bevel Modifiers (2-3 seg)  │ • Live Boolean operations         │
│ • Double Support Loops       │ • Face Weighted Normals      │ • High-res Voxel Remesh (0.005m)  │
│ • Catmull-Clark Subdivision  │ • No High-to-Low Bake needed │ • Polishing & Edge Relaxing       │
│ • Flawless Reflections       │ • Low Polycount, Fast Iter   │ • ZRemesh / QuadriFlow Retopo     │
└──────────────────────────────┴──────────────────────────────┴───────────────────────────────────┘
```

---

## 2. Subdivision Surface (SubD) Mathematics & Support Loops

When modeling for Catmull-Clark subdivision, edge sharpness is controlled by proximity support loops rather than creasing (unless using OpenSubdiv crease weights for film).

### Support Loop Distance Rule
For a primary boundary edge $E_0$, two parallel support loops $E_{-1}$ and $E_{+1}$ must be inserted at distance $\delta$:

$$\delta = k \cdot r_{\text{bevel}}$$

Where $r_{\text{bevel}}$ is the desired radius of curvature and $k \approx 0.5$.

### The 3-Edge Corner Rule
At any 90-degree corner with subdivision level 2+, three edges must converge into an outer Quad corner rather than a triangle or 5-pole star to eliminate reflection pinching:

```text
CORRECT 3-EDGE CORNER QUAD FLOW:
┌───────┬───────┐
│       │       │
├───────┼───────┤
│       │ ┌─────┘  <-- Clean 3-loop corner turn
│       │ │
```

---

## 3. Bevel Modifier & Weighted Normal Workflow

For mid-poly game assets (vehicles, machinery, sci-fi modular corridors), the **Weighted Normal** workflow produces crisp, baked-like chamfer highlights directly in real-time shaders without needing a high-to-low bake.

### Blender Implementation Protocol
1. **Mark Sharp**: Mark all structural boundary edges (>30° angle) as **Sharp** (`use_edge_sharp = True`).
2. **Bevel Modifier**:
   - `width`: $0.005\text{ m}$ to $0.02\text{ m}$ (depending on asset scale).
   - `segments`: 2 (standard game) or 3 (hero asset).
   - `profile`: 0.70 (produces rounder highlights).
   - `limit_method`: `'WEIGHT'` or `'ANGLE'` ($35^\circ$).
   - `harden_normals`: `True` (transfers large face normals across chamfer strips).
3. **Weighted Normal Modifier**:
   - `mode`: `'FACE_AREA_WITH_ANGLE'` (weights vertex normal contribution by face area multiplied by corner angle).
   - `keep_sharp`: `True`.

### Normal Weighting Mathematical Formulation
For vertex $v$ shared by faces $f_1, f_2, \dots, f_m$:

$$\vec{N}(v) = \frac{\sum_{i=1}^m \text{Area}(f_i) \cdot \theta_i(v) \cdot \vec{N}(f_i)}{\left\| \sum_{i=1}^m \text{Area}(f_i) \cdot \theta_i(v) \cdot \vec{N}(f_i) \right\|}$$

Where $\theta_i(v)$ is the interior angle of face $f_i$ at vertex $v$.

---

## 4. Floaters & Micro-Detail Projection

Floaters are detached geometric meshes (bolts, vents, panel seams, latches) placed slightly above the main high-poly surface ($0.1\text{ mm} \le h \le 1.0\text{ mm}$).

### Floater Golden Rules:
1. **Sinking Angle**: Floater edges must flare outward or chamfer at $\ge 45^\circ$ toward the base surface so rays cast from the baking cage hit the chamfer.
2. **Floating Height**: Never hover more than $0.5 \times \text{Cage Max Distance}$ above the primary hull to prevent ray miss artifacts.
3. **Backface Culling**: Delete back-facing geometry on floaters to reduce polygon overhead during high-poly ray traversal.

---

## 5. CAD / NURBS to Polygon Conversion Standards

When converting STEP/IGES CAD data (from SolidWorks, Fusion 360, Rhino, Alias) into game-ready geometry:

1. **Tessellation Chordal Deviation**: Set max distance tolerance $\le 0.1\text{ mm}$ ($0.0001\text{ m}$).
2. **Angular Deviation**: Set max angle tolerance $\le 12.0^\circ$.
3. **Planar Decimation**: Co-planar adjacent triangles on flat surfaces must be dissolved into clean n-gons or quads, provided normal angles deviate by $< 0.05^\circ$.
4. **Curvature Retention**: High density must be concentrated strictly along filleted bevels and cylinders. Cylindrical radial spans must follow the 8-Multiple Rule (16, 24, 32, 48, 64 segments).


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
