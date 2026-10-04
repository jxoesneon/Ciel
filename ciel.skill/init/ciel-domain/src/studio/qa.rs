//! Port of geometry_qa_validator.py — single-pass topological QA audit,
//! programmatic --fix repair, CIEL artifact vault routing, compact JSON.

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::geometry::{
    compute_polygon_area_3d, compute_single_pass_bounding_box, parse_obj_buffered, MeshData,
};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

pub fn audit_geometry(mesh: &MeshData, profile: &str, is_watertight_required: bool) -> Value {
    let verts = &mesh.vertices;
    let faces = &mesh.faces;
    let num_verts = verts.len();
    let num_faces = faces.len();

    if num_verts == 0 || num_faces == 0 {
        return json!({"status": "FAIL", "error": "Empty geometry: 0 vertices or 0 faces.", "metrics": {}});
    }

    let (_min_pt, _max_pt, dims) = compute_single_pass_bounding_box(verts);

    let mut triangles_count = 0usize;
    let mut quads_count = 0usize;
    let mut ngons_count = 0usize;
    let mut degenerate_faces: Vec<usize> = Vec::new();
    let mut total_surface_area = 0.0f64;

    let mut edge_to_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    let mut vertex_to_faces: HashSet<usize> = HashSet::new();
    let mut vertex_valence: HashMap<usize, usize> = HashMap::new();

    for (f_idx, face) in faces.iter().enumerate() {
        let k = face.len();
        if k == 3 {
            triangles_count += 1;
        } else if k == 4 {
            quads_count += 1;
        } else if k > 4 {
            ngons_count += 1;
        }
        let poly_pts: Vec<[f64; 3]> = face
            .iter()
            .filter(|&&vi| vi < num_verts)
            .map(|&vi| verts[vi])
            .collect();
        let f_area = compute_polygon_area_3d(&poly_pts);
        total_surface_area += f_area;
        if f_area <= 1e-7 {
            degenerate_faces.push(f_idx);
        }
        let m = face.len();
        for i in 0..m {
            let v_curr = face[i];
            let v_next = face[(i + 1) % m];
            let edge_key = if v_curr < v_next {
                (v_curr, v_next)
            } else {
                (v_next, v_curr)
            };
            edge_to_faces.entry(edge_key).or_default().push(f_idx);
            vertex_to_faces.insert(v_curr);
        }
    }

    let mut non_manifold_edges: Vec<Value> = Vec::new();
    let mut boundary_edges: Vec<Value> = Vec::new();
    for (edge, f_list) in &edge_to_faces {
        if f_list.len() > 2 {
            non_manifold_edges.push(json!({"edge": [edge.0, edge.1], "count": f_list.len()}));
        } else if f_list.len() == 1 {
            boundary_edges.push(json!([edge.0, edge.1]));
        }
        *vertex_valence.entry(edge.0).or_insert(0) += 1;
        *vertex_valence.entry(edge.1).or_insert(0) += 1;
    }

    let (mut val_3, mut val_4, mut val_5, mut star_poles) = (0usize, 0usize, 0usize, 0usize);
    for v_idx in 0..num_verts {
        let val = vertex_valence.get(&v_idx).copied().unwrap_or(0);
        match val {
            3 => val_3 += 1,
            4 => val_4 += 1,
            5 => val_5 += 1,
            v if v >= 6 => star_poles += 1,
            _ => {}
        }
    }
    let loose_verts: Vec<usize> = (0..num_verts)
        .filter(|vi| !vertex_to_faces.contains(vi))
        .collect();

    let mut qa_flags: Vec<String> = Vec::new();
    let mut passed = true;

    if !non_manifold_edges.is_empty() {
        passed = false;
        qa_flags.push(format!(
            "CRITICAL: {} non-manifold edges (>2 faces).",
            non_manifold_edges.len()
        ));
    }
    if !degenerate_faces.is_empty() {
        passed = false;
        qa_flags.push(format!(
            "ERROR: {} degenerate (zero-area) faces.",
            degenerate_faces.len()
        ));
    }
    if !loose_verts.is_empty() {
        passed = false;
        qa_flags.push(format!(
            "ERROR: {} loose unreferenced vertices.",
            loose_verts.len()
        ));
    }
    if ["aaa_game", "character", "deformable"].contains(&profile) && star_poles > 0 {
        passed = false;
        qa_flags.push(format!(
            "CRITICAL: {} star poles (valence >= 6) in deformable mesh.",
            star_poles
        ));
    }
    if ["character", "deformable"].contains(&profile) && ngons_count > 0 {
        passed = false;
        qa_flags.push(format!(
            "CRITICAL: {} n-gons in deformable character mesh.",
            ngons_count
        ));
    }
    if is_watertight_required && !boundary_edges.is_empty() {
        qa_flags.push(format!(
            "WARNING: Mesh has {} open boundary edges.",
            boundary_edges.len()
        ));
    }

    let pct = |n: usize| -> f64 {
        if num_faces > 0 {
            n as f64 / num_faces as f64 * 100.0
        } else {
            0.0
        }
    };

    json!({
        "status": if passed { "PASS" } else { "FAIL" },
        "profile": profile,
        "summary": {
            "total_vertices": num_verts,
            "total_edges": edge_to_faces.len(),
            "total_faces": num_faces,
            "surface_area_m2": py::round_py(total_surface_area, 6),
            "bounding_box_meters": {
                "x": py::round_py(dims[0], 4),
                "y": py::round_py(dims[1], 4),
                "z": py::round_py(dims[2], 4)
            }
        },
        "topology_breakdown": {
            "triangles": triangles_count,
            "triangle_percentage": py::round_py(pct(triangles_count), 2),
            "quads": quads_count,
            "quad_percentage": py::round_py(pct(quads_count), 2),
            "ngons": ngons_count,
            "ngon_percentage": py::round_py(pct(ngons_count), 2)
        },
        "valence_histogram": {
            "valence_3_n_poles": val_3,
            "valence_4_regular": val_4,
            "valence_5_e_poles": val_5,
            "valence_6_plus_star_poles": star_poles
        },
        "defect_counts": {
            "non_manifold_edges": non_manifold_edges.len(),
            "boundary_edges": boundary_edges.len(),
            "loose_vertices": loose_verts.len(),
            "degenerate_faces": degenerate_faces.len()
        },
        "qa_flags": qa_flags
    })
}

pub fn repair_mesh_data(mesh: &MeshData, out_repaired_path: &str) -> Result<Value, String> {
    let mut valid_faces: Vec<&Vec<usize>> = Vec::new();
    let mut valid_face_uvs: Vec<&Vec<Option<usize>>> = Vec::new();
    let mut valid_face_normals: Vec<&Vec<Option<usize>>> = Vec::new();
    let mut referenced: HashSet<usize> = HashSet::new();

    for (idx, f) in mesh.faces.iter().enumerate() {
        let poly_pts: Vec<[f64; 3]> = f
            .iter()
            .filter(|&&vi| vi < mesh.vertices.len())
            .map(|&vi| mesh.vertices[vi])
            .collect();
        let uniq: HashSet<usize> = f.iter().copied().collect();
        if compute_polygon_area_3d(&poly_pts) > 1e-7 && uniq.len() >= 3 {
            valid_faces.push(f);
            if idx < mesh.face_uvs.len() {
                valid_face_uvs.push(&mesh.face_uvs[idx]);
            }
            if idx < mesh.face_normals.len() {
                valid_face_normals.push(&mesh.face_normals[idx]);
            }
            for &vi in f {
                referenced.insert(vi);
            }
        }
    }

    let mut old_to_new: HashMap<usize, usize> = HashMap::new();
    let mut new_verts: Vec<[f64; 3]> = Vec::new();
    for (old_idx, v) in mesh.vertices.iter().enumerate() {
        if referenced.contains(&old_idx) {
            old_to_new.insert(old_idx, new_verts.len());
            new_verts.push(*v);
        }
    }

    let mut out = String::new();
    out.push_str("# Autonomous 3D Studio - Auto-Repaired Mesh\n");
    out.push_str(&format!("# Repaired: {}\n", py::utcnow_iso_z()));
    for v in &new_verts {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for vt in &mesh.texcoords {
        out.push_str(&format!("vt {:.6} {:.6}\n", vt[0], vt[1]));
    }
    for vn in &mesh.normals {
        out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", vn[0], vn[1], vn[2]));
    }
    for (f_idx, f_elem) in valid_faces.iter().enumerate() {
        let f_uv: &[Option<usize>] = valid_face_uvs
            .get(f_idx)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let f_norm: &[Option<usize>] = valid_face_normals
            .get(f_idx)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let mut tokens = Vec::new();
        for (i, &old_vi) in f_elem.iter().enumerate() {
            let new_vi = old_to_new[&old_vi] + 1;
            let vt_part = match f_uv.get(i).copied().flatten() {
                Some(vt) => (vt + 1).to_string(),
                None => String::new(),
            };
            let vn_part = match f_norm.get(i).copied().flatten() {
                Some(vn) => (vn + 1).to_string(),
                None => String::new(),
            };
            if !vn_part.is_empty() {
                tokens.push(format!("{}/{}/{}", new_vi, vt_part, vn_part));
            } else if !vt_part.is_empty() {
                tokens.push(format!("{}/{}", new_vi, vt_part));
            } else {
                tokens.push(format!("{}", new_vi));
            }
        }
        out.push_str(&format!("f {}\n", tokens.join(" ")));
    }
    fs::write(out_repaired_path, out).map_err(|e| e.to_string())?;

    Ok(json!({
        "repaired_path": out_repaired_path,
        "removed_degenerate_faces": mesh.faces.len() - valid_faces.len(),
        "pruned_loose_vertices": mesh.vertices.len() - new_verts.len()
    }))
}

pub fn record_instinct_observation(mesh_path: &str, report: &Value) {
    let dir = py::expanduser("~/.ciel/instincts");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let entry = json!({
        "ts": py::utcnow_iso_z(),
        "asset": Path::new(mesh_path).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
        "status": report.get("status").cloned().unwrap_or(Value::Null),
        "profile": report.get("profile").cloned().unwrap_or(Value::Null),
        "defects": report.get("defect_counts").cloned().unwrap_or(Value::Null),
        "flags": report.get("qa_flags").cloned().unwrap_or(Value::Null)
    });
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("3d_studio_observations.jsonl"))
    {
        let _ = writeln!(f, "{}", jsonfmt::dumps(&entry));
    }
}

pub fn route_to_ciel_vault(report: &Value, asset_name: &str) -> Option<String> {
    let vault_dir = py::expanduser("~/.ciel/artifacts/audits");
    if fs::create_dir_all(&vault_dir).is_err() {
        return None;
    }
    let vault_file = vault_dir.join(format!("{}_qa_audit.json", asset_name));
    fs::write(&vault_file, jsonfmt::dumps_indent(report, 2))
        .ok()
        .map(|_| vault_file.to_string_lossy().into_owned())
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio geometry-qa",
        argv,
        &[
            ArgSpec::value("input", Some('i'), "input").req(),
            ArgSpec::value("profile", Some('p'), "profile")
                .def("aaa_game")
                .choices(&[
                    "aaa_game",
                    "character",
                    "hard_surface",
                    "nanite",
                    "vfx_film",
                ]),
            ArgSpec::flag("watertight", None, "watertight"),
            ArgSpec::flag("fix", None, "fix"),
            ArgSpec::value("out", Some('o'), "out"),
            ArgSpec::flag("vault", None, "vault"),
            ArgSpec::flag("compact", None, "compact"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let input = args.get("input").unwrap();
    if !Path::new(input).exists() {
        eprintln!("Error: Input file '{}' not found.", input);
        return 1;
    }
    let mesh = match parse_obj_buffered(input, 1_000_000) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let profile = args.get_or("profile", "aaa_game");
    let mut report = audit_geometry(&mesh, &profile, args.flag("watertight"));
    record_instinct_observation(input, &report);

    if args.flag("vault") {
        let stem = Path::new(input)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let vault_path = route_to_ciel_vault(&report, &stem);
        report["vault_saved_path"] = vault_path.map(Value::String).unwrap_or(Value::Null);
    }

    if args.flag("fix") {
        let out_fix = match args.get("out") {
            Some(o) => o.to_string(),
            None => format!(
                "{}_repaired.obj",
                Path::new(input).with_extension("").to_string_lossy()
            ),
        };
        match repair_mesh_data(&mesh, &out_fix) {
            Ok(r) => report["fix_result"] = r,
            Err(e) => {
                eprintln!("Error: {}", e);
                return 1;
            }
        }
    }

    if args.flag("compact") {
        let compact = json!({
            "status": report["status"],
            "profile": report["profile"],
            "v": report["summary"]["total_vertices"],
            "f": report["summary"]["total_faces"],
            "quads_pct": report["topology_breakdown"]["quad_percentage"],
            "star_poles": report["valence_histogram"]["valence_6_plus_star_poles"],
            "defects": report["defect_counts"],
            "flags": report["qa_flags"]
        });
        println!("{}", jsonfmt::dumps_compact(&compact));
    } else if args.flag("json") {
        println!("{}", jsonfmt::dumps_indent(&report, 2));
    } else {
        let base = Path::new(input)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        println!("\n{:=<70}", "");
        println!(" AAA STUDIO GEOMETRY QA REPORT: {}", base);
        println!("{:=<70}", "");
        println!(" Status:       {}", report["status"].as_str().unwrap_or(""));
        println!(
            " Profile:      {}",
            report["profile"].as_str().unwrap_or("")
        );
        println!(
            " Vertices:     {}",
            py::comma_int(report["summary"]["total_vertices"].as_i64().unwrap_or(0))
        );
        println!(
            " Faces:        {} (Quads: {}%)",
            py::comma_int(report["summary"]["total_faces"].as_i64().unwrap_or(0)),
            report["topology_breakdown"]["quad_percentage"]
        );
        println!(
            " Star Poles:   {}",
            report["valence_histogram"]["valence_6_plus_star_poles"]
        );
        println!(
            " Defects:      Non-Manifold: {} | Degenerate: {}",
            report["defect_counts"]["non_manifold_edges"],
            report["defect_counts"]["degenerate_faces"]
        );
        if let Some(flags) = report["qa_flags"].as_array() {
            for flag in flags {
                println!("  • {}", flag.as_str().unwrap_or(""));
            }
        }
        println!("{:=<70}\n", "");
    }

    if report["status"].as_str() == Some("FAIL") && !args.flag("fix") {
        return 2;
    }
    0
}
