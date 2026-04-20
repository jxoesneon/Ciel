# OUTCOME_SCORING

Post-execution scoring of routing + skill execution outcomes.

## Scored Dimensions

| Dimension | Weight | Measurement |
| --- | --- | --- |
| success | 0.35 | exit code 0 + no error markers |
| correctness | 0.25 | LLM-judged match to intent (optional, sampled) |
| side_effects | 0.15 | declared vs observed side effects |
| efficiency | 0.10 | ms vs skill's avg; tokens vs budget |
| user_satisfaction | 0.10 | implicit (no retry / correction) or explicit |
| safety_observed | 0.05 | no Safety flags triggered post-hoc |

`outcome_score = Σ weight_i * dim_i` in 0..1.

## Storage

Per-invocation:

```json
{
  "run_id": "...",
  "skill": "git/SKILL.md",
  "ts": "...",
  "dims": { "success": 1.0, "correctness": 0.9, ... },
  "score": 0.92,
  "tokens": 1240,
  "ms": 420
}
```

in MemPalace partition key `ciel/traces/outcomes/<run_id>`.

## Baseline

A skill's rolling average across last 100 invocations. Updated as exponential moving average (α = 0.1). Stored in `registry/REGISTRY.md` `performance.success_rate` etc.

## LLM-Judged Correctness

Sampled, not every invocation — `outcome_scoring.sample_rate` (default 0.1 for low-risk, 1.0 for high-risk). Uses `prompts/self_improvement/outcome_scoring.md`.

## Feedback Loop

Scores update:

- trust score (`acquisition/TRUST_MODEL.md`),
- router confidence priors,
- skill `success_rate` and `avg_ms`,
- trigger signals (`TRIGGERS.md`).

Score drops beyond thresholds fire `outcome_regression` trigger.
