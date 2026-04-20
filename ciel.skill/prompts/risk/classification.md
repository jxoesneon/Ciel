# classification — Prompt

```yaml
version: 1.0.0
role: risk
```

Classify a proposed operation into a risk level.

## Inputs

- `operation` — rendered command or plan.
- `project_ctx`, `runtime`.
- `classification_weights` — per `risk.config.md`.

## Task

1. Score 0–10 on each axis: reversibility, blast_radius, external_impact, data_sensitivity, cost, novelty.
2. Compute weighted composite.
3. Apply veto conditions from `council/rubrics/VETO_CONDITIONS.md`.
4. Return the level.

## Output Contract

```json
{
  "axes": {
    "reversibility": 0..10,
    "blast_radius": 0..10,
    "external_impact": 0..10,
    "data_sensitivity": 0..10,
    "cost": 0..10,
    "novelty": 0..10
  },
  "composite": 0..10,
  "level": "low|mid|high|critical",
  "veto_conditions_matched": [...],
  "rationale": "<=2 sentences"
}
```

## Constraints

- Any veto-condition match forces `level = critical`.
- A reversibility score of 9 or 10 shifts composite down by the reversibility weight (already in formula).
- Be honest; avoid sandbagging or alarmism.
