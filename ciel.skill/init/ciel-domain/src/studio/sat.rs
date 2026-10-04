//! Port of substance_sat_baker.py — Adobe Substance Automation Toolkit (SAT)
//! `sbsbaker` command construction and batch shell recipe emission.

use serde_json::json;
use std::fs;
use std::path::Path;

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

pub fn find_sat_baker_binary() -> Option<String> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            for name in ["sbsbaker", "sbsbaker.exe"] {
                let cand = dir.join(name);
                if cand.exists() {
                    return Some(cand.to_string_lossy().into_owned());
                }
            }
        }
    }
    for p in [
        "/opt/Allegorithmic/Substance_Automation_Toolkit/sbsbaker",
        "/usr/local/bin/sbsbaker",
        r"C:\Program Files\Allegorithmic\Substance Automation Toolkit\sbsbaker.exe",
    ] {
        if Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    None
}

fn shlex_quote(token: &str) -> String {
    // Python shlex.quote: safe chars pass through, else single-quote wrapped.
    let safe = |c: char| c.is_ascii_alphanumeric() || "@%_+=:,./-".contains(c);
    if !token.is_empty() && token.chars().all(safe) {
        return token.to_string();
    }
    format!("'{}'", token.replace('\'', "'\"'\"'"))
}

pub fn build_sat_bake_command(
    high_mesh: &str,
    low_mesh: &str,
    out_dir: &str,
    map_type: &str,
    resolution: i64,
    format_out: &str,
) -> Vec<String> {
    let sat_baker_type = match map_type {
        "normal" => "normal-from-mesh",
        "ao" => "ambient-occlusion",
        "curvature" => "curvature",
        "world_normal" => "world-space-normals",
        "position" => "position",
        "thickness" => "thickness",
        "color_id" => "color-from-mesh",
        _ => "normal-from-mesh",
    };
    let stem = Path::new(low_mesh)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    vec![
        "sbsbaker".to_string(),
        sat_baker_type.to_string(),
        "--high-mesh".to_string(),
        py::abspath(high_mesh),
        "--low-mesh".to_string(),
        py::abspath(low_mesh),
        "--output-path".to_string(),
        py::abspath(out_dir),
        "--output-name".to_string(),
        format!("T_{}_{}", stem, map_type.to_uppercase()),
        "--output-format".to_string(),
        format_out.to_string(),
        "--output-size".to_string(),
        format!("{}x{}", resolution, resolution),
        "--antialiasing".to_string(),
        "4x4".to_string(),
        "--ray-distance".to_string(),
        "0.01".to_string(),
        "--tangent-space-mode".to_string(),
        "mikktspace".to_string(),
    ]
}

pub fn generate_sat_batch_script(
    high_mesh: &str,
    low_mesh: &str,
    out_dir: &str,
    out_script_path: &str,
    resolution: i64,
) -> Result<serde_json::Value, String> {
    let maps = [
        "normal",
        "ao",
        "curvature",
        "world_normal",
        "position",
        "thickness",
        "color_id",
    ];
    let mut lines = vec![
        "#!/bin/bash".to_string(),
        "# Adobe Substance Automation Toolkit (SAT) Headless Batch Bake".to_string(),
        "set -e".to_string(),
        format!("mkdir -p {}", shlex_quote(&py::abspath(out_dir))),
    ];

    let mut command_list: Vec<serde_json::Value> = Vec::new();
    for m in &maps {
        let cmd_vec = build_sat_bake_command(high_mesh, low_mesh, out_dir, m, resolution, "png");
        lines.push(
            cmd_vec
                .iter()
                .map(|t| shlex_quote(t))
                .collect::<Vec<_>>()
                .join(" "),
        );
        command_list.push(json!(cmd_vec));
    }

    let script_content = lines.join("\n") + "\n";
    fs::write(out_script_path, script_content).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(out_script_path, fs::Permissions::from_mode(0o755));
    }

    Ok(json!({
        "status": "SUCCESS",
        "recipe_script": out_script_path,
        "high_mesh": high_mesh,
        "low_mesh": low_mesh,
        "out_dir": out_dir,
        "resolution": resolution,
        "maps_configured": maps,
        "commands": command_list
    }))
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio sat-bake",
        argv,
        &[
            ArgSpec::value("high", Some('H'), "high").req(),
            ArgSpec::value("low", Some('L'), "low").req(),
            ArgSpec::value("outdir", Some('o'), "outdir").def("./sat_bakes"),
            ArgSpec::int("res", Some('r'), "res")
                .def("4096")
                .choices(&["1024", "2048", "4096", "8192"]),
            ArgSpec::value("script_out", None, "script-out").def("./run_sat_bakes.sh"),
            ArgSpec::flag("json", None, "json"),
        ],
    );

    let high = args.get("high").unwrap();
    let low = args.get("low").unwrap();
    let outdir = args.get_or("outdir", "./sat_bakes");
    let res = args.int("res", 4096);
    let script_out = args.get_or("script_out", "./run_sat_bakes.sh");

    let _ = fs::create_dir_all(&outdir);

    match generate_sat_batch_script(high, low, &outdir, &script_out, res) {
        Ok(result) => {
            if args.flag("json") {
                println!("{}", jsonfmt::dumps_indent(&result, 2));
            } else {
                println!("\n[Substance SAT Baker Bridge] Generated SAT batch recipe:");
                println!(
                    " -> Shell Script: {}",
                    result["recipe_script"].as_str().unwrap_or("")
                );
                println!(
                    " -> Target Resolution: {}x{} (MikkTSpace Normal, AO, Curvature, Position, Thickness, ID)",
                    res, res
                );
                println!(
                    " -> Run via SAT CLI: `./{}`\n",
                    Path::new(result["recipe_script"].as_str().unwrap_or(""))
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default()
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
