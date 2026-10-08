# DOCKET — Release v1.3.0

**Date:** 2026-10-07
**Scope:** release (publish gate — Council of Five sign-off required before tag/publish)
**Origin:** accumulated surface since `v1.2.0` (2026-09-24): 77 commits, ~1,096 files, ~268k insertions. SemVer classification: **minor** — multiple `feat()` changes; a patch release is fixes-only and does not apply.

## Release surface

- **System-1 active decision tier**: laya-serve participates in PreToolUse and router decisions behind a fail-open boundary; six calibrated surfaces under the tau lattice; hosted-Jev primary with persisted-breaker failover to local laya-serve; launchd/systemd packaging.
- **Registry-overlay checkpoint serving** (`init/system1/ciel_laya_serve.py`): digest-pinned `models.local.json` overlay; explicit-`model=`-only reachability; upstream `LAYA_EXTRA_MODELS` PR open (laya#1047), shim authoritative until it lands.
- **Rust expansion**: `ciel-dev` (12 subcommands + MiniLM embed), `ciel-domain` (4 binaries), `ciel-rs` `verify-3d`/`config-heal`, `system1.rs` context-surface port wired to production call sites.
- **Retraining toolchain**: corpus build, teacher labeling, Track-1 head-only / Track-2 LoRA retrain, offline eval harness, RLCD closure.
- **Memory backend**, **runtime adapters** (Xcode, opencode, PowerShell), **11 council-approved skill acquisitions** incl. validated `e2e`.
- **CI debt clearance**: ruff 0.16.8 pin, default-ruleset debt, shfmt, markdownlint (0 issues / 830 files), spec+frontmatter gates.
- **Working-tree release hygiene diff**: 203 files — mechanical lint/format remediation (ruff autofixes, shfmt, markdownlint), one scanner-literal fix in `secrets.rs`, version bump, changelogs.

## Gate evidence

| Gate | Result |
| --- | --- |
| `python3 -m unittest discover tests` (system1 venv) | 658 tests, OK |
| `tests.test_release_gate_v130` (venv / system python) | 114 OK / 114 OK (21 torch-gated skips) |
| `ruff check` (CI-scoped paths, ruff 0.16.8) | clean |
| `cargo fmt --check` / `clippy -D warnings` / `cargo test` (ciel-rs) | clean / clean / 66 passed |
| `markdownlint-cli2` (830 files) | 0 issues |
| `shfmt -d -i 2 -ci` | clean |
| `validate-spec.sh` / `validate-frontmatter.sh` | PASSED / 320 blocks, 0 invalid |
| Diff-scoped coverage vs `v1.2.0` | **100%** — 1,957/1,957 lines; two verified-unreachable defensive blocks `# pragma: no cover`-annotated (`system1.py` env-cache path guard; `system1_head_retrain.py` probe-side bar-10 duplicate) |

## Council of Five — release sign-off

Run `council-20261007-v130-release-signoff` (in-session deliberation, declared inline mode; `council_verify` → VERIFIED).

- **Verdict: PASS** — weighted **7.2** ≥ 6.5; votes coherence 7, capability 9, safety 6, efficiency 6, evolution 8; Safety > 3 (no veto). Pivotal lens: Safety.
- **Conditions**: (c1) pragma-annotated dead blocks are removal-candidates at next touch — not a standing pattern; (c2) coverage artifacts excluded from the release commit; (c3) eval-fixture gap for `memory_salience`/`context_compaction`/`mandate_canary` recorded as backlog.
- Stage-2 note: Safety dropped 7→6 on the coverage-pragma precedent (dead safety logic shipped annotated); all other lenses held.

## Honesty notes (binding for release notes)

- `ciel-context-lora` is registered and servable; **production pin remains `typed-decisions`** — the bar-2 shadow-accrual floor (200 decisions/gated surface) has not cleared. Release ships capability, not the pin move.
- laya PR #1047 is **open**, not merged; the local overlay remains authoritative.
- Bar-1 tolerance ruling (0.001 margin over tau) is recorded on DOCKET_20261005.

## Verdict

**PASS** — Council of Five signed off (`council-20261007-v130-release-signoff`, weighted 7.2, no veto). Conditions c1–c3 recorded above. Proceed to tag `v1.3.0` and push.
