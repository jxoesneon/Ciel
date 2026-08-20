# Reference: Visual QA Turnaround & Multimodal Audit Protocol

## 1. Autonomous Visual QA Architecture

Visual inspection is the ultimate verification gate. While mathematical topology scripts catch non-manifold edges and UV overlaps, multimodal visual turnarounds verify aesthetic silhouette quality, normal continuity, reflection smoothness, and organic/hard-surface expression.

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 5-PASS VISUAL AUDIT TURNAROUND RIG                               │
├─────────────────┬─────────────────┬──────────────────┬──────────────────┬────────────────────────┤
│ PASS 1: Beauty  │ PASS 2: Clay    │ PASS 3: Wireframe│ PASS 4: Normal   │ PASS 5: Specular       │
├─────────────────┼─────────────────┼──────────────────┼──────────────────┼────────────────────────┤
│ • Full PBR      │ • Neutral 50%   │ • Overlay wire   │ • Tangent Normal │ • Mirror roughness (0) │
│ • 3-Point Light │   grey albedo   │ • Inspect quad   │ • Check tangent  │ • High-frequency       │
│ • HDRI Studio   │ • Check forms   │   flow & poles   │   polarity       │   reflection pinch     │
└─────────────────┴─────────────────┴──────────────────┴──────────────────┴────────────────────────┘
```

---

## 2. Standardized Studio Camera & Lighting Rig

To ensure zero optical distortion and repeatable evaluation across assets:

### Camera Rig Calibration

- **Focal Length**: $85.0\text{ mm}$ (eliminates wide-angle perspective fish-eye distortion).
- **Sensor Size**: $36.0\text{ mm} \times 24.0\text{ mm}$ (Full Frame 35mm equivalent).
- **Elevation Angle**: $+15.0^\circ$ above ground plane.
- **Azimuth Rotation Steps**: 8 camera angles ($0^\circ, 45^\circ, 90^\circ, 135^\circ, 180^\circ, 225^\circ, 270^\circ, 315^\circ$).
- **Framing**: Bounding sphere fits within $85\%$ of vertical frame height.

### 3-Point Studio Lighting Specification

- **Key Light**: 45° off-axis, Intensity $= 1000\text{ W}$, Temperature $= 5500\text{ K}$, Soft Area Light.
- **Fill Light**: -60° off-axis, Intensity $= 350\text{ W}$, Temperature $= 6500\text{ K}$.
- **Rim / Back Light**: 160° behind asset, Intensity $= 1200\text{ W}$, Temperature $= 4500\text{ K}$ (sharp silhouette separation).
- **Environment**: Neutral Grey Studio HDRI, Energy $= 0.25$.

---

## 3. The 5 Turnaround Passes & Defect Diagnostic Matrix

| Inspection Pass | Target Defect to Detect | Remediation Protocol |
| :--- | :--- | :--- |
| **Pass 1: Beauty** | Incorrect PBR values (e.g. gray metals, overly bright dielectrics), texture seams | Recalibrate albedo values according to physical PBR charts; re-bake seams. |
| **Pass 2: Clay** | Lumpy surfaces, uneven bevel widths, un-smoothed faceting | Apply targeted relaxation smoothing or increase subdivision support loops. |
| **Pass 3: Wireframe** | High-valence star poles ($\ge 6$), spiral loops, non-quad polygons, density jumps | Reroute edge loops along curvature; apply 4-to-2 quad reduction patterns. |
| **Pass 4: Normal Map** | Inverted green channel (DirectX/OpenGL flip), ray miss black spots, cage clipping | Flip green channel in shader; increase cage projection distance $d_{\text{push}}$. |
| **Pass 5: Specular** | Shading pinching, reflection warping, waviness across flat planes | Enforce coplanar vertex alignment; apply weighted normal modifier with face-area weight. |

---

## 4. Visual Inspection Matrix & Scoring

Every asset turnaround is compiled into a multi-panel visual sheet and graded:

$$\text{VisualScore} = 0.30 \cdot S_{\text{Silhouette}} + 0.25 \cdot S_{\text{Reflection}} + 0.20 \cdot S_{\text{PBR}} + 0.15 \cdot S_{\text{Normal}} + 0.10 \cdot S_{\text{Topology}}$$

- **Passing Threshold**: $\text{VisualScore} \ge 0.95$ (AAA Studio Quality).
- Any individual category score $< 0.85$ triggers an automatic RED flag and halts downstream export until remediated.

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
