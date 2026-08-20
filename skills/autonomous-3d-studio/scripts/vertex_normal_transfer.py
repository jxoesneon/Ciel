#!/usr/bin/env python3
"""
vertex_normal_transfer.py - CAD/SubD High-Poly Vertex Normal Projection Engine

Transfers smooth, face-area weighted vertex normals from a dense CAD/NURBS or SubD
source mesh onto a retopologized real-time target mesh, preserving crisp highlights
across complex boolean chamfers without ray cage distortion.
"""

import sys
import os
import math
import json
import argparse

try:
    from geometry_core import parse_obj_buffered, hash_grid_3d, compute_single_pass_bounding_box
except ImportError:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, script_dir)
    from geometry_core import parse_obj_buffered, hash_grid_3d, compute_single_pass_bounding_box

def find_nearest_normal_spatial(target_vert, source_verts, source_normals, grid_map, cell_size, min_pt):
    gx = int((target_vert[0] - min_pt[0]) / cell_size) if cell_size > 0 else 0
    gy = int((target_vert[1] - min_pt[1]) / cell_size) if cell_size > 0 else 0
    gz = int((target_vert[2] - min_pt[2]) / cell_size) if cell_size > 0 else 0

    best_dist_sq = float('inf')
    best_norm = [0.0, 0.0, 1.0]

    # Search 3x3x3 neighbor grid cells
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            for dz in (-1, 0, 1):
                h = hash_grid_3d(gx + dx, gy + dy, gz + dz)
                if h in grid_map:
                    for src_idx in grid_map[h]:
                        sv = source_verts[src_idx]
                        dist_sq = (sv[0] - target_vert[0])**2 + (sv[1] - target_vert[1])**2 + (sv[2] - target_vert[2])**2
                        if dist_sq < best_dist_sq:
                            best_dist_sq = dist_sq
                            if src_idx < len(source_normals):
                                best_norm = source_normals[src_idx]

    return best_norm

def transfer_vertex_normals(source_mesh_path, target_mesh_path, out_mesh_path):
    src = parse_obj_buffered(source_mesh_path)
    tgt = parse_obj_buffered(target_mesh_path)

    min_pt, max_pt, dims = compute_single_pass_bounding_box(src.vertices)
    max_dim = max(dims[0], dims[1], dims[2])
    grid_res = max(10, int(math.pow(len(src.vertices), 1.0 / 3.0)))
    cell_size = max_dim / grid_res if grid_res > 0 else 0.1

    # Spatial index for source vertices
    grid_map = {}
    for vi, v in enumerate(src.vertices):
        gx = int((v[0] - min_pt[0]) / cell_size) if cell_size > 0 else 0
        gy = int((v[1] - min_pt[1]) / cell_size) if cell_size > 0 else 0
        gz = int((v[2] - min_pt[2]) / cell_size) if cell_size > 0 else 0
        h = hash_grid_3d(gx, gy, gz)
        if h not in grid_map:
            grid_map[h] = []
        grid_map[h].append(vi)

    transferred_normals = []
    for tv in tgt.vertices:
        norm = find_nearest_normal_spatial(tv, src.vertices, src.normals, grid_map, cell_size, min_pt)
        transferred_normals.append(norm)

    # Write target mesh with transferred smooth normals
    with open(out_mesh_path, 'w', encoding='utf-8') as f:
        f.write("# Autonomous 3D Studio - CAD Vertex Normal Transfer Mesh\n")
        for v in tgt.vertices:
            f.write(f"v {v[0]:.6f} {v[1]:.6f} {v[2]:.6f}\n")
        for vt in tgt.texcoords:
            f.write(f"vt {vt[0]:.6f} {vt[1]:.6f}\n")
        for vn in transferred_normals:
            f.write(f"vn {vn[0]:.6f} {vn[1]:.6f} {vn[2]:.6f}\n")

        for f_idx, face in enumerate(tgt.faces):
            tokens = []
            f_uv = tgt.face_uvs[f_idx] if f_idx < len(tgt.face_uvs) else []
            for i, vi in enumerate(face):
                vt_part = str(f_uv[i] + 1) if i < len(f_uv) and f_uv[i] is not None else ""
                vn_part = str(vi + 1) # Mapped to vertex normal
                tokens.append(f"{vi + 1}/{vt_part}/{vn_part}")
            f.write(f"f {' '.join(tokens)}\n")

    return {
        "status": "SUCCESS",
        "source_mesh": source_mesh_path,
        "target_mesh": target_mesh_path,
        "output_mesh": out_mesh_path,
        "transferred_normals_count": len(transferred_normals)
    }

def main():
    parser = argparse.ArgumentParser(description="CAD/SubD Vertex Normal Transfer Engine")
    parser.add_argument("--source", "-s", required=True, help="Dense High-Poly CAD/SubD mesh (.obj)")
    parser.add_argument("--target", "-t", required=True, help="Low-Poly Game mesh (.obj)")
    parser.add_argument("--out", "-o", default="./target_transferred_normals.obj", help="Output OBJ file")
    parser.add_argument("--json", action="store_true", help="Output JSON telemetry")

    args = parser.parse_args()

    if not os.path.exists(args.source) or not os.path.exists(args.target):
        print("Error: Source or Target mesh file not found.", file=sys.stderr)
        sys.exit(1)

    result = transfer_vertex_normals(args.source, args.target, args.out)
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"\n[Vertex Normal Transfer] Successfully projected smooth CAD normals:")
        print(f" -> Output Mesh: {args.out}")
        print(f" -> Normals Transferred: {result['transferred_normals_count']:,}\n")

if __name__ == "__main__":
    main()
