# risk.config — Global Risk

```yaml

# <anchor:start>

risk:
  mid_threshold: 3.0
  high_threshold: 6.0
  critical_threshold: 8.5
  judge_confidence_floor: 0.70
  mid_judge_model: auto              # auto|haiku|sonnet|gemini-flash|gemini-pro
  mid_cost_usd_threshold: 0.10
  high_cost_usd_threshold: 1.00
  cost_threshold: 0.20
  plan_mode_gate: false
  classification_weights:
    reversibility: 0.25
    blast_radius: 0.20
    external_impact: 0.20
    data_sensitivity: 0.15
    cost: 0.10
    novelty: 0.10
critical:
  watch_hours: 24
  require_post_mortem: true
  require_alternative_proposal: true
  accept_remote_approval: false      # Constitutional: locked false

# <anchor:end>

```

## Notes

- Thresholds shift the boundary between levels. `critical_threshold` cannot be raised beyond 9.0 (Constitutional floor).
- `accept_remote_approval: false` is locked — critical approvals must be in-terminal.
- `plan_mode_gate: true` forces mid-risk ops through plan mode before execution.

See `risk/CLASSIFICATION.md` (locked) for level definitions.
