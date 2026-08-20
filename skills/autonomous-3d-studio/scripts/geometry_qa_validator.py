#!/usr/bin/env python3
"""
geometry_qa_validator.py - AAA Studio 3D Geometry Quality Assurance Inspector & Auto-Repair

Performs rigorous single-pass topological and geometric audits on 3D meshes (OBJ, FBX, glTF, PLY).
Features:
  - Single-pass linear O(V + E + F) topological analysis
  - Programmatic `--fix` auto-remediation (degenerate dissolution & vertex pruning)
  - CIEL Artifact Vault routing (`--vault` -> ~/.ciel/artifacts/audits/)
  - Compact JSON telemetry (`--compact`) for token-efficient LLM context
"""

import sys
import os
import math
import json
import argparse
from datetime import datetime
from collections import defaultdict

try:
    from geometry_core import parse_obj_buffered, compute_single_pass_bounding_box, compute_polygon_area_3d
except ImportError:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, script_dir)
    from geometry_core import parse_obj_buffered, compute_single_pass_bounding_box, compute_polygon_area_3d

def audit_geometry(mesh_data, profile="aaa_game", is_watertight_required=True):
    verts = mesh_data.vertices
    faces = mesh_data.faces
    num_verts = len(verts)
    num_faces = len(faces)

    if num_verts == 0 or num_faces == 0:
        return {"status": "FAIL", "error": "Empty geometry: 0 vertices or 0 faces.", "metrics": {}}

    # 1. Single-Pass Bounding Box
    min_pt, max_pt, dims = compute_single_pass_bounding_box(verts)

    # 2. Single-Pass Polygon Classification & Topology Adjacency
    triangles_count = 0
    quads_count = 0
    ngons_count = 0
    degenerate_faces = []
    total_surface_area = 0.0

    edge_to_faces = defaultdict(list)
    vertex_to_faces = defaultdict(list)
    vertex_valence = defaultdict(int)

    for f_idx, face in enumerate(faces):
        k = len(face)
        if k == 3: triangles_count += 1
        elif k == 4: quads_count += 1
        elif k > 4: ngons_count += 1

        poly_pts = [verts[vi] for vi in face if vi < num_verts]
        f_area = compute_polygon_area_3d(poly_pts)
        total_surface_area += f_area
        if f_area <= 1e-7:
            degenerate_faces.append(f_idx)

        m = len(face)
        for i in range(m):
            v_curr = face[i]
            v_next = face[(i + 1) % m]
            edge_key = (min(v_curr, v_next), max(v_curr, v_next))
            edge_to_faces[edge_key].append(f_idx)
            vertex_to_faces[v_curr].append(f_idx)

    # 3. Manifold & Boundary Audit
    non_manifold_edges = []
    boundary_edges = []
    for edge, f_list in edge_to_faces.items():
        if len(f_list) > 2:
            non_manifold_edges.append({"edge": list(edge), "count": len(f_list)})
        elif len(f_list) == 1:
            boundary_edges.append(list(edge))
        vertex_valence[edge[0]] += 1
        vertex_valence[edge[1]] += 1

    # 4. Valence & Poles
    val_3, val_4, val_5, star_poles = 0, 0, 0, 0
    for v_idx in range(num_verts):
        val = vertex_valence[v_idx]
        if val == 3: val_3 += 1
        elif val == 4: val_4 += 1
        elif val == 5: val_5 += 1
        elif val >= 6: star_poles += 1

    loose_verts = [vi for vi in range(num_verts) if vi not in vertex_to_faces]

    # 5. AAA Quality Hard Gates
    qa_flags = []
    passed = True

    if len(non_manifold_edges) > 0:
        passed = False
        qa_flags.append(f"CRITICAL: {len(non_manifold_edges)} non-manifold edges (>2 faces).")

    if len(degenerate_faces) > 0:
        passed = False
        qa_flags.append(f"ERROR: {len(degenerate_faces)} degenerate (zero-area) faces.")

    if len(loose_verts) > 0:
        passed = False
        qa_flags.append(f"ERROR: {len(loose_verts)} loose unreferenced vertices.")

    if profile in ["aaa_game", "character", "deformable"] and star_poles > 0:
        passed = False
        qa_flags.append(f"CRITICAL: {star_poles} star poles (valence >= 6) in deformable mesh.")

    if profile in ["character", "deformable"] and ngons_count > 0:
        passed = False
        qa_flags.append(f"CRITICAL: {ngons_count} n-gons in deformable character mesh.")

    if is_watertight_required and len(boundary_edges) > 0:
        qa_flags.append(f"WARNING: Mesh has {len(boundary_edges)} open boundary edges.")

    quad_pct = (quads_count / num_faces * 100.0) if num_faces > 0 else 0.0
    tri_pct = (triangles_count / num_faces * 100.0) if num_faces > 0 else 0.0
    ngon_pct = (ngons_count / num_faces * 100.0) if num_faces > 0 else 0.0

    return {
        "status": "PASS" if passed else "FAIL",
        "profile": profile,
        "summary": {
            "total_vertices": num_verts,
            "total_edges": len(edge_to_faces),
            "total_faces": num_faces,
            "surface_area_m2": round(total_surface_area, 6),
            "bounding_box_meters": {
                "x": round(dims[0], 4),
                "y": round(dims[1], 4),
                "z": round(dims[2], 4)
            }
        },
        "topology_breakdown": {
            "triangles": triangles_count,
            "triangle_percentage": round(tri_pct, 2),
            "quads": quads_count,
            "quad_percentage": round(quad_pct, 2),
            "ngons": ngons_count,
            "ngon_percentage": round(ngon_pct, 2)
        },
        "valence_histogram": {
            "valence_3_n_poles": val_3,
            "valence_4_regular": val_4,
            "valence_5_e_poles": val_5,
            "valence_6_plus_star_poles": star_poles
        },
        "defect_counts": {
            "non_manifold_edges": len(non_manifold_edges),
            "boundary_edges": len(boundary_edges),
            "loose_vertices": len(loose_verts),
            "degenerate_faces": len(degenerate_faces)
        },
        "qa_flags": qa_flags
    }

def repair_mesh_data(mesh, out_repaired_path):
    valid_faces = []
    valid_face_uvs = []
    valid_face_normals = []
    referenced_verts = set()

    for idx, f in enumerate(mesh.faces):
        poly_pts = [mesh.vertices[vi] for vi in f if vi < len(mesh.vertices)]
        if compute_polygon_area_3d(poly_pts) > 1e-7 and len(set(f)) >= 3:
            valid_faces.append(f)
            if idx < len(mesh.face_uvs): valid_face_uvs.append(mesh.face_uvs[idx])
            if idx < len(mesh.face_normals): valid_face_normals.append(mesh.face_normals[idx])
            for vi in f: referenced_verts.add(vi)

    old_to_new = {}
    new_verts = []
    for old_idx, v in enumerate(mesh.vertices):
        if old_idx in referenced_verts:
            old_to_new[old_idx] = len(new_verts)
            new_verts.append(v)

    with open(out_repaired_path, 'w', encoding='utf-8') as f:
        f.write("# Autonomous 3D Studio - Auto-Repaired Mesh\n")
        f.write(f"# Repaired: {datetime.utcnow().isoformat()}Z\n")
        for v in new_verts:
            f.write(f"v {v[0]:.6f} {v[1]:.6f} {v[2]:.6f}\n")
        for vt in mesh.texcoords:
            f.write(f"vt {vt[0]:.6f} {vt[1]:.6f}\n")
        for vn in mesh.normals:
            f.write(f"vn {vn[0]:.6f} {vn[1]:.6f} {vn[2]:.6f}\n")

        for f_idx, f_elem in enumerate(valid_faces):
            tokens = []
            f_uv = valid_face_uvs[f_idx] if f_idx < len(valid_face_uvs) else []
            f_norm = valid_face_normals[f_idx] if f_idx < len(valid_face_normals) else []
            for i, old_vi in enumerate(f_elem):
                new_vi = old_to_new[old_vi] + 1
                vt_part = str(f_uv[i] + 1) if i < len(f_uv) and f_uv[i] is not None else ""
                vn_part = str(f_norm[i] + 1) if i < len(f_norm) and f_norm[i] is not None else ""
                if vn_part: tokens.append(f"{new_vi}/{vt_part}/{vn_part}")
                elif vt_part: tokens.append(f"{new_vi}/{vt_part}")
                else: tokens.append(f"{new_vi}")
            f.write(f"f {' '.join(tokens)}\n")

    return {
        "repaired_path": out_repaired_path,
        "removed_degenerate_faces": len(mesh.faces) - len(valid_faces),
        "pruned_loose_vertices": len(mesh.vertices) - len(new_verts)
    }

def record_instinct_observation(mesh_path, report):
    try:
        instinct_dir = os.path.expanduser("~/.ciel/instincts")
        os.makedirs(instinct_dir, exist_ok=True)
        with open(os.path.join(instinct_dir, "3d_studio_observations.jsonl"), 'a', encoding='utf-8') as f:
            f.write(json.dumps({
                "ts": datetime.utcnow().isoformat() + "Z",
                "asset": os.path.basename(mesh_path),
                "status": report["status"],
                "profile": report["profile"],
                "defects": report["defect_counts"],
                "flags": report["qa_flags"]
            }) + "\n")
    except Exception:
        pass

def route_to_ciel_vault(report, asset_name):
    """Saves audit report to CIEL Artifact Vault (~/.ciel/artifacts/audits/)."""
    try:
        vault_dir = os.path.expanduser("~/.ciel/artifacts/audits")
        os.makedirs(vault_dir, exist_ok=True)
        vault_file = os.path.join(vault_dir, f"{asset_name}_qa_audit.json")
        with open(vault_file, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2)
        return vault_file
    except Exception:
        return None

def main():
    parser = argparse.ArgumentParser(description="AAA Studio 3D Geometry QA Validator & Auto-Repair")
    parser.add_argument("--input", "-i", required=True, help="Path to input 3D mesh (.obj)")
    parser.add_argument("--profile", "-p", default="aaa_game", choices=["aaa_game", "character", "hard_surface", "nanite", "vfx_film"])
    parser.add_argument("--watertight", action="store_true", default=False)
    parser.add_argument("--fix", action="store_true", default=False)
    parser.add_argument("--out", "-o", help="Output path for repaired mesh")
    parser.add_argument("--vault", action="store_true", help="Route audit report to CIEL Artifact Vault (~/.ciel/artifacts/audits/)")
    parser.add_argument("--compact", action="store_true", help="Output token-efficient compact JSON")
    parser.add_argument("--json", action="store_true", help="Output JSON telemetry")

    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f"Error: Input file '{args.input}' not found.", file=sys.stderr)
        sys.exit(1)

    mesh = parse_obj_buffered(args.input)
    report = audit_geometry(mesh, profile=args.profile, is_watertight_required=args.watertight)
    record_instinct_observation(args.input, report)

    if args.vault:
        vault_path = route_to_ciel_vault(report, os.path.splitext(os.path.basename(args.input))[0])
        report["vault_saved_path"] = vault_path

    if args.fix:
        out_fix = args.out if args.out else os.path.splitext(args.input)[0] + "_repaired.obj"
        report["fix_result"] = repair_mesh_data(mesh, out_fix)

    if args.compact:
        compact_summary = {
            "status": report["status"],
            "profile": report["profile"],
            "v": report["summary"]["total_vertices"],
            "f": report["summary"]["total_faces"],
            "quads_pct": report["topology_breakdown"]["quad_percentage"],
            "star_poles": report["valence_histogram"]["valence_6_plus_star_poles"],
            "defects": report["defect_counts"],
            "flags": report["qa_flags"]
        }
        print(json.dumps(compact_summary, separators=(',', ':')))
    elif args.json:
        print(json.dumps(report, indent=2))
    else:
        print("\n" + "="*70)
        print(f" AAA STUDIO GEOMETRY QA REPORT: {os.path.basename(args.input)}")
        print("="*70)
        print(f" Status:       {report['status']}")
        print(f" Profile:      {report['profile']}")
        print(f" Vertices:     {report['summary']['total_vertices']:,}")
        print(f" Faces:        {report['summary']['total_faces']:,} (Quads: {report['topology_breakdown']['quad_percentage']}%)")
        print(f" Star Poles:   {report['valence_histogram']['valence_6_plus_star_poles']}")
        print(f" Defects:      Non-Manifold: {report['defect_counts']['non_manifold_edges']} | Degenerate: {report['defect_counts']['degenerate_faces']}")
        if report['qa_flags']:
            for flag in report['qa_flags']: print(f"  • {flag}")
        print("="*70 + "\n")

    if report["status"] == "FAIL" and not args.fix:
        sys.exit(2)

if __name__ == "__main__":
    main()
