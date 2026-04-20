# outcome_scoring — Prompt

```yaml
version: 1.0.0
role: self_improvement
phase: outcome
```

Score the outcome of an executed operation.

## Inputs

- `op` — operation description + plan.
- `result` — stdout/stderr/exit + effects observed.
- `intent` — what the user or router was trying to accomplish.
- `baseline` — past averages for this skill (if any).

## Task

Score each dimension 0..1: `success`, `correctness`, `side_effects`, `efficiency`, `user_satisfaction`, `safety_observed`. Compute weighted total per `OUTCOME_SCORING.md`.

## Output Contract

```json
{
  "dims": {
    "success": 0.0..1.0,
    "correctness": 0.0..1.0,
    "side_effects": 0.0..1.0,
    "efficiency": 0.0..1.0,
    "user_satisfaction": 0.0..1.0,
    "safety_observed": 0.0..1.0
  },
  "score": 0.0..1.0,
  "rationale": "<=3 sentences",
  "flags": ["unexpected_side_effect" | "partial_success" | "retry_recommended" | ...]
}
```

## Constraints

- Base `success` on objective signals (exit code, error markers).
- `correctness` is LLM-judged; be explicit about the evidence.
- Do not conflate `side_effects` with `safety_observed` — first is breadth, second is severity.
