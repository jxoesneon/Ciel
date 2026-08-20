# Changelog — Autonomous 3D Studio

## [2.0.0] - 2026-08-20 (Council Security & Capability Refinement - The Ultimate AAA+ Release)
### Added
- `scripts/geometry_core.py`: Memory-efficient shared 3D parser with zero-dependency DSU trees and 3D integer spatial hashing for O(V+F) nearest-neighbor. Hard 500k vertex OOM ceiling implemented.
- `scripts/collision_hull_generator.py`: Automated V-HACD compound convex hulls for UE5 Chaos physics.
- `scripts/vertex_normal_transfer.py`: Face-area weighted CAD/NURBS normal transfer.
- `scripts/facs_blendshape_mirror.py`: Bilateral symmetry mirroring for ARKit/FACS 52 blendshapes.
- `scripts/udim_pack_analyzer.py`: Multi-tile UDIM distribution and boundary check.
- `scripts/usd_variant_manager.py`: OpenUSD stage generator for LOD and Material variants.
- `scripts/distill_3d_instincts.py`: Autonomous instinct consolidation engine distilling `3d_studio_observations.jsonl` into `.ciel/rules/3d_spatial_rules.md`.
- `tests/test_3d_studio_regression.py`: Synthetic test geometry generator for regression testing on QA audit and auto-repair.

### Hardened & Secured
- Refactored `blender_mcp_addon.py` and `blender_mcp_server.py` with `secrets` cryptographic token authentication and length-prefixed JSON-RPC 2.0 binary framing (`!I`).
- Refactored `generative_3d_adapter.py`, `retopology_quadriflow.py`, `geometry_qa_validator.py`, and `uv_texel_analyzer.py` to use `geometry_core.py`, eliminating memory-heavy pure Python BFS overheads.
- `scripts/autonomous_refinement_loop.py` updated with strict 120s subprocess timeouts and `--vault` routing to prevent unconstrained processes and disk thrashing.
- `scripts/substance_sat_baker.py` hardened with `shlex.quote`.
- Dense reciprocal cross-linking implemented across all `references/*.md` documentation.

## [1.2.0] - 2026-08-18 (Deep Research Enhancements)
### Added
- `scripts/blender_mcp_server.py` & `assets/blender_mcp_addon.py`: Live interactive Model Context Protocol (MCP) TCP socket bridge for zero-latency Blender viewport control and real-time execution.
- `scripts/generative_3d_adapter.py`: AI-generative 3D foundation model adapter for Microsoft TRELLIS.2 (O-Voxel sparse voxels), Tencent Hunyuan3D-2.0 (two-stage DiT + Paint), and Hyper3D Rodin Gen-2.5 with photometric delighting and disconnected shell filtering.
- `scripts/substance_sat_baker.py`: Adobe Substance Automation Toolkit (SAT) headless GPU baking bridge with automated recipe generation for Normal, Curvature, AO, Position, Thickness, and Color ID maps.
- `scripts/retopology_quadriflow.py`: Curvature-guided quad retopology and automated 5-tier LOD hierarchy generator (LOD0 to LOD4) with screen-size transition thresholds and JSON manifests.

## [1.1.0] - 2026-08-18 (Council Audit Refinements)
### Added
- `scripts/autonomous_refinement_loop.py`: Closed-loop iterative convergence engine that orchestrates multi-pass auto-repair until all studio hard gates pass.
- Programmatic `--fix` auto-remediation in `scripts/geometry_qa_validator.py` (removes zero-area degenerate faces and prunes unreferenced loose vertices).
- Continuous learning instinct telemetry bridge logging geometry observations to `~/.ciel/instincts/3d_studio_observations.jsonl`.
- Automated atomic backup snapshots in `/root/.gemini/hooks/ciel_3d_preflight.sh` saving timestamped `.bak` files in `$HOME/.ciel/backups/3d_assets/`.

### Hardened & Secured
- Python script generation in `high_to_low_baker.py` and `unreal_engine_bridge.py` hardened with `json.dumps()` escaping to eliminate template injection risks.
- HTML variable sanitization in `turnaround_qa_renderer.py` using `html.escape()`.
- Process group session management and SIGTERM/SIGKILL tree termination in `blender_pipeline_executor.py` upon timeouts.
- Resource guards in `ciel_3d_postflight.sh` with strict 15s timeouts and 50MB file-size limits.

## [1.0.0] - 2026-08-18
### Added
- Initial release of `autonomous-3d-studio`, CIEL's AAA+ master 3D production skill.
- Full 7-stage production pipeline documentation from high-poly sculpting to engine integration.
- 12 comprehensive reference blueprints covering hard-surface, characters, topology, UV/texel density, baking, MaterialX, Geometry Nodes, rigging, USD/UE5, generative 3D, and visual QA.
- 8 deterministic production scripts for headless Blender execution, geometry validation, turnaround rendering, UV analysis, high-to-low baking, procedural kit generation, USD/MaterialX assembly, and Unreal Engine 5 remote execution.
- Standardized asset configuration presets for DCC export, texel density matrices, and studio QA checklists.
