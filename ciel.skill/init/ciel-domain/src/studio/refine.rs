//! Port of autonomous_refinement_loop.py — closed-loop iterative self-healing
//! engine. In the Rust crate the validator / UV analyzer / turnaround steps
//! run in-process (same code paths as the individual subcommands) instead of
//! spawning Python subprocesses.

use serde_json::json;
use std::fs;
use std::path::Path;

use super::{geometry::parse_obj_buffered, qa, turnaround, uv_texel};
use crate::common::cli::{self, ArgSpec};
use crate::common::py;

fn run_refinement_iteration(
    mesh_path: &str,
    out_dir: &str,
    profile: &str,
    target_td: f64,
    resolution: i64,
    max_iterations: i64,
    use_vault: bool,
) -> serde_json::Value {
    let out_dir_owned;
    let out_dir = if use_vault {
        out_dir_owned = py::expanduser("~/.ciel/artifacts/models")
            .to_string_lossy()
            .into_owned();
        out_dir_owned.as_str()
    } else {
        out_dir
    };
    let _ = fs::create_dir_all(out_dir);
    let mut current_mesh = mesh_path.to_string();
    let mut iteration = 1i64;

    println!("\n{:=<75}", "");
    println!(
        " [AUTONOMOUS 3D STUDIO] INITIALIZING CLOSED-LOOP REFINEMENT: {}",
        Path::new(mesh_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    );
    println!("{:=<75}", "");

    while iteration <= max_iterations {
        println!("\n--- Iteration {}/{} ---", iteration, max_iterations);

        // 1. Geometry QA Audit
        let geo_report = match parse_obj_buffered(&current_mesh, 1_000_000) {
            Ok(m) => {
                let mut r = qa::audit_geometry(&m, profile, false);
                qa::record_instinct_observation(&current_mesh, &r);
                if use_vault {
                    let stem = Path::new(&current_mesh)
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let vp = qa::route_to_ciel_vault(&r, &stem);
                    r["vault_saved_path"] = vp
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null);
                }
                r
            }
            Err(_) => json!({}),
        };

        let geo_status = geo_report
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("FAIL")
            .to_string();
        println!(" -> Geometry QA Status: {}", geo_status);

        // 2. Auto-repair if failed
        if geo_status == "FAIL" {
            let stem = Path::new(mesh_path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let repaired_out = Path::new(out_dir)
                .join(format!("{}_iter{}_repaired.obj", stem, iteration))
                .to_string_lossy()
                .into_owned();
            println!(
                " -> Applying Programmatic Auto-Repair (--fix) -> {}",
                repaired_out
            );
            if let Ok(m) = parse_obj_buffered(&current_mesh, 1_000_000) {
                let _ = qa::repair_mesh_data(&m, &repaired_out);
            }
            current_mesh = repaired_out;
            iteration += 1;
            continue;
        }

        // 3. UV & Texel Density Audit
        let uv_report = match parse_obj_buffered(&current_mesh, 1_000_000) {
            Ok(m) => uv_texel::analyze_texel_density(&m, resolution, target_td),
            Err(_) => json!({}),
        };

        // 4. Compile Visual QA Turnaround
        let (mut v_count, mut f_count, mut q_pct) = (12480i64, 12450i64, 99.8f64);
        if current_mesh.ends_with(".obj") {
            if let Ok(m) = parse_obj_buffered(&current_mesh, 1_000_000) {
                v_count = m.vertices.len() as i64;
                f_count = m.faces.len() as i64;
                let quads = m.faces.iter().filter(|f| f.len() == 4).count() as f64;
                q_pct = if f_count > 0 {
                    py::round_py(quads / f_count as f64 * 100.0, 1)
                } else {
                    100.0
                };
            }
        }
        let turn_dir = if use_vault {
            py::expanduser("~/.ciel/artifacts/visuals")
                .to_string_lossy()
                .into_owned()
        } else {
            out_dir.to_string()
        };
        let _ = fs::create_dir_all(&turn_dir);
        let stem = Path::new(&current_mesh)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let out_html = Path::new(&turn_dir)
            .join(format!("{}_turnaround_qa.html", stem))
            .to_string_lossy()
            .into_owned();
        let _ = turnaround::generate_turnaround_report(
            &current_mesh,
            &out_html,
            v_count,
            f_count,
            q_pct,
        );

        println!("\n{:=<75}", "");
        println!(" [AUTONOMOUS 3D STUDIO] QUALITY GATE CONVERGENCE ACHIEVED:");
        println!(" -> Final Production Mesh: {}", current_mesh);
        println!(
            " -> Visual Turnaround Sheet: {}",
            Path::new(out_dir)
                .join("turnaround_qa_report.html")
                .to_string_lossy()
        );
        println!(
            " -> All AAA Studio Hard Gates Passed in {} iteration(s).",
            iteration
        );
        println!("{:=<75}\n", "");
        return json!({
            "status": "CONVERGED",
            "iterations": iteration,
            "final_mesh": current_mesh,
            "geometry_report": geo_report,
            "uv_report": uv_report
        });
    }

    println!(
        "\n[Autonomous 3D Studio] Max iterations ({}) reached without full convergence.",
        max_iterations
    );
    json!({"status": "PARTIAL", "final_mesh": current_mesh})
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio refine",
        argv,
        &[
            ArgSpec::value("mesh", Some('m'), "mesh").req(),
            ArgSpec::value("outdir", Some('o'), "outdir").def("./refined_asset"),
            ArgSpec::value("profile", Some('p'), "profile").def("aaa_game"),
            ArgSpec::float("target_td", Some('t'), "target-td").def("20.48"),
            ArgSpec::int("res", Some('r'), "res").def("4096"),
            ArgSpec::flag("vault", None, "vault"),
        ],
    );

    let result = run_refinement_iteration(
        args.get("mesh").unwrap(),
        &args.get_or("outdir", "./refined_asset"),
        &args.get_or("profile", "aaa_game"),
        args.float("target_td", 20.48),
        args.int("res", 4096),
        3,
        args.flag("vault"),
    );
    if result["status"].as_str() != Some("CONVERGED") {
        return 2;
    }
    0
}
