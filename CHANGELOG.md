# CHANGELOG

<!-- markdownlint-disable MD024 -->

All notable changes to Ciel are tracked here. Ciel appends an entry on every self-mutation commit. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with SemVer.

## [1.3.0] — 2026-10-07

System-1 grows from advisory shadow to an active, governed decision tier: laya-serve participates in PreToolUse and router decisions behind a fail-open boundary, six calibrated surfaces sit under the tau lattice (`pre_tool_risk`, `council_prescreen`, `context_select`, `memory_salience`, `context_compaction`, `mandate_canary`), and an RLCD pipeline plus Track-1/Track-2 retraining toolchain closes the improvement loop. A hosted Jev endpoint is primary with persisted-breaker failover to local laya-serve; platform packaging ships `launchd`/`systemd` units and an env-pinned launcher. Local sibling checkpoints are servable through a digest-pinned registry overlay (`ciel_laya_serve.py`) pending upstream `laya` PR #1047. Two new Rust crates (`ciel-dev`: 12 dev-tools + MiniLM embed; `ciel-domain`: 4 domain binaries) extend the native tier, and `ciel-rs` gains `verify-3d`/`config-heal`. Also: adapted Obsidian-vault memory backend with hybrid search; Xcode, opencode, and PowerShell adapter coverage; eleven council-approved skill acquisitions including validated `e2e` agentic testing; and a cleared CI debt backlog (ruff pin, lint, format, spec gates). Note: the `ciel-context-lora` candidate is registered but production remains pinned to `typed-decisions` until the shadow-accrual acceptance bar accrues.

Full per-change detail: `ciel.skill/CHANGELOG.md` [1.3.0].

## [1.2.0] — 2026-09-24

Comprehensive Rust migration: a single `ciel` binary (`init/ciel-rs/`) now carries every per-invocation hot path — `pretool`/`prompt-submit` hook bodies, consolidated `session-start`, and primitive subcommands (`risk-eval`, `risk-check`, `grant-state`, `secret-scan`, `attribution-scan`, `store-perms`, `ledger`, `watchdog`, `log-rotate`, `sanitize`, `compile-policy`, `council-verify`, `system1`). Shell hooks remain adapters resolving `$CIEL_BIN` → `~/.ciel/bin/ciel` → `~/.cargo/bin/ciel` → `init/bin/ciel` with byte-identical Python fallback. Measured ~2.7× faster per PreToolUse firing, ~4–7× per session start. Distribution: POSIX release matrix (linux/darwin × x86_64/arm64) with checksum-verified prebuilt downloads and `cargo install` support; `install.ps1` unchanged (Windows runs the Python fallback). 54-case Rust↔Python parity suite plus 85-case red-team corpus through the `.sh` wrappers in both engine modes; council review across four stages resolved all must-fix findings. Python remains need-basis: `system1_embed.py` (sentence-transformers), hook adapters, the fallback engine, and cold operator scripts.

Full per-change detail: `ciel.skill/CHANGELOG.md` [1.2.0].

## [1.1.0] — 2026-09-24

Policy-as-code pre-tool gate (hard/soft/advisory tiers) with hook red-team harness, System-1 advisory shadow tier with calibration + RLCD export, council-run verification, session watchdog, transcript sanitizer, store-permission self-heal, secret-ingress and attribution gates, requirement ledger with completion evidence, and 100% diff-scoped test coverage on changed code. Release-council hardening: grant-control files under non-overridable rules, destructive-verb coverage, anchored policy resolution, detached deferred sanitize, extended sanitizer store coverage.

Full per-change detail: `ciel.skill/CHANGELOG.md` [1.1.0].

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
