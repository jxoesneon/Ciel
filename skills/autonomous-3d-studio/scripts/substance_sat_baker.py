#!/usr/bin/env python3
"""
substance_sat_baker.py - Substance Automation Toolkit (SAT) Headless Baker Bridge

Automates Adobe Substance Automation Toolkit (SAT) CLI tools (`sbsbaker`, `sbscooker`, `sbsrender`)
for high-speed multi-threaded GPU baking of:
  - Tangent Space Normal (DirectX / OpenGL)
  - Ambient Occlusion (Ray-traced cosine distribution)
  - Curvature & World Normal
  - Position & Thickness (for Subsurface Scattering)
  - Color ID / Material ID
"""

import sys
import os
import shlex
import json
import shutil
import argparse
import subprocess

def find_sat_baker_binary():
    """Locates Adobe Substance Automation Toolkit sbsbaker executable."""
    bin_path = shutil.which("sbsbaker")
    if bin_path:
        return bin_path

    common_paths = [
        "/opt/Allegorithmic/Substance_Automation_Toolkit/sbsbaker",
        "/usr/local/bin/sbsbaker",
        r"C:\Program Files\Allegorithmic\Substance Automation Toolkit\sbsbaker.exe"
    ]
    for p in common_paths:
        if os.path.exists(p):
            return p
    return None

def build_sat_bake_command(high_mesh, low_mesh, out_dir, map_type="normal", resolution=4096, format_out="png"):
    map_types = {
        "normal": "normal-from-mesh",
        "ao": "ambient-occlusion",
        "curvature": "curvature",
        "world_normal": "world-space-normals",
        "position": "position",
        "thickness": "thickness",
        "color_id": "color-from-mesh"
    }

    sat_baker_type = map_types.get(map_type, "normal-from-mesh")
    cmd = [
        "sbsbaker", sat_baker_type,
        "--high-mesh", os.path.abspath(high_mesh),
        "--low-mesh", os.path.abspath(low_mesh),
        "--output-path", os.path.abspath(out_dir),
        "--output-name", f"T_{os.path.splitext(os.path.basename(low_mesh))[0]}_{map_type.upper()}",
        "--output-format", format_out,
        "--output-size", f"{resolution}x{resolution}",
        "--antialiasing", "4x4",
        "--ray-distance", "0.01",
        "--tangent-space-mode", "mikktspace"
    ]
    return cmd

def generate_sat_batch_script(high_mesh, low_mesh, out_dir, out_script_path, resolution=4096):
    maps = ["normal", "ao", "curvature", "world_normal", "position", "thickness", "color_id"]
    lines = ["#!/bin/bash", "# Adobe Substance Automation Toolkit (SAT) Headless Batch Bake", "set -e", f"mkdir -p {shlex.quote(os.path.abspath(out_dir))}"]

    command_list = []
    for m in maps:
        cmd_vec = build_sat_bake_command(high_mesh, low_mesh, out_dir, map_type=m, resolution=resolution)
        quoted_line = " ".join(shlex.quote(token) for token in cmd_vec)
        lines.append(quoted_line)
        command_list.append(cmd_vec)

    script_content = "\n".join(lines) + "\n"
    with open(out_script_path, 'w', encoding='utf-8') as f:
        f.write(script_content)
    os.chmod(out_script_path, 0o755)
    
    return {
        "status": "SUCCESS",
        "recipe_script": out_script_path,
        "high_mesh": high_mesh,
        "low_mesh": low_mesh,
        "out_dir": out_dir,
        "resolution": resolution,
        "maps_configured": maps,
        "commands": command_list
    }

def main():
    parser = argparse.ArgumentParser(description="Substance Automation Toolkit (SAT) Headless Baker Bridge")
    parser.add_argument("--high", "-H", required=True, help="High-poly source mesh (.obj / .fbx)")
    parser.add_argument("--low", "-L", required=True, help="Low-poly target mesh (.obj / .fbx)")
    parser.add_argument("--outdir", "-o", default="./sat_bakes", help="Output directory for baked PBR maps")
    parser.add_argument("--res", "-r", type=int, default=4096, choices=[1024, 2048, 4096, 8192], help="Texture Map Resolution")
    parser.add_argument("--script-out", default="./run_sat_bakes.sh", help="Path for generated SAT shell recipe")
    parser.add_argument("--json", action="store_true", help="Output structured JSON telemetry")

    args = parser.parse_args()
    os.makedirs(args.outdir, exist_ok=True)

    result = generate_sat_batch_script(args.high, args.low, args.outdir, args.script_out, resolution=args.res)
    
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"\n[Substance SAT Baker Bridge] Generated SAT batch recipe:")
        print(f" -> Shell Script: {result['recipe_script']}")
        print(f" -> Target Resolution: {args.res}x{args.res} (MikkTSpace Normal, AO, Curvature, Position, Thickness, ID)")
        print(f" -> Run via SAT CLI: `./{os.path.basename(result['recipe_script'])}`\n")

if __name__ == "__main__":
    main()
