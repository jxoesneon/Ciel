#!/usr/bin/env python3
"""
uv_texel_analyzer.py - AAA Studio UV Unwrapping & Texel Density Analyzer

Audits UV layout quality, texture packing efficiency, and calculates precise Texel Density.
Features:
  - Adaptive Texel Density & VRAM budgeting heuristics based on surface area & screen distance
  - Instinct telemetry logging (~/.ciel/instincts/3d_studio_observations.jsonl)
  - CIEL Artifact Vault routing (--vault -> ~/.ciel/artifacts/audits/)
"""

import sys
import os
import math
import json
import argparse
from datetime import datetime
from collections import defaultdict

try:
    from geometry_core import parse_obj_buffered, compute_triangle_area_3d
except ImportError:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, script_dir)
    from geometry_core import parse_obj_buffered, compute_triangle_area_3d

def compute_triangle_2d_signed_area(u0, u1, u2):
    return 0.5 * ((u1[0] - u0[0]) * (u2[1] - u0[1]) - (u2[0] - u0[0]) * (u1[1] - u0[1]))

def estimate_adaptive_vram_budget(total_surface_area_m2, target_td_px_cm=20.48):
    """
    Computes optimal texture resolution and VRAM footprint based on physical surface area.
    Formula: Res_optimal = sqrt(Area_cm2 * (TD)^2) rounded to nearest power of 2.
    """
    area_cm2 = total_surface_area_m2 * 10000.0
    ideal_pixels_1d = math.sqrt(area_cm2) * target_td_px_cm
    
    # Snap to standard power of 2 (512, 1024, 2048, 4096, 8192)
    powers = [512, 1024, 2048, 4096, 8192]
    closest_res = min(powers, key=lambda p: abs(p - ideal_pixels_1d))
    
    # 4 textures (BC, ORM, N_16bit, E): approx VRAM MB
    raw_vram_mb = (closest_res * closest_res * 4 * 4) / (1024 * 1024)
    compressed_vram_mb = raw_vram_mb / 4.0 # BC7 / DXT5 approx 4:1
    
    return {
        "ideal_resolution": closest_res,
        "recommended_texture_size": f"{closest_res}x{closest_res}",
        "estimated_vram_compressed_mb": round(compressed_vram_mb, 2)
    }

def analyze_texel_density(mesh, target_resolution=4096, target_td_px_cm=20.48):
    verts = mesh.vertices
    texcoords = mesh.texcoords
    faces = mesh.faces
    face_uvs = mesh.face_uvs

    if not texcoords or not face_uvs:
        return {"status": "FAIL", "error": "Mesh has no UV coordinates (vt missing).", "metrics": {}}

    total_3d_area_m2 = 0.0
    total_uv_normalized_area = 0.0
    face_td_values = []
    flipped_uv_faces = 0
    missing_uv_faces = 0

    uv_min_u, uv_max_u = float('inf'), float('-inf')
    uv_min_v, uv_max_v = float('inf'), float('-inf')

    for f_idx, face in enumerate(faces):
        f_vt = face_uvs[f_idx] if f_idx < len(face_uvs) else []
        if None in f_vt or len(f_vt) < 3:
            missing_uv_faces += 1
            continue

        v0_3d = verts[face[0]]
        vt0_2d = texcoords[f_vt[0]]
        
        for i in range(1, len(face) - 1):
            v1_3d = verts[face[i]]
            v2_3d = verts[face[i + 1]]
            area_3d = compute_triangle_area_3d(v0_3d, v1_3d, v2_3d)

            vt1_2d = texcoords[f_vt[i]]
            vt2_2d = texcoords[f_vt[i + 1]]
            signed_uv = compute_triangle_2d_signed_area(vt0_2d, vt1_2d, vt2_2d)

            if signed_uv < 0.0:
                flipped_uv_faces += 1
            
            abs_uv = abs(signed_uv)
            total_3d_area_m2 += area_3d
            total_uv_normalized_area += abs_uv

            for vt in [vt0_2d, vt1_2d, vt2_2d]:
                uv_min_u = min(uv_min_u, vt[0])
                uv_max_u = max(uv_max_u, vt[0])
                uv_min_v = min(uv_min_v, vt[1])
                uv_max_v = max(uv_max_v, vt[1])

            area_3d_cm2 = area_3d * 10000.0
            if area_3d_cm2 > 1e-6 and abs_uv > 1e-8:
                pixel_area = abs_uv * (target_resolution ** 2)
                td_cm = math.sqrt(pixel_area) / math.sqrt(area_3d_cm2)
                face_td_values.append(td_cm)

    if not face_td_values:
        return {"status": "FAIL", "error": "Zero valid UV area.", "metrics": {}}

    avg_td = sum(face_td_values) / len(face_td_values)
    min_td = min(face_td_values)
    max_td = max(face_td_values)
    variance_pct = ((max_td - min_td) / avg_td * 100.0) if avg_td > 0 else 0.0

    vram_heuristics = estimate_adaptive_vram_budget(total_3d_area_m2, target_td_px_cm=target_td_px_cm)

    qa_warnings = []
    passed = True

    if missing_uv_faces > 0:
        passed = False
        qa_warnings.append(f"CRITICAL: {missing_uv_faces} faces are missing UV texture mappings.")

    if flipped_uv_faces > 0:
        qa_warnings.append(f"WARNING: {flipped_uv_faces} UV sub-triangles are flipped.")

    if variance_pct > 15.0:
        qa_warnings.append(f"WARNING: High Texel Density variance ({round(variance_pct, 1)}%). Recommended <= 10%.")

    return {
        "status": "PASS" if passed else "FAIL",
        "target_resolution": target_resolution,
        "target_td_px_cm": target_td_px_cm,
        "texel_density": {
            "average_px_cm": round(avg_td, 3),
            "average_px_m": round(avg_td * 100.0, 1),
            "min_px_cm": round(min_td, 3),
            "max_px_cm": round(max_td, 3),
            "variance_percentage": round(variance_pct, 2)
        },
        "adaptive_vram_budget": vram_heuristics,
        "uv_space_metrics": {
            "total_3d_surface_area_m2": round(total_3d_area_m2, 4),
            "uv_coverage_percentage": round(total_uv_normalized_area * 100.0, 2),
            "uv_bounds": {
                "u_min": round(uv_min_u, 4), "u_max": round(uv_max_u, 4),
                "v_min": round(uv_min_v, 4), "v_max": round(uv_max_v, 4)
            },
            "flipped_uv_faces": flipped_uv_faces,
            "missing_uv_faces": missing_uv_faces
        },
        "qa_warnings": qa_warnings
    }

def record_uv_instinct(mesh_path, report):
    try:
        instinct_dir = os.path.expanduser("~/.ciel/instincts")
        os.makedirs(instinct_dir, exist_ok=True)
        with open(os.path.join(instinct_dir, "3d_studio_observations.jsonl"), 'a', encoding='utf-8') as f:
            f.write(json.dumps({
                "ts": datetime.utcnow().isoformat() + "Z",
                "asset": os.path.basename(mesh_path),
                "domain": "uv_texel_density",
                "avg_td_px_cm": report["texel_density"]["average_px_cm"],
                "variance_pct": report["texel_density"]["variance_percentage"],
                "warnings": report["qa_warnings"]
            }) + "\n")
    except Exception:
        pass

def main():
    parser = argparse.ArgumentParser(description="AAA Studio UV Unwrapping & Texel Density Analyzer")
    parser.add_argument("--mesh", "-m", required=True, help="Path to 3D mesh (.obj)")
    parser.add_argument("--res", "-r", type=int, default=4096, choices=[512, 1024, 2048, 4096, 8192])
    parser.add_argument("--target-td", "-t", type=float, default=20.48)
    parser.add_argument("--vault", action="store_true", help="Route report to CIEL Artifact Vault")
    parser.add_argument("--json", action="store_true", default=False)

    args = parser.parse_args()

    if not os.path.exists(args.mesh):
        print(f"Error: Mesh file '{args.mesh}' not found.", file=sys.stderr)
        sys.exit(1)

    mesh = parse_obj_buffered(args.mesh)
    report = analyze_texel_density(mesh, target_resolution=args.res, target_td_px_cm=args.target_td)
    record_uv_instinct(args.mesh, report)

    if args.vault:
        vault_dir = os.path.expanduser("~/.ciel/artifacts/audits")
        os.makedirs(vault_dir, exist_ok=True)
        vault_path = os.path.join(vault_dir, f"{os.path.splitext(os.path.basename(args.mesh))[0]}_texel_audit.json")
        with open(vault_path, 'w', encoding='utf-8') as vf:
            json.dump(report, vf, indent=2)
        report["vault_saved_path"] = vault_path

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print("\n" + "="*70)
        print(f" AAA STUDIO UV & TEXEL DENSITY REPORT: {os.path.basename(args.mesh)}")
        print("="*70)
        print(f" Status:              {report['status']}")
        print(f" Measured Average TD: {report['texel_density']['average_px_cm']} px/cm ({report['texel_density']['average_px_m']} px/m)")
        print(f" TD Variance:         {report['texel_density']['variance_percentage']}%")
        print(f" Adaptive VRAM Budget: {report['adaptive_vram_budget']['recommended_texture_size']} (~{report['adaptive_vram_budget']['estimated_vram_compressed_mb']} MB VRAM)")
        if report['qa_warnings']:
            for w in report['qa_warnings']: print(f"  • {w}")
        print("="*70 + "\n")

    if report["status"] == "FAIL":
        sys.exit(2)

if __name__ == "__main__":
    main()
