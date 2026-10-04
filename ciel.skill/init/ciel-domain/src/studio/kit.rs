//! Port of procedural_kit_generator.py — chamfered modular wall panel
//! generator with UVs and per-face normals.

use std::fs;

use crate::common::cli::{self, ArgSpec};
use crate::common::py;

fn generate_modular_wall_panel(
    width: f64,
    height: f64,
    depth: f64,
    bevel_width: f64,
    out_path: &str,
) -> Result<String, String> {
    let (hw, hh, hd, bw) = (width / 2.0, height / 2.0, depth / 2.0, bevel_width);

    let verts: [[f64; 3]; 12] = [
        // Front face inner inset
        [-hw + bw, -hh + bw, hd],
        [hw - bw, -hh + bw, hd],
        [hw - bw, hh - bw, hd],
        [-hw + bw, hh - bw, hd],
        // Front face outer bevel boundary
        [-hw, -hh, hd - bw],
        [hw, -hh, hd - bw],
        [hw, hh, hd - bw],
        [-hw, hh, hd - bw],
        // Back face outer boundary
        [-hw, -hh, -hd],
        [hw, -hh, -hd],
        [hw, hh, -hd],
        [-hw, hh, -hd],
    ];

    let uvs: [[f64; 2]; 12] = [
        [0.1, 0.1],
        [0.9, 0.1],
        [0.9, 0.9],
        [0.1, 0.9], // Inset
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0], // Outer bevel
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0], // Back
    ];

    let faces: [[usize; 4]; 10] = [
        [1, 2, 3, 4],  // Front center inset face
        [5, 6, 2, 1],  // Bottom bevel
        [6, 7, 3, 2],  // Right bevel
        [7, 8, 4, 3],  // Top bevel
        [8, 5, 1, 4],  // Left bevel
        [9, 10, 6, 5], // Side walls
        [10, 11, 7, 6],
        [11, 12, 8, 7],
        [12, 9, 5, 8],
        [12, 11, 10, 9], // Back face
    ];

    let mut f_normals = Vec::with_capacity(faces.len());
    for face in &faces {
        let v0 = verts[face[0] - 1];
        let v1 = verts[face[1] - 1];
        let v2 = verts[face[2] - 1];
        let (ax, ay, az) = (v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]);
        let (bx, by, bz) = (v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);
        let nx = ay * bz - az * by;
        let ny = az * bx - ax * bz;
        let nz = ax * by - ay * bx;
        let length = (nx * nx + ny * ny + nz * nz).sqrt();
        if length > 1e-7 {
            f_normals.push([nx / length, ny / length, nz / length]);
        } else {
            f_normals.push([0.0, 0.0, 1.0]);
        }
    }

    let mut out = String::new();
    out.push_str("# AAA Studio Procedural Modular Wall Panel\n");
    out.push_str(&format!(
        "# Dimensions: {}m x {}m x {}m (Bevel: {}m)\n",
        py::py_num(width),
        py::py_num(height),
        py::py_num(depth),
        py::py_num(bevel_width)
    ));
    for v in &verts {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for vt in &uvs {
        out.push_str(&format!("vt {:.6} {:.6}\n", vt[0], vt[1]));
    }
    for fn_ in &f_normals {
        out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", fn_[0], fn_[1], fn_[2]));
    }
    for (f_idx, face) in faces.iter().enumerate() {
        let norm_idx = f_idx + 1;
        let f_str = face
            .iter()
            .map(|idx| format!("{}/{}/{}", idx, idx, norm_idx))
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!("f {}\n", f_str));
    }

    fs::write(out_path, out).map_err(|e| e.to_string())?;
    Ok(out_path.to_string())
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio kit",
        argv,
        &[
            ArgSpec::value("type", Some('t'), "type")
                .def("modular_panel")
                .choices(&["modular_panel", "sci_fi_crate", "conduit_beam"]),
            ArgSpec::float("width", Some('W'), "width").def("2.0"),
            ArgSpec::float("height", Some('H'), "height").def("3.0"),
            ArgSpec::float("depth", Some('D'), "depth").def("0.2"),
            ArgSpec::float("bevel", Some('b'), "bevel").def("0.025"),
            ArgSpec::value("out", Some('o'), "out").def("./modular_panel.obj"),
        ],
    );

    let typ = args.get_or("type", "modular_panel");
    let width = args.float("width", 2.0);
    let height = args.float("height", 3.0);
    let depth = args.float("depth", 0.2);
    let bevel = args.float("bevel", 0.025);
    let out = args.get_or("out", "./modular_panel.obj");

    match generate_modular_wall_panel(width, height, depth, bevel, &out) {
        Ok(out_file) => {
            println!("\n[Procedural Kit Generator] Created procedural {}:", typ);
            println!(" -> Output Mesh: {}", out_file);
            println!(
                " -> Modular Grid Fit: {}m x {}m x {}m (Chamfer: {}m)\n",
                py::py_num(width),
                py::py_num(height),
                py::py_num(depth),
                py::py_num(bevel)
            );
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
