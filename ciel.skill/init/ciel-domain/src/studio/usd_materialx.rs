//! Port of usd_materialx_bridge.py — OpenUSD ASCII stage assembler and
//! OpenPBR MaterialX document emitter.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::common::cli::{self, ArgSpec};

pub fn convert_obj_to_usda(
    obj_path: &str,
    out_usda_path: &str,
    material_name: &str,
) -> Result<String, String> {
    let mut verts: Vec<String> = Vec::new();
    let mut texcoords: Vec<String> = Vec::new();
    let mut normals: Vec<String> = Vec::new();
    let mut face_v_indices: Vec<i64> = Vec::new();
    let mut face_v_counts: Vec<usize> = Vec::new();
    let mut face_vt_indices: Vec<i64> = Vec::new();
    let mut face_vn_indices: Vec<i64> = Vec::new();

    let file = fs::File::open(obj_path).map_err(|e| e.to_string())?;
    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "v" => {
                let x: f64 = tokens[1].parse().unwrap_or(0.0);
                let y: f64 = tokens[2].parse().unwrap_or(0.0);
                let z: f64 = tokens[3].parse().unwrap_or(0.0);
                verts.push(format!("({:.6}, {:.6}, {:.6})", x, y, z));
            }
            "vt" => {
                let u: f64 = tokens[1].parse().unwrap_or(0.0);
                let v: f64 = if tokens.len() > 2 {
                    tokens[2].parse().unwrap_or(0.0)
                } else {
                    0.0
                };
                texcoords.push(format!("({:.6}, {:.6})", u, v));
            }
            "vn" => {
                normals.push(format!(
                    "({:.6}, {:.6}, {:.6})",
                    tokens[1].parse::<f64>().unwrap_or(0.0),
                    tokens[2].parse::<f64>().unwrap_or(0.0),
                    tokens[3].parse::<f64>().unwrap_or(0.0)
                ));
            }
            "f" => {
                let mut face_v = Vec::new();
                let mut face_vt = Vec::new();
                let mut face_vn = Vec::new();
                for p in &tokens[1..] {
                    let parts: Vec<&str> = p.split('/').collect();
                    let v_idx = if !parts[0].is_empty() {
                        parts[0].parse::<i64>().unwrap_or(1) - 1
                    } else {
                        0
                    };
                    let vt_idx = if parts.len() > 1 && !parts[1].is_empty() {
                        parts[1].parse::<i64>().unwrap_or(1) - 1
                    } else {
                        0
                    };
                    let vn_idx = if parts.len() > 2 && !parts[2].is_empty() {
                        parts[2].parse::<i64>().unwrap_or(1) - 1
                    } else {
                        0
                    };
                    face_v.push(v_idx);
                    face_vt.push(vt_idx);
                    face_vn.push(vn_idx);
                }
                face_v_counts.push(face_v.len());
                face_v_indices.extend(face_v);
                face_vt_indices.extend(face_vt);
                face_vn_indices.extend(face_vn);
            }
            _ => {}
        }
    }

    let mesh_name = Path::new(obj_path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let counts_str = face_v_counts
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let idx_str = face_v_indices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let pts_str = verts.join(", ");

    let usda_content = format!(
        r#"#usda 1.0
(
    defaultPrim = "Root"
    metersPerUnit = 1.0
    upAxis = "Z"
)

def Xform "Root"
{{
    def Scope "Looks"
    {{
        def Material "{material_name}"
        {{
            token outputs:surface.connect = </Root/Looks/{material_name}/PBRShader.outputs:surface>

            def Shader "PBRShader"
            {{
                uniform token info:id = "UsdPreviewSurface"
                color3f inputs:diffuseColor = (0.8, 0.8, 0.8)
                float inputs:metallic = 0.0
                float inputs:roughness = 0.4
                token outputs:surface
            }}
        }}
    }}

    def Scope "Geometry"
    {{
        def Mesh "{mesh_name}" (
            prepend apiSchemas = ["MaterialBindingAPI"]
        )
        {{
            uniform bool doubleSided = 0
            int[] faceVertexCounts = [{counts_str}]
            int[] faceVertexIndices = [{idx_str}]
            point3f[] points = [{pts_str}]

            rel material:binding = </Root/Looks/{material_name}>
        }}
    }}
}}
"#,
        material_name = material_name,
        mesh_name = mesh_name,
        counts_str = counts_str,
        idx_str = idx_str,
        pts_str = pts_str
    );

    fs::write(out_usda_path, usda_content).map_err(|e| e.to_string())?;
    Ok(out_usda_path.to_string())
}

pub fn generate_materialx_openpbr(mat_name: &str, out_mtlx_path: &str) -> Result<String, String> {
    let mtlx_code = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<materialx version="1.38">
  <!-- OpenPBR Surface Standard -->
  <open_pbr_surface name="SR_{mat_name}" type="surfaceshader">
    <input name="base_color" type="color3" value="0.8, 0.8, 0.8" />
    <input name="base_metalness" type="float" value="0.0" />
    <input name="specular_roughness" type="float" value="0.35" />
    <input name="specular_ior" type="float" value="1.5" />
  </open_pbr_surface>

  <surfacematerial name="{mat_name}" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="SR_{mat_name}" />
  </surfacematerial>
</materialx>
"#,
        mat_name = mat_name
    );
    fs::write(out_mtlx_path, mtlx_code).map_err(|e| e.to_string())?;
    Ok(out_mtlx_path.to_string())
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio usd-bridge",
        argv,
        &[
            ArgSpec::value("mesh", Some('m'), "mesh").req(),
            ArgSpec::value("usd", Some('u'), "usd").def("./stage.usda"),
            ArgSpec::value("matx", Some('x'), "matx").def("./material.mtlx"),
        ],
    );

    let mesh = args.get("mesh").unwrap();
    let usd = args.get_or("usd", "./stage.usda");
    let matx = args.get_or("matx", "./material.mtlx");

    if let Err(e) = convert_obj_to_usda(mesh, &usd, "M_CyberAsset") {
        eprintln!("Error: {}", e);
        return 1;
    }
    if let Err(e) = generate_materialx_openpbr("M_Asset", &matx) {
        eprintln!("Error: {}", e);
        return 1;
    }

    println!("\n[OpenUSD & MaterialX Bridge] Assembled production stage:");
    println!(" -> OpenUSD Stage: {}", usd);
    println!(" -> MaterialX Shader: {}\n", matx);
    0
}
