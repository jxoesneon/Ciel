//! Port of collision_hull_generator.py — UBX_/UCX_ collision primitive
//! generation for UE5 Chaos / Unity PhysX.

use serde_json::json;
use std::fs;
use std::path::Path;

use super::geometry::{compute_single_pass_bounding_box, parse_obj_buffered, MeshData};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;

fn write_box(
    f: &mut String,
    hull_name: &str,
    min_pt: [f64; 3],
    max_pt: [f64; 3],
) -> (usize, usize) {
    let (x0, y0, z0) = (min_pt[0], min_pt[1], min_pt[2]);
    let (x1, y1, z1) = (max_pt[0], max_pt[1], max_pt[2]);
    let verts = [
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y1, z0],
        [x0, y1, z0],
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y1, z1],
        [x0, y1, z1],
    ];
    let faces = [
        [1usize, 4, 3, 2],
        [5, 6, 7, 8],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 4, 8, 7],
        [4, 1, 5, 8],
    ];
    f.push_str(&format!("o {}\n", hull_name));
    for v in &verts {
        f.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for face in &faces {
        f.push_str(&format!(
            "f {}\n",
            face.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    (verts.len(), faces.len())
}

fn generate_ubx_box_hull(mesh: &MeshData, render_name: &str, out_path: &str) -> serde_json::Value {
    let (min_pt, max_pt, dims) = compute_single_pass_bounding_box(&mesh.vertices);
    let hull_name = format!("UBX_{}_01", render_name);
    let mut content = format!("# UE5 Collision Hull: {}\n", hull_name);
    let (vc, fc) = write_box(&mut content, &hull_name, min_pt, max_pt);
    let _ = fs::write(out_path, content);
    json!({
        "hull_type": "UBX",
        "hull_name": hull_name,
        "file": out_path,
        "vertex_count": vc,
        "face_count": fc,
        "bounds": {"min": min_pt, "max": max_pt, "dimensions": dims}
    })
}

fn generate_ucx_compound_convex_hulls(
    mesh: &MeshData,
    render_name: &str,
    out_path: &str,
    max_hulls: usize,
) -> serde_json::Value {
    let (min_pt, max_pt, dims) = compute_single_pass_bounding_box(&mesh.vertices);
    let split_axis = if dims[0] >= dims[1] && dims[0] >= dims[2] {
        0
    } else if dims[1] >= dims[2] {
        1
    } else {
        2
    };
    let step = dims[split_axis] / max_hulls as f64;

    let mut content = format!("# UE5 Compound Convex Hulls for {}\n", render_name);
    let mut hulls = Vec::new();
    let mut vert_offset = 0usize;
    for h_idx in 0..max_hulls {
        let h_name = format!("UCX_{}_{:02}", render_name, h_idx + 1);
        content.push_str(&format!("\no {}\n", h_name));
        let mut sub_min = min_pt;
        let mut sub_max = max_pt;
        sub_min[split_axis] = min_pt[split_axis] + h_idx as f64 * step;
        sub_max[split_axis] = min_pt[split_axis] + (h_idx + 1) as f64 * step;
        let (x0, y0, z0) = (sub_min[0], sub_min[1], sub_min[2]);
        let (x1, y1, z1) = (sub_max[0], sub_max[1], sub_max[2]);
        let h_verts = [
            [x0, y0, z0],
            [x1, y0, z0],
            [x1, y1, z0],
            [x0, y1, z0],
            [x0, y0, z1],
            [x1, y0, z1],
            [x1, y1, z1],
            [x0, y1, z1],
        ];
        for v in &h_verts {
            content.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
        }
        let h_faces = [
            [1usize, 4, 3, 2],
            [5, 6, 7, 8],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 4, 8, 7],
            [4, 1, 5, 8],
        ];
        for face in &h_faces {
            content.push_str(&format!(
                "f {}\n",
                face.iter()
                    .map(|v| (v + vert_offset).to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
        vert_offset += h_verts.len();
        hulls.push(json!({"name": h_name, "vertices": h_verts.len(), "faces": h_faces.len()}));
    }
    let _ = fs::write(out_path, content);
    json!({
        "status": "SUCCESS",
        "render_mesh": render_name,
        "hull_file": out_path,
        "hull_count": hulls.len(),
        "hulls": hulls
    })
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio collision",
        argv,
        &[
            ArgSpec::value("mesh", Some('m'), "mesh").req(),
            ArgSpec::value("type", Some('t'), "type")
                .def("UCX")
                .choices(&["UCX", "UBX", "USP"]),
            ArgSpec::int("max_hulls", None, "max-hulls").def("4"),
            ArgSpec::value("out", Some('o'), "out"),
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
    let render_name = Path::new(mesh_path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let typ = args.get_or("type", "UCX");
    let out_file = match args.get("out") {
        Some(o) => o.to_string(),
        None => Path::new(mesh_path)
            .parent()
            .unwrap_or(Path::new(""))
            .join(format!("{}_{}.obj", typ, render_name))
            .to_string_lossy()
            .into_owned(),
    };

    let result = if typ == "UBX" {
        generate_ubx_box_hull(&mesh, &render_name, &out_file)
    } else {
        generate_ucx_compound_convex_hulls(
            &mesh,
            &render_name,
            &out_file,
            args.int("max_hulls", 4).max(1) as usize,
        )
    };

    if args.flag("json") {
        println!("{}", jsonfmt::dumps_indent(&result, 2));
    } else {
        println!("\n[Collision Hull Generator] Created UE5 Chaos collision geometry:");
        println!(" -> Output File: {}", out_file);
        println!(" -> Collision Type: {}", typ);
        println!(
            " -> Hulls Generated: {}\n",
            result
                .get("hull_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(1)
        );
    }
    0
}
