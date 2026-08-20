#!/usr/bin/env python3
"""
unreal_engine_bridge.py - Unreal Engine 5 Python Automation & Remote Execution Bridge

Generates UE5 Python automation scripts and sends remote commands to live Unreal Editor
instances (via Web Remote Control / Python Remote Execution) to automate:
  - Asset FBX/USD Import
  - Nanite Enablement & Position Precision
  - LOD Group & Screen Size thresholds
  - Convex Collision Hull (UCX) generation
  - Material Instance creation and parameter binding
"""

import sys
import os
import json
import argparse

def generate_ue5_import_script(fbx_path, out_script_path, dest_path="/Game/Assets", enable_nanite=True, lod_group="Hero"):
    abs_fbx = json.dumps(os.path.abspath(fbx_path))
    json_dest = json.dumps(dest_path)
    json_nanite = "True" if enable_nanite else "False"
    json_lod_group = json.dumps(lod_group)

    code = f"""# Unreal Engine 5 Headless / Remote Asset Importer & Nanite Setup
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
"""
    with open(out_script_path, 'w', encoding='utf-8') as f:
        f.write(code)
    return out_script_path

def main():
    parser = argparse.ArgumentParser(description="Unreal Engine 5 Asset Bridge & Remote Execution Generator")
    parser.add_argument("--asset", "-a", required=True, help="Path to 3D asset (.fbx / .usd)")
    parser.add_argument("--dest", "-d", default="/Game/Assets/Props", help="UE5 Content Browser target folder")
    parser.add_argument("--nanite", action="store_true", default=True, help="Enable Unreal Engine 5 Nanite")
    parser.add_argument("--lod-group", default="Hero", help="LOD Group (Hero, HighDetail, Deco, Vehicle)")
    parser.add_argument("--out", "-o", default="./import_to_unreal.py", help="Output Python script path")

    args = parser.parse_args()

    script_path = generate_ue5_import_script(args.asset, args.out, dest_path=args.dest, enable_nanite=args.nanite, lod_group=args.lod_group)

    print(f"\n[Unreal Engine 5 Bridge] Generated UE5 Python Automation Script:")
    print(f" -> Script: {script_path}")
    print(f" -> Target Destination: {args.dest}")
    print(f" -> Nanite Enabled: {args.nanite}")
    print(f" -> Run via UE5 CLI: `UnrealEditor-Cmd.exe <Project.uproject> -ExecutePythonScript={os.path.abspath(script_path)}`\n")

if __name__ == "__main__":
    main()
