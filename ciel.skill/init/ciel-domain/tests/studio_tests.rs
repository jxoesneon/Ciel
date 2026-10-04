//! Unit tests for the 3D-studio ports: pure mesh math (geometry), the OBJ
//! parser, QA audit, FACS symmetry mapping, and generated-script content
//! (no Blender/Unreal binaries required — we assert on emitted text).

use ciel_domain::studio::{facs, geometry, pipeline, qa, unreal};
use serde_json::Value;
use std::io::Write;

fn tmp_path(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("ciel-domain-studio-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name).to_string_lossy().into_owned()
}

// ---------- geometry ----------

#[test]
fn dsu_union_find() {
    let mut dsu = geometry::DisjointSetUnion::new(4);
    assert_ne!(dsu.find(0), dsu.find(1));
    assert!(dsu.union(0, 1)); // new merge → true
    assert!(!dsu.union(0, 1)); // already same root → false
    assert_eq!(dsu.find(0), dsu.find(1));
    assert_ne!(dsu.find(0), dsu.find(2));
}

#[test]
fn hash_grid_deterministic() {
    let h1 = geometry::hash_grid_3d(1, 2, 3);
    let h2 = geometry::hash_grid_3d(1, 2, 3);
    assert_eq!(h1, h2);
    assert_ne!(
        geometry::hash_grid_3d(1, 2, 3),
        geometry::hash_grid_3d(3, 2, 1)
    );
    // Negative coords must not panic (Python wraps into 31-bit space).
    let _ = geometry::hash_grid_3d(-5, -7, -9);
}

#[test]
fn triangle_and_polygon_area() {
    let a = geometry::compute_triangle_area_3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert!((a - 0.5).abs() < 1e-9);
    // Unit quad (2 triangles).
    let q = geometry::compute_polygon_area_3d(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]);
    assert!((q - 1.0).abs() < 1e-9);
}

#[test]
fn bounding_box() {
    let verts: Vec<geometry::Vec3> = vec![[0.0, 0.0, 0.0], [2.0, 3.0, -1.0], [-1.0, 1.0, 4.0]];
    let (min_pt, max_pt, dims) = geometry::compute_single_pass_bounding_box(&verts);
    assert_eq!(min_pt, [-1.0, 0.0, -1.0]);
    assert_eq!(max_pt, [2.0, 3.0, 4.0]);
    assert_eq!(dims, [3.0, 3.0, 5.0]);
}

#[test]
fn grid_coord_trunc_semantics() {
    // Python uses int() — truncation toward zero, not math.floor.
    assert_eq!(geometry::grid_coord(0.0, 0.0, 1.0), 0);
    assert_eq!(geometry::grid_coord(0.99, 0.0, 1.0), 0);
    assert_eq!(geometry::grid_coord(1.0, 0.0, 1.0), 1);
    assert_eq!(geometry::grid_coord(-0.01, 0.0, 1.0), 0); // int(-0.01) == 0
    assert_eq!(geometry::grid_coord(-1.5, 0.0, 1.0), -1); // int(-1.5) == -1
}

#[test]
fn parse_obj_basic() {
    let path = tmp_path("tri.obj");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "v 0 0 0").unwrap();
    writeln!(f, "v 1 0 0").unwrap();
    writeln!(f, "v 0 1 0").unwrap();
    writeln!(f, "vn 0 0 1").unwrap();
    writeln!(f, "vt 0 0").unwrap();
    writeln!(f, "f 1/1/1 2/1/1 3/1/1").unwrap();
    drop(f);
    let mesh = geometry::parse_obj_buffered(&path, 1_000_000).unwrap();
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.faces.len(), 1);
    assert_eq!(mesh.faces[0], vec![0, 1, 2]);
    assert_eq!(mesh.texcoords.len(), 1);
    assert_eq!(mesh.normals.len(), 1);
}

#[test]
fn parse_obj_missing() {
    assert!(geometry::parse_obj_buffered("/nonexistent/x.obj", 10).is_err());
}

// ---------- qa ----------

fn cube_mesh() -> geometry::MeshData {
    // A closed unit cube: 8 verts, 12 triangles (two per face).
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];
    let faces = vec![
        vec![0, 1, 2],
        vec![0, 2, 3], // bottom
        vec![4, 6, 5],
        vec![4, 7, 6], // top
        vec![0, 4, 5],
        vec![0, 5, 1], // front
        vec![2, 6, 7],
        vec![2, 7, 3], // back
        vec![0, 3, 7],
        vec![0, 7, 4], // left
        vec![1, 5, 6],
        vec![1, 6, 2], // right
    ];
    geometry::MeshData {
        vertices,
        faces,
        ..Default::default()
    }
}

#[test]
fn qa_audit_closed_cube() {
    let report = qa::audit_geometry(&cube_mesh(), "game", true);
    assert_eq!(
        report["status"],
        Value::String("PASS".into()),
        "report: {}",
        report
    );
    assert_eq!(report["summary"]["total_vertices"], serde_json::json!(8));
    assert_eq!(report["summary"]["total_faces"], serde_json::json!(12));
    assert_eq!(
        report["defect_counts"]["boundary_edges"],
        serde_json::json!(0)
    );
}

#[test]
fn qa_audit_empty_mesh() {
    let mesh = geometry::MeshData::default();
    let report = qa::audit_geometry(&mesh, "game", true);
    assert_eq!(report["status"], Value::String("FAIL".into()));
}

#[test]
fn qa_audit_open_mesh_warns_not_fails() {
    // Single triangle — every edge is a boundary edge: watertight is a
    // WARNING, not a hard fail (Python parity).
    let mesh = geometry::MeshData {
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        faces: vec![vec![0, 1, 2]],
        ..Default::default()
    };
    let report = qa::audit_geometry(&mesh, "game", true);
    assert_eq!(report["status"], Value::String("PASS".into()));
    assert_eq!(
        report["defect_counts"]["boundary_edges"],
        serde_json::json!(3)
    );
    let flags: Vec<&str> = report["qa_flags"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(flags.iter().any(|f| f.contains("open boundary edges")));
}

#[test]
fn qa_audit_non_manifold_fails() {
    // Three triangles sharing edge (0,1) → non-manifold → FAIL.
    let mesh = geometry::MeshData {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 1, 3], vec![0, 2, 1], vec![0, 1, 2]],
        ..Default::default()
    };
    let report = qa::audit_geometry(&mesh, "game", true);
    assert_eq!(
        report["status"],
        Value::String("FAIL".into()),
        "report: {}",
        report
    );
    assert!(
        report["defect_counts"]["non_manifold_edges"]
            .as_i64()
            .unwrap()
            > 0
    );
}

// ---------- facs ----------

#[test]
fn facs_symmetry_map_python_parity() {
    // Verified against Python find_symmetry_vertex_map on the same verts:
    // at tight tolerance the grid yields identity; at tolerance >= mirror
    // distance it finds the pair.
    let verts: Vec<[f64; 3]> = vec![[-1.0, 2.0, 0.0], [1.0, 2.0, 0.0], [0.0, 1.0, 0.0]];
    let tight = facs::find_symmetry_vertex_map(&verts, 0.05);
    assert_eq!(tight[&0], 0);
    assert_eq!(tight[&1], 1);
    assert_eq!(tight[&2], 2);
    let loose = facs::find_symmetry_vertex_map(&verts, 3.0);
    assert_eq!(loose[&0], 1);
    assert_eq!(loose[&1], 0);
    assert_eq!(loose[&2], 2);
}

// ---------- generated scripts ----------

#[test]
fn unreal_script_contains_payload() {
    let out = tmp_path("ue5_import.py");
    let path =
        unreal::generate_ue5_import_script("/tmp/asset.fbx", &out, "/Game/Imported", true, "LOD0");
    let code = std::fs::read_to_string(&path).unwrap();
    assert!(code.contains("import unreal"));
    assert!(code.contains("/Game/Imported"));
    assert!(code.contains("True")); // nanite enabled
    assert!(code.contains("LOD0"));
    assert!(!code.contains("import bpy")); // Unreal-only payload
}

#[test]
fn pipeline_scene_script_contains_payload() {
    let out = tmp_path("scene_build.py");
    pipeline::generate_scene_build_script("/tmp/scene.json", &out);
    let code = std::fs::read_to_string(&out).unwrap();
    assert!(code.contains("import bpy"));
    assert!(code.contains("scene.json"));
    assert!(code.contains("read_factory_settings"));
}
