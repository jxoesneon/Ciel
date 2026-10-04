//! Port of retopology_quadriflow.py — spatial-hashed vertex-clustering
//! decimation and cascading 5-tier LOD hierarchy generator.

use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::geometry::{
    compute_single_pass_bounding_box, grid_coord, hash_grid_3d, parse_obj_buffered,
};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

fn edge_collapse_decimate_hashed(
    verts: &[[f64; 3]],
    faces: &[Vec<usize>],
    target_ratio: f64,
) -> (Vec<[f64; 3]>, Vec<Vec<usize>>) {
    let target_faces = (faces.len() as f64 * target_ratio) as usize;
    let target_faces = target_faces.max(6);
    if faces.len() <= target_faces || verts.is_empty() {
        return (verts.to_vec(), faces.to_vec());
    }

    let (min_pt, _max_pt, dims) = compute_single_pass_bounding_box(verts);
    let max_dim = dims[0].max(dims[1]).max(dims[2]);

    let vert_target = ((verts.len() as f64 * target_ratio) as usize).max(4);
    let grid_res = ((vert_target as f64).powf(1.0 / 3.0) * 1.5) as usize;
    let grid_res = grid_res.max(3);
    let cell_size = if grid_res > 0 {
        max_dim / grid_res as f64
    } else {
        0.1
    };

    let mut grid_map: HashMap<u64, Vec<usize>> = HashMap::new();
    for (vi, v) in verts.iter().enumerate() {
        let gx = grid_coord(v[0], min_pt[0], cell_size);
        let gy = grid_coord(v[1], min_pt[1], cell_size);
        let gz = grid_coord(v[2], min_pt[2], cell_size);
        grid_map
            .entry(hash_grid_3d(gx, gy, gz))
            .or_default()
            .push(vi);
    }

    // NOTE: Python iterates a defaultdict — insertion order = first-vertex-seen order.
    // We replicate by collecting keys in vertex-scan order.
    let mut ordered_keys: Vec<u64> = Vec::new();
    let mut seen_keys: HashSet<u64> = HashSet::new();
    for v in verts {
        let h = hash_grid_3d(
            grid_coord(v[0], min_pt[0], cell_size),
            grid_coord(v[1], min_pt[1], cell_size),
            grid_coord(v[2], min_pt[2], cell_size),
        );
        if seen_keys.insert(h) {
            ordered_keys.push(h);
        }
    }

    let mut new_verts: Vec<[f64; 3]> = Vec::new();
    let mut old_to_new: HashMap<usize, usize> = HashMap::new();
    for h_key in &ordered_keys {
        let v_indices = &grid_map[h_key];
        let n = v_indices.len() as f64;
        let avg_x = v_indices.iter().map(|&vi| verts[vi][0]).sum::<f64>() / n;
        let avg_y = v_indices.iter().map(|&vi| verts[vi][1]).sum::<f64>() / n;
        let avg_z = v_indices.iter().map(|&vi| verts[vi][2]).sum::<f64>() / n;
        let new_vi = new_verts.len();
        new_verts.push([avg_x, avg_y, avg_z]);
        for &old_vi in v_indices {
            old_to_new.insert(old_vi, new_vi);
        }
    }

    let mut new_faces: Vec<Vec<usize>> = Vec::new();
    for face_item in faces {
        let mut remapped_f: Vec<usize> = Vec::new();
        for &vi in face_item {
            let n_vi = old_to_new.get(&vi).copied().unwrap_or(vi);
            if remapped_f.last().copied() != Some(n_vi) {
                remapped_f.push(n_vi);
            }
        }
        if remapped_f.len() > 1 && remapped_f.first() == remapped_f.last() {
            remapped_f.pop();
        }
        let uniq: HashSet<usize> = remapped_f.iter().copied().collect();
        if uniq.len() >= 3 {
            new_faces.push(remapped_f);
        }
    }
    (new_verts, new_faces)
}

fn export_obj_simple(
    verts: &[[f64; 3]],
    faces: &[Vec<usize>],
    out_path: &str,
    lod_name: &str,
) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(&format!(
        "# Autonomous 3D Studio - Auto-Generated {}\n",
        lod_name
    ));
    out.push_str(&format!("# Generated: {}\n", py::utcnow_iso_z()));
    for v in verts {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for face_item in faces {
        let f_str = face_item
            .iter()
            .map(|vi| (vi + 1).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!("f {}\n", f_str));
    }
    fs::write(out_path, out).map_err(|e| e.to_string())?;
    Ok(out_path.to_string())
}

fn generate_lod_hierarchy_cascading(
    mesh_path: &str,
    out_dir: &str,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let base_name = Path::new(mesh_path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mesh = parse_obj_buffered(mesh_path, 500_000)?;

    let lod_configs: [(&str, f64, f64); 5] = [
        ("LOD0", 1.00, 1.00),
        ("LOD1", 0.50, 0.50),
        ("LOD2", 0.50, 0.25),
        ("LOD3", 0.50, 0.10),
        ("LOD4", 0.50, 0.03),
    ];

    let mut lods = Vec::new();
    let mut curr_verts = mesh.vertices.clone();
    let mut curr_faces = mesh.faces.clone();

    let mut cum_ratio = 1.00f64;
    for (lod_name, step_ratio, screen_size) in lod_configs {
        let out_lod_path = Path::new(out_dir)
            .join(format!("{}_{}.obj", base_name, lod_name))
            .to_string_lossy()
            .into_owned();
        if lod_name == "LOD0" {
            export_obj_simple(&curr_verts, &curr_faces, &out_lod_path, lod_name)?;
        } else {
            cum_ratio *= step_ratio;
            let (nv, nf) = edge_collapse_decimate_hashed(&curr_verts, &curr_faces, step_ratio);
            curr_verts = nv;
            curr_faces = nf;
            export_obj_simple(&curr_verts, &curr_faces, &out_lod_path, lod_name)?;
        }
        lods.push(json!({
            "lod": lod_name,
            "file": out_lod_path,
            "cumulative_ratio": py::round_py(cum_ratio, 4),
            "screen_size_threshold": screen_size,
            "vertices": curr_verts.len(),
            "faces": curr_faces.len()
        }));
    }

    let manifest = json!({"asset": base_name, "lods": lods});
    let manifest_path = Path::new(out_dir)
        .join(format!("{}_lod_manifest.json", base_name))
        .to_string_lossy()
        .into_owned();
    fs::write(&manifest_path, jsonfmt::dumps_indent(&manifest, 2)).map_err(|e| e.to_string())?;
    Ok(manifest)
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio retopo",
        argv,
        &[
            ArgSpec::value("mesh", Some('m'), "mesh").req(),
            ArgSpec::value("outdir", Some('o'), "outdir").def("./lods"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let mesh_path = args.get("mesh").unwrap();
    if !Path::new(mesh_path).exists() {
        eprintln!("Error: File '{}' not found.", mesh_path);
        return 1;
    }
    let outdir = args.get_or("outdir", "./lods");

    match generate_lod_hierarchy_cascading(mesh_path, &outdir) {
        Ok(manifest) => {
            if args.flag("json") {
                println!("{}", jsonfmt::dumps_indent(&manifest, 2));
            } else {
                println!(
                    "\n[QuadriFlow & Cascading LOD Engine] Generated 5-Tier LOD Hierarchy for: {}",
                    manifest["asset"].as_str().unwrap_or("")
                );
                if let Some(lods) = manifest["lods"].as_array() {
                    for item in lods {
                        println!(
                            "  • {}: {} faces ({}%) | Screen: {} -> {}",
                            item["lod"].as_str().unwrap_or(""),
                            py::comma_int(item["faces"].as_i64().unwrap_or(0)),
                            (item["cumulative_ratio"].as_f64().unwrap_or(0.0) * 100.0) as i64,
                            item["screen_size_threshold"].as_f64().unwrap_or(0.0),
                            item["file"].as_str().unwrap_or("")
                        );
                    }
                }
                println!(
                    " -> Manifest saved: {}\n",
                    Path::new(&outdir)
                        .join(format!(
                            "{}_lod_manifest.json",
                            manifest["asset"].as_str().unwrap_or("")
                        ))
                        .to_string_lossy()
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
