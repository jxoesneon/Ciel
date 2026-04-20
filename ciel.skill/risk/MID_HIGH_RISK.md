# MID_HIGH_RISK

Mid and high risk policies.

## Mid Risk

- Fast LLM-judge pass before proceeding (see `LLM_JUDGE.md`).
- Judge is Ciel auditing Ciel — a lightweight single-model prompt, not the full Council.
- On judge pass → execute.
- On judge fail → upgrade to high or escalate.

## High Risk

- Full Council of Five invocation via `council/invocation_scopes/HIGH_RISK_OPS.md`.
- Safety member is primary.
- Execute only on Council pass + Safety non-veto.
- Post-execution: elevated outcome scoring weight; mandatory post-mortem in `~/.ciel/high_risk/<run_id>.md`.

## Dry-Run Preference

For operations that support it, mid/high risk prefers dry-run first:

- `git push --dry-run`
- `npm publish --dry-run`
- `terraform plan`
- Plan mode on Gemini CLI.

Dry-run results feed into the judge / Council.

## Adapter Integration

- Claude Code: `PreToolUse` hook → judge (mid) or Council (high).
- Gemini CLI: `tool.preinvoke` hook + Plan mode gate.
- Generic: inline prompt check.

## Config

```yaml
risk:
  mid_judge_model: gemini-3-flash | haiku
  mid_judge_timeout_s: 10
  high_council_timeout_s: 120
  mid_cost_usd_threshold: 0.10
  high_cost_usd_threshold: 1.00
```

Cost exceedances escalate regardless of level.
