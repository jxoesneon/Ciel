# CHANGELOG

<!-- markdownlint-disable MD024 -->

All notable changes to Ciel are tracked here. Ciel appends an entry on every self-mutation commit. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with SemVer.

## [Unreleased]

### Added

- **`ciel` Rust binary** (`init/ciel-rs/`): single-binary port of the per-invocation hook bodies — `pretool --runtime {devin|antigravity}` (payload parse → policy evaluate → activity append → detached System-1 shadow spawn → advisory attribution dispatch → decision emit) and `prompt-submit` (secret ingress scan + canary context), plus primitive subcommands `risk-eval`, `risk-check`, `grant-state`, `secret-scan`, `attribution-scan`. Dual-engine regex: `regex` fast path with `fancy-regex` fallback for the policy's `(?<!...)` lookbehind rules (a `[^C]`-rewrite was rejected — it cannot fire on `of=/etc` where the predecessor char is locked inside `\bof=`). Hook `.sh` adapters resolve `$CIEL_BIN` → `~/.ciel/bin/ciel` → `init/bin/ciel` and fall through to the Python bodies on missing binary or nonzero exit. Measured ~2.7× faster per PreToolUse firing (~134ms vs ~360ms end-to-end, ~90ms vs ~366ms evaluator-only); Python remains the fallback until parity is proven everywhere.
- **Rust↔Python parity suite** (`tests/test_rust_parity.py`): differential tests feeding identical inputs through `risk_policy.evaluate`/`secret_scan.scan`/`attribution_scan.scan` and the binary — the full 85-case red-team corpus under both override states, path normalization forms, malformed JSON, fallback-policy source, and CLI contract. `test_hooks_redteam.py` gained `test_corpus_rust_fastpath` — the same corpus through the `.sh` wrappers with `CIEL_BIN` set.
- **§9c install step** (`init/scripts/install.sh`): builds `init/ciel-rs` with cargo when the toolchain is present, honors `CIEL_BIN_URL`/`CIEL_RELEASE_BASE` for prebuilt artifacts, and warns-skips to the Python fallback otherwise — the binary is never an install blocker.
- **`ciel session-start`** — the whole SessionStart batch collapsed into one Rust process: attribution-flag heal + store-permission sweep + grant-state report + watchdog check (stalled-session hints, incremental transcript secret sweep, detached `--sanitize-pending`) + activity-log rotation + runtime canary emission for both `devin` (`session_start.sh`) and `antigravity` (`pre_invocation.sh`). New primitive subcommands `store-perms`, `ledger {add|done|list|pending}`, `watchdog [--resume|--sanitize-pending]`, `log-rotate` mirror `store_perms.py`/`requirements.py`/`session_watchdog.py`/`activity_log_rotate.py` exactly — same stdout contracts, same state files. `session_start.sh` and `pre_invocation.sh` take the same `$CIEL_BIN`-first/Python-fallback adapter shape as the pre-tool hooks. Measured ~4–7× faster per session start (~0.5–0.8s vs ~2–5.9s end-to-end on a live home).
- **Session-ops parity tests** (`test_rust_parity.py::TestSessionOpsParity`): perms repair counts, ledger add/pending/done event flow, watchdog stalled/secret-sweep hints, daily log rotation (archive + `log_rotate` marker), and SessionStart JSON shape for both runtimes — all asserted identical across the Python and Rust engines. Caught a real bug: `OpenOptions::append` without `create` silently dropped the post-rotation marker (the log is renamed away first).
- **Phase-3 operator ports** — three new subcommands under the same binary: `ciel sanitize [--scan|--redact] [--dry] [--deep]` (full `transcript_sanitize.py` port: store iteration, `.log.gz` decompress→redact→recompress, `.bak` backups, perms re-tighten, surrogateescape-equivalent redaction via valid-UTF-8 span substitution so protobuf-in-BLOB bytes round-trip verbatim, LIKE+`instr()` two-tier SQL prefilter and SQL-level `sessions.db` redaction with BEGIN IMMEDIATE busy-retry and `wal_checkpoint(TRUNCATE)` via bundled rusqlite), `ciel compile-policy [--check]` (`policy.yaml`→`policy.json` byte-exact render — `serde_yaml` + insertion-ordered `serde_json` matching `json.dumps(indent=2)`), `ciel council-verify <run|dir>` (full artifact verification + signal emission, `isinstance`-faithful bool-score semantics). `watchdog --sanitize-pending` now consumes the deferred flag in-process (retries=2, wait=3s) and `check` re-execs the binary for the detached leg — no Python spawn left in the deferred-sanitize path. `TestOperatorParity` proves: identical JSON contracts for scan/redact/dry, byte-identical redacted outputs across two mirrored homes, BLOB length preservation and invalid-byte passthrough, locked-DB pending-flag write, byte-exact policy.json render, and all four council-verdict shapes.

## [1.1.0] — 2026-09-24

### Added

- **Policy-as-code pre-tool gate** (`risk/policy.yaml` + `init/hooks/lib/risk_policy.py`): single source of truth for every runtime's PreToolUse gate — hard/soft tiers, command and path matchers, tool scoping, `allow_privileged` override on soft rules only, embedded hard-rule fallback when the policy file is absent. Devin and Antigravity hooks now consume the shared evaluator; activity.log entries carry `rule_id`/`tier`/`policy` fields. `scripts/compile_policy.py` produces the stdlib-only `policy.json` twin (sync-tested).
- **Hook red-team harness** (`tests/test_hooks_redteam.py` + `tests/fixtures/hook_redteam_cases.json`): 85 cases fired as real subprocesses against both pre-tool gates in a sandboxed HOME.
- **Paired-evaluation commit gate** (`scripts/paired_eval.py`, `evals/tasks/`): skill mutations/promotions run a task set with and without the candidate in isolated workspaces, scored by per-task `verify.sh`; any regression fails the gate. Required before `sandboxed` → `validated` per `acquisition/TRUST_MODEL.md`.
- **Skill-scan baseline** (`risk/skill_scan_baseline.json`): individually reviewed findings with justifications are subtracted from the scan gate; the 11 findings across 7 skills were triaged (documentation prose and an intentional token-gated bridge).
- **Static skill security scan** (`scripts/scan_skills.py`, `skillfrisk`): mandatory gate before `untrusted` → `sandboxed`, plus a CI `skill-scan` job and JSON report output.
- **System-1 shadow tier** (`risk/SYSTEM1.md`, `hooks/lib/system1.py`): optional semantic second opinion for the pre-tool gate via the Jev protocol (`POST /v1/systemone`, typed `choice`/`noul`/`score` questions). Strictly advisory — never influences `evaluate()`; hooks fire a detached `system1.py --ask` subprocess (bounded to 2 in-flight, response cache) that logs verdicts to `~/.ciel/system1/events.jsonl`. Backends: local `laya-serve` (default `http://127.0.0.1:8765`), hosted Jev, or AutoJev — swap via `CIEL_SYSTEM1_URL`/`CIEL_SYSTEM1_KEY`; `CIEL_SYSTEM1_DISABLED=1` opts out. Lattice design in `architecture/ADR_20260923_SYSTEM1_DEEP_INTEGRATION.md`.
- **System-1 calibration harness** (`scripts/system1_eval.py`): scores the endpoint against the hook red-team corpus and writes `risk/system1_calibration.json` (confusion matrix, precision/recall/F1, confidence calibration, latency, per-tau operating-point sweeps) — the committed evidence artifact required before any surface promotion.
- **Advisory banding + review queue** (`system1.py` `SURFACE_FLAGS`/`_band`, `scripts/system1_review.py`): every shadow event carries `flag` ∈ `pass`/`uncertain`/`flag` — per-surface flagged choices plus a tau uncertainty floor (`CIEL_SYSTEM1_TAU`, default 0.2 from the calibration sweep, where low-confidence `safe` is review-worthy). The review script tails `events.jsonl` into a flagged/uncertain queue with per-surface stats.
- **`council_prescreen` shadow surface** (`system1.council_prescreen`): detached routine/escalate verdicts for council-scope events. Calibrated wording ("when in doubt, escalate") lifts escalate recall 0.00→0.67 on the 20-case corpus — neutral phrasing collapses to a `routine` bias.
- **Multi-surface calibration** (`scripts/system1_eval.py`): pre_tool_risk / council_prescreen / router corpora (`tests/fixtures/system1_*_cases.json`) with per-surface metrics — risk P=0.74 R=0.79 after path-emphasis wording fix (was 0.70/0.66), prescreen P=0.80 R=0.67, router accuracy 0.73 on 9 candidates.
- **RLCD pair exporter** (`scripts/system1_export.py`): converts `events.jsonl` shadow traffic into contrastive training pairs (`meta.expected` ground truth or `regex_decision` labels, `allow` flagged weak) — the dataset path for a future domain checkpoint.
- **Agent Skills spec conformance**: Ciel-specific frontmatter moved to per-skill `ciel.yaml` sidecars across all 165 skills; `SKILL.md` files now validate against the official spec (`scripts/migrate_skill_sidecar.py --check`).
- **Bounded activity-log rotation** (`init/hooks/lib/activity_log_rotate.py`): spec-aligned daily/size triggers, zstd archives under `~/.ciel/archive/logs/`, 90-day retention, sweep markers.
- **Devin and Antigravity hook payloads** shipped from source (`init/hooks/devin/`, `init/hooks/antigravity/`, `init/hooks/lib/`) with installer support on POSIX and Windows.
- **Integrity sweep tool** (`init/scripts/integrity.py`) implementing `init/INTEGRITY.md`: ok / unknown-drift / expected-drift / missing / unexpected classification, timestamped reports, `--write` manifest regeneration.
- **Double-loop skill**: outer supervisor loop (decompose, dispatch, independently verify, correct) driving a bounded pool of hot-swappable worker loops for large worklists.
- **Antigravity `allow_privileged` parity**: `pre_tool_use.sh` honors the local override file like the devin gate.
- **No-attribution enforcement**: install.sh sets `attribution: false` in the devin config and verify.sh re-checks it.
- **Local lint script** (`scripts/lint.sh`) mirroring all CI gates.
- **Store-permission self-heal** (`hooks/lib/store_perms.py`): session-start sweep asserting `0700`/`0600` across transcript, summary, log, conversation-database, `sessions.db`/`-wal`/`-shm`, and Ciel state stores — previously world-readable state-bearing files are tightened automatically and the repair is surfaced in session context.
- **Secret-ingress scanner** (`hooks/lib/secret_scan.py` on `user_prompt_submit`): 12 deterministic, local-only pattern classes (GitHub/AWS/GCP/Slack/npm/crates tokens, JWTs, private-key blocks, password/secret assignments). Warns into session context and logs category names only — no matched content is persisted, and detection never blocks the prompt.
- **Grant-state surfacing and provenance** (`risk_policy.py --grant-state`, `~/.ciel/grants.log`): the `allow_privileged` override is announced in session-start context when active, and first-seen/removed transitions are logged so an unexpected grant becomes visible instead of silent.
- **Attribution gate** (`hooks/lib/attribution_scan.py` + `advisory` policy tier in `risk/policy.yaml`): scans publish-shaped commands, staged-diff additions, and unpushed messages for attribution trailers, identity tokens, and emoji. Compiled through the `policy.yaml → policy.json` path as a non-denying tier; `~/.ciel/risk/attribution_gate` selects shadow/deny mode, an allowlist file tunes false positives, and `CIEL_ATTRIBUTION_SKIP=1` is the documented bypass.
- **Council-run enforcement** (`scripts/council_verify.py`, `adapters/devin/COUNCIL_INVOCATION.md`): a council verdict is verifiable only when each member ran as an isolated subagent — spawn receipts, per-member stage artifacts, and veto-consistency are checked, verdicts emit improvement signals, and inline fallbacks must declare `mode: "inline"` (flagged `unverified_member_isolation`, never silently verified).
- **Requirement ledger + completion evidence** (`hooks/lib/requirements.py`, `COMPLETION_EVIDENCE.md`): session-scoped pending/done ledger on the checkpoints store, reconciled by the stop hook with a hard cap of 2 nudges per session; the evidence matrix binds each task class to its required fresh artifact (code → suite run, policy change → fired event, release → coverage + council sign-off).
- **Session watchdog** (`hooks/lib/session_watchdog.py` + `ciel-watchdog.{service,timer}`): detects stalled sessions (pending ledger items past the idle threshold) and transcripts ending on API-shaped error signatures, surfaces resume hints at SessionStart, and supports bounded opt-in headless resume (`devin -c -p`, gated on `CIEL_WATCHDOG_AUTORESUME=1`, ≤1/session/day, ≤3/day, ≥30min backoff). Timer units are installed but deliberately not enabled.
- **Transcript sanitizer** (`scripts/transcript_sanitize.py`): advisory scan plus in-place `[REDACTED:<category>]` redaction across transcript/summary stores, plain and `.gz` logs (decompress–redact–recompress), conversation databases (length-preserving byte substitution to protect length-delimited wire fields), and `sessions.db` via SQL-level updates (byte-level `instr()` needles so BLOB values are found, surrogateescape round-trip, `wal_checkpoint(TRUNCATE)` afterward). When the live database is write-locked it flags `sessions_db_sanitize_pending`, which the watchdog consumes at the next SessionStart or timer fire.
- **Scheduled conversation audit** (`CONVERSATION_AUDIT.md`): monthly leg under `scheduled_sweep` — repeats the transcript/prompt audit method to catch recurring workflow drift, emitting findings into the improvement signal store.
- **Release-gate diff coverage** (`tests/test_release_gate_coverage.py`): every line added or changed since v1.0.0 is exercised — CLI mains, subprocess entry points, error branches, and binary/SQLite edge paths — measured at 100% diff coverage via `coverage` + `diff-cover` against the last tag.

### Changed

- **Hook portability**: hook payloads reference `${HOME}` instead of hardcoded absolute paths.
- **Devin adapter docs**: hook references corrected to the real `~/.ciel/hooks/devin/<event>.sh` layout.

### Fixed

- **Gate self-escalation chain closed** (release-council must-fix): `~/.ciel/allow_privileged`, `.grant_state`, `grants.log`, and the `risk/` advisory-gate controls now sit under hard-tier `grant_control_tamper`/`grant_control_command` rules — the override sentinel can no longer be created, modified, or deleted by a gated call on either the path or shell-command vector, so the soft tier's self-tamper guard can no longer be switched off by its own subject.
- **Destructive-verb coverage**: hard-tier `destructive_sensitive_path`/`find_delete_sensitive_path` rules cover `rm`/`unlink`/`shred`/`rmdir`/`find -delete` against credential stores, `~/.ciel` (root, hooks, risk, grant controls), `~/.config/devin`, and protected system paths — previously `rm -rf ~/.ciel/risk` matched no rule. `shell_sensitive_path_write` also gained the destructive verbs at soft tier plus the bare `~/.ciel` root (closes the mv-aside + symlink-swap substitution vector).
- **Policy ancestor-substitution closed**: `_candidate_policy_files` now generates candidates only under named-anchor ancestors (`.ciel`, `ciel.skill`) instead of every ancestor to `/` — a planted `~/risk/policy.json` can no longer replace the deployed policy after an unguarded delete. `_normalize_path` now applies `os.path.normpath` so `/./`, `//`, and `/../` spellings cannot slip past path rules, and `${HOME}` brace expansion is matched alongside `$HOME`/`~` in command rules; `mkdir`/`sed`/`perl`/`python`/`ruby` join the grant-control verb list (the sentinel can no longer be minted as a directory or written by an interpreter one-liner).
- **Deferred sessions.db sanitize detached**: the pending redact now spawns as a detached `--sanitize-pending` child from SessionStart instead of running inline under the watchdog's `timeout 4` budget, where lock retries plus the multi-GB `message_nodes` sweep could never finish.
- **Sanitizer store coverage extended** (`transcript_sanitize.py`, watchdog sweep): `~/.ciel/system1/*.jsonl` (shadow events, RLCD pairs), `~/.ciel/checkpoints/*.jsonl`, `activity.log`, and `grants.log` are now swept — full command text persisted by the shadow tier no longer sits outside sanitization.
- Ruff auto-fixable findings across `scripts/` and `ciel.skill/init/scripts/` cleared.
- `system1_review.py` no longer crashes on shadow events whose `answers` payload is a non-dict value.
- `transcript_sanitize.py` sessions.db prefilter now reaches BLOB values (SQL `LIKE` never matches a blob operand and stops at embedded NULs — `CAST` plus byte-level `instr()` needles cover both cases).

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
