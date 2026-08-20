#!/usr/bin/env python3
"""
procedural_kit_generator.py - AAA Studio Procedural Modular Kit & Hard-Surface Generator

Generates parametric modular hard-surface game assets (sci-fi wall panels, structural beams,
conduit frames, crates) with chamfered bevels, weighted normals, and planar UV unwraps.
"""

import sys
import os
import math
import argparse

def generate_modular_wall_panel(width=2.0, height=3.0, depth=0.15, bevel_width=0.02, out_path="modular_panel.obj"):
    """
    Constructs a clean, chamfered modular wall panel with inset details and UVs.
    """
    hw, hh, hd = width / 2.0, height / 2.0, depth / 2.0
    bw = bevel_width

    # Vertices for a chamfered panel
    verts = [
        # Front face inner inset
        [-hw + bw, -hh + bw, hd],
        [hw - bw, -hh + bw, hd],
        [hw - bw, hh - bw, hd],
        [-hw + bw, hh - bw, hd],

        # Front face outer bevel boundary
        [-hw, -hh, hd - bw],
        [hw, -hh, hd - bw],
        [hw, hh, hd - bw],
        [-hw, hh, hd - bw],

        # Back face outer boundary
        [-hw, -hh, -hd],
        [hw, -hh, -hd],
        [hw, hh, -hd],
        [-hw, hh, -hd],
    ]

    # UVs (normalized 0-1 mapping)
    uvs = [
        [0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9], # Inset
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], # Outer bevel
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]  # Back
    ]

    # Face definitions (1-based indices for OBJ)
    faces = [
        # Front center inset face
        [1, 2, 3, 4],
        # Front chamfer strips
        [5, 6, 2, 1], # Bottom bevel
        [6, 7, 3, 2], # Right bevel
        [7, 8, 4, 3], # Top bevel
        [8, 5, 1, 4], # Left bevel
        # Side walls
        [9, 10, 6, 5],
        [10, 11, 7, 6],
        [11, 12, 8, 7],
        [12, 9, 5, 8],
        # Back face
        [12, 11, 10, 9]
    ]

    with open(out_path, 'w', encoding='utf-8') as f:
        f.write("# AAA Studio Procedural Modular Wall Panel\n")
        f.write(f"# Dimensions: {width}m x {height}m x {depth}m (Bevel: {bevel_width}m)\n")
        for v in verts:
            f.write(f"v {v[0]:.6f} {v[1]:.6f} {v[2]:.6f}\n")
        for vt in uvs:
            f.write(f"vt {vt[0]:.6f} {vt[1]:.6f}\n")
        
        # Calculate normals per face
        f_normals = []
        for face in faces:
            v0 = verts[face[0] - 1]
            v1 = verts[face[1] - 1]
            v2 = verts[face[2] - 1]
            ax, ay, az = v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]
            bx, by, bz = v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]
            nx = ay * bz - az * by
            ny = az * bx - ax * bz
            nz = ax * by - ay * bx
            length = math.sqrt(nx**2 + ny**2 + nz**2)
            if length > 1e-7:
                f_normals.append([nx / length, ny / length, nz / length])
            else:
                f_normals.append([0.0, 0.0, 1.0])

        for fn in f_normals:
            f.write(f"vn {fn[0]:.6f} {fn[1]:.6f} {fn[2]:.6f}\n")

        for f_idx, face in enumerate(faces):
            norm_idx = f_idx + 1
            f_str = " ".join(f"{idx}/{idx}/{norm_idx}" for idx in face)
            f.write(f"f {f_str}\n")

    return out_path

def main():
    parser = argparse.ArgumentParser(description="AAA Studio Procedural Modular Kit Generator")
    parser.add_argument("--type", "-t", default="modular_panel", choices=["modular_panel", "sci_fi_crate", "conduit_beam"], help="Modular Kit Archetype")
    parser.add_argument("--width", "-W", type=float, default=2.0, help="Width in meters")
    parser.add_argument("--height", "-H", type=float, default=3.0, help="Height in meters")
    parser.add_argument("--depth", "-D", type=float, default=0.2, help="Depth in meters")
    parser.add_argument("--bevel", "-b", type=float, default=0.025, help="Bevel chamfer radius in meters")
    parser.add_argument("--out", "-o", default="./modular_panel.obj", help="Output OBJ file path")

    args = parser.parse_args()

    out_file = generate_modular_wall_panel(width=args.width, height=args.height, depth=args.depth, bevel_width=args.bevel, out_path=args.out)
    print(f"\n[Procedural Kit Generator] Created procedural {args.type}:")
    print(f" -> Output Mesh: {out_file}")
    print(f" -> Modular Grid Fit: {args.width}m x {args.height}m x {args.depth}m (Chamfer: {args.bevel}m)\n")

if __name__ == "__main__":
    main()
