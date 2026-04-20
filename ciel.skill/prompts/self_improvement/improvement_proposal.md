# improvement_proposal — Prompt

```yaml
version: 1.0.0
role: self_improvement
phase: proposal
```

Generate a concrete improvement proposal from accumulated signals.

## Inputs

- `signals` — the aggregated trigger events from `~/.ciel/improvements/signals/`.
- `current_state` — relevant config / registry excerpts.
- `constraints` — Constitutional floors, per-day caps.

## Task

1. Identify the root cause the signals point to.
2. Propose a minimal, specific change.
3. Classify it: `trivial|standard|structural|constitutional`.
4. Provide before/after metric projections.
5. Include a diff preview (no literal file system writes — just the intended diff).

## Output Contract

```json
{
  "title": "short subject (imperative)",
  "category": "trivial|standard|structural|constitutional",
  "root_cause": "<=2 sentences",
  "proposal": {
    "target_files": ["..."],
    "diff_preview": "unified diff or yaml diff snippet"
  },
  "expected_effect": {
    "metric": "<metric name>",
    "before": 0.0,
    "after": 0.0,
    "delta_pct": 0
  },
  "watch_plan": {
    "window_invocations": 20,
    "window_hours": 48
  },
  "risks": ["..."]
}
```

## Constraints

- No changes to locked files unless `category = constitutional`.
- `trivial` category changes must fit within `auto_tune_range`.
- Do not fabricate metrics; only cite signals actually present in input.
