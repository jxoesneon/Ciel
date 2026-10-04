//! Port of udim_pack_analyzer.py — multi-tile UDIM distribution audit and
//! 4-channel packing presets (ORM / RMA / Packed_Mask).

use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use super::geometry::{compute_triangle_area_3d, parse_obj_buffered, MeshData};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

fn get_udim_tile_id(u: f64, v: f64) -> Option<i64> {
    let u_tile = u.floor() as i64;
    let v_tile = v.floor() as i64;
    if !(0..10).contains(&u_tile) || v_tile < 0 {
        return None;
    }
    Some(1001 + u_tile + 10 * v_tile)
}

pub fn analyze_udim_distribution(mesh: &MeshData, resolution: i64) -> serde_json::Value {
    // Python dicts preserve insertion order; tiles are later sorted by key.
    let mut tile_to_faces: HashMap<i64, Vec<usize>> = HashMap::new();
    let mut tile_surface_area: HashMap<i64, f64> = HashMap::new();
    let mut boundary_crossing_faces: Vec<usize> = Vec::new();

    let verts = &mesh.vertices;
    let texcoords = &mesh.texcoords;
    let faces = &mesh.faces;
    let face_uvs = &mesh.face_uvs;

    for (f_idx, face) in faces.iter().enumerate() {
        let f_vt: &[Option<usize>] = face_uvs.get(f_idx).map(|v| v.as_slice()).unwrap_or(&[]);
        if f_vt.iter().any(|x| x.is_none()) || f_vt.len() < 3 {
            continue;
        }

        let mut face_udims: HashSet<i64> = HashSet::new();
        // Preserve Python `list(set)` first-seen order for the primary tile.
        let mut face_udims_order: Vec<i64> = Vec::new();
        for vt_i in f_vt.iter().flatten() {
            let (u, v) = (texcoords[*vt_i][0], texcoords[*vt_i][1]);
            if let Some(tile) = get_udim_tile_id(u, v) {
                if face_udims.insert(tile) {
                    face_udims_order.push(tile);
                }
            }
        }

        if face_udims.len() > 1 {
            boundary_crossing_faces.push(f_idx);
        }

        let primary_tile = face_udims_order.first().copied().unwrap_or(1001);
        tile_to_faces.entry(primary_tile).or_default().push(f_idx);

        let v0 = verts[face[0]];
        for i in 1..face.len() - 1 {
            let v1 = verts[face[i]];
            let v2 = verts[face[i + 1]];
            *tile_surface_area.entry(primary_tile).or_insert(0.0) +=
                compute_triangle_area_3d(v0, v1, v2);
        }
    }

    let mut udim_reports = Vec::new();
    let sorted: BTreeMap<i64, &Vec<usize>> = tile_to_faces.iter().map(|(k, v)| (*k, v)).collect();
    for (tile_id, face_list) in &sorted {
        let area_m2 = tile_surface_area.get(tile_id).copied().unwrap_or(0.0);
        udim_reports.push(json!({
            "udim_tile": tile_id,
            "face_count": face_list.len(),
            "surface_area_m2": py::round_py(area_m2, 4),
            "target_texture_resolution": format!("{}x{}", resolution, resolution)
        }));
    }

    json!({
        "status": if boundary_crossing_faces.is_empty() { "PASS" } else { "WARNING" },
        "total_udim_tiles": tile_to_faces.len(),
        "udim_tiles": udim_reports,
        "boundary_crossing_faces": boundary_crossing_faces.len(),
        "channel_packing_presets": {
            "ORM": {"R": "Ambient Occlusion", "G": "Roughness", "B": "Metallic", "A": "None"},
            "RMA": {"R": "Roughness", "G": "Metallic", "B": "Ambient Occlusion", "A": "None"},
            "Packed_Mask": {"R": "Curvature", "G": "Edge Wear", "B": "Cavity Dirt", "A": "Emissive Mask"}
        }
    })
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio udim",
        argv,
        &[
            ArgSpec::value("mesh", Some('m'), "mesh").req(),
            ArgSpec::int("res", Some('r'), "res").def("4096"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let mesh_path = args.get("mesh").unwrap();
    if !Path::new(mesh_path).exists() {
        eprintln!("Error: Mesh '{}' not found.", mesh_path);
        return 1;
    }
    let mesh = match parse_obj_buffered(mesh_path, 1_000_000) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let report = analyze_udim_distribution(&mesh, args.int("res", 4096));

    if args.flag("json") {
        println!("{}", jsonfmt::dumps_indent(&report, 2));
    } else {
        let base = Path::new(mesh_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        println!("\n[UDIM & Channel Packing Analyzer] Evaluated: {}", base);
        println!(" -> Active UDIM Tiles: {}", report["total_udim_tiles"]);
        if let Some(tiles) = report["udim_tiles"].as_array() {
            for udim in tiles {
                println!(
                    "    • UDIM {}: {} faces | Area: {} m2 ({})",
                    udim["udim_tile"],
                    py::comma_int(udim["face_count"].as_i64().unwrap_or(0)),
                    py::py_float(udim["surface_area_m2"].as_f64().unwrap_or(0.0)),
                    udim["target_texture_resolution"].as_str().unwrap_or("")
                );
            }
        }
        let bcf = report["boundary_crossing_faces"].as_i64().unwrap_or(0);
        if bcf > 0 {
            println!(" -> WARNING: {} faces cross UDIM tile borders!", bcf);
        }
        println!();
    }
    0
}
