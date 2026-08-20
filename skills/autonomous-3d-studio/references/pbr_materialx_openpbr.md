# Reference: PBR, MaterialX & OpenPBR Standards

## 1. Physically Based Rendering (PBR) Fundamentals

AAA texturing adheres strictly to physical energy conservation and microfacet theory (Cook-Torrance BRDF).

$$f_r(\omega_i, \omega_o) = k_d \frac{c}{\pi} + k_s \frac{D(h) F(\omega_o, h) G(\omega_i, \omega_o, h)}{4 (\vec{n} \cdot \omega_i)(\vec{n} \cdot \omega_o)}$$

### The Strict Binary Metallic Rule

- **Dielectrics (Non-Metals)**: Wood, plastic, cloth, stone, skin, glass.
  - $\text{Metallic} = 0.0$.
  - Specular Reflectance at normal incidence $F_0 \approx 0.04$ ($4\%$).
  - Base Color defines diffuse reflected albedo ($30 \le \text{sRGB} \le 240$; pure 0 or 255 is physically impossible).
- **Conductors (Pure Metals)**: Iron, gold, copper, aluminum, chrome.
  - $\text{Metallic} = 1.0$.
  - $F_0 \in [0.70, 1.00]$ (specular reflection inherits Base Color tint).
  - Diffuse Albedo $k_d = 0.0$ (no diffuse reflection).
- **Semiconductors / Transition Zones (Values between 0.0 and 1.0)**:
  - ONLY permitted across narrow 1-2 pixel transition edges representing dust, rust, oxidation, or grease films covering metal.

---

## 2. Channel Packing Standards

To minimize texture sampling draw calls in real-time GPU pixel shaders, multiple grayscale PBR channels are packed into single multi-channel RGBA textures:

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             STANDARD ORM PACKING CONFIGURATION                                   │
├─────────────────┬─────────────────┬─────────────────┬────────────────────────────────────────────┤
│ RED CHANNEL     │ GREEN CHANNEL   │ BLUE CHANNEL    │ ALPHA CHANNEL (Optional)                   │
├─────────────────┼─────────────────┼─────────────────┼────────────────────────────────────────────┤
│ Ambient         │ Roughness       │ Metallic        │ Thickness / Opacity / Detail Mask          │
│ Occlusion (AO)  │ (Micro-surface) │ (Conductivity)  │                                            │
└─────────────────┴─────────────────┴─────────────────┴────────────────────────────────────────────┘
```

### Color Space Assignment Table

| Texture File | Format | Channels | Color Space | Bit Depth |
| :--- | :--- | :--- | :--- | :--- |
| `T_<Name>_BC` (Base Color) | PNG / TGA | RGB | **sRGB / sRGB-Linear** | 8-bit |
| `T_<Name>_ORM` (AO, Rough, Metal) | PNG / TGA | RGB | **Linear / Raw / Non-Color** | 8-bit |
| `T_<Name>_N` (Normal Map) | PNG / EXR | RGB | **Linear / Raw / Non-Color** | **16-bit Float** |
| `T_<Name>_E` (Emissive) | PNG / EXR | RGB | **sRGB / sRGB-Linear** | 8-bit / 16-bit |
| `T_<Name>_H` (Displacement) | EXR / TIFF | R | **Linear / Raw / Non-Color** | **16/32-bit Float** |

---

## 3. MaterialX (`.mtlx`) & OpenPBR Surface Specification

MaterialX is the open standard for rich procedural and texture-driven shader definitions across Maya, Houdini, Unreal Engine, and WebGPU.

### Production OpenPBR MaterialX Document Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<materialx version="1.38">
  <!-- Master OpenPBR Surface Shader Node -->
  <open_pbr_surface name="SR_CyberArmor" type="surfaceshader">
    <!-- Base Color Connection -->
    <input name="base_color" type="color3" nodename="node_tex_basecolor" />
    <input name="base_metalness" type="float" nodename="node_extract_metalness" />
    <input name="specular_roughness" type="float" nodename="node_extract_roughness" />
    <input name="specular_ior" type="float" value="1.5" />
    
    <!-- Subsurface Scattering (For Organics/Polymer) -->
    <input name="subsurface_weight" type="float" value="0.0" />
    <input name="subsurface_color" type="color3" value="1.0, 0.2, 0.1" />
    <input name="subsurface_radius" type="float" value="0.01" />
    
    <!-- Geometry Normals -->
    <input name="geometry_normal" type="vector3" nodename="node_normalmap" />
  </open_pbr_surface>

  <!-- Texture Samplers -->
  <image name="node_tex_basecolor" type="color3">
    <input name="file" type="filename" value="textures/T_Armor_BC.png" colorspace="srgb_texture" />
  </image>

  <image name="node_tex_orm" type="color3">
    <input name="file" type="filename" value="textures/T_Armor_ORM.png" colorspace="lin_rec709" />
  </image>

  <!-- Channel Extractors from ORM -->
  <extract name="node_extract_roughness" type="float">
    <input name="in" type="color3" nodename="node_tex_orm" />
    <input name="index" type="integer" value="1" />
  </extract>

  <extract name="node_extract_metalness" type="float">
    <input name="in" type="color3" nodename="node_tex_orm" />
    <input name="index" type="integer" value="2" />
  </extract>

  <!-- MikkTSpace Normal Map Evaluator -->
  <normalmap name="node_normalmap" type="vector3">
    <input name="in" type="vector3" nodename="node_tex_normal" />
    <input name="scale" type="float" value="1.0" />
    <input name="space" type="string" value="tangent" />
  </normalmap>

  <image name="node_tex_normal" type="vector3">
    <input name="file" type="filename" value="textures/T_Armor_N.png" colorspace="lin_rec709" />
  </image>

  <!-- Material Assignment -->
  <surfacematerial name="M_CyberArmor" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="SR_CyberArmor" />
  </surfacematerial>
</materialx>
```

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
