# AAA+ Spatial & 3D Engineering Masterclass Reference

This master technical document codifies studio-grade curricula and production standards synthesized from premier institutions (The Gnomon Workshop, CG Master Academy, Think Tank Training Centre, Rebelway, Vertex School, AnimSchool). It serves as Ciel's definitive technical art and 3D modeling operational manual across four core disciplines.

---

# Module 1: Character & Creature Production Pipeline

```
[Character Workflow]
Anatomical Sculpt (ZBrush) ──► Production Retopo & FACS ──► Cloth & Groom (MD / Cards) ──► MikkTSpace PBR ──► Engine Rig
```

## 1.1 Inside-Out Anatomy & Form Hierarchy
- **Primary Forms (Gesture & Silhouette)**: Establish bony landmarks (clavicle, acromion, C7 vertebra, iliac crest, greater trochanter, ulna styloid) using Dynamesh (Res 32–64) or Sculptris Pro. Silhouette must read clearly from 360 degrees before subdividing.
- **Secondary Forms (Biomechanical Volumes)**: Model muscles honoring origin and insertion points (e.g., sternocleidomastoid to mastoid process; pectoralis major twisting into the bicipital groove). Maintain skin tension vs. compression dynamics; keep adipose distinct from muscle volume.
- **Tertiary Forms & Micro-Detailing**: Align wrinkles and skin pores with Langer's Lines of cleavage. Detail non-destructively in ZBrush 3D Layers, using HD Geometry and scan projection for micro-pores and epidermal strain.
- **Comparative Zoology**: Map skeletal homologies (human ankle = quadruped hock; human metacarpals = ungulate cannon bone). Balance center of gravity and thoracic/pelvic tilt across plantigrade, digitigrade, and unguligrade locomotion.

## 1.2 Deformation Topology & FACS Facial Blendshapes
- **Concentric Loops**: Complete loop flows around ocular apertures (Orbicularis oculi extending into glabella) and oral apertures (Orbicularis oris transitioning into nasolabial folds).
- **Joint Articulations**: 3-loop hinge configuration across elbows and knees to eliminate volume collapse during 140° flexions.
- **Pole Placement**: Zero 6-sided poles. Position 5-sided poles strictly on flat bony planes with minimal deformation; never on active flexing hinges.
- **52 ARKit / FACS Blendshapes**: Author neutral-clamped (0.0–1.0) delta shapes for speech and emotional expression across Eye/Brow, Jaw, Mouth Primary, Mouth Articulation, and Cheek/Nose/Tongue. Implement Pose-Space Deformation (PSD) correctives for extreme compound shapes (e.g., jawOpen + mouthSmile).

## 1.3 Real-Time Grooming & Marvelous Designer Cloth
- **Hair Cards Pipeline**:
  - 3-tier layering: Layer 1 (Dense Base Cap), Layer 2 (Transition & Clump Volumes), Layer 3 (Flyaways & Breakup).
  - 4K Atlas baking: Albedo (root-to-tip gradient), Opacity (crisp high-contrast mask), Flow/Tangent Map (directional flow for anisotropic specular), and Root/Depth Map.
  - Dual-Specular Marschner / Kajiya-Kay shading model: Primary R specular (shifted 2°–4° rootward by cuticle tilt) + Secondary TRT colored cortex reflection + TT backlight transmission.
- **Marvelous Designer / Clo3D**:
  - Construction based on true 2D tailoring flat patterns.
  - Multi-phase particle distance: Phase 1 Blockout (20mm) -> Phase 2 Crease/Pleats (10mm) -> Phase 3 Final Freeze (5mm).
  - Quad retopology aligned to 2D pattern UVs, welded seams, and micro-cloth displacement.

---

# Module 2: Hard Surface, Weapons & Vehicle Engineering

```
[Hard Surface Workflow]
CAD/Sub-D Blockout ──► Live Booleans & Bevels ──► High-Poly Polish (ZBrush/Dynamesh) ──► Low-Poly Game Mesh ──► Skew-Painted Bake
```

## 2.1 Non-Destructive Modifiers & High-Poly Polish
- **Modifier Stack Architecture**: Base Mesh -> Live Booleans (Union/Difference/Intersection) -> Bevel Modifier (Weight/Angle 30°–45°, 2–3 segments, profile 0.70) -> Weighted Normal Modifier (Face Area + Corner Angle, Keep Sharp).
- **Face-Weighted Normals (FWN)**: Force vertex normals perpendicular to major planar faces, compressing curvature strictly into bevel strips to eliminate flat-surface shading gradients without perimeter support loops.
- **DynaMesh Polish Pipeline**: Merge boolean assemblies in ZBrush at 5M+ polys, auto-group by normals, and execute Polish by Features with ClayPolish to achieve uniform manufactured micro-bevels (0.5mm–2.0mm).

## 2.2 CAD / NURBS to Game-Ready Geometry
- **Tessellation Controls**: Angle tolerance 5.0°–12.0° with chordal deviation limits to prevent sliver triangles. Export with planar N-gons and dense fillet quads.
- **Retopology Translation**: Quad Remesher with hard-surface edge crease detection; dissolve coplanar edges and standardize radial cylinder sides (8, 12, 16, 24, 32) relative to camera importance.

## 2.3 Mechanical Rigging, Kinematics & Baking Protocols
- **Hydraulic & Suspension Systems**: Opposing Aim/Look-At constraints between cylinder base and piston rod with pole vector isolation to eliminate axial flipping during multi-axis compression.
- **Firearm Kinematics**: Rotation-driven trigger-sear-hammer mechanical curves, linear bolt carrier translation, and spring-damped extractor clearance.
- **Artifact-Free Baking**:
  - *Hard Edge = UV Seam Cut*: Every smoothing group split must have a corresponding UV cut with >= 16px padding on 4K maps to prevent mipmap black line bleeding.
  - *Skew Painting*: Paint projection ray angles 90° perpendicular to low-poly faces to prevent skewed circular bolt and port bakes.
  - *Match by Name*: Exploded sub-mesh suffix matching (`_high` / `_low`) to isolate AO and raycasting between adjacent assemblies.

---

# Module 3: Environment Art & Procedural World Building

```
[Environment Workflow]
Metric Modular Kit ──► Trimsheet & Layered Shader ──► Photogrammetry Delighting ──► Houdini/PCG Graphs ──► Nanite Virtualization
```

## 3.1 Modular Kits & Dimensional Discipline
- **Metric Grid Standard**: Powers of 10 cm in 1 unit = 1 cm engines. Wall modules (400cm W x 300/400cm H, 20/40cm thickness). Standardized doorways (120x220cm) and windows (100x150cm) on 10cm grid snapping.
- **Pivot Anchoring**: Bottom-left corner for walls, center-bottom for columns. L-shaped corner caps (20x20cm) and 20cm thickness offsets to eliminate corner light leaks.
- **Face-Weighted Normals**: 2–4cm chamfers with FWN on structural pieces to provide smooth bevel roll-offs without unique normal maps.

## 3.2 Advanced Trimsheets & Layered Vertex Shaders
- **Texel Density Formulation**: Uniform 10.24 px/cm for architecture (2048px / 2.0m). Unfold UVs into rectified horizontal strips along V-axis with infinite U-axis tiling.
- **4-Layer Vertex Color Blending**: Red (Plaster/Dirt), Green (Moss/Vegetation), Blue (Wetness/Puddles), Alpha (Micro-damage). Modulate transitions with HeightLerp contrast algorithms and inject AO/curvature crevice masks.

## 3.3 Photogrammetry, Delighting & Scanning
- **Cross-Polarized Capture**: Linear polarizer over lights + circular polarizer on lens at 90° to eliminate specular reflection. Calibrate exposure and white balance against X-Rite Neutral 5 (18% gray, sRGB 118–122).
- **Delighting Passes**: Spherical harmonics decomposition and inverse shadow lifting in Substance Sampler to yield raw, shadowless Albedo/BaseColor.

## 3.4 Procedural Content Generation (PCG) & Nanite Streaming
- **Houdini Digital Assets (HDAs)**: Export point clouds with `unreal_instance`, `unreal_material`, `transform`, and VEX-driven spline extrusions.
- **Unreal PCG Graphs**: Surface sampling -> Normal/Slope filtering (grass < 30°, rock > 35°) -> Perlin/Worley density noise -> Road/Spline exclusion differences -> Instanced static spawners.
- **Nanite Virtualization**: Modeled solid geometry over alpha cards on foliage to reduce VisBuffer pixel overdraw. Clamp World Position Offset (WPO) distance curves and use Nanite Skinning to prevent Virtual Shadow Map invalidation.

---

# Module 4: LookDev, PBR Material Design & Technical Art

```
[LookDev Workflow]
Substance Designer Frequency Graphs ──► MikkTSpace Multi-Channel Baking ──► Cook-Torrance BRDF ──► GPU Memory & Quad Optimization
```

## 4.1 Procedural Texture Synthesis (Substance 3D Designer)
- **3-Tier Frequency Graph**:
  - *Macro Frequency*: Primal mass patterns using Tile Sampler and Vector Map Displace.
  - *Meso Frequency*: Erosion and mechanical chipping via Slope Blur Grayscale and Non-Uniform Directional Warp.
  - *Micro Frequency*: Tertiary grain, pores, and micro-scratches through Highpass Grayscale.
- **Normal & Roughness Derivation**: Sobel derivative filtering for Height-to-Normal conversion. HeightBlend with depth-contrast masks. Systematically derive Roughness from Curvature, Cavity, and Micro-Normal variance.

## 4.2 Multi-Channel Baking Suite
- **MikkTSpace Normal Mapping**: Strict tangent/bitangent derivation ($B = (N \times T) \cdot \text{Sign}$) with $Y+$ (OpenGL/Godot/Blender) or $Y-$ (DirectX/Unreal) orientation.
- **Full Channel Output**: Tangent Normal, Ambient Occlusion (HBAO/Raytraced), Bent Normals (directional AO), Thickness/Translucency (internal SSS depth), Cavity/Curvature, Position, and ID Maps.

## 4.3 PBR Shading & Microfacet BRDF Math
- **Cook-Torrance Specular Model**:
  $$f_r(\mathbf{V}, \mathbf{L}) = \frac{D(\mathbf{H}) \cdot F(\mathbf{V}, \mathbf{H}) \cdot G(\mathbf{V}, \mathbf{L}, \mathbf{H})}{4 (\mathbf{N} \cdot \mathbf{V}) (\mathbf{N} \cdot \mathbf{L})}$$
  - *NDF ($D$)*: GGX / Trowbridge-Reitz distribution with $\alpha = \text{Roughness}^2$.
  - *Fresnel ($F$)*: Fresnel-Schlick approximation with dielectric $F_0 = 0.04$ and metallic $F_0 = \text{Albedo}$.
  - *Visibility ($G$)*: Smith-GGX correlated visibility factor.
- **Advanced Shading Layers**: Dual-Lobe SSS (Burley profile), dual-lobe Clearcoat, Anisotropic strand flow, and chromatic dispersion refraction for transmissive glass.

## 4.4 Engine Profiling & Optimization Budgets
- **ORM Texture Packing**: Red (Ambient Occlusion), Green (Roughness), Blue (Metallic), Alpha (Height/Curvature). Compress with BC1 (BaseColor), BC5 (Normals: $R$ & $G$, $B$ reconstructed as $\sqrt{1 - X^2 - Y^2}$), and BC7 (High-quality Albedo/Translucency).
- **GPU Scheduling & ALU Pressure**: Calculate invariant vectors in Vertex Shaders. Eliminate runtime branch divergences via branchless `mix()`, `step()`, and compile-time `#ifdef`. Avoid sub-8px triangles to prevent $2 \times 2$ pixel quad overdraw waste.
