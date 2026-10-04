//! Port of unreal_engine_bridge.py — thin-split: the Rust side owns arg
//! parsing, paths, logging and writes the UE5 Python automation script; the
//! emitted script text is the `import unreal`-only part that stays inside
//! the Unreal Editor embedded VM.

use std::fs;

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

/// Emits the minimal `import unreal`-only automation script. The returned
/// script contains the entire embedded-VM payload — nothing else stays in
/// Python.
pub fn generate_ue5_import_script(
    fbx_path: &str,
    out_script_path: &str,
    dest_path: &str,
    enable_nanite: bool,
    lod_group: &str,
) -> String {
    let abs_fbx = jsonfmt::dumps(&serde_json::json!(py::abspath(fbx_path)));
    let json_dest = jsonfmt::dumps(&serde_json::json!(dest_path));
    let json_nanite = if enable_nanite { "True" } else { "False" };
    let json_lod_group = jsonfmt::dumps(&serde_json::json!(lod_group));

    let code = format!(
        r#"# Unreal Engine 5 Headless / Remote Asset Importer & Nanite Setup
import unreal

def import_and_configure_asset(fbx_path, destination_path="/Game/Assets", enable_nanite=True, lod_group="Hero"):
    print(f"[UE5 Automation] Importing asset: {{fbx_path}} to {{destination_path}}")

    # 1. Setup Asset Import Task
    task = unreal.AssetImportTask()
    task.filename = fbx_path
    task.destination_path = destination_path
    task.destination_name = ""
    task.replace_existing = True
    task.automated = True
    task.save = True

    # 2. Setup FBX Import Options
    options = unreal.FbxImportUI()
    options.import_mesh = True
    options.import_textures = True
    options.import_materials = True
    options.import_as_skeletal = False
    options.static_mesh_import_data.combine_meshes = True
    options.static_mesh_import_data.auto_generate_collision = True # Uses UCX_ if present
    options.static_mesh_import_data.generate_lightmap_u_vs = False

    # Enable Nanite if requested
    if enable_nanite:
        options.static_mesh_import_data.nanite_settings.enable_nanite = True

    task.options = options

    # 3. Execute Import
    unreal.AssetToolsHelpers.get_asset_tools().import_asset_tasks([task])

    # 4. Configure Static Mesh Settings
    imported_assets = task.imported_object_paths
    for asset_path in imported_assets:
        mesh = unreal.EditorAssetLibrary.load_asset(asset_path)
        if isinstance(mesh, unreal.StaticMesh):
            print(f"[UE5 Automation] Configuring Nanite & LODs on: {{asset_path}}")
            mesh.set_editor_property("lod_group", lod_group)
            if enable_nanite:
                nanite_settings = mesh.get_editor_property("nanite_settings")
                nanite_settings.set_editor_property("enable_nanite", True)
                mesh.set_editor_property("nanite_settings", nanite_settings)

            unreal.EditorAssetLibrary.save_asset(asset_path)

    print("[UE5 Automation] Asset import & Nanite setup completed successfully.")

if __name__ == "__main__":
    import_and_configure_asset(
        fbx_path={abs_fbx},
        destination_path={json_dest},
        enable_nanite={json_nanite},
        lod_group={json_lod_group}
    )
"#,
        abs_fbx = abs_fbx,
        json_dest = json_dest,
        json_nanite = json_nanite,
        json_lod_group = json_lod_group
    );
    let _ = fs::write(out_script_path, code);
    out_script_path.to_string()
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio unreal-bridge",
        argv,
        &[
            ArgSpec::value("asset", Some('a'), "asset").req(),
            ArgSpec::value("dest", Some('d'), "dest").def("/Game/Assets/Props"),
            // argparse `store_true` with default=True
            ArgSpec::flag("nanite", None, "nanite"),
            ArgSpec::value("lod_group", None, "lod-group").def("Hero"),
            ArgSpec::value("out", Some('o'), "out").def("./import_to_unreal.py"),
        ],
    );

    let asset = args.get("asset").unwrap();
    let dest = args.get_or("dest", "/Game/Assets/Props");
    let nanite = true; // Python: action="store_true", default=True → always True
    let lod_group = args.get_or("lod_group", "Hero");
    let out = args.get_or("out", "./import_to_unreal.py");

    let script_path = generate_ue5_import_script(asset, &out, &dest, nanite, &lod_group);

    println!("\n[Unreal Engine 5 Bridge] Generated UE5 Python Automation Script:");
    println!(" -> Script: {}", script_path);
    println!(" -> Target Destination: {}", dest);
    println!(" -> Nanite Enabled: {}", nanite);
    println!(
        " -> Run via UE5 CLI: `UnrealEditor-Cmd.exe <Project.uproject> -ExecutePythonScript={}`\n",
        py::abspath(&script_path)
    );
    0
}
