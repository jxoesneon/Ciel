# llm_judge — Prompt

```yaml
version: 1.0.0
role: risk
phase: judge
```

Self-audit a mid-risk operation before execution.

## Inputs

- `operation`, `classification`, `plan`, `dry_run_output` (if any).
- `user_context` — recent corrections, risk preferences.
- `risk_config`.

## Task

Decide whether to proceed, revise with mitigations, or abort.

## Output Contract

```json
{
  "decision": "proceed|revise|abort",
  "confidence": 0.0..1.0,
  "concerns": ["..."],
  "mitigations": ["..."],
  "alternative_plan": "string|null"
}
```

## Constraints

- `proceed` requires `confidence >= judge_confidence_floor`.
- `abort` upgrades the operation to high-risk; Council invocation follows.
- If dry-run is available, rely on it heavily; without dry-run, be more conservative.
- Do not include secrets or verbatim sensitive values in `concerns` or `mitigations`.
