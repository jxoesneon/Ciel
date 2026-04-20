# Template — risk assessment record

Structured record of a risk classification + judge/Council outcome.

```json
{
  "run_id": "{{uuid}}",
  "ts": "{{iso8601}}",
  "operation": "{{rendered command or plan}}",
  "classification": {
    "axes": {
      "reversibility": 0,
      "blast_radius": 0,
      "external_impact": 0,
      "data_sensitivity": 0,
      "cost": 0,
      "novelty": 0
    },
    "composite": 0.0,
    "level": "low|mid|high|critical",
    "veto_conditions_matched": []
  },
  "gate": {
    "kind": "none|judge|council|user",
    "outcome": "proceed|revise|abort|pass|reject|approved|rejected",
    "details_ref": "{{path to details}}"
  },
  "execution": {
    "executed": true,
    "outcome_score": 0.0,
    "elapsed_ms": 0,
    "commit": "{{sha}}|null"
  },
  "post_mortem_ref": "{{path}}|null"
}
```

## Storage

`~/.ciel/risk/<run_id>.json`. Mirrored into MemPalace `ciel/risk/` for recall.
