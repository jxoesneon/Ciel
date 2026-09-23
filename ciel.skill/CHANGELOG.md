# CHANGELOG

<!-- markdownlint-disable MD024 -->

All notable changes to Ciel are tracked here. Ciel appends an entry on every self-mutation commit. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with SemVer.

## [Unreleased]

### Added

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
