//! Port of vertex_normal_transfer.py — face-area-weighted smooth normal
//! projection from a dense CAD/SubD source onto a game-res target mesh via
//! 3x3x3 spatial-hash nearest-vertex lookup.

use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::geometry::{
    compute_single_pass_bounding_box, grid_coord, hash_grid_3d, parse_obj_buffered, Vec3,
};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

fn find_nearest_normal_spatial(
    target_vert: Vec3,
    source_verts: &[Vec3],
    source_normals: &[Vec3],
    grid_map: &HashMap<u64, Vec<usize>>,
    cell_size: f64,
    min_pt: Vec3,
) -> Vec3 {
    let gx = grid_coord(target_vert[0], min_pt[0], cell_size);
    let gy = grid_coord(target_vert[1], min_pt[1], cell_size);
    let gz = grid_coord(target_vert[2], min_pt[2], cell_size);

    let mut best_dist_sq = f64::INFINITY;
    let mut best_norm = [0.0, 0.0, 1.0];

    for dx in -1i64..=1 {
        for dy in -1i64..=1 {
            for dz in -1i64..=1 {
                let h = hash_grid_3d(gx + dx, gy + dy, gz + dz);
                if let Some(indices) = grid_map.get(&h) {
                    for &src_idx in indices {
                        let sv = source_verts[src_idx];
                        let dist_sq = (sv[0] - target_vert[0]).powi(2)
                            + (sv[1] - target_vert[1]).powi(2)
                            + (sv[2] - target_vert[2]).powi(2);
                        if dist_sq < best_dist_sq {
                            best_dist_sq = dist_sq;
                            if src_idx < source_normals.len() {
                                best_norm = source_normals[src_idx];
                            }
                        }
                    }
                }
            }
        }
    }
    best_norm
}

pub fn transfer_vertex_normals(
    source_mesh_path: &str,
    target_mesh_path: &str,
    out_mesh_path: &str,
) -> Result<serde_json::Value, String> {
    let src = parse_obj_buffered(source_mesh_path, 1_000_000)?;
    let tgt = parse_obj_buffered(target_mesh_path, 1_000_000)?;

    let (min_pt, _max_pt, dims) = compute_single_pass_bounding_box(&src.vertices);
    let max_dim = dims[0].max(dims[1]).max(dims[2]);
    let grid_res = ((src.vertices.len() as f64).powf(1.0 / 3.0) as usize).max(10);
    let cell_size = if grid_res > 0 {
        max_dim / grid_res as f64
    } else {
        0.1
    };

    let mut grid_map: HashMap<u64, Vec<usize>> = HashMap::new();
    for (vi, v) in src.vertices.iter().enumerate() {
        let h = hash_grid_3d(
            grid_coord(v[0], min_pt[0], cell_size),
            grid_coord(v[1], min_pt[1], cell_size),
            grid_coord(v[2], min_pt[2], cell_size),
        );
        grid_map.entry(h).or_default().push(vi);
    }

    let mut transferred_normals = Vec::with_capacity(tgt.vertices.len());
    for tv in &tgt.vertices {
        transferred_normals.push(find_nearest_normal_spatial(
            *tv,
            &src.vertices,
            &src.normals,
            &grid_map,
            cell_size,
            min_pt,
        ));
    }

    let mut out = String::new();
    out.push_str("# Autonomous 3D Studio - CAD Vertex Normal Transfer Mesh\n");
    for v in &tgt.vertices {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for vt in &tgt.texcoords {
        out.push_str(&format!("vt {:.6} {:.6}\n", vt[0], vt[1]));
    }
    for vn in &transferred_normals {
        out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", vn[0], vn[1], vn[2]));
    }
    for (f_idx, face) in tgt.faces.iter().enumerate() {
        let f_uv: &[Option<usize>] = tgt.face_uvs.get(f_idx).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut tokens = Vec::new();
        for (i, &vi) in face.iter().enumerate() {
            let vt_part = match f_uv.get(i).copied().flatten() {
                Some(vt) => (vt + 1).to_string(),
                None => String::new(),
            };
            let vn_part = (vi + 1).to_string(); // Mapped to vertex normal
            tokens.push(format!("{}/{}/{}", vi + 1, vt_part, vn_part));
        }
        out.push_str(&format!("f {}\n", tokens.join(" ")));
    }
    fs::write(out_mesh_path, out).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "SUCCESS",
        "source_mesh": source_mesh_path,
        "target_mesh": target_mesh_path,
        "output_mesh": out_mesh_path,
        "transferred_normals_count": transferred_normals.len()
    }))
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio vnt",
        argv,
        &[
            ArgSpec::value("source", Some('s'), "source").req(),
            ArgSpec::value("target", Some('t'), "target").req(),
            ArgSpec::value("out", Some('o'), "out").def("./target_transferred_normals.obj"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let source = args.get("source").unwrap();
    let target = args.get("target").unwrap();
    let out = args.get_or("out", "./target_transferred_normals.obj");

    if !Path::new(source).exists() || !Path::new(target).exists() {
        eprintln!("Error: Source or Target mesh file not found.");
        return 1;
    }

    match transfer_vertex_normals(source, target, &out) {
        Ok(result) => {
            if args.flag("json") {
                println!("{}", jsonfmt::dumps_indent(&result, 2));
            } else {
                println!("\n[Vertex Normal Transfer] Successfully projected smooth CAD normals:");
                println!(" -> Output Mesh: {}", out);
                println!(
                    " -> Normals Transferred: {}\n",
                    py::comma_int(result["transferred_normals_count"].as_i64().unwrap_or(0))
                );
            }
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
