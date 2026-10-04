# ADR 2026-10-03 — Platform-Agnostic Agent Scripts

Status: accepted (Council of Five, RUN_20261003_PLATFORM_AGNOSTIC_SCRIPTS —
pass, weighted 8.0, 5/5 lenses, no veto; docket
`ciel.skill/council/dockets/DOCKET_20261003_PLATFORM_AGNOSTIC_SKILL_SCRIPTS.md`)

## Context

`skills/ciel/scripts/` grew two divergent surfaces: hollow `.sh` lifecycle
hooks (variables stripped, dead code) and a Windows-only `.ps1` policy that
disagreed with the shared risk core (`ask` vs `deny`, fail-open `catch`,
different log schema). `verify_evidence.sh` resolved the System-1 completion
gate via a repo-relative path that does not exist in installed copies, so the
gate was silently skipped. `ciel.yaml` declared `system: [bash]` and omitted
the runtimes whose hooks already exist (antigravity, devin).

## Decision

1. **Invariant.** Agent-facing scripts are single-source, cross-platform.
   Per-OS shims are permitted only as thin dispatchers forwarding arguments
   and exit codes unchanged to the shared core; they must never carry their
   own policy logic. Codified in `skills/ciel/SKILL.md` and the PR checklist
   in `CONTRIBUTING.md`.
2. **One Python implementation per lifecycle surface**, marked as
   transitional fallback shims pending Rust parity:
   - `ciel_preflight.py` — stdin `{tool,command,path}` (the `ciel risk-eval`
     contract); binary-first via `ciel pretool` (the production hook body,
     which already applies `system1_failsafe`); in-process fallback applies
     `risk_policy.evaluate` + `risk_policy.system1_failsafe`. Exit codes
     `0`/`2`/`3` (allow/deny/unavailable-or-malformed, fail-closed).
   - `ciel_audit.py` — emits the hooks' byte-compatible activity.log schema
     through the shared `ciel_root.append_log` writer + rotation.
   - `verify_evidence.py` — resolves the completion gate across candidate
     roots and the `ciel verify-completion` binary; exit `0`/`2`/`3`
     (pass/rejected/gate-unreachable under enforce; loud + logged under
     shadow).
3. **Shared discovery lives in `ciel.skill/init/hooks/lib/ciel_root.py`**
   (CIEL_ROOT/CIEL_BIN orders, `append_log`) — deployed with the hooks to
   `~/.ciel/hooks/lib`, usable by every consumer; `skills/ciel/scripts/
   _bootstrap.py` only locates that directory.
4. **`verify_completion.py` moved into the bundle**
   (`ciel.skill/init/scripts/`) and installed to `~/.ciel/scripts/` by
   `install.sh`, so the gate is reachable on fresh hosts.
5. **`.ps1` guard coverage regression-ported** into `ciel.skill/risk/
   policy.yaml` (win_volume_format, win_diskpart, drop_database,
   win_remove_item_recurse_drive, win_rmdir_s, ps_alias_recurse_drive) and
   exercised by the shared red-team corpus on both engines. POSIX
   `rm -rf /tmp/x` stays `allow` per existing corpus contract — the alias
   rules key on drive-qualified paths.
6. All seven superseded files deleted in the same commit.

## Consequences

- One verdict, one log schema, one discovery order on every OS.
- The System-1 completion gate can no longer be skipped silently in
  installed copies: unreachable ⇒ exit 3 (enforce) or loud logged warn
  (shadow).
- `ciel_preflight.py`'s stdin/stdout contract matches `ciel risk-eval`/`ciel
  pretool` so the Python layer can be demoted without call-site changes.

## Roadmap (conditions of the verdict)

- `ciel audit` subcommand and a `windows-latest` prebuilt binary target in
  `release.yml`; once both exist, the Python shims demote to fallback-only.
- Preflight-parity and exit-code tests run on the windows-latest/macos CI
  matrix (added in `ci.yml`), not Linux only.
