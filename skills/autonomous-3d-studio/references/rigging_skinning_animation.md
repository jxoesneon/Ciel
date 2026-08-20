# Reference: Rigging, Skinning & Facial Blendshape Standards

## 1. Skeletal Hierarchy & Armature Topology

A standardized skeletal hierarchy ensures compatibility across animation retargeting systems (Unreal Engine 5 IK Rig / Control Rig, Unity Humanoid, Maya HumanIK).

```text
root (World origin 0,0,0)
 └── pelvis
      ├── spine_01
      │    └── spine_02
      │         └── spine_03
      │              ├── neck_01
      │              │    └── head
      │              │         ├── eye_l / eye_r
      │              │         └── jaw
      │              ├── clavicle_l / clavicle_r
      │              │    └── upperarm_l / upperarm_r
      │              │         └── lowerarm_l / lowerarm_r
      │              │              └── hand_l / hand_r
      │              │                   └── thumb / index / middle / ring / pinky (01-03)
      └── thigh_l / thigh_r
           └── calf_l / calf_r
                └── foot_l / foot_r
                     └── toe_l / toe_r
```

---

## 2. Joint Orientation & Bone Roll Rules

Improper bone roll alignment results in erratic axis flipping during IK (Inverse Kinematics) solving and twist extraction.

### Blender / FBX Standard Bone Alignment:
- **Primary / Roll Axis (Length)**: Points strictly from parent joint toward the direct child joint ($+Y$ axis in Blender).
- **Bend / Hinge Axis**: Points perpendicular to the anatomical hinge bend plane ($+Z$ or $+X$ axis).
  - Elbows: Hinge axis must bend backward strictly along negative primary plane.
  - Knees: Hinge axis must bend forward strictly along positive primary plane.
- **Bone Roll Angle**: Must be symmetrical ($-\theta_{\text{Left}} = +\theta_{\text{Right}}$) or auto-aligned using bone roll calculation vectors.

---

## 3. Skinning Mathematics & Weight Normalization

### Linear Blend Skinning (LBS) vs Dual Quaternion Skinning (DQS)

1. **Linear Blend Skinning (LBS)**:
   $$v' = \sum_{j=1}^{k} w_j \cdot \mathbf{M}_j \mathbf{B}_j^{-1} v$$
   - *Limitation*: Suffers from the "candy-wrapper" volume collapse artifact when joints twist $>90^\circ$ (e.g. forearms, shoulders).
   - *Mitigation*: Insert dedicated twist bones (`twist_upperarm_01`, `twist_lowerarm_01`) that absorb $50\%$ of the axial twist rotation.

2. **Dual Quaternion Skinning (DQS)**:
   $$\hat{\mathbf{q}}_{\text{blend}} = \frac{\sum_{j=1}^{k} w_j \hat{\mathbf{q}}_j}{\left\| \sum_{j=1}^{k} w_j \hat{\mathbf{q}}_j \right\|}$$
   - Preserves 100% cylindrical volume during extreme twist deformation.

### Strict AAA Skinning Hard Limits:
- **Maximum Bone Influences Per Vertex**:
  - Real-time mobile / VR: Max **4 influences** per vertex.
  - AAA Game Engines (UE5/Unity): Max **8 influences** per vertex.
  - Cinematic / VFX (Maya/USD): Max **12 influences** per vertex.
- **Weight Normalization**:
  - The sum of all bone weights affecting any single vertex MUST strictly equal $1.0$:
    $$\left| \sum_{j=1}^{k} w_j - 1.0 \right| \le 1.0 \times 10^{-6}$$
- **Prune Tiny Weights**:
  - Any influence with weight $w_j < 0.005$ ($0.5\%$) must be pruned to 0.0 and remaining weights re-normalized.

---

## 4. FACS Facial Blendshape Delta Calculation

Morph targets (Blendshapes) encode relative vertex offsets $\Delta \vec{v}_i$ from the neutral base mesh:

$$\Delta \vec{v}_i = \vec{v}_{i, \text{target}} - \vec{v}_{i, \text{neutral}}$$

### Blendshape Topology Integrity:
- **Vertex Order Invariant**: The vertex count, vertex indexing, and edge connectivity of every blendshape target MUST match the neutral base mesh with $100\%$ precision ($N_{\text{verts, target}} = N_{\text{verts, base}}$).
- **Sparsity Optimization**: Blendshapes must store only non-zero deltas ($\|\Delta \vec{v}_i\| > 1.0 \times 10^{-5}\text{ m}$) to minimize memory footprint during engine runtime streaming.


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
