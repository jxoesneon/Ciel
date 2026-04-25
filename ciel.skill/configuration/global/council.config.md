# council.config — Global Council

```yaml

# <anchor:start>

council:
  pass_score: 6
  weighted_pass: 6.5
  reject_threshold: 4.5
  majority_required: 3
  weights:
    coherence: 0.20
    capability: 0.20
    safety: 0.25       # Constitutional floor: 0.20
    efficiency: 0.15
    evolution: 0.20
  anonymize_stage2: true   # Constitutional: locked true
  stage_timeout_s: 60
  local_quorum_min: 3

# <anchor:end>

```

## Notes

- `weights.safety` cannot be lowered below 0.20 (Constitutional).
- `anonymize_stage2` cannot be disabled (Constitutional).
- `stage_timeout_s` — per-stage wall clock. Exceeding counts as abstention.
- `local_quorum_min` — minimum non-abstaining members for local-only improvements.

## Related Rubrics

- `council/rubrics/SCORING.md`
- `council/rubrics/VETO_CONDITIONS.md` (locked)
- `council/rubrics/PROMOTION_RUBRIC.md`
- `council/rubrics/CONFLICT_RUBRIC.md`
