//! Port of facs_blendshape_mirror.py — ARKit/FACS 52 blendshape mirroring
//! across the sagittal (X=0) plane via spatially-hashed symmetry mapping.

use serde_json::json;
use std::collections::HashMap;
use std::fs;

use super::geometry::{
    compute_single_pass_bounding_box, grid_coord, hash_grid_3d, parse_obj_buffered,
};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;

/// Spatially-hashed mirror map across the sagittal plane — `vertex -> its
/// (-x, y, z) counterpart within tolerance`.
pub fn find_symmetry_vertex_map(
    neutral_verts: &[[f64; 3]],
    tolerance: f64,
) -> HashMap<usize, usize> {
    let (min_pt, _max_pt, dims) = compute_single_pass_bounding_box(neutral_verts);
    let max_dim = dims[0].max(dims[1]).max(dims[2]);
    let cell_size = (tolerance * 2.0).max(max_dim / 100.0);

    // Spatial hash keyed on mirrored (-x, y, z) positions of every vertex.
    let mut grid_map: HashMap<u64, Vec<usize>> = HashMap::new();
    for (vi, v) in neutral_verts.iter().enumerate() {
        let gx = grid_coord(-v[0], min_pt[0], cell_size);
        let gy = grid_coord(v[1], min_pt[1], cell_size);
        let gz = grid_coord(v[2], min_pt[2], cell_size);
        grid_map
            .entry(hash_grid_3d(gx, gy, gz))
            .or_default()
            .push(vi);
    }

    let mut sym_map = HashMap::new();
    for (vi, v) in neutral_verts.iter().enumerate() {
        let (mx, my, mz) = (-v[0], v[1], v[2]);
        let gx = grid_coord(mx, min_pt[0], cell_size);
        let gy = grid_coord(my, min_pt[1], cell_size);
        let gz = grid_coord(mz, min_pt[2], cell_size);

        let mut best_match = vi;
        let mut best_dist = f64::INFINITY;
        for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                for dz in -1i64..=1 {
                    let h = hash_grid_3d(gx + dx, gy + dy, gz + dz);
                    if let Some(cands) = grid_map.get(&h) {
                        for &cand_vi in cands {
                            let cv = neutral_verts[cand_vi];
                            let dist = ((cv[0] - mx).powi(2)
                                + (cv[1] - my).powi(2)
                                + (cv[2] - mz).powi(2))
                            .sqrt();
                            if dist < best_dist && dist <= tolerance {
                                best_dist = dist;
                                best_match = cand_vi;
                            }
                        }
                    }
                }
            }
        }
        sym_map.insert(vi, best_match);
    }
    sym_map
}

fn mirror_blendshape(
    neutral_path: &str,
    source_shape_path: &str,
    out_mirrored_path: &str,
    tolerance: f64,
) -> Result<serde_json::Value, String> {
    let neutral = parse_obj_buffered(neutral_path, 1_000_000)?;
    let source = parse_obj_buffered(source_shape_path, 1_000_000)?;

    if neutral.vertices.len() != source.vertices.len() {
        return Err(format!(
            "Topology mismatch: Neutral has {} verts, Source has {} verts.",
            neutral.vertices.len(),
            source.vertices.len()
        ));
    }

    let sym_map = find_symmetry_vertex_map(&neutral.vertices, tolerance);

    let mut deltas = Vec::with_capacity(neutral.vertices.len());
    for vi in 0..neutral.vertices.len() {
        let nv = neutral.vertices[vi];
        let sv = source.vertices[vi];
        deltas.push([sv[0] - nv[0], sv[1] - nv[1], sv[2] - nv[2]]);
    }

    let mut mirrored_verts = neutral.vertices.clone();
    for (&vi, &sym_vi) in &sym_map {
        let [dx, dy, dz] = deltas[vi];
        mirrored_verts[sym_vi][0] += -dx;
        mirrored_verts[sym_vi][1] += dy;
        mirrored_verts[sym_vi][2] += dz;
    }

    let mut out = String::new();
    out.push_str("# Autonomous 3D Studio - FACS Mirrored Blendshape\n");
    for v in &mirrored_verts {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for vt in &neutral.texcoords {
        out.push_str(&format!("vt {:.6} {:.6}\n", vt[0], vt[1]));
    }
    for vn in &neutral.normals {
        out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", vn[0], vn[1], vn[2]));
    }
    for (f_idx, face) in neutral.faces.iter().enumerate() {
        let f_uv: &[Option<usize>] = neutral
            .face_uvs
            .get(f_idx)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let f_norm: &[Option<usize>] = neutral
            .face_normals
            .get(f_idx)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let mut tokens = Vec::new();
        for (i, &vi) in face.iter().enumerate() {
            let vt_part = match f_uv.get(i).copied().flatten() {
                Some(vt) => (vt + 1).to_string(),
                None => String::new(),
            };
            let vn_part = match f_norm.get(i).copied().flatten() {
                Some(vn) => (vn + 1).to_string(),
                None => String::new(),
            };
            if !vn_part.is_empty() {
                tokens.push(format!("{}/{}/{}", vi + 1, vt_part, vn_part));
            } else if !vt_part.is_empty() {
                tokens.push(format!("{}/{}", vi + 1, vt_part));
            } else {
                tokens.push(format!("{}", vi + 1));
            }
        }
        out.push_str(&format!("f {}\n", tokens.join(" ")));
    }
    fs::write(out_mirrored_path, out).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "SUCCESS",
        "neutral_mesh": neutral_path,
        "source_shape": source_shape_path,
        "mirrored_shape": out_mirrored_path,
        "symmetry_mapped_vertices": sym_map.len()
    }))
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio facs-mirror",
        argv,
        &[
            ArgSpec::value("neutral", Some('n'), "neutral").req(),
            ArgSpec::value("source_shape", Some('s'), "source-shape").req(),
            ArgSpec::value("out", Some('o'), "out").def("./blendshape_mirrored.obj"),
            ArgSpec::float("tol", None, "tol").def("0.002"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let neutral = args.get("neutral").unwrap();
    let source = args.get("source_shape").unwrap();
    let out = args.get_or("out", "./blendshape_mirrored.obj");
    let tol = args.float("tol", 0.002);

    match mirror_blendshape(neutral, source, &out, tol) {
        Ok(result) => {
            if args.flag("json") {
                println!("{}", jsonfmt::dumps_indent(&result, 2));
            } else {
                println!("\n[FACS Blendshape Mirror] Generated bilateral expression:");
                println!(" -> Mirrored Shape: {}", out);
                println!(
                    " -> Symmetrical Vertex Pairs Mapped: {}\n",
                    crate::common::py::comma_int(
                        result["symmetry_mapped_vertices"].as_i64().unwrap_or(0)
                    )
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
