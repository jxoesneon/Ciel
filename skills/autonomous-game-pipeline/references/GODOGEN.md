---
name: godogen
version: 1.0.0
description: |
  Autonomous game development pipeline for Godot 4, Bevy, and Babylon.js.
  Describe a game; the agent builds it, generates assets, runs the engine,
  and proves the result — as a live game you watch and steer, or as a
  recorded video when you're not there.
triggers: ["make a game", "build a game", "generate a game", "game from description", "godot game", "bevy game", "babylon game", "game pipeline", "autonomous game dev"]
tags: ["gamedev", "godot", "bevy", "babylon", "procedural", "asset-generation", "csharp", "rust", "typescript", "autonomous", "pipeline"]
runtimes: ["claude_code", "codex", "gemini_cli", "windsurf", "generic"]
license: MIT
source:
  tier: 3
  origin: web-extraction
  url: https://github.com/htdt/godogen
  fetched: 2026-08-27
  hash: sha256:pending
dependencies:
  skills:
    - godot/SKILL.md
  mcp: []
  system:
    - "Godot 4 (.NET build) on PATH for Godot projects"
    - "Python 3 with pip for asset generation tools"
    - "GOOGLE_API_KEY env var for Gemini image generation"
    - "XAI_API_KEY env var for Grok image/video generation"
    - "TRIPO3D_API_KEY env var for Tripo3D 3D model generation"
    - "ffmpeg + imagemagick for video/frame processing"
    - "xvfb + vulkan-tools for headless capture on Linux"
io_contract:
  input: "Natural language game description"
  output: "Complete game project (Godot/Bevy/Babylon) with assets + proof video"
---

# Godogen — Autonomous Game Generator

Generate complete games from natural language descriptions. Supports Godot 4 (C#/.NET), Bevy (Rust), and Babylon.js (TypeScript).

## Pipeline

```
User request
 │
 ├─ Check if PLAN.md exists (resume check)
 │   ├─ If yes: read PLAN.md, STRUCTURE.md, MEMORY.md → skip to task execution
 │   └─ If no: continue with fresh pipeline
 │
 ├─ Generate visual target → reference.png + ASSETS.md (art direction only)
 ├─ Analyze risks + define verification criteria → PLAN.md
 ├─ Design architecture → STRUCTURE.md + project.godot + stubs
 │
 ├─ If budget provided (and no asset tables in ASSETS.md):
 │   ├─ Plan and generate assets → ASSETS.md + updated PLAN.md
 │
 ├─ Task execution loop:
 │   ├─ Pick next task from PLAN.md
 │   ├─ Implement (write code, generate assets, build scenes)
 │   ├─ Verify (build, run, capture screenshot)
 │   └─ Update PLAN.md + MEMORY.md
 │
 ├─ Capture proof video (15-20s of gameplay)
 ├─ Visual QA (multimodal review of screenshots)
 └─ Deliver
```

## Engine Guides

Read the appropriate engine guide before writing code:

| Engine | Guide | Stack |
|--------|-------|-------|
| Godot 4 | `engines/godot.md` | C#/.NET, Jolt Physics, build-time scene generation |
| Bevy | `engines/bevy.md` | Rust, ECS, code-first scenes |
| Babylon.js | `engines/babylon.md` | TypeScript/Vite, browser-served |

## Sub-Files

| File | Purpose | When to read |
|------|---------|--------------|
| `engines/godot.md` | Godot engine guide | Before writing Godot code |
| `engines/bevy.md` | Bevy engine guide | Before writing Bevy code |
| `engines/babylon.md` | Babylon.js engine guide | Before writing Babylon code |
| `asset-gen/SKILL.md` | Asset generation CLI reference | When generating assets |
| `asset-gen/rembg.md` | Background removal | When an asset needs transparency |
| `prompts/runtime.md` | Runtime manifest | Pipeline start |

## Asset Generation

Uses paid APIs — confirm spend with user before first generation:

| Service | Use | Cost |
|---------|-----|------|
| Gemini | Precise references, characters | 5-15¢ per image |
| xAI Grok | Textures, simple objects | 2¢ per image |
| Tripo3D | Image-to-3D GLB models | 30-60¢ per model |
| Tripo3D Rig | Rigged biped characters | +25¢ |
| Tripo3D Retarget | Animation clips | 10¢ per clip |
| Grok Video | Animated sprites | 5¢/second |

## Key Principles

1. **Proof over claims** — judge progress from the running game, never from a clean build.
2. **Visual QA closes the loop** — capture screenshots, use multimodal review to catch defects.
3. **Scenes are generated at build time** — C# SceneTree scripts that emit .tscn, not hand-authored.
4. **Silent-failure awareness** — Godot serialization drops nodes without errors; validate packs.
5. **Asset manifest** — track every asset with an in-game Size column in README.md.

## Godot-Specific Gotchas

- `ArrayMesh.GenerateNormals()` required for procedural meshes to receive shadows.
- MultiMeshInstance3D + GLB loses mesh on pack/save; use individual instances.
- Raycasts don't reliably hit ConcavePolygonShape3D; use shape queries or analytical height.
- `SetScript()` disposes the C# wrapper — set scripts last, after hierarchy is built.
- Owner chain: every node must have Owner set to scene root or it won't serialize.
- Validate the pack: count nodes before packing, Instantiate() after, compare counts.
- Never `CreateTrimeshShape()` on imported GLB meshes (drops to <1 FPS); use primitive shapes.
- Frame-rate-independent damping: `speed *= Mathf.Exp(-rate * delta)`, not `speed *= (1 - drag)`.
