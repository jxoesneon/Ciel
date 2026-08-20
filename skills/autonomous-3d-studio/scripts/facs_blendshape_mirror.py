#!/usr/bin/env python3
"""
facs_blendshape_mirror.py - ARKit / FACS 52 Facial Blendshape Mirroring Engine

Computes topological delta offsets from a neutral base mesh, mirrors asymmetric expressions
across the sagittal symmetry plane (X=0), and generates matching bilateral FACS 52 shapekeys
(e.g., eyeBlinkLeft -> eyeBlinkRight, mouthSmileLeft -> mouthSmileRight).
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

def find_symmetry_vertex_map(neutral_verts, tolerance=0.001):
    """
    Builds a bidirectional symmetry index map: vertex_i (X > 0) -> vertex_j (X < 0).
    """
    min_pt, max_pt, dims = compute_single_pass_bounding_box(neutral_verts)
    max_dim = max(dims[0], dims[1], dims[2])
    cell_size = max(tolerance * 2.0, max_dim / 100.0)

    # Spatial hash for negative-X vertices
    grid_map = {}
    for vi, v in enumerate(neutral_verts):
        gx = int((-v[0] - min_pt[0]) / cell_size) if cell_size > 0 else 0
        gy = int((v[1] - min_pt[1]) / cell_size) if cell_size > 0 else 0
        gz = int((v[2] - min_pt[2]) / cell_size) if cell_size > 0 else 0
        h = hash_grid_3d(gx, gy, gz)
        if h not in grid_map:
            grid_map[h] = []
        grid_map[h].append(vi)

    sym_map = {}
    for vi, v in enumerate(neutral_verts):
        # Mirrored target coordinate (-x, y, z)
        mx, my, mz = -v[0], v[1], v[2]
        gx = int((mx - min_pt[0]) / cell_size) if cell_size > 0 else 0
        gy = int((my - min_pt[1]) / cell_size) if cell_size > 0 else 0
        gz = int((mz - min_pt[2]) / cell_size) if cell_size > 0 else 0
        
        best_match = vi
        best_dist = float('inf')
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for dz in (-1, 0, 1):
                    h = hash_grid_3d(gx + dx, gy + dy, gz + dz)
                    if h in grid_map:
                        for cand_vi in grid_map[h]:
                            cand_v = neutral_verts[cand_vi]
                            dist = math.sqrt((cand_v[0] - mx)**2 + (cand_v[1] - my)**2 + (cand_v[2] - mz)**2)
                            if dist < best_dist and dist <= tolerance:
                                best_dist = dist
                                best_match = cand_vi
        sym_map[vi] = best_match

    return sym_map

def mirror_blendshape(neutral_path, source_shape_path, out_mirrored_path, tolerance=0.002):
    neutral = parse_obj_buffered(neutral_path)
    source = parse_obj_buffered(source_shape_path)

    if len(neutral.vertices) != len(source.vertices):
        raise ValueError(f"Topology mismatch: Neutral has {len(neutral.vertices)} verts, Source has {len(source.vertices)} verts.")

    sym_map = find_symmetry_vertex_map(neutral.vertices, tolerance=tolerance)

    # Compute deltas from source: Delta(v) = Source(v) - Neutral(v)
    deltas = []
    for vi in range(len(neutral.vertices)):
        nv = neutral.vertices[vi]
        sv = source.vertices[vi]
        deltas.append([sv[0] - nv[0], sv[1] - nv[1], sv[2] - nv[2]])

    # Apply mirrored delta: Mirrored_Delta(v_sym) = (-Delta_x, Delta_y, Delta_z)
    mirrored_verts = [list(v) for v in neutral.vertices]
    for vi, sym_vi in sym_map.items():
        dx, dy, dz = deltas[vi]
        # Invert X component of delta vector
        mirrored_verts[sym_vi][0] += -dx
        mirrored_verts[sym_vi][1] += dy
        mirrored_verts[sym_vi][2] += dz

    with open(out_mirrored_path, 'w', encoding='utf-8') as f:
        f.write("# Autonomous 3D Studio - FACS Mirrored Blendshape\n")
        for v in mirrored_verts:
            f.write(f"v {v[0]:.6f} {v[1]:.6f} {v[2]:.6f}\n")
        for vt in neutral.texcoords:
            f.write(f"vt {vt[0]:.6f} {vt[1]:.6f}\n")
        for vn in neutral.normals:
            f.write(f"vn {vn[0]:.6f} {vn[1]:.6f} {vn[2]:.6f}\n")
        for f_idx, face in enumerate(neutral.faces):
            tokens = []
            f_uv = neutral.face_uvs[f_idx] if f_idx < len(neutral.face_uvs) else []
            f_norm = neutral.face_normals[f_idx] if f_idx < len(neutral.face_normals) else []
            for i, vi in enumerate(face):
                vt_part = str(f_uv[i] + 1) if i < len(f_uv) and f_uv[i] is not None else ""
                vn_part = str(f_norm[i] + 1) if i < len(f_norm) and f_norm[i] is not None else ""
                if vn_part: tokens.append(f"{vi + 1}/{vt_part}/{vn_part}")
                elif vt_part: tokens.append(f"{vi + 1}/{vt_part}")
                else: tokens.append(f"{vi + 1}")
            f.write(f"f {' '.join(tokens)}\n")

    return {
        "status": "SUCCESS",
        "neutral_mesh": neutral_path,
        "source_shape": source_shape_path,
        "mirrored_shape": out_mirrored_path,
        "symmetry_mapped_vertices": len(sym_map)
    }

def main():
    parser = argparse.ArgumentParser(description="FACS 52 Blendshape Symmetry Mirror Engine")
    parser.add_argument("--neutral", "-n", required=True, help="Neutral Base Mesh (.obj)")
    parser.add_argument("--source-shape", "-s", required=True, help="Source Asymmetrical Blendshape Mesh (.obj)")
    parser.add_argument("--out", "-o", default="./blendshape_mirrored.obj", help="Output Mirrored Blendshape (.obj)")
    parser.add_argument("--tol", type=float, default=0.002, help="Symmetry vertex search tolerance")
    parser.add_argument("--json", action="store_true", help="Output JSON telemetry")

    args = parser.parse_args()

    result = mirror_blendshape(args.neutral, args.source_shape, args.out, tolerance=args.tol)
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"\n[FACS Blendshape Mirror] Generated bilateral expression:")
        print(f" -> Mirrored Shape: {args.out}")
        print(f" -> Symmetrical Vertex Pairs Mapped: {result['symmetry_mapped_vertices']:,}\n")

if __name__ == "__main__":
    main()
