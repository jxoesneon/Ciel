//! Port of uv_texel_analyzer.py — UV layout / texel-density audit with
//! adaptive VRAM budgeting, instinct telemetry, and CIEL vault routing.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::geometry::{compute_triangle_area_3d, parse_obj_buffered, MeshData, Vec2};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

fn compute_triangle_2d_signed_area(u0: Vec2, u1: Vec2, u2: Vec2) -> f64 {
    0.5 * ((u1[0] - u0[0]) * (u2[1] - u0[1]) - (u2[0] - u0[0]) * (u1[1] - u0[1]))
}

fn estimate_adaptive_vram_budget(total_surface_area_m2: f64, target_td_px_cm: f64) -> Value {
    let area_cm2 = total_surface_area_m2 * 10000.0;
    let ideal_pixels_1d = area_cm2.sqrt() * target_td_px_cm;

    let powers = [512i64, 1024, 2048, 4096, 8192];
    let closest_res = powers
        .iter()
        .copied()
        .min_by(|a, b| {
            ((*a as f64) - ideal_pixels_1d)
                .abs()
                .partial_cmp(&((*b as f64) - ideal_pixels_1d).abs())
                .unwrap()
        })
        .unwrap();

    let raw_vram_mb = (closest_res * closest_res * 4 * 4) as f64 / (1024.0 * 1024.0);
    let compressed_vram_mb = raw_vram_mb / 4.0;

    json!({
        "ideal_resolution": closest_res,
        "recommended_texture_size": format!("{}x{}", closest_res, closest_res),
        "estimated_vram_compressed_mb": py::round_py(compressed_vram_mb, 2)
    })
}

pub fn analyze_texel_density(
    mesh: &MeshData,
    target_resolution: i64,
    target_td_px_cm: f64,
) -> Value {
    let verts = &mesh.vertices;
    let texcoords = &mesh.texcoords;
    let faces = &mesh.faces;
    let face_uvs = &mesh.face_uvs;

    if texcoords.is_empty() || face_uvs.is_empty() {
        return json!({"status": "FAIL", "error": "Mesh has no UV coordinates (vt missing).", "metrics": {}});
    }

    let mut total_3d_area_m2 = 0.0f64;
    let mut total_uv_normalized_area = 0.0f64;
    let mut face_td_values: Vec<f64> = Vec::new();
    let mut flipped_uv_faces = 0usize;
    let mut missing_uv_faces = 0usize;

    let mut uv_min_u = f64::INFINITY;
    let mut uv_max_u = f64::NEG_INFINITY;
    let mut uv_min_v = f64::INFINITY;
    let mut uv_max_v = f64::NEG_INFINITY;

    for (f_idx, face) in faces.iter().enumerate() {
        let f_vt: &[Option<usize>] = face_uvs.get(f_idx).map(|v| v.as_slice()).unwrap_or(&[]);
        if f_vt.iter().any(|x| x.is_none()) || f_vt.len() < 3 {
            missing_uv_faces += 1;
            continue;
        }

        let v0_3d = verts[face[0]];
        let vt0_2d = texcoords[f_vt[0].unwrap()];

        for i in 1..face.len() - 1 {
            let v1_3d = verts[face[i]];
            let v2_3d = verts[face[i + 1]];
            let area_3d = compute_triangle_area_3d(v0_3d, v1_3d, v2_3d);

            let vt1_2d = texcoords[f_vt[i].unwrap()];
            let vt2_2d = texcoords[f_vt[i + 1].unwrap()];
            let signed_uv = compute_triangle_2d_signed_area(vt0_2d, vt1_2d, vt2_2d);

            if signed_uv < 0.0 {
                flipped_uv_faces += 1;
            }

            let abs_uv = signed_uv.abs();
            total_3d_area_m2 += area_3d;
            total_uv_normalized_area += abs_uv;

            for vt in [vt0_2d, vt1_2d, vt2_2d] {
                uv_min_u = uv_min_u.min(vt[0]);
                uv_max_u = uv_max_u.max(vt[0]);
                uv_min_v = uv_min_v.min(vt[1]);
                uv_max_v = uv_max_v.max(vt[1]);
            }

            let area_3d_cm2 = area_3d * 10000.0;
            if area_3d_cm2 > 1e-6 && abs_uv > 1e-8 {
                let pixel_area = abs_uv * (target_resolution as f64).powi(2);
                let td_cm = pixel_area.sqrt() / area_3d_cm2.sqrt();
                face_td_values.push(td_cm);
            }
        }
    }

    if face_td_values.is_empty() {
        return json!({"status": "FAIL", "error": "Zero valid UV area.", "metrics": {}});
    }

    let avg_td = face_td_values.iter().sum::<f64>() / face_td_values.len() as f64;
    let min_td = face_td_values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_td = face_td_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let variance_pct = if avg_td > 0.0 {
        (max_td - min_td) / avg_td * 100.0
    } else {
        0.0
    };

    let vram_heuristics = estimate_adaptive_vram_budget(total_3d_area_m2, target_td_px_cm);

    let mut qa_warnings: Vec<String> = Vec::new();
    let mut passed = true;

    if missing_uv_faces > 0 {
        passed = false;
        qa_warnings.push(format!(
            "CRITICAL: {} faces are missing UV texture mappings.",
            missing_uv_faces
        ));
    }
    if flipped_uv_faces > 0 {
        qa_warnings.push(format!(
            "WARNING: {} UV sub-triangles are flipped.",
            flipped_uv_faces
        ));
    }
    if variance_pct > 15.0 {
        qa_warnings.push(format!(
            "WARNING: High Texel Density variance ({}%). Recommended <= 10%.",
            py::round_py(variance_pct, 1)
        ));
    }

    json!({
        "status": if passed { "PASS" } else { "FAIL" },
        "target_resolution": target_resolution,
        "target_td_px_cm": target_td_px_cm,
        "texel_density": {
            "average_px_cm": py::round_py(avg_td, 3),
            "average_px_m": py::round_py(avg_td * 100.0, 1),
            "min_px_cm": py::round_py(min_td, 3),
            "max_px_cm": py::round_py(max_td, 3),
            "variance_percentage": py::round_py(variance_pct, 2)
        },
        "adaptive_vram_budget": vram_heuristics,
        "uv_space_metrics": {
            "total_3d_surface_area_m2": py::round_py(total_3d_area_m2, 4),
            "uv_coverage_percentage": py::round_py(total_uv_normalized_area * 100.0, 2),
            "uv_bounds": {
                "u_min": py::round_py(uv_min_u, 4), "u_max": py::round_py(uv_max_u, 4),
                "v_min": py::round_py(uv_min_v, 4), "v_max": py::round_py(uv_max_v, 4)
            },
            "flipped_uv_faces": flipped_uv_faces,
            "missing_uv_faces": missing_uv_faces
        },
        "qa_warnings": qa_warnings
    })
}

fn record_uv_instinct(mesh_path: &str, report: &Value) {
    let dir = py::expanduser("~/.ciel/instincts");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let entry = json!({
        "ts": py::utcnow_iso_z(),
        "asset": Path::new(mesh_path).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
        "domain": "uv_texel_density",
        "avg_td_px_cm": report["texel_density"]["average_px_cm"],
        "variance_pct": report["texel_density"]["variance_percentage"],
        "warnings": report["qa_warnings"]
    });
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("3d_studio_observations.jsonl"))
    {
        let _ = writeln!(f, "{}", jsonfmt::dumps(&entry));
    }
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio uv-texel",
        argv,
        &[
            ArgSpec::value("mesh", Some('m'), "mesh").req(),
            ArgSpec::int("res", Some('r'), "res")
                .def("4096")
                .choices(&["512", "1024", "2048", "4096", "8192"]),
            ArgSpec::float("target_td", Some('t'), "target-td").def("20.48"),
            ArgSpec::flag("vault", None, "vault"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let mesh_path = args.get("mesh").unwrap();
    if !Path::new(mesh_path).exists() {
        eprintln!("Error: Mesh file '{}' not found.", mesh_path);
        return 1;
    }
    let mesh = match parse_obj_buffered(mesh_path, 1_000_000) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let mut report =
        analyze_texel_density(&mesh, args.int("res", 4096), args.float("target_td", 20.48));
    record_uv_instinct(mesh_path, &report);

    if args.flag("vault") {
        let vault_dir = py::expanduser("~/.ciel/artifacts/audits");
        let _ = fs::create_dir_all(&vault_dir);
        let stem = Path::new(mesh_path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let vault_path = vault_dir.join(format!("{}_texel_audit.json", stem));
        if fs::write(&vault_path, jsonfmt::dumps_indent(&report, 2)).is_ok() {
            report["vault_saved_path"] = json!(vault_path.to_string_lossy());
        }
    }

    if args.flag("json") {
        println!("{}", jsonfmt::dumps_indent(&report, 2));
    } else {
        let base = Path::new(mesh_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        println!("\n{:=<70}", "");
        println!(" AAA STUDIO UV & TEXEL DENSITY REPORT: {}", base);
        println!("{:=<70}", "");
        println!(
            " Status:              {}",
            report["status"].as_str().unwrap_or("")
        );
        println!(
            " Measured Average TD: {} px/cm ({} px/m)",
            report["texel_density"]["average_px_cm"], report["texel_density"]["average_px_m"]
        );
        println!(
            " TD Variance:         {}%",
            report["texel_density"]["variance_percentage"]
        );
        println!(
            " Adaptive VRAM Budget: {} (~{} MB VRAM)",
            report["adaptive_vram_budget"]["recommended_texture_size"]
                .as_str()
                .unwrap_or(""),
            report["adaptive_vram_budget"]["estimated_vram_compressed_mb"]
        );
        if let Some(warnings) = report["qa_warnings"].as_array() {
            for w in warnings {
                println!("  • {}", w.as_str().unwrap_or(""));
            }
        }
        println!("{:=<70}\n", "");
    }

    if report["status"].as_str() == Some("FAIL") {
        return 2;
    }
    0
}
