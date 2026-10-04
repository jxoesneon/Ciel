# DOCKET 2026-10-03 — Platform-Agnostic `skills/ciel/scripts` Consolidation

- **Scope:** SELF_MODIFICATION (non-locked; touches Ciel's own skill surface and System-1 docs)
- **Thresholds:** pass_weighted_score 7.5, safety_min 7, majority_required 4/5 at pass_score 6
- **Repo:** `/home/eduardo/Ciel` (HEAD rebased on origin/main 1296089; 5 local commits unpushed)

## Governing principle (stated by Master)

> Ciel is platform agnostic, so it should run the same across all platforms, with platform
> specifics not affecting the others.

Clarification from Master: this is a skill for AI agents — the agent usually decides when to run
scripts; some are invoked by hooks, pre-hooks, workflows, System-1 or System-2. Unwired scripts are
not a defect per se.

## Evidence (verified by grep, 27 files)

| Script | Invoker | References |
|---|---|---|
| `skills/ciel/scripts/pre_tool_hook.sh` | agent | only `skills/ciel/SKILL.md:138` |
| `skills/ciel/scripts/post_tool_hook.sh` | agent | only `SKILL.md:140` |
| `skills/ciel/scripts/post_failure_hook.sh` | none | unreferenced |
| `skills/ciel/scripts/verify_evidence.sh` | agent + System-1 | `SKILL.md:136`, `ciel.skill/risk/SYSTEM1.md:21`, `ciel.skill/self_improvement/COMPLETION_EVIDENCE.md:36` |
| `skills/ciel/scripts/verify_3d_asset.py` | agent | `SKILL.md:101,136` |
| `skills/ciel/scripts/read_agy_conversation.js` (incoming) | agent | `SKILL.md:144`, `references/antigravity_conversations.md` |
| `ciel_pre_tool.ps1`, `ciel_post_tool.ps1`, `ciel_pre_invocation.ps1` (incoming) | none | not referenced anywhere, not even SKILL.md |

Facts:

1. The three `.sh` agent hooks are **hollow** — every `$VAR` was stripped (`case "" in`,
   `TIMESTAMP="2026-08-19T15:08:05Z"`, `if [ -d "" ]`). They do nothing useful and are bash-only.
2. The incoming `.ps1` files implement a **second, Windows-only policy**: 5 ad-hoc regexes
   (`Remove-Item` pattern requires `-Recurse` before the drive path; misses `rm -r`, `ri -fo`),
   decision `ask` where the shared core says `deny`, `catch` fails open to `allow`, log schema
   `{ts,kind,tool,decision}` vs the hooks' `{ts,runtime,event,tool,risk,rule_id,tier,policy,...}`,
   assigns to PowerShell automatic variable `$args`, different canary text.
3. The shared core already exists and is parity-tested: `ciel risk-eval` (Rust, stdin
   `{tool,command,path}` → verdict JSON) mirrors `hooks/lib/risk_policy.evaluate`;
   `tests/test_rust_parity.py` asserts equality. Production hooks
   (`ciel.skill/init/hooks/{antigravity,devin}/pre_tool_use.sh`) already use binary→python fallback.
4. `verify_evidence.sh` is bash-only and resolves `scripts/verify_completion.py` via
   `$SCRIPT_DIR/../../..`; in the installed copy (`~/.agents/skills/ciel/`) that path does not exist,
   so the System-1 completion gate is **silently skipped** in real installs.
5. `read_agy_conversation.js` is cross-platform (node, `os.homedir`) but: prefix resolution matches
   `.db-shm`/`.db-wal` siblings → "Ambiguous prefix" on every live conversation (reproduced);
   `--raw` without transcript crashes (`readFileSync(undefined)`); header comment says "Windows".
6. CI PSScriptAnalyzer globs only `scripts/*.ps1` and `ciel.skill/init/scripts/*.ps1` — `skills/**`
   unlinted. `skills/ciel/ciel.yaml` declares `system: [bash, git]` (platform-bound, and omits
   python3/node actually required).
7. Incoming `8c624d1` (cfg-gating `perms.rs`/`watchdog.rs`) is correctly isolated; `cargo check` and
   `clippy` are clean.

## Proposal under review

1. `ciel_preflight.py` — replaces `pre_tool_hook.sh` + `ciel_pre_tool.ps1`. Calls `ciel risk-eval`,
   falls back to `risk_policy.evaluate`. One verdict, one log schema.
2. `ciel_audit.py` — replaces `post_tool_hook.sh` + `post_failure_hook.sh` + `ciel_post_tool.ps1`.
   Emits the hooks' existing activity.log schema.
3. Delete `ciel_pre_invocation.ps1` (duplicates the host-hook canary with divergent wording).
4. `verify_evidence.py` — cross-platform port of `verify_evidence.sh`; resolves the gate via
   `CIEL_ROOT` → `~/.ciel` → repo, and **reports loudly** when the gate is unreachable instead of
   silently skipping. Update `SYSTEM1.md` and `COMPLETION_EVIDENCE.md` references.
5. Fix the two `read_agy_conversation.js` bugs and its comment.
6. `SKILL.md` + `ciel.yaml`: repoint references, codify the platform-agnostic invariant
   ("agent-facing scripts are single-source cross-platform; per-OS shims only as thin dispatchers to
   the shared core"), fix declared deps (`python3` required, `node` optional for reader; drop `bash`).
7. Tests + CI: preflight binary/python parity test; `verify_evidence.py` exit-code contract test;
   extend PSScriptAnalyzer + shellcheck globs to `skills/**`.

Out of scope (flagged only): stale installed copy at `~/.agents/skills/ciel`; `install.ps1` not
building `ciel` / registering hooks; `perms.rs` reporting "clean" instead of "unsupported" on
non-unix.

## Alternatives the Council may weigh

- **A. Keep per-OS twins** (fix `.sh`, keep `.ps1`, add parity tests across both).
- **B. Rust-only** (agents call `ciel risk-eval` / new `ciel audit` subcommand directly; no Python
  wrapper). Requires the binary to be installed — not guaranteed on fresh agent hosts.
- **C. Proposal as written** (Python single-source, binary fast path, Python fallback).

## Verdict (RUN_20261003_PLATFORM_AGNOSTIC_SCRIPTS)

**PASS — weighted 8.0** (self-modification thresholds: ≥7.5 weighted, Safety ≥7, ≥4/5 at pass_score 6)

| Lens | Stage 1 | Stage 2 | Delta |
|---|---|---|---|
| Coherence | 8 | 8 | 0 |
| Capability | 8 | 8 | 0 |
| Safety | 8 | 8 | 0 (veto: false) |
| Efficiency | 8 | 8 | 0 |
| Evolution | 8 | 8 | 0 |

Pivotal lens: **safety** — its Stage-1 finding that `ciel risk-eval` omits `system1_failsafe`
(present in production hooks via pretool.rs) drove the binding contract amendments.

Stage-2 challenges resolved into amendments: Coherence→A (shared helper lives in
`hooks/lib/ciel_root.py`, not `scripts/`); Efficiency→C (extract shared `append_log`
into `hooks/lib` — no fifth inline writer; binary-first ordering); Capability→E
(`risk-eval` exits 0 unconditionally — define documented exit codes 0/2/3 instead of
mirroring it).

Binding amendments and follow-ups: `~/.ciel/council/RUN_20261003_PLATFORM_AGNOSTIC_SCRIPTS/final_verdict.json`.

Follow-up (separate commit): restore governance `members/EFFICIENCY.md` overwritten by
the design lens in merge 7b1b6f1.

## Implementation status

Implemented per the PASS verdict and binding amendments:

- `skills/ciel/scripts/`: `ciel_preflight.py` (0 allow / 2 deny / 3
  unavailable-or-malformed; `ciel pretool` fast path, in-process
  `risk_policy.evaluate` + `system1_failsafe` fallback), `ciel_audit.py`,
  `verify_evidence.py` (0/2/3 gate contract, enforce fail-loud, shadow
  warn+log), `_bootstrap.py` resolving `hooks/lib` across repo/bundle/
  installed layouts.
- `ciel.skill/init/hooks/lib/ciel_root.py`: shared root/bin/log primitives;
  `CIEL_BIN` semantics match the production hooks (explicit non-empty
  override is authoritative — no stale-binary rescue).
- Deleted the seven superseded lifecycle/policy scripts (3 `.sh`, 3 `.ps1`,
  `verify_evidence.sh`).
- `read_agy_conversation.js`: `os.homedir()` storage, `.db`-only prefix match,
  `--raw` without transcript exits 1.
- `verify_completion.py` moved into `ciel.skill/init/scripts/`; `install.sh`
  deploys it to `$CIEL_HOME/scripts`.
- Policy: PowerShell destructive patterns ported to `risk/policy.yaml`
  (24 rules, `policy.json` in sync); red-team corpus extended while
  preserving the POSIX `/tmp` cleanup allows.
- Runtime metadata: `antigravity` + `devin` declared; `python3` required,
  `node` optional.
- Tests: `tests/test_agent_scripts.py` (20 tests incl. shim-forwarding
  invariant); CI gains `skills/ciel/scripts/` ruff glob, a
  ubuntu/windows/macos matrix job, and `test_agent_scripts` in the rust
  parity leg.

Verification: 20/20 agent-script tests, 54/54 rust-parity tests, 491-test
suite green, `compile_policy.py --check` in sync, spec/frontmatter/sidecar
validators pass.
