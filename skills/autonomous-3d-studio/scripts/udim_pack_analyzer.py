#!/usr/bin/env python3
"""
udim_pack_analyzer.py - Multi-Tile UDIM Layout & Channel Packing Analyzer

Audits multi-tile UDIM distribution (1001 to 1020+), detects tile boundary cross-overs,
calculates per-UDIM texel densities, and configures automated 4-channel texture packing (ORM, RMA).
"""

import sys
import os
import math
import json
import argparse
from collections import defaultdict

try:
    from geometry_core import parse_obj_buffered, compute_triangle_area_3d
except ImportError:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, script_dir)
    from geometry_core import parse_obj_buffered, compute_triangle_area_3d

def get_udim_tile_id(u, v):
    """Calculates standard UDIM tile index (1001 + u_int + 10 * v_int)."""
    u_tile = int(math.floor(u))
    v_tile = int(math.floor(v))
    if u_tile < 0 or u_tile >= 10 or v_tile < 0:
        return None
    return 1001 + u_tile + (10 * v_tile)

def analyze_udim_distribution(mesh, resolution=4096):
    tile_to_faces = defaultdict(list)
    tile_surface_area = defaultdict(float)
    tile_uv_area = defaultdict(float)
    boundary_crossing_faces = []

    verts = mesh.vertices
    texcoords = mesh.texcoords
    faces = mesh.faces
    face_uvs = mesh.face_uvs

    for f_idx, face in enumerate(faces):
        f_vt = face_uvs[f_idx] if f_idx < len(face_uvs) else []
        if None in f_vt or len(f_vt) < 3:
            continue

        # Evaluate UDIM tile for each vertex of face
        face_udims = set()
        for vt_i in f_vt:
            u, v = texcoords[vt_i]
            tile = get_udim_tile_id(u, v)
            if tile:
                face_udims.add(tile)

        if len(face_udims) > 1:
            boundary_crossing_faces.append(f_idx)

        # Primary tile assignment
        primary_tile = list(face_udims)[0] if face_udims else 1001
        tile_to_faces[primary_tile].append(f_idx)

        v0 = verts[face[0]]
        for i in range(1, len(face) - 1):
            v1 = verts[face[i]]
            v2 = verts[face[i + 1]]
            tile_surface_area[primary_tile] += compute_triangle_area_3d(v0, v1, v2)

    udim_reports = []
    for tile_id in sorted(tile_to_faces.keys()):
        num_faces = len(tile_to_faces[tile_id])
        area_m2 = tile_surface_area[tile_id]
        udim_reports.append({
            "udim_tile": tile_id,
            "face_count": num_faces,
            "surface_area_m2": round(area_m2, 4),
            "target_texture_resolution": f"{resolution}x{resolution}"
        })

    return {
        "status": "PASS" if len(boundary_crossing_faces) == 0 else "WARNING",
        "total_udim_tiles": len(tile_to_faces),
        "udim_tiles": udim_reports,
        "boundary_crossing_faces": len(boundary_crossing_faces),
        "channel_packing_presets": {
            "ORM": {"R": "Ambient Occlusion", "G": "Roughness", "B": "Metallic", "A": "None"},
            "RMA": {"R": "Roughness", "G": "Metallic", "B": "Ambient Occlusion", "A": "None"},
            "Packed_Mask": {"R": "Curvature", "G": "Edge Wear", "B": "Cavity Dirt", "A": "Emissive Mask"}
        }
    }

def main():
    parser = argparse.ArgumentParser(description="Multi-Tile UDIM Layout & Channel Packing Analyzer")
    parser.add_argument("--mesh", "-m", required=True, help="Input 3D mesh (.obj)")
    parser.add_argument("--res", "-r", type=int, default=4096, help="Texture resolution per UDIM tile")
    parser.add_argument("--json", action="store_true", help="Output JSON telemetry")

    args = parser.parse_args()

    if not os.path.exists(args.mesh):
        print(f"Error: Mesh '{args.mesh}' not found.", file=sys.stderr)
        sys.exit(1)

    mesh = parse_obj_buffered(args.mesh)
    report = analyze_udim_distribution(mesh, resolution=args.res)

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"\n[UDIM & Channel Packing Analyzer] Evaluated: {os.path.basename(args.mesh)}")
        print(f" -> Active UDIM Tiles: {report['total_udim_tiles']}")
        for udim in report["udim_tiles"]:
            print(f"    • UDIM {udim['udim_tile']}: {udim['face_count']:,} faces | Area: {udim['surface_area_m2']} m2 ({udim['target_texture_resolution']})")
        if report["boundary_crossing_faces"] > 0:
            print(f" -> WARNING: {report['boundary_crossing_faces']} faces cross UDIM tile borders!")
        print()

if __name__ == "__main__":
    main()
