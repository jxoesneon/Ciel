# Reference: OpenUSD & Unreal Engine 5 Integration Standards

## 1. OpenUSD (Universal Scene Description) Stage Architecture

OpenUSD is the foundational interchange format for modern VFX studios, feature films, and scalable multi-tool pipelines (NVIDIA Omniverse, Pixar USD, Maya, Houdini, Unreal Engine).

```text
/Root (UsdGeomXform)
 ├── /Geometry (UsdGeomScope)
 │    └── /SM_Prop_Generator_01 (UsdGeomMesh)
 │         ├── points (float3[])
 │         ├── faceVertexCounts (int[])
 │         ├── faceVertexIndices (int[])
 │         ├── normals (normal3[] - faceVarying)
 │         └── primvars:st (texCoord2f[] - faceVarying)
 └── /Looks (UsdShadeMaterial)
      └── /M_Generator_01 (UsdShadeMaterial)
           └── /PBRShader (UsdShadeShader - UsdPreviewSurface / OpenPBR)
```

### UsdGeomMesh Schema Specification (USDA format):

```python
# UsdGeomMesh definition example
"""
def Mesh "SM_Prop_Generator_01" (
    prepend apiSchemas = ["MaterialBindingAPI"]
)
{
    uniform bool doubleSided = 0
    float3[] extent = [(-1.5, -0.8, 0.0), (1.5, 0.8, 2.2)]
    int[] faceVertexCounts = [4, 4, 4, 4]
    int[] faceVertexIndices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    normal3f[] normals = [(0, 0, 1), (0, 0, 1), (0, 0, 1), (0, 0, 1)] (
        interpolation = "faceVarying"
    )
    point3f[] points = [(-1.0, -0.5, 0.0), (1.0, -0.5, 0.0), (1.0, 0.5, 0.0), (-1.0, 0.5, 0.0)]
    texCoord2f[] primvars:st = [(0, 0), (1, 0), (1, 1), (0, 1)] (
        interpolation = "faceVarying"
    )
    rel material:binding = </Root/Looks/M_Generator_01>
}
"""
```

---

## 2. Unreal Engine 5 Nanite & Lumen Optimization

### Nanite Geometry Requirements:
1. **Watertight Solids**: Open sheets or 1-sided planes can cause cluster boundary shading discontinuities.
2. **Cluster Size**: Nanite operates on clusters of 128 triangles. Geometry with high curvature requires uniform triangle distribution to avoid micro-clusters.
3. **Materials**: Opaque or Masked blend modes. Translucent materials cannot be rendered via Nanite in UE5.

### Lumen Mesh Card & Distance Field Rules:
1. **Minimum Wall Thickness**: Geometry must have at least **$5.0\text{ cm}$ physical thickness**. Zero-thickness 1-sided planes cause severe Lumen Global Illumination light leaking.
2. **Mesh Distance Field Resolution**: Set `DistanceFieldResolutionScale` to $1.0 - 2.0$ for hero props with tight crevices to ensure accurate ambient shadowing.

---

## 3. Chaos Physics Collision Geometry (`UCX_` / `UBX_` / `USP_`)

Physics collisions should never calculate against dense render geometry. Lightweight convex collision hulls must be authored alongside the visual mesh:

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             PHYSICS COLLISION HULL SPECIFICATIONS                                │
├──────────────────────────┬───────────────────────────────┬───────────────────────────────────────┤
│ HULL TYPE                │ NAMING CONVENTION             │ CONSTRAINTS & RULES                   │
├──────────────────────────┼───────────────────────────────┼───────────────────────────────────────┤
│ Convex Hull              │ `UCX_<RenderMeshName>_<Index>`│ 100% strictly convex. No inward dents.│
│                          │                               │ Concave shapes require multiple UCX.  │
├──────────────────────────┼───────────────────────────────┼───────────────────────────────────────┤
│ Box Primitive            │ `UBX_<RenderMeshName>_<Index>`│ Oriented 6-sided bounding box.        │
│                          │                               │ Lowest CPU solver overhead.           │
├──────────────────────────┼───────────────────────────────┼───────────────────────────────────────┤
│ Sphere Primitive         │ `USP_<RenderMeshName>_<Index>`│ 3D Sphere geometry (1-parameter r).   │
├──────────────────────────┼───────────────────────────────┼───────────────────────────────────────┤
│ Capsule Primitive        │ `UCP_<RenderMeshName>_<Index>`│ Cylinder with hemispherical end caps. │
└──────────────────────────┴───────────────────────────────┴───────────────────────────────────────┘
```

### Automated Convex Decomposition (V-HACD):
For complex concave meshes (e.g. hollow pipes, chairs, open doorways), the pipeline runs **V-HACD (Volumetric Hierarchical Approximate Convex Decomposition)**:
- `max_convex_hulls`: 8 to 16 hulls per asset.
- `max_vertices_per_hull`: 32 vertices.
- `resolution`: 100,000 voxels.


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
