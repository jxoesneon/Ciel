---
name: godot-mcp
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Godot 4 MCP editor bridging, visual screenshot verification, live GDScript execution, and runtime error telemetry. Bridges LLM reasoning with live editor execution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems", "godot", "mcp", "visual-verification", "debugger"]
triggers:
  - pattern: "(godot[- ]?mcp|godot[- ]screenshot|visual[- ]verification|godot[- ]editor[- ]bridge|mcp[- ]godot)"
    confidence: 0.9
  - pattern: "(take_screenshot|execute_gdscript|get_debugger_errors|focus_editor_viewport)"
    confidence: 0.95
source: { tier: 1, origin: self-synthesized }
side_effects: ["shell", "network"]
dependencies:
  skills: ["godot-engine"]
  mcp: ["godot-mcp"]
  system: []
---

# CIEL ADAPTATION: Godot MCP & Visual Verification

Bridges LLM reasoning with live Godot 4 editor execution via the `godot-mcp` MCP server. Provides real-time introspection, visual screenshot verification, live GDScript evaluation, and runtime error telemetry. Ciel's runtime safety gates handle execution gating; this skill declares shell/network side effects from the TCP bridge (port 6505) and GDScript execution.

## Tool Domains

- **Visual Verification**: `take_screenshot`, `focus_editor_viewport_3d` — high-res capture of 3D/2D viewports, running scenes, editor windows with artifact auto-save.
- **Live Editor Bridge**: `get_editor_selection`, `set_editor_selection`, `get_active_script_editor`, `execute_gdscript` — introspection, node selection, camera framing, live GDScript eval with UndoRedo.
- **Debugger Telemetry**: `get_debugger_errors`, `clear_debugger_errors`, `get_debugger_error_counts` — runtime error detection, warning audits, break session tracking.
- **Scene/Node Authoring**: `add_node`, `modify_node_properties`, `delete_node`, `reparent_node`, `instantiate_scene` — programmatic scene tree construction.
- **Shaders & Audio**: `create_shader_material`, `set_shader_parameter`, `configure_audio_bus`, `create_audio_stream_player` — material overrides, uniform binding, audio bus routing.

## Screenshot Target Modes

- `viewport_3d`: Active 3D editor viewport (frame target via `focus_editor_viewport_3d` first).
- `viewport_2d`: 2D scene canvas (CanvasLayer UI, Control hierarchy, HUDs).
- `main_screen` / `editor`: Full editor interface (docks, scene tree, inspector).
- `scene` + `scene_path`: Headless/offscreen `SubViewport` render of any `.tscn` — no editor focus required.
- `output_path`: Direct disk write (e.g. `<artifactDir>/screenshots/view.png`); returns markdown image link `![Label](file://...)`.
- Options: `format` (png/jpg/webp), `max_width`/`max_height` (Lanczos downscale), `quality` (0.0–1.0).

## Live Script Execution (`execute_gdscript`)

- Instantiates a clean `@tool` `GDScript` object via `script.new()`.
- Runs synchronously on the Godot editor thread with full access to `EditorInterface`, `ClassDB`, `ProjectSettings`, `RenderingServer`.
- Use for: querying live node properties, inspecting singleton states, one-shot verification routines.

## Debugger Error Auditing

- Retrieve active errors: `get_debugger_errors` with `severity` and `exclude_session_events`.
- Clear before test run: `clear_debugger_errors`.
- **Always verify zero runtime errors before completing any task.**

## Best Practices

- **Visual-first validation**: On shader/material/camera/UI updates, capture a screenshot and embed in the walkthrough artifact.
- **Process lifecycle**: Preserve the Godot Editor process (`--editor`) and MCP server. Kill test/runtime instances cleanly via `godot_safe_run.sh` or `quit()`.
- **Dual bridge awareness**: Routes to Live In-Editor TCP Bridge (port 6505) when editor active; falls back to headless CLI when closed.

## Anti-Patterns

- **Log-only verification**: Relying on stdout for graphical/shader/UI features. Always capture a screenshot.
- **Killing the editor process**: Terminating `--editor` drops the MCP bridge. Kill only runtime/test instances.
- **Trusting client-side GDScript eval**: `execute_gdscript` runs with editor privileges — validate inputs, avoid destructive calls without UndoRedo.
- **Ignoring error counts**: Completing tasks without `get_debugger_errors` audit. Silent runtime errors compound.
