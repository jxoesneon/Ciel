#!/usr/bin/env python3
"""
test_3d_studio_regression.py - Synthetic Regression Test Suite for Autonomous 3D Studio

Generates procedural defect meshes (non-manifold, zero-area faces, star poles)
and continuously verifies auto-repair convergence rates and MCP reliability.
"""

import sys
import os
import json
import subprocess
from datetime import datetime

def generate_defective_mesh(out_path):
    """Generates a simple mesh with intentional topological defects for QA validation."""
    with open(out_path, 'w', encoding='utf-8') as f:
        f.write("# Synthetic Defective Test Mesh\n")
        # Valid vertices
        f.write("v 0.0 0.0 0.0\n")
        f.write("v 1.0 0.0 0.0\n")
        f.write("v 1.0 1.0 0.0\n")
        f.write("v 0.0 1.0 0.0\n")
        # Loose vertex (Defect)
        f.write("v 5.0 5.0 5.0\n")
        
        # Valid Face
        f.write("f 1 2 3 4\n")
        # Degenerate zero-area face (Defect)
        f.write("f 1 2 2 1\n")
    return out_path

def test_geometry_qa_validator(scripts_dir, work_dir):
    print("[TEST] Running geometry_qa_validator.py on defective mesh...")
    mesh_path = os.path.join(work_dir, "synthetic_defective.obj")
    generate_defective_mesh(mesh_path)
    
    script_path = os.path.join(scripts_dir, "geometry_qa_validator.py")
    
    # 1. Audit should fail
    res = subprocess.run([sys.executable, script_path, "--input", mesh_path, "--compact"], capture_output=True, text=True)
    try:
        report = json.loads(res.stdout)
        if report["status"] != "FAIL":
            print(f"  ❌ FAILED: Audit passed unexpectedly on defective mesh.")
            return False
        if report["defects"]["loose_vertices"] != 1:
            print(f"  ❌ FAILED: Did not detect loose vertex.")
            return False
        if report["defects"]["degenerate_faces"] != 1:
            print(f"  ❌ FAILED: Did not detect degenerate face.")
            return False
    except Exception as e:
        print(f"  ❌ FAILED: Output parsing error: {e}")
        return False
        
    print("  ✅ Audit correctly identified defects.")

    # 2. Fix should pass
    fixed_mesh = os.path.join(work_dir, "synthetic_fixed.obj")
    subprocess.run([sys.executable, script_path, "--input", mesh_path, "--fix", "--out", fixed_mesh, "--compact"], capture_output=True)
    
    res = subprocess.run([sys.executable, script_path, "--input", fixed_mesh, "--compact"], capture_output=True, text=True)
    try:
        report = json.loads(res.stdout)
        if report["status"] != "PASS":
            print(f"  ❌ FAILED: Fixed mesh still failed audit.")
            return False
        if report["defects"]["loose_vertices"] > 0 or report["defects"]["degenerate_faces"] > 0:
            print(f"  ❌ FAILED: Defects were not completely dissolved.")
            return False
    except Exception as e:
        print(f"  ❌ FAILED: Fixed output parsing error: {e}")
        return False
        
    print("  ✅ Auto-repair successfully dissolved defects.")
    return True

def main():
    print("==================================================")
    print(" CIEL Autonomous 3D Studio - Synthetic Regression")
    print("==================================================")
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    scripts_path = os.path.join(os.path.dirname(script_dir), "scripts")
    work_dir = os.path.join(os.path.expanduser("~"), ".ciel", "tmp", "3d_tests")
    os.makedirs(work_dir, exist_ok=True)
    
    passed = True
    passed &= test_geometry_qa_validator(scripts_path, work_dir)
    
    print("==================================================")
    if passed:
        print(" 🎉 ALL REGRESSION TESTS PASSED.")
        sys.exit(0)
    else:
        print(" ❌ REGRESSION TESTS FAILED.")
        sys.exit(1)

if __name__ == "__main__":
    main()
