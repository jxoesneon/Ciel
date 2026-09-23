# CHANGELOG

<!-- markdownlint-disable MD024 -->

All notable changes to Ciel are tracked here. Ciel appends an entry on every self-mutation commit. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with SemVer.

## [Unreleased]

### Added

- **Policy-as-code pre-tool gate** (`risk/policy.yaml` + `init/hooks/lib/risk_policy.py`): single source of truth for every runtime's PreToolUse gate — hard/soft tiers, command and path matchers, tool scoping, `allow_privileged` override on soft rules only, embedded hard-rule fallback when the policy file is absent. Devin and Antigravity hooks now consume the shared evaluator; activity.log entries carry `rule_id`/`tier`/`policy` fields. `scripts/compile_policy.py` produces the stdlib-only `policy.json` twin (sync-tested).
- **Hook red-team harness** (`tests/test_hooks_redteam.py` + `tests/fixtures/hook_redteam_cases.json`): 41 cases fired as real subprocesses against both pre-tool gates in a sandboxed HOME.
- **Paired-evaluation commit gate** (`scripts/paired_eval.py`, `evals/tasks/`): skill mutations/promotions run a task set with and without the candidate in isolated workspaces, scored by per-task `verify.sh`; any regression fails the gate. Required before `sandboxed` → `validated` per `acquisition/TRUST_MODEL.md`.
- **Skill-scan baseline** (`risk/skill_scan_baseline.json`): individually reviewed findings with justifications are subtracted from the scan gate; the 11 findings across 7 skills were triaged (documentation prose and an intentional token-gated bridge).
- **Static skill security scan** (`scripts/scan_skills.py`, `skillfrisk`): mandatory gate before `untrusted` → `sandboxed`, plus a CI `skill-scan` job and JSON report output.
- **System-1 shadow tier** (`risk/SYSTEM1.md`, `hooks/lib/system1.py`): optional semantic second opinion for the pre-tool gate via the Jev protocol (`POST /v1/systemone`, typed `choice`/`noul`/`score` questions). Strictly advisory — never influences `evaluate()`; hooks fire a detached `system1.py --ask` subprocess (bounded to 2 in-flight, response cache) that logs verdicts to `~/.ciel/system1/events.jsonl`. Backends: local `laya-serve` (default `http://127.0.0.1:8765`), hosted Jev, or AutoJev — swap via `CIEL_SYSTEM1_URL`/`CIEL_SYSTEM1_KEY`; `CIEL_SYSTEM1_DISABLED=1` opts out. Lattice design in `architecture/ADR_20260923_SYSTEM1_DEEP_INTEGRATION.md`.
- **System-1 calibration harness** (`scripts/system1_eval.py`): scores the endpoint against the hook red-team corpus and writes `risk/system1_calibration.json` (confusion matrix, precision/recall/F1, confidence calibration, latency, per-tau operating-point sweeps) — the committed evidence artifact required before any surface promotion.
- **Advisory banding + review queue** (`system1.py` `SURFACE_FLAGS`/`_band`, `scripts/system1_review.py`): every shadow event carries `flag` ∈ `pass`/`uncertain`/`flag` — per-surface flagged choices plus a tau uncertainty floor (`CIEL_SYSTEM1_TAU`, default 0.2 from the calibration sweep, where low-confidence `safe` is review-worthy). The review script tails `events.jsonl` into a flagged/uncertain queue with per-surface stats.
- **`council_prescreen` shadow surface** (`system1.council_prescreen`): detached routine/escalate verdicts for council-scope events. Live probe: model biases toward `routine` (2/4 — both misses under-triage), confirming shadow-only status until fine-tuned.
- **Agent Skills spec conformance**: Ciel-specific frontmatter moved to per-skill `ciel.yaml` sidecars across all 165 skills; `SKILL.md` files now validate against the official spec (`scripts/migrate_skill_sidecar.py --check`).
- **Bounded activity-log rotation** (`init/hooks/lib/activity_log_rotate.py`): spec-aligned daily/size triggers, zstd archives under `~/.ciel/archive/logs/`, 90-day retention, sweep markers.
- **Devin and Antigravity hook payloads** shipped from source (`init/hooks/devin/`, `init/hooks/antigravity/`, `init/hooks/lib/`) with installer support on POSIX and Windows.
- **Integrity sweep tool** (`init/scripts/integrity.py`) implementing `init/INTEGRITY.md`: ok / unknown-drift / expected-drift / missing / unexpected classification, timestamped reports, `--write` manifest regeneration.
- **Double-loop skill**: outer supervisor loop (decompose, dispatch, independently verify, correct) driving a bounded pool of hot-swappable worker loops for large worklists.
- **Antigravity `allow_privileged` parity**: `pre_tool_use.sh` honors the local override file like the devin gate.
- **No-attribution enforcement**: install.sh sets `attribution: false` in the devin config and verify.sh re-checks it.
- **Local lint script** (`scripts/lint.sh`) mirroring all CI gates.

### Changed

- **Hook portability**: hook payloads reference `${HOME}` instead of hardcoded absolute paths.
- **Devin adapter docs**: hook references corrected to the real `~/.ciel/hooks/devin/<event>.sh` layout.

### Fixed

- Ruff auto-fixable findings across `scripts/` and `ciel.skill/init/scripts/` cleared.

### Removed

- **Foreign backlog**: `ciel.skill/backlog/` Blindsight task files dropped (not Ciel content).

## [1.0.0] — Genesis

### Added

- **Full CIEL 1.0 Ecosystem**: Integrated and harmonized 140 high-density frameworks from 262 disparate sources.
- **Partner Intelligence Model**: Evolved core cognitive architecture with explicit Identity, Constitution, and Autonomy layers.
- **Elite Guild System**: Consolidated 100+ specialized agents into 10 High-Signal Guilds (Systems, Web, Cloud, Security, Intelligence, Experience, Strategy, Quality, Mobile, Data).
- **Master Router**: Hybrid routing system (Fast / Reasoning / Acquisition) with multi-runtime support (Gemini CLI, Claude Code, Windsurf).
- **Council of Five**: Governance layer for autonomous auditing (Capability, Coherence, Safety, Efficiency, Evolution).
- **The Iron Law**: Mandatory verification evidence protocol for all completion claims.
- **TDD 80% Floor**: Mandatory baseline testing and 80% coverage mandate across all logic paths.
- **High-Density Frameworks**:

  - `ciel-swarm-orchestration`: Parallel multi-agent task decomposition.
  - `ciel-hitl-protocol`: Human-in-the-Loop risk gating.
  - `ciel-root-cause-debugger`: Empirical RCA and trace analysis.
  - `ciel-project-scaffolder`: Standardized architectural initialization.

- **MemPalace-rs**: Primary memory backend with AAAK 3.2 compression and SQLite/Filesystem fallbacks.
- **Validation Suite**: Platform-agnostic shell and PowerShell scripts for spec compliance and frontmatter integrity.
- **Automation**: Enterprise-grade CI/CD pipelines for release, verification, and mandate enforcement.
- **Observability**: Activity logging and OTEL-compatible tracing for transparent autonomous execution.

### Fixed

- Resolved 'Blind Deletion' and 'Amnesiac Recovery' failures via self-improvement and git-based restoration.
- Corrected logic bugs in `lint-fix.py` fence-tracking.
- Synchronized licensing (MIT) and versioning (1.0.0) across all 140 skills and manifest files.

### Notes

- This version represents the terminal state of the CIEL 1.0 harmonization process. The core CIEL 1.0 ecosystem is harmonized and ready for deployment.
