//! Port of generative_3d_adapter.py — AI foundation-model mesh post-processor
//! with DSU disconnected-shell pruning and routing telemetry.

use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::geometry::{parse_obj_buffered, DisjointSetUnion, MeshData};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

fn filter_disconnected_shells_dsu(
    mesh: &MeshData,
    min_area_ratio: f64,
    max_vertices: usize,
) -> (Vec<[f64; 3]>, Vec<Vec<usize>>, usize) {
    let num_verts = mesh.vertices.len();
    if num_verts > max_vertices {
        eprintln!(
            "[AI Adapter Warning] Mesh exceeds {} vertex limit. Processing primary hull...",
            crate::common::py::comma_int(max_vertices as i64)
        );
    }

    let mut dsu = DisjointSetUnion::new(num_verts);
    for f in &mesh.faces {
        if f.len() > 1 {
            let v0 = f[0];
            for &v_next in &f[1..] {
                if v0 < num_verts && v_next < num_verts {
                    dsu.union(v0, v_next);
                }
            }
        }
    }

    let mut component_sizes: HashMap<usize, usize> = HashMap::new();
    for vi in 0..num_verts {
        let root = dsu.find(vi);
        *component_sizes.entry(root).or_insert(0) += 1;
    }
    if component_sizes.is_empty() {
        return (mesh.vertices.clone(), mesh.faces.clone(), 0);
    }

    let mut sorted_roots: Vec<usize> = component_sizes.keys().copied().collect();
    sorted_roots.sort_by_key(|r| std::cmp::Reverse(component_sizes[r]));
    let primary_size = component_sizes[&sorted_roots[0]];
    let threshold_size = (primary_size as f64 * min_area_ratio) as usize;
    let threshold_size = threshold_size.max(1);

    let valid_roots: std::collections::HashSet<usize> = sorted_roots
        .iter()
        .copied()
        .filter(|r| component_sizes[r] >= threshold_size)
        .collect();
    let pruned_components = sorted_roots.len() - valid_roots.len();

    let mut old_to_new: HashMap<usize, usize> = HashMap::new();
    let mut new_verts = Vec::new();
    for (vi, v) in mesh.vertices.iter().enumerate() {
        if valid_roots.contains(&dsu.find(vi)) {
            old_to_new.insert(vi, new_verts.len());
            new_verts.push(*v);
        }
    }

    let mut new_faces = Vec::new();
    for f in &mesh.faces {
        if f.iter().all(|vi| old_to_new.contains_key(vi)) {
            new_faces.push(f.iter().map(|vi| old_to_new[vi]).collect());
        }
    }
    (new_verts, new_faces, pruned_components)
}

fn export_cleaned_obj(
    verts: &[[f64; 3]],
    faces: &[Vec<usize>],
    out_path: &str,
) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("# Autonomous 3D Studio - AI Foundation Model Cleaned Mesh\n");
    out.push_str(&format!("# Processed: {}\n", py::utcnow_iso_z()));
    for v in verts {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for f in faces {
        out.push_str(&format!(
            "f {}\n",
            f.iter()
                .map(|vi| (vi + 1).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    fs::write(out_path, out).map_err(|e| e.to_string())
}

fn process_ai_generated_mesh(
    mesh_path: &str,
    out_path: &str,
    model_source: &str,
    min_shell_ratio: f64,
) -> Result<serde_json::Value, String> {
    let mesh = parse_obj_buffered(mesh_path, 500_000)?;
    let (clean_verts, clean_faces, pruned_shells) =
        filter_disconnected_shells_dsu(&mesh, min_shell_ratio, 500_000);
    export_cleaned_obj(&clean_verts, &clean_faces, out_path)?;

    let telemetry = json!({
        "status": "SUCCESS",
        "model_source": model_source,
        "input_mesh": mesh_path,
        "cleaned_mesh": out_path,
        "original_vertices": mesh.vertices.len(),
        "cleaned_vertices": clean_verts.len(),
        "original_faces": mesh.faces.len(),
        "cleaned_faces": clean_faces.len(),
        "pruned_disconnected_shells": pruned_shells
    });

    // Foundation Model Routing Telemetry
    let dir = py::expanduser("~/.ciel/instincts");
    if fs::create_dir_all(&dir).is_ok() {
        let mut entry = serde_json::Map::new();
        entry.insert("ts".to_string(), json!(py::utcnow_iso_z()));
        for (k, v) in telemetry.as_object().unwrap() {
            entry.insert(k.clone(), v.clone());
        }
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("3d_foundation_model_telemetry.jsonl"))
        {
            let _ = writeln!(f, "{}", jsonfmt::dumps(&serde_json::Value::Object(entry)));
        }
    }
    Ok(telemetry)
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio generative",
        argv,
        &[
            ArgSpec::value("input", Some('i'), "input").req(),
            ArgSpec::value("out", Some('o'), "out").def("./ai_cleaned_mesh.obj"),
            ArgSpec::value("source", Some('s'), "source")
                .def("trellis_v2")
                .choices(&["trellis_v2", "hunyuan3d_v2", "rodin_v2_5", "instant_mesh"]),
            ArgSpec::float("min_shell", None, "min-shell").def("0.05"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let input = args.get("input").unwrap();
    if !Path::new(input).exists() {
        eprintln!("Error: File '{}' not found.", input);
        return 1;
    }
    let out = args.get_or("out", "./ai_cleaned_mesh.obj");
    let source = args.get_or("source", "trellis_v2");
    let min_shell = args.float("min_shell", 0.05);

    match process_ai_generated_mesh(input, &out, &source, min_shell) {
        Ok(result) => {
            if args.flag("json") {
                println!("{}", jsonfmt::dumps_indent(&result, 2));
            } else {
                println!(
                    "\n[AI Generative 3D Bridge] Processed raw AI foundation mesh ({}):",
                    source
                );
                println!(" -> Output: {}", out);
                println!(
                    " -> Pruned Disconnected Shells: {}",
                    result["pruned_disconnected_shells"]
                );
                println!(
                    " -> Vertices: {} -> {}",
                    result["original_vertices"], result["cleaned_vertices"]
                );
                println!(
                    " -> Faces:    {} -> {}\n",
                    result["original_faces"], result["cleaned_faces"]
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
