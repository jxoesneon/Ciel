#!/usr/bin/env python3
"""
===============================================================================
AUTO-WIRE HOOK & SCENE ATTACHER
===============================================================================
Automatically scans a project workspace for scene files (.tscn, index.html)
and outputs a ready-to-inject procedural audio node wiring snippet.
===============================================================================
"""

import sys
import os
import re
import json

def inspect_and_wire(file_path: str) -> dict:
    if not os.path.exists(file_path):
        return {"error": f"File not found: {file_path}"}

    with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    has_buttons = bool(re.search(r'type="(Button|TextureButton|CheckButton)"|<button|class=".*btn.*"', content, re.IGNORECASE))
    has_physics = bool(re.search(r'type="(RigidBody2D|RigidBody3D|CharacterBody2D|CharacterBody3D)"|rigidbody', content, re.IGNORECASE))
    has_particles = bool(re.search(r'type="(GPUParticles2D|GPUParticles3D|CPUParticles2D|CPUParticles3D)"|particle', content, re.IGNORECASE))

    return {
        "target_file": file_path,
        "interactive_buttons_found": has_buttons,
        "physics_bodies_found": has_physics,
        "particle_emitters_found": has_particles,
        "recommended_wiring": {
            "ui_click": "Connect BaseButton.pressed to procedural click generator" if has_buttons else "None",
            "physics_impact": "Connect RigidBody.body_entered to modal impact generator" if has_physics else "None",
            "weather_ambience": "Attach granular particle audio synthesizer" if has_particles else "Standard Drone"
        }
    }

def main():
    if len(sys.argv) > 1:
        result = inspect_and_wire(sys.argv[1])
        print(json.dumps(result, indent=2))
    else:
        print("Usage: python3 auto_wire_hook.py <path_to_scene_file>")

if __name__ == "__main__":
    main()
