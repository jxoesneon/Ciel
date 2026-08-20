# Reference: Character, Anatomy & Organic Modeling Standards

## 1. Organic & Character Modeling Philosophy

Organic and character modeling requires strict alignment between topological edge flow and muscular/skeletal anatomical contraction lines. A character mesh is evaluated not just statically, but by its deformation integrity under extreme joint articulation and facial expression.

---

## 2. Facial Topology Loop Architecture

The human face topology must follow the 5 Primary Facial Anchor Loops to guarantee expressive, wrinkle-accurate deformation without topological pinching:

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                PRIMARY FACIAL ANCHOR LOOPS                                       │
├───────────────────────────────────┬──────────────────────────────────────────────────────────────┤
│ 1. Orbicularis Oculi Loop         │ Continuous concentric quad rings encircling the eye socket.  │
│                                   │ Outer boundary converges into the zygomatic arch.            │
├───────────────────────────────────┼──────────────────────────────────────────────────────────────┤
│ 2. Orbicularis Oris Loop          │ Concentric radial quad rings encircling the lips and mouth   │
│                                   │ opening. Meets the philtrum column seamlessly.               │
├───────────────────────────────────┼──────────────────────────────────────────────────────────────┤
│ 3. Nasolabial Fold Loop           │ Extends from the top of the nose bridge, contours around     │
│                                   │ the alar base of the nostril, down around the mouth corners. │
├───────────────────────────────────┼──────────────────────────────────────────────────────────────┤
│ 4. Jawline / Mandibular Loop      │ Sweeps from the chin apex along the lower jaw mandible to   │
│                                   │ behind the earlobe, separating the neck from the face.       │
├───────────────────────────────────┼──────────────────────────────────────────────────────────────┤
│ 5. Forehead / Frontalis Loops     │ Horizontal quad spans traversing the forehead from temple    │
│                                   │ to temple to accommodate eyebrow elevation creasing.         │
└───────────────────────────────────┴──────────────────────────────────────────────────────────────┘
```

---

## 3. Joint Articulation & Deformation Loop Mechanics

When organic joints flex (elbows, knees, finger phalanges, shoulders), single-edge loops collapse and lose internal volume. All articulating joints MUST utilize **Triple-Span Deformation Topology**:

```text
EXTENDED ARM (0° Flexion):
    │   │   │   │   │
────┼───┼───┼───┼───┼────  <-- Outer loop (Extensor side)
    │   │   │   │   │
────┼───┼───┼───┼───┼────  <-- Center hinge loop
    │   │   │   │   │
────┼───┼───┼───┼───┼────  <-- Inner loop (Flexor / Crease side)
    │   │   │   │   │

FLEXED ARM (120° Flexion):
Outer extensor quads stretch uniformly without tearing.
Inner flexor quads fold cleanly into a 3-loop crease without vertex penetration.
```

### Joint Polycount Budget Guidelines

- **Knees / Elbows**: Minimum 3 dedicated edge loops centered across the pivot axis.
- **Shoulders / Clavicle**: Radial concentric loops branching into the pectoralis major and latissimus dorsi.
- **Finger Joints**: 3 tight parallel loops per interphalangeal joint.

---

## 4. FACS 52 Facial Blendshape Standard

For AAA game and cinematic performance capture, characters must provide the standardized **52 FACS (Facial Action Coding System) ShapeKeys / Morph Targets** (compatible with Apple ARKit, MetaHuman, and Maya LiveLink):

### Core FACS Morph Target Manifest

```text
Eye & Brow:
- eyeBlinkLeft, eyeBlinkRight
- eyeLookDownLeft, eyeLookDownRight, eyeLookInLeft, eyeLookInRight
- eyeLookOutLeft, eyeLookOutRight, eyeLookUpLeft, eyeLookUpRight
- eyeSquintLeft, eyeSquintRight, eyeWideLeft, eyeWideRight
- browDownLeft, browDownRight, browInnerUp, browOuterUpLeft, browOuterUpRight

Jaw & Mouth:
- jawOpen, jawForward, jawLeft, jawRight
- mouthClose, mouthFunnel, mouthPucker
- mouthLeft, mouthRight, mouthSmileLeft, mouthSmileRight
- mouthFrownLeft, mouthFrownRight, mouthDimpleLeft, mouthDimpleRight
- mouthStretchLeft, mouthStretchRight, mouthRollLower, mouthRollUpper
- mouthShrugLower, mouthShrugUpper, mouthPressLeft, mouthPressRight
- mouthLowerDownLeft, mouthLowerDownRight, mouthUpperUpLeft, mouthUpperUpRight

Cheek, Nose & Tongue:
- cheekPuff, cheekSquintLeft, cheekSquintRight
- noseSneerLeft, noseSneerRight, tongueOut
```

---

## 5. Real-Time Hair Cards & Groom Curves

1. **Card Hierarchy**:
   - **Base Cap / Scalp**: Dense opaque scalp mesh with painted root density.
   - **Volume Layer**: Large 3D curved cards establishing primary silhouette, clumps, and flow.
   - **Detail / Breakup Layer**: Medium density cards with strand variations, flyaways, and directional transitions.
   - **Stray / Wisps Layer**: Ultra-thin single-pixel alpha cards to break edge silhouette sharpness.
2. **Hair Card Normals**:
   - Vertex normals on hair cards must be **sphericalized** or transferred from the underlying scalp mesh to eliminate harsh paper-thin shading artifacts under dynamic directional light.
3. **Strand Groom Curves (UE5 Groom / Maya XGen)**:
   - Interpolated guides: 100-300 guide curves per groom asset.
   - Point count per curve: 16-32 CV points.
   - Root-to-tip width: $0.05\text{ mm}$ at root $\to 0.01\text{ mm}$ at tip.

---

## 6. Cloth Simulation Geometry Standards

Meshes intended for real-time cloth simulation (Chaos Cloth / Havok / Marvelous Designer export) must obey:

1. **Regularized Delaunay Triangulation or Isotropic Quads**: Edge lengths must be strictly uniform ($\Delta l \le 10\%$). Long, stretched sliver triangles cause severe solver instability and tearing.
2. **Particle Distance**: $5\text{ mm}$ for hero garments; $10\text{ mm}$ to $15\text{ mm}$ for background clothing.
3. **Weight Map Masking**: Dedicated vertex color channel (Alpha or Vertex Color Red) defining cloth stiffness and pin constraints ($1.0 = \text{rigid attached to body}$, $0.0 = \text{fully simulated dynamic cloth}$).

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
