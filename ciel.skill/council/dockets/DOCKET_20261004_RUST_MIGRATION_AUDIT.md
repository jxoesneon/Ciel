# DOCKET 2026-10-04 — Rust Migration Audit (all Python components)

- **Scope:** SELF_MODIFICATION — standing directive "migrate everything in
  Python to Rust unless technologically impossible; ask on confirmed
  impossibility". Audit recorded; Master's rulings on the confirmed
  impossibilities and policy questions are binding below.
- **Repo:** `/home/eduardo/Ciel`
- **Status:** AUDIT COMPLETE — porting proceeds core-first

## Audit result

**~6.6k lines** of core-runtime Python (`hooks/lib`, `init/scripts`,
`skills/ciel/scripts`, repo `scripts/`) plus **~40 domain-skill files**
(skills/*). Classification:

### Tier A — ported; Python survives as the documented fallback

Per ADR_20261003 the binary is primary and the shell hooks fall back to the
`.py` bodies when `ciel` is absent or exits nonzero:

| Python | Rust twin |
| --- | --- |
| `system1.py` (1336) | `system1.rs` + `system1 --ask/--decide`, `route-choice` |
| `risk_policy.py` | `risk-eval` / `grant-state` / `risk-check` |
| `secret_scan.py` | `secret-scan` |
| `attribution_scan.py` | `attribution-scan` |
| `session_watchdog.py` | `watchdog` |
| `activity_log_rotate.py` | `log-rotate` |
| `store_perms.py` | `store-perms` |
| `requirements.py` | `ledger` |
| `ciel_root.py` | `paths.rs` |
| `verify_completion.py` | `verify-completion` |
| `compile_policy.py` | `compile-policy` (byte-exact twin) |
| `council_verify.py` | `council-verify` |
| `transcript_sanitize.py` | `sanitize` |
| hook heredocs (pretool/prompt-submit/session-start) | `pretool` / `prompt-submit` / `session-start` |

### Tier B — portable, not yet ported (this session's work queue)

Core runtime:

- `integrity.py` (164) — hash sweep + `git ls-files` classification
- `setup.py` (185) — installer; provisioning the laya venv is Python-ecosystem
  work, but the orchestrator itself ports (spawn `pip`/`uv`)
- `ciel_preflight.py` — documented transitional shim → `ciel preflight`
- `ciel_audit.py` — documented transitional shim → `ciel audit`
- `verify_evidence.py` — completion-gate runner → `ciel verify-evidence`
- `_bootstrap.py` — `sys.path` plumbing; obsoletes once its callers are Rust
- `system1_embed.py` — MiniLM bi-encoder via `sentence_transformers`;
  portable via `candle` (MiniLM support exists) — or served by the
  `ciel-system1d` server from DOCKET_20261004_SYSTEM1_RUST_INFERENCE

Repo dev-tools (all pure logic, portable): `system1_eval.py`,
`system1_export.py`, `system1_review.py`, `ciel_rlcd_pipeline.py`,
`paired_eval.py`, `scan_skills.py`, `harmonize_skills.py`, `fix_md_lint.py`,
`lint-fix.py`, `migrate_skill_sidecar.py`, `build_registry_index.py`.

Domain skills (per Master's scope ruling below): procedural-audio DSP
(librosa/scipy/matplotlib → rustfft/fundsp/plotters), autonomous-3d-studio
orchestrators (`blender_mcp_server.py`, `blender_pipeline_executor.py`,
`high_to_low_baker.py` — their `import bpy` sites are *generated script
strings* shipped to `blender -b -P`, not real imports; the files port),
ui-ux-pro-max catalog tools, `devin-conversation-recovery`, `brag`,
`hyperframes-creative`, `verify_3d_asset.py` (stub).

### Tier C — confirmed technologically impossible

Two files run *inside* vendor-embedded CPython VMs with no Rust/native ABI:

1. `skills/autonomous-3d-studio/assets/blender_mcp_addon.py` — top-level
   `import bpy`; executes inside Blender's addon runtime.
2. `skills/autonomous-3d-studio/scripts/unreal_engine_bridge.py` — top-level
   `import unreal` (`AssetImportTask`, `FbxImportUI`,
   `EditorAssetLibrary`); executes inside Unreal Editor's Python VM.

The sibling `laya-serve` PyTorch daemon is NOT in Tier C — the ONNX parity
evidence from DOCKET_20261004_SYSTEM1_RUST_INFERENCE shows the graph ports;
replacement (`ciel-system1d`) is gated on host performance, not feasibility.

## Master's rulings (binding)

1. **Blender addon — EXEMPT.** `blender_mcp_addon.py` stays Python forever;
   all surrounding orchestration (MCP server, pipeline executor, baker)
   migrates to Rust, optionally including a Rust MCP server.
2. **Unreal bridge — THIN-SPLIT.** Keep only the `unreal.*` calls in a
   minimal Python script; move argument handling, paths, and logging to a
   Rust orchestrator that invokes `UnrealEditor-Cmd -run=pythonscript`.
3. **Fallback policy — DELETE AFTER PARITY SOAK.** Tier-A Python twins are
   deleted once a defined soak shows zero fallback events; the binary then
   becomes a hard dependency. Requires fallback-event instrumentation
   (count every shell-hook descent to the Python body) before the soak
   clock can start.
4. **Scope — EVERYTHING, CORE FIRST.** Domain-skill Python is in scope for
   this migration without re-asking; ordering is core runtime → repo
   dev-tools → domain skills.

## Soak gate definition (for ruling 3)

A fallback event is any shell-hook path that reaches a `.py` body after a
binary attempt fails or the binary is absent. Instrumentation: each hook's
fallback branch appends one line to `~/.ciel/fallback_events.jsonl`
(`{"ts","hook","reason"}`). Soak = **14 consecutive days with zero events**
plus a forced-fallback drill proving the path still works immediately
before deletion (delete-order: remove twins only after the drill passes).
