# PAIRED_EVAL — paired-evaluation commit gate

Every skill mutation or promotion is gated on execution evidence: the eval
task set runs **with** and **without** the candidate skill, and each arm is
scored by the task's own `verify.sh`. Model self-evaluation is not accepted
as evidence.

## Runner

```bash
python3 scripts/paired_eval.py --skill skills/<candidate> \
    [--tasks evals/tasks ...] [--report evidence.json]
```

- **Control arm**: clean temp workspace seeded from the task's `workspace/`.
- **Treatment arm**: identical workspace plus the candidate skill installed
  at `.devin/skills/<name>/`.
- The runner template (`--runner`) executes inside the workspace; `{prompt}`
  is replaced by the task prompt. Default:
  `devin -p --permission-mode accept-edits --respect-workspace-trust false -- {prompt}`.
  (The `--` separator keeps the prompt out of the `[PATH]` positionals; the
  trust flag is required because eval workspaces are fresh temp dirs that
  print mode cannot prompt to trust.)
- `verify.sh` decides pass/fail per arm — deterministic, offline-checkable.

## Outcomes and verdict

| control | treatment | outcome |
| --- | --- | --- |
| pass | pass | preserved-pass |
| fail | pass | improvement |
| pass | fail | **regression → gate fails** |
| fail | fail | preserved-fail |

`--require-improvement` additionally fails the gate when no task improves —
this encodes the SkillsBench finding that skills which cannot demonstrate a
measurable gain should not be merged.

## Task set

`evals/tasks/<id>/`: `prompt.md` + `verify.sh` + optional `workspace/` seed.
Candidate skills may ship their own `evals/` directory and pass it via
`--tasks` alongside the shared set. Keep the set small and focused — a
handful of tasks that exercise the skill's domain beats an exhaustive suite
(focused bundles outperform exhaustive loading).

## Evidence

The `--report` JSON (verdict, per-task arms, timings, log tails) is attached
to the promotion docket and kept with the trust record. CI runs the harness
against stub runners only (`tests/test_paired_eval.py`); real agent arms run
at promotion time on a host with an agent CLI.
