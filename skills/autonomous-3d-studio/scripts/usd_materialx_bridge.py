#!/usr/bin/env python3
"""
usd_materialx_bridge.py - OpenUSD & MaterialX Production Stage Assembler

Assembles OpenUSD (.usda/.usd) stages, binds UsdGeomMesh primitives, and generates
standard OpenPBR / UsdPreviewSurface MaterialX (.mtlx) shader networks.
"""

import sys
import os
import math
import argparse

def convert_obj_to_usda(obj_path, out_usda_path, material_name="M_CyberAsset"):
    """
    Translates an OBJ 3D mesh into a clean OpenUSD ASCII (.usda) stage.
    """
    verts = []
    texcoords = []
    normals = []
    face_v_indices = []
    face_v_counts = []
    face_vt_indices = []
    face_vn_indices = []

    with open(obj_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            tokens = line.strip().split()
            if not tokens:
                continue
            if tokens[0] == 'v':
                verts.append(f"({float(tokens[1]):.6f}, {float(tokens[2]):.6f}, {float(tokens[3]):.6f})")
            elif tokens[0] == 'vt':
                u = float(tokens[1])
                v = float(tokens[2]) if len(tokens) > 2 else 0.0
                texcoords.append(f"({u:.6f}, {v:.6f})")
            elif tokens[0] == 'vn':
                normals.append(f"({float(tokens[1]):.6f}, {float(tokens[2]):.6f}, {float(tokens[3]):.6f})")
            elif tokens[0] == 'f':
                face_v = []
                face_vt = []
                face_vn = []
                for p in tokens[1:]:
                    parts = p.split('/')
                    v_idx = int(parts[0]) - 1 if parts[0] else 0
                    vt_idx = int(parts[1]) - 1 if len(parts) > 1 and parts[1] else 0
                    vn_idx = int(parts[2]) - 1 if len(parts) > 2 and parts[2] else 0
                    face_v.append(v_idx)
                    face_vt.append(vt_idx)
                    face_vn.append(vn_idx)
                face_v_counts.append(len(face_v))
                face_v_indices.extend(face_v)
                face_vt_indices.extend(face_vt)
                face_vn_indices.extend(face_vn)

    mesh_name = os.path.splitext(os.path.basename(obj_path))[0]

    usda_content = f"""#usda 1.0
(
    defaultPrim = "Root"
    metersPerUnit = 1.0
    upAxis = "Z"
)

def Xform "Root"
{{
    def Scope "Looks"
    {{
        def Material "{material_name}"
        {{
            token outputs:surface.connect = </Root/Looks/{material_name}/PBRShader.outputs:surface>

            def Shader "PBRShader"
            {{
                uniform token info:id = "UsdPreviewSurface"
                color3f inputs:diffuseColor = (0.8, 0.8, 0.8)
                float inputs:metallic = 0.0
                float inputs:roughness = 0.4
                token outputs:surface
            }}
        }}
    }}

    def Scope "Geometry"
    {{
        def Mesh "{mesh_name}" (
            prepend apiSchemas = ["MaterialBindingAPI"]
        )
        {{
            uniform bool doubleSided = 0
            int[] faceVertexCounts = [{", ".join(map(str, face_v_counts))}]
            int[] faceVertexIndices = [{", ".join(map(str, face_v_indices))}]
            point3f[] points = [{", ".join(verts)}]
            
            rel material:binding = </Root/Looks/{material_name}>
        }}
    }}
}}
"""
    with open(out_usda_path, 'w', encoding='utf-8') as f:
        f.write(usda_content)

    return out_usda_path

def generate_materialx_openpbr(mat_name, out_mtlx_path, basecolor_tex="", orm_tex="", normal_tex=""):
    """
    Generates an OpenPBR Surface MaterialX (.mtlx) document.
    """
    mtlx_code = f"""<?xml version="1.0" encoding="UTF-8"?>
<materialx version="1.38">
  <!-- OpenPBR Surface Standard -->
  <open_pbr_surface name="SR_{mat_name}" type="surfaceshader">
    <input name="base_color" type="color3" value="0.8, 0.8, 0.8" />
    <input name="base_metalness" type="float" value="0.0" />
    <input name="specular_roughness" type="float" value="0.35" />
    <input name="specular_ior" type="float" value="1.5" />
  </open_pbr_surface>

  <surfacematerial name="{mat_name}" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="SR_{mat_name}" />
  </surfacematerial>
</materialx>
"""
    with open(out_mtlx_path, 'w', encoding='utf-8') as f:
        f.write(mtlx_code)
    return out_mtlx_path

def main():
    parser = argparse.ArgumentParser(description="OpenUSD Stage & MaterialX Assembler")
    parser.add_argument("--mesh", "-m", required=True, help="Input OBJ mesh path")
    parser.add_argument("--usd", "-u", default="./stage.usda", help="Output OpenUSD (.usda) path")
    parser.add_argument("--matx", "-x", default="./material.mtlx", help="Output MaterialX (.mtlx) path")

    args = parser.parse_args()

    convert_obj_to_usda(args.mesh, args.usd)
    generate_materialx_openpbr("M_Asset", args.matx)

    print(f"\n[OpenUSD & MaterialX Bridge] Assembled production stage:")
    print(f" -> OpenUSD Stage: {args.usd}")
    print(f" -> MaterialX Shader: {args.matx}\n")

if __name__ == "__main__":
    main()
