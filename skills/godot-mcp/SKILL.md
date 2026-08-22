---
name: godot-mcp
version: 2.0.0
description: Professional Godot 4 MCP integration for live editor introspection, visual viewport screenshot verification, and runtime debugging
triggers: ["godot-mcp", "godot_mcp", "godot-screenshot", "visual-verification", "godot-editor-bridge", "mcp-godot"]
tags: ["godot", "mcp", "screenshots", "visual-verification", "debugger", "gdscript", "in-editor"]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
source:
  tier: 1
  origin: self-synthesized
dependencies:
  skills: ["godot"]
  mcp: ["godot-mcp"]
  system: []
---

# Godot MCP Pro & Visual Verification Intelligence

The definitive guide and operating manual for using **Godot MCP** (`godot-mcp`) in AI-assisted game development workflows. It bridges LLM reasoning with live Godot 4 editor execution, real-time error telemetry, and comprehensive visual screenshot verification.

## Core Capabilities & Tool Index

Godot MCP exposes 70+ specialized tools across 5 major functional domains:

| Domain | Key Tools | Primary Purpose |
| :--- | :--- | :--- |
| **Visual Verification** | `take_screenshot`, `focus_editor_viewport_3d` | High-res Base64/PNG capture of 3D/2D viewports, running scenes, and editor windows with artifact auto-saving. |
| **Live Editor Bridge** | `get_editor_selection`, `set_editor_selection`, `get_active_script_editor`, `execute_gdscript` | Real-time introspection, node selection, camera framing, and live GDScript evaluation with UndoRedo support. |
| **Debugger Telemetry** | `get_debugger_errors`, `clear_debugger_errors`, `get_debugger_error_counts` | Zero-latency runtime error detection, warning audits, and break session tracking. |
| **Scene & Node Authoring** | `add_node`, `modify_node_properties`, `delete_node`, `reparent_node`, `instantiate_scene` | Programmatic scene tree construction and property modification. |
| **Shaders & Audio** | `create_shader_material`, `set_shader_parameter`, `configure_audio_bus`, `create_audio_stream_player` | Material override attachments, uniform binding, and dynamic audio bus routing. |

---

## 1. Visual Verification & Screenshot Capture (`take_screenshot`)

Godot MCP provides multi-target viewport capture allowing agents to visually inspect scene layout, shader effects, lighting cascades, HUD elements, and planet atmospheres.

### Target Modes & Usage

```json
{
  "target": "auto",           // "viewport_3d", "viewport_2d", "main_screen", "scene", "auto"
  "viewport_index": 0,        // 0-3 for 3D multi-viewports
  "scene_path": "res://...",  // Optional: render any .tscn headlessly/offscreen
  "output_path": "/path/...", // Direct file path on disk (e.g. into artifact dir)
  "format": "png",            // "png", "jpg", "webp"
  "max_width": 1920,          // High quality Lanczos downscaling
  "max_height": 1080,
  "quality": 0.85             // Compression quality (0.0 - 1.0)
}
```

### Modes Breakdown

1. **`target: "viewport_3d"`**: Captures the active 3D editor viewport. Best used after calling `focus_editor_viewport_3d` to frame a specific node.
2. **`target: "viewport_2d"`**: Captures the 2D scene canvas (CanvasLayer UI, Control hierarchy, HUDs).
3. **`target: "main_screen"` / `"editor"`**: Captures the complete Godot editor interface including Docks, Scene Tree, and Inspector.
4. **`target: "scene"`**: Renders a standalone `.tscn` offscreen in a `SubViewport`, settles rendering passes, and captures the frame without requiring editor focus.
5. **`output_path`**: When specified, writes the image directly to disk (e.g. `<artifactDir>/screenshots/view.png`) and returns a ready-to-use markdown image link:
   ```markdown
   ![Godot Viewport Screenshot](file:///path/to/screenshot.png)
   ```

---

## 2. Dynamic Live Script Execution (`execute_gdscript`)

Use `execute_gdscript` to query live node properties, inspect singleton states, or run one-shot verification routines inside the editor:

```gdscript
func eval():
    var root = EditorInterface.get_edited_scene_root()
    var camera = root.get_node_or_null("Camera3D")
    return {
        "camera_pos": camera.global_position if camera else Vector3.ZERO,
        "nodes_in_scene": root.get_child_count()
    }
```

- Instantiates a clean `@tool` `GDScript` object via `script.new()`.
- Runs synchronously within the Godot editor thread with full access to `EditorInterface`, `ClassDB`, `ProjectSettings`, and `RenderingServer`.

---

## 3. Real-Time Debugger Error Auditing (`get_debugger_errors`)

Always verify zero runtime errors before completing any task:

```json
// Retrieve active errors
"godot-mcp:get_debugger_errors": {
  "severity": "error",
  "exclude_session_events": true
}

// Clear error log before a test run
"godot-mcp:clear_debugger_errors": {}
```

---

## 4. Best Practices for Pair-Programming & Verification

1. **Visual-First Validation:** Whenever shaders, materials, camera clipping, planet LODs, or UI layouts are updated, capture a screenshot and embed it into the walkthrough artifact.
2. **Process Lifecycle Safety:**
   - Preserve the Godot Editor process (`--editor`) and MCP server.
   - Kill test/runtime instances cleanly using `godot_safe_run.sh` or automated test scripts with `quit()`.
3. **Dual Bridge Awareness:** Godot MCP automatically routes requests to the Live In-Editor TCP Bridge (port 6505) when the editor is active, and seamlessly falls back to headless CLI execution when the editor is closed.
