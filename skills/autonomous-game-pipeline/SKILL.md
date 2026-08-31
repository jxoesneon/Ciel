---
name: autonomous-game-pipeline
version: 1.0.0
format: skill/1.0
description: CIEL's framework for autonomous game generation from natural language descriptions across Godot 4, Bevy, and Babylon.js. Builds complete games with assets and proof video.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai", "gamedev", "godot", "bevy", "babylon", "autonomous", "procedural", "asset-generation"]
triggers:
  - pattern: "(make|build|generate) a game|game from description|autonomous game (dev|pipeline)|godot|bevy|babylon game"
    confidence: 0.9
  - pattern: "(godogen|game pipeline|procedural game|asset generation pipeline)"
    confidence: 0.85
source: { tier: 3, origin: "https://github.com/htdt/godogen" }
side_effects: ["shell", "network", "external_api", "fs"]
dependencies:
  skills: ["godot-engine"]
  mcp: []
  system: ["Godot 4 (.NET)", "Python 3", "ffmpeg", "imagemagick", "xvfb", "vulkan-tools"]
---

# CIEL ADAPTATION: Autonomous Game Generator

Generates complete games from natural language descriptions across Godot 4 (C#/.NET), Bevy (Rust), and Babylon.js (TypeScript). The agent builds the project, generates assets via paid APIs, runs the engine, and proves the result with a recorded gameplay video.

**Risk note**: Ciel's risk classifier gates autonomous execution as high → Council. Ciel's SANDBOX.md enforces acquisition-time isolation for Tier 3 sources. This skill declares side effects; Ciel's runtime handles execution gating — sandbox requirements are NOT baked into this body.

## Pipeline

1. **Resume check**: If `PLAN.md` exists, read `PLAN.md` + `STRUCTURE.md` + `MEMORY.md` → skip to task execution.
2. **Fresh pipeline**: Generate visual target → `reference.png` + `ASSETS.md` (art direction only).
3. **Risk analysis**: Define verification criteria → `PLAN.md`.
4. **Architecture**: Design → `STRUCTURE.md` + `project.godot` + stubs.
5. **Asset generation** (if budget provided, no asset tables in `ASSETS.md`): Plan + generate → updated `PLAN.md`.
6. **Task loop**: Pick next task → implement (code/assets/scenes) → verify (build/run/screenshot) → update `PLAN.md` + `MEMORY.md`.
7. **Proof**: Capture 15–20s gameplay video → visual QA (multimodal review) → deliver.

## Engine Guides

- **Godot 4** (`engines/godot.md`): C#/.NET, Jolt Physics, build-time scene generation.
- **Bevy** (`engines/bevy.md`): Rust, ECS, code-first scenes.
- **Babylon.js** (`engines/babylon.md`): TypeScript/Vite, browser-served.

Read the appropriate guide before writing any engine-specific code.

## Asset Generation (Paid APIs — confirm spend before first run)

- **Gemini**: Precise references, characters — 5–15¢/image.
- **xAI Grok**: Textures, simple objects — 2¢/image.
- **Tripo3D**: Image-to-3D GLB — 30–60¢/model; rigged biped +25¢; animation clips 10¢/clip.
- **Grok Video**: Animated sprites — 5¢/second.

## Key Principles

- **Proof over claims**: Judge progress from the running game, never a clean build.
- **Visual QA closes the loop**: Capture screenshots; multimodal review catches defects.
- **Scenes generated at build time**: C# SceneTree scripts emit `.tscn`, not hand-authored.
- **Silent-failure awareness**: Godot serialization drops nodes without errors; validate packs.
- **Asset manifest**: Track every asset with an in-game Size column in `README.md`.

## Godot-Specific Gotchas

- `ArrayMesh.GenerateNormals()` required for procedural meshes to receive shadows.
- `MultiMeshInstance3D` + GLB loses mesh on pack/save; use individual instances.
- Raycasts don't reliably hit `ConcavePolygonShape3D`; use shape queries or analytical height.
- `SetScript()` disposes the C# wrapper — set scripts last, after hierarchy built.
- Owner chain: every node must have Owner set to scene root or it won't serialize.
- Validate the pack: count nodes before packing, `Instantiate()` after, compare counts.
- Never `CreateTrimeshShape()` on imported GLB meshes (drops to <1 FPS); use primitives.
- Frame-rate-independent damping: `speed *= Mathf.Exp(-rate * delta)`, not `speed *= (1 - drag)`.

## Anti-Patterns

- **Claiming done from a clean build**: Build success ≠ working game. Capture proof video.
- **Hand-authoring scenes**: Use build-time C# SceneTree scripts; manual `.tscn` doesn't scale.
- **Skipping pack validation**: Silent node drops on serialization. Count before/after.
- **Trimesh on imported GLB**: `<1 FPS`. Always primitive shapes for collision.
- **Linear damping**: `speed *= (1 - drag)` is frame-dependent. Use exponential form.
