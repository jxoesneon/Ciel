# Template — improvement proposal

Structured form of a self-improvement proposal.

```yaml
id: {{uuid}}
ts: {{iso8601}}
title: "{{imperative subject}}"
category: trivial|standard|structural|constitutional
trigger_signals:

  - kind: {{route_miss|confidence_floor_breach|outcome_regression|...}}

    evidence: "{{citation or summary}}"
root_cause: "<=2 sentences"
proposal:
  target_files:

    - path/to/file

  diff_preview: |
    --- a/...
    +++ b/...
    @@
    ...
expected_effect:
  metric: "{{name}}"
  before: 0.0
  after: 0.0
  delta_pct: 0
watch_plan:
  window_invocations: 20
  window_hours: 48
risks:

  - "..."

council:
  scope: {{self_modification|skill_integration|promotion|null}}
  run_id: {{id|null}}
  verdict: {{pass|reject|deadlock|pending}}
status: draft|queued|approved|applied|rolled_back|rejected
```

## Storage

`~/.ciel/improvements/<id>.yaml`. Linked commits reference it in trailer `Proposal-Id: {{id}}`.
