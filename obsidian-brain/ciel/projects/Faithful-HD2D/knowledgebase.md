---
title: Faithful-HD2D — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-09
source: "https://github.com/jxoesneon/Faithful-HD2D"
---

# Faithful-HD2D — Knowledgebase

Synthesized expansion from the read-only subagent exploration of the local clone.

## Summary

Faithful-HD2D is an ambitious **god simulation game** with HD-2D+ (high-fidelity 2.5D isometric) deferred rendering, planetary-scale ecosystem simulation, and a hybrid TypeScript/React frontend with a Rust/WASM simulation core. The player is a deity guiding mortal civilizations, managing faith systems, casting miracles, and ascending divine levels.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/Faithful-HD2D` |
| Local path | `C:/Users/josee/Faithful` |
| Visibility | PUBLIC |
| License | None |
| Stack | TypeScript/React 19, Pixi.js 8, Rust/WASM, Vite 6, Tailwind CSS 4 |

## Top-level structure

- `src/` — TypeScript source (100+ engine modules, React components, assets).
  - `engine/` — ECS, simulation, renderer, audio, AI, behavior, combat, economy, faith, world systems, VFX, etc.
  - `components/` — React UI components (AdaptiveUI, AssetInspector, Minimap, StartMenuOverlay, etc.).
- `faithful-engine/` — Rust/WASM simulation core (`lib.rs`, `ecs.rs`, `fractal.rs`, `simulation.rs`, `gods.rs`).
- `docs/` — 30+ design docs (technical architecture, game cycle, audit, gap analysis, sprites, UI/UX).
- `tests/` — E2E tests.
- `package.json`, `tsconfig.json`, `vite.config.ts`, `playwright.config.ts`, `vitest.setup.ts`.
- `README.md`, `HANDOFF.md`.

## Architecture

### Hybrid stack

- **Frontend**: React 19 + TypeScript + Tailwind CSS 4 + Motion.
- **Rendering**: Pixi.js 8 (WebGL 2D) with deferred rendering pipeline, G-Buffer (albedo/normal), dynamic lighting, god rays, instanced rendering for 100k+ entities, LOD L0–L4.
- **Simulation core**: Rust compiled to WASM; Web Worker with SharedArrayBuffer for zero-copy memory (100k entities).
- **Terrain**: Ridged multifractal noise, particle-based hydraulic erosion, thermal erosion, tectonic stress, exponential zoom detail.
- **ECS**: custom Map-based component storage in both TypeScript and Rust; supports queries and state export/import.

### Major systems

- **Engine coordinator** (`engineCoordinator.ts`) — manages 25+ subsystem managers.
- **World**: day/night, seasons, weather, ecology, disease, wind, vegetation, chunk-based spatial partitioning.
- **Faith**: five faith systems (Animism, Elementalism, Interventionist, Secular, Nihilism), belief matrix, faith fog, shrines, piety, missionary spread, dogma.
- **Economy**: resources (wood, stone, food, metal, crystal, divine essence), gathering, crafting, tech tree (0–10), trade, population, inventory.
- **AI / behavior**: behavior trees, GOAP, blackboard, sensation, combat AI, squad formations, A* pathfinding, preset trees for wolf/stag/villager.

### Core component types

`Position`, `Physics`, `Biology`, `Society`, `Faith`, `Flora`, `Fauna`, `Structure`, `Movement`.

## Build / test

```bash
npm install
npm run dev        # Vite dev server on port 3000
npm run build
npm run preview
npm run lint       # TypeScript type check
npm run test       # Vitest unit tests
npm run test:coverage
npm run test:e2e   # Playwright

# Rust/WASM
cd faithful-engine
cargo build --release
```

WASM release profile: `opt-level = 3`, LTO enabled, `wasm-opt` disabled.

## Current implementation status

**Complete** (from `HANDOFF.md`):

- HD-2D+ deferred rendering pipeline.
- Dynamic lighting and god rays.
- Rust/WASM geological simulation (erosion, tectonics).
- Battery-saver mode, 3D-aware hitboxes, asset registry editor, mobile-responsive UI.

**Stub / missing** (from `AAA-GAP-ANALYSIS-FINAL.md`):

- Pathfinding (P0 critical)
- Behavior trees / GOAP (P0 critical)
- Resource gathering (P0 critical)
- Crafting (P1 high)
- Technology tree (P1 high, currently stub)
- Combat AI (P1 high)
- Population dynamics (P1 high)

## Recent git state (manual snapshot)

- **No `.git` directory** was found in the read-only exploration; this appears to be a working copy without git history or cloned without `.git`.
- Key docs dated 2026-05/06:
  - `HANDOFF.md` (2026-05-24): Phase 1 & 2 complete.
  - `ARCHITECTURAL_AUDIT.md` (2026-05-24): Phase 1 active.
  - `AAA-GAP-ANALYSIS-FINAL.md` (2026-06-03): comprehensive gap analysis.
- Working tree: several untracked files (`.fastembed_cache_old`, `.mempalace`, coverage outputs, debug scripts, scratch, screenshot, gap analysis docs).

## Key files for deeper context

1. `docs/technical-architecture.md` — spatial partitioning, ECS dormancy, rendering, memory.
2. `docs/GAMECYCLE.md` — game mechanics, divine progression, demographics, miracles.
3. `docs/AAA-GAP-ANALYSIS-FINAL.md` — gap analysis of missing systems.
4. `HANDOFF.md` — architectural achievements and next steps.
5. `src/engine/engineCoordinator.ts` — 25+ subsystem orchestrator.
6. `src/engine/simulation.ts` — Web Worker + SharedArrayBuffer simulation.
7. `src/engine/renderer.ts` — deferred Pixi.js renderer.
8. `src/types.ts` — component definitions.
9. `src/engine/gods_data.ts` — deity definitions and mechanics.
10. `faithful-engine/src/fractal.rs` — terrain generation with erosion.

## Related

- [[ciel/projects/Faithful-HD2D/Faithful-HD2D.md|Faithful-HD2D overview]]
- [[ciel/projects.md|Projects index]]
