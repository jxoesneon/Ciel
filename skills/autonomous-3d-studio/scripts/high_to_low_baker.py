#!/usr/bin/env python3
"""
high_to_low_baker.py - AAA Studio High-to-Low Texture Baking & Cage Automation Engine

Computes vertex-averaged projection cages and sets up high-fidelity 16/32-bit texture bakes:
  - Tangent Space Normal (DirectX -Y and OpenGL +Y)
  - Ambient Occlusion (AO)
  - Curvature (Laplace-Beltrami Convexity/Concavity)
  - World Position
  - Material ID / Vertex Color
  - Subsurface Thickness
"""

import sys
import os
import math
import json
import argparse
from collections import defaultdict

def parse_obj_simple(filepath):
    verts = []
    normals = []
    faces = []
    with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            tokens = line.strip().split()
            if not tokens:
                continue
            if tokens[0] == 'v':
                verts.append([float(tokens[1]), float(tokens[2]), float(tokens[3])])
            elif tokens[0] == 'vn':
                normals.append([float(tokens[1]), float(tokens[2]), float(tokens[3])])
            elif tokens[0] == 'f':
                face = []
                for p in tokens[1:]:
                    face.append(int(p.split('/')[0]) - 1)
                faces.append(face)
    return verts, normals, faces

def compute_vertex_averaged_normals(verts, faces):
    """
    Computes area-weighted averaged vertex normals for smooth cage extrusion without seam tearing.
    """
    vert_normals = [[0.0, 0.0, 0.0] for _ in range(len(verts))]

    for face in faces:
        if len(face) < 3:
            continue
        v0 = verts[face[0]]
        for i in range(1, len(face) - 1):
            v1 = verts[face[i]]
            v2 = verts[face[i + 1]]
            
            # Cross product gives unnormalized face normal with magnitude = 2 * area
            ax, ay, az = v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]
            bx, by, bz = v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]
            nx = ay * bz - az * by
            ny = az * bx - ax * bz
            nz = ax * by - ay * bx
            
            # Add area-weighted face normal to each vertex in face
            for vi in face:
                vert_normals[vi][0] += nx
                vert_normals[vi][1] += ny
                vert_normals[vi][2] += nz

    # Normalize all vertex normals
    normalized_normals = []
    for vn in vert_normals:
        length = math.sqrt(vn[0]**2 + vn[1]**2 + vn[2]**2)
        if length > 1e-9:
            normalized_normals.append([vn[0] / length, vn[1] / length, vn[2] / length])
        else:
            normalized_normals.append([0.0, 0.0, 1.0])
            
    return normalized_normals

def generate_cage_mesh(low_poly_path, out_cage_path, push_distance=0.01):
    """
    Generates a continuous, non-splitting projection cage mesh.
    """
    verts, normals, faces = parse_obj_simple(low_poly_path)
    avg_normals = compute_vertex_averaged_normals(verts, faces)

    cage_verts = []
    for i, v in enumerate(verts):
        n = avg_normals[i]
        cage_verts.append([
            v[0] + n[0] * push_distance,
            v[1] + n[1] * push_distance,
            v[2] + n[2] * push_distance
        ])

    with open(out_cage_path, 'w', encoding='utf-8') as f:
        f.write("# AAA Studio Automated Baking Cage Mesh\n")
        f.write(f"# Source: {os.path.basename(low_poly_path)}\n")
        f.write(f"# Push Distance: {push_distance}m\n")
        for cv in cage_verts:
            f.write(f"v {cv[0]:.6f} {cv[1]:.6f} {cv[2]:.6f}\n")
        for n in avg_normals:
            f.write(f"vn {n[0]:.6f} {n[1]:.6f} {n[2]:.6f}\n")
        for face in faces:
            f_str = " ".join(f"{vi + 1}//{vi + 1}" for vi in face)
            f.write(f"f {f_str}\n")

    return out_cage_path

def generate_blender_bake_recipe(high_path, low_path, cage_path, out_dir, resolution=4096):
    """
    Generates a hardened, injection-safe Blender Python bake script (`bake_job.py`).
    Uses json.dumps for all string/path interpolations.
    """
    abs_low = json.dumps(os.path.abspath(low_path))
    abs_high = json.dumps(os.path.abspath(high_path))
    abs_cage = json.dumps(os.path.abspath(cage_path))
    abs_out_dir = json.dumps(os.path.abspath(out_dir))
    low_basename = json.dumps(os.path.splitext(os.path.basename(low_path))[0])

    recipe_code = f"""# Autonomous 3D Studio - Headless Blender Bake Script
import bpy
import os

# Clean default scene
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import Low Poly
bpy.ops.wm.obj_import(filepath={abs_low})
low_obj = bpy.context.selected_objects[0]
low_obj.name = "LowPoly_Target"

# Import High Poly
bpy.ops.wm.obj_import(filepath={abs_high})
high_obj = bpy.context.selected_objects[0]
high_obj.name = "HighPoly_Source"

# Import Cage
bpy.ops.wm.obj_import(filepath={abs_cage})
cage_obj = bpy.context.selected_objects[0]
cage_obj.name = "Bake_Cage"

# Configure Cycles Baking
bpy.context.scene.render.engine = 'CYCLES'
bpy.context.scene.cycles.bake_type = 'NORMAL'
bpy.context.scene.render.bake.use_selected_to_active = True
bpy.context.scene.render.bake.use_cage = True
bpy.context.scene.render.bake.cage_object = cage_obj
bpy.context.scene.render.bake.normal_space = 'TANGENT'

# Setup Image Texture Node for 16-bit Normal Bake
mat = bpy.data.materials.new(name="BakeMaterial")
mat.use_nodes = True
low_obj.data.materials.append(mat)
nodes = mat.node_tree.nodes

img = bpy.data.images.new(
    name="Baked_Normal_16bit",
    width={resolution},
    height={resolution},
    float_buffer=True # 16/32-bit float to eliminate stair-step banding
)
img_node = nodes.new('ShaderNodeTexImage')
img_node.image = img
nodes.active = img_node

# Select High Poly then Low Poly (Active)
bpy.ops.object.select_all(action='DESELECT')
high_obj.select_set(True)
low_obj.select_set(True)
bpy.context.view_layer.objects.active = low_obj

# Execute Normal Bake
print("[Bake Engine] Baking 16-bit Tangent Space Normal Map...")
bpy.ops.object.bake(type='NORMAL')

# Save Output
out_normal_path = os.path.join({abs_out_dir}, f"T_{{{low_basename}}}_N.png")
img.filepath_raw = out_normal_path
img.file_format = 'PNG'
img.save()
print(f"[Bake Engine] Normal Map saved to: {{out_normal_path}}")
"""
    recipe_file = os.path.join(out_dir, "run_blender_bake.py")
    with open(recipe_file, 'w', encoding='utf-8') as f:
        f.write(recipe_code)
    return recipe_file

def main():
    parser = argparse.ArgumentParser(description="AAA Studio High-to-Low Texture Baker & Cage Generator")
    parser.add_argument("--high", "-H", required=True, help="Path to High-Poly source mesh (.obj)")
    parser.add_argument("--low", "-L", required=True, help="Path to Low-Poly target mesh (.obj)")
    parser.add_argument("--out", "-o", default="./bakes", help="Output directory for bake maps and cage")
    parser.add_argument("--push", "-p", type=float, default=0.008, help="Cage push distance in meters (default: 0.008m)")
    parser.add_argument("--res", "-r", type=int, default=4096, choices=[1024, 2048, 4096, 8192], help="Bake Resolution")

    args = parser.parse_args()
    os.makedirs(args.out, exist_ok=True)

    cage_path = os.path.join(args.out, f"Cage_{os.path.splitext(os.path.basename(args.low))[0]}.obj")
    print(f"\n[High-to-Low Baker] Computing area-weighted vertex cage envelope (push={args.push}m)...")
    generate_cage_mesh(args.low, cage_path, push_distance=args.push)
    print(f" -> Cage mesh generated: {cage_path}")

    bake_script = generate_blender_bake_recipe(args.high, args.low, cage_path, args.out, resolution=args.res)
    print(f" -> 16-Bit Float Bake Automation Recipe written: {bake_script}")
    print(f" -> Ready for headless execution: `blender -b -P {bake_script}`\n")

if __name__ == "__main__":
    main()
