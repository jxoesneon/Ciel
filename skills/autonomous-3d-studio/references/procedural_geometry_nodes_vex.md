# Reference: Procedural Geometry Nodes & Houdini VEX Standards

## 1. Procedural 3D Architecture

Procedural 3D systems allow autonomous agents to generate infinite non-destructive variations of modular kits, environment architecture, pipes/cabling, and parametric props while maintaining deterministic geometric quality.

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             PROCEDURAL GENERATION PIPELINE DATA FLOW                             │
├─────────────────┬──────────────────┬──────────────────┬─────────────────┬────────────────────────┤
│ 1. Curve / Seed │ 2. Extrusion &   │ 3. Boolean &     │ 4. Attribute    │ 5. Realize &           │
│ Input           │ Profile Meshing  │ Bevel Modifiers  │ Computation     │ Mesh Clean             │
├─────────────────┼──────────────────┼──────────────────┼─────────────────┼────────────────────────┤
│ • Guide paths   │ • Curve-to-Mesh  │ • Cutter sockets │ • UV coordinates│ • Weld vertices        │
│ • Grid matrices │ • Wall thickness │ • Chamfer bevels │ • AO / Wear mask│ • Sharp edge tag       │
│ • Poisson points│ • Sweep profiles │ • Intersections  │ • Material IDs  │ • Nanite export        │
└─────────────────┴──────────────────┴──────────────────┴─────────────────┴────────────────────────┘
```

---

## 2. Blender Geometry Nodes Node Tree Patterns

### Pattern A: Procedural Cable / Conduit Harness
1. **Input**: Bezier curve with handle tangents.
2. **Resample Curve**: `Mode = 'LENGTH'`, `Length = 0.05 m`.
3. **Curve to Mesh**:
   - `Profile Curve`: `Curve Circle` (Radius $= r_{\text{cable}}$, Resolution $= 12$).
4. **UV Unwrapping on Curves**:
   - $U = \frac{\theta}{2\pi} \in [0, 1]$ (Circular profile coordinate).
   - $V = \text{Spline Parameter (Length)} \times \text{Tiling Factor}$.
   - Store as Named Attribute `UVMap` (Vector 2D, Face Corner domain).

### Pattern B: Poisson Disk Scatter with Slope/Normal Masking
To scatter debris, bolts, foliage, or modular detail assets without inter-penetration:
```python
# Geometry Nodes Field Evaluation Logic
import mathutils

def evaluate_scatter_points(mesh, min_distance=0.2, max_density=50.0, max_slope_angle=30.0):
    """
    Simulates Poisson Disk Distribution over surface geometry.
    Rejects points where surface normal dot product with Up (0,0,1) < cos(max_slope_angle).
    """
    # 1. Distribute Points on Faces (Poisson Disk)
    # 2. Filter by Normal: dot(Normal, (0,0,1)) >= cos(radians(max_slope_angle))
    # 3. Align Euler to Vector: Z-axis aligned to surface normal, random Z rotation [0, 2*pi]
    # 4. Instance on Points -> Realize Instances -> Weld Coincident Vertices
    pass
```

---

## 3. Houdini VEX & HDA Scripting Patterns

For Houdini procedural generation, agents employ VEX Point and Primitive Wrangles:

### VEX Wrangle 1: Procedural Edge Chamfer & Surface Normal Tangent
```c
// Point Wrangle: Calculate curvature and displace along smooth surface normal
vector pos = @P;
vector norm = @N;
float curvature = 0.0;

// Neighbor search for local curvature estimation
int neighbors[] = neighbours(0, @ptnum);
int count = len(neighbors);

if (count > 0) {
    vector avg_neighbor_pos = set(0, 0, 0);
    for (int i = 0; i < count; i++) {
        avg_neighbor_pos += point(0, "P", neighbors[i]);
    }
    avg_neighbor_pos /= float(count);
    curvature = length(pos - avg_neighbor_pos);
}

// Bind curvature to vertex attribute for texturing mask
f@curvature = fit(curvature, 0.0, 0.05, 0.0, 1.0);
```

### VEX Wrangle 2: Ray-Casting Alignment against Terrains
```c
// Point Wrangle: Project procedural kit roots to ground collision mesh
vector ground_pos;
vector ground_norm;
int hit = intersect(1, @P + set(0, 50, 0), set(0, -100, 0), ground_pos, ground_norm);

if (hit >= 0) {
    @P = ground_pos;
    // Align instance orientation to ground normal
    vector up = set(0, 1, 0);
    vector cross_prod = cross(up, ground_norm);
    @N = ground_norm;
    @up = normalize(cross(ground_norm, cross_prod));
}
```

---

## 4. Modular Kit Snapping & Grid Math

To ensure seamless tiling without light leaks or visible gaps in real-time game engines:

$$\text{Snap}(x, G) = \left\lfloor \frac{x}{G} + 0.5 \right\rfloor \cdot G$$

Where $G$ is the grid snap increment ($1.0\text{ m}$, $0.5\text{ m}$, $0.25\text{ m}$, or $0.1\text{ m}$).

### Metric Wall/Floor Modular Rule:
- All wall panels must align to $400\text{ cm} \times 300\text{ cm}$ or $200\text{ cm} \times 300\text{ cm}$ increments.
- Pivot points must be placed at the bottom-left corner ($X_{\min}, Y_{\min}, Z_{\min}$) or bottom-center ($X_{\text{mid}}, Y_{\text{mid}}, Z_{\min}$) of the bounding box.
- All connecting edges must have coplanar vertex normals ($\vec{N} = (0, \pm 1, 0)$ or $(\pm 1, 0, 0)$) to prevent visible seam splits across tiled modular sections under dynamic lighting.


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
