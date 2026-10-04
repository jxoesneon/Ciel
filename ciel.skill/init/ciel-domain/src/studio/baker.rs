//! Port of high_to_low_baker.py — vertex-averaged projection cage generation
//! and Blender bake-recipe emission.

use std::fs;
use std::path::Path;

use super::geometry::{parse_obj_buffered, Vec3};
use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

type ParsedMesh = (Vec<Vec3>, Vec<Vec3>, Vec<Vec<usize>>);

fn parse_obj_simple(filepath: &str) -> Result<ParsedMesh, String> {
    // Same grammar as high_to_low_baker's local parser: v / vn / f only.
    let mesh = parse_obj_buffered(filepath, 1_000_000)?;
    Ok((mesh.vertices, mesh.normals, mesh.faces))
}

fn compute_vertex_averaged_normals(verts: &[Vec3], faces: &[Vec<usize>]) -> Vec<Vec3> {
    let mut vert_normals = vec![[0.0f64; 3]; verts.len()];
    for face in faces {
        if face.len() < 3 {
            continue;
        }
        let v0 = verts[face[0]];
        for i in 1..face.len() - 1 {
            let v1 = verts[face[i]];
            let v2 = verts[face[i + 1]];
            let (ax, ay, az) = (v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]);
            let (bx, by, bz) = (v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);
            let nx = ay * bz - az * by;
            let ny = az * bx - ax * bz;
            let nz = ax * by - ay * bx;
            for &vi in face {
                vert_normals[vi][0] += nx;
                vert_normals[vi][1] += ny;
                vert_normals[vi][2] += nz;
            }
        }
    }
    vert_normals
        .iter()
        .map(|vn| {
            let len = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
            if len > 1e-9 {
                [vn[0] / len, vn[1] / len, vn[2] / len]
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect()
}

fn generate_cage_mesh(
    low_poly_path: &str,
    out_cage_path: &str,
    push_distance: f64,
) -> Result<String, String> {
    let (verts, _normals, faces) = parse_obj_simple(low_poly_path)?;
    let avg_normals = compute_vertex_averaged_normals(&verts, &faces);

    let mut out = String::new();
    out.push_str("# AAA Studio Automated Baking Cage Mesh\n");
    out.push_str(&format!(
        "# Source: {}\n",
        Path::new(low_poly_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    ));
    out.push_str(&format!(
        "# Push Distance: {}m\n",
        py::py_num(push_distance)
    ));
    for (i, v) in verts.iter().enumerate() {
        let n = avg_normals[i];
        out.push_str(&format!(
            "v {:.6} {:.6} {:.6}\n",
            v[0] + n[0] * push_distance,
            v[1] + n[1] * push_distance,
            v[2] + n[2] * push_distance
        ));
    }
    for n in &avg_normals {
        out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", n[0], n[1], n[2]));
    }
    for face in &faces {
        let parts: Vec<String> = face
            .iter()
            .map(|vi| format!("{}//{}", vi + 1, vi + 1))
            .collect();
        out.push_str(&format!("f {}\n", parts.join(" ")));
    }
    fs::write(out_cage_path, out).map_err(|e| e.to_string())?;
    Ok(out_cage_path.to_string())
}

fn generate_blender_bake_recipe(
    high_path: &str,
    low_path: &str,
    cage_path: &str,
    out_dir: &str,
    resolution: i64,
) -> Result<String, String> {
    let abs_low = jsonfmt::dumps(&serde_json::json!(py::abspath(low_path)));
    let abs_high = jsonfmt::dumps(&serde_json::json!(py::abspath(high_path)));
    let abs_cage = jsonfmt::dumps(&serde_json::json!(py::abspath(cage_path)));
    let abs_out_dir = jsonfmt::dumps(&serde_json::json!(py::abspath(out_dir)));
    let stem = Path::new(low_path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let low_basename = jsonfmt::dumps(&serde_json::json!(stem));

    let recipe_code = format!(
        r#"# Autonomous 3D Studio - Headless Blender Bake Script
import bpy
import os

# Clean default scene
bpy.ops.wm.read_factory_settings(use_empty=True)

# Import Low Poly
bpy.ops.wm.obj_import(filepath={abs_low})
low_obj = bpy.context.selected_objects[0]
low_obj.name = "LowPoly_Target"

# Import High Poly
bpy.ops.wm.obj_import(filepath={abs_high})
high_obj = bpy.context.selected_objects[0]
high_obj.name = "HighPoly_Source"

# Import Cage
bpy.ops.wm.obj_import(filepath={abs_cage})
cage_obj = bpy.context.selected_objects[0]
cage_obj.name = "Bake_Cage"

# Configure Cycles Baking
bpy.context.scene.render.engine = 'CYCLES'
bpy.context.scene.cycles.bake_type = 'NORMAL'
bpy.context.scene.render.bake.use_selected_to_active = True
bpy.context.scene.render.bake.use_cage = True
bpy.context.scene.render.bake.cage_object = cage_obj
bpy.context.scene.render.bake.normal_space = 'TANGENT'

# Setup Image Texture Node for 16-bit Normal Bake
mat = bpy.data.materials.new(name="BakeMaterial")
mat.use_nodes = True
low_obj.data.materials.append(mat)
nodes = mat.node_tree.nodes

img = bpy.data.images.new(
    name="Baked_Normal_16bit",
    width={resolution},
    height={resolution},
    float_buffer=True # 16/32-bit float to eliminate stair-step banding
)
img_node = nodes.new('ShaderNodeTexImage')
img_node.image = img
nodes.active = img_node

# Select High Poly then Low Poly (Active)
bpy.ops.object.select_all(action='DESELECT')
high_obj.select_set(True)
low_obj.select_set(True)
bpy.context.view_layer.objects.active = low_obj

# Execute Normal Bake
print("[Bake Engine] Baking 16-bit Tangent Space Normal Map...")
bpy.ops.object.bake(type='NORMAL')

# Save Output
out_normal_path = os.path.join({abs_out_dir}, f"T_{{{low_basename}}}_N.png")
img.filepath_raw = out_normal_path
img.file_format = 'PNG'
img.save()
print(f"[Bake Engine] Normal Map saved to: {{out_normal_path}}")
"#,
        abs_low = abs_low,
        abs_high = abs_high,
        abs_cage = abs_cage,
        resolution = resolution,
        abs_out_dir = abs_out_dir,
        low_basename = low_basename
    );

    let recipe_file = Path::new(out_dir).join("run_blender_bake.py");
    fs::write(&recipe_file, recipe_code).map_err(|e| e.to_string())?;
    Ok(recipe_file.to_string_lossy().into_owned())
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio bake",
        argv,
        &[
            ArgSpec::value("high", Some('H'), "high").req(),
            ArgSpec::value("low", Some('L'), "low").req(),
            ArgSpec::value("out", Some('o'), "out").def("./bakes"),
            ArgSpec::float("push", Some('p'), "push").def("0.008"),
            ArgSpec::int("res", Some('r'), "res")
                .def("4096")
                .choices(&["1024", "2048", "4096", "8192"]),
        ],
    );

    let high = args.get("high").unwrap();
    let low = args.get("low").unwrap();
    let out = args.get_or("out", "./bakes");
    let push = args.float("push", 0.008);
    let res = args.int("res", 4096);

    let _ = fs::create_dir_all(&out);
    let stem = Path::new(low)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let cage_path = Path::new(&out).join(format!("Cage_{}.obj", stem));
    let cage_str = cage_path.to_string_lossy().into_owned();

    println!(
        "\n[High-to-Low Baker] Computing area-weighted vertex cage envelope (push={}m)...",
        py::py_num(push)
    );
    if let Err(e) = generate_cage_mesh(low, &cage_str, push) {
        eprintln!("Error: {}", e);
        return 1;
    }
    println!(" -> Cage mesh generated: {}", cage_str);

    match generate_blender_bake_recipe(high, low, &cage_str, &out, res) {
        Ok(bake_script) => {
            println!(
                " -> 16-Bit Float Bake Automation Recipe written: {}",
                bake_script
            );
            println!(
                " -> Ready for headless execution: `blender -b -P {}`\n",
                bake_script
            );
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
