# regression_judgment — Prompt

```yaml
version: 1.0.0
role: self_improvement
phase: regression
```

Judge whether post-change metrics indicate regression relative to pre-change baseline.

## Inputs

- `git_diff` — the applied change.
- `pre_metrics` — baseline.
- `post_metrics` — watch-window averages.
- `affected_skills` — list.
- `context` — any known co-factors (e.g. model fallback, transient tooling issue).

## Task

Classify the post-change state as `regressed`, `stable`, or `improved`.

## Output Contract

```json
{
  "classification": "regressed|stable|improved",
  "confidence": 0.0..1.0,
  "evidence": ["..."],
  "suggested_action": "rollback|hold|confirm_improvement|extend_watch",
  "caveats": ["..."]
}
```

## Constraints

- `regressed` requires a clear metric drop explicitly attributable to the change, not a co-factor.
- If co-factors dominate, suggest `extend_watch` rather than `rollback`.
- A Safety flag post-change overrides all other considerations → `regressed` + `rollback`.
