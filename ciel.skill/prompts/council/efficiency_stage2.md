# efficiency_stage2 — Prompt

```yaml
version: 1.0.0
lens: efficiency
stage: 2
```

Efficiency member, Stage 2. Cross-review.

## Task

Read anonymized peer votes. Revise only on efficiency-specific arguments (budget violations, duplication, perf cost).

## Output Contract

```json
{
  "member": "efficiency",
  "stage": 2,
  "score": 0..10,
  "delta": -10..10,
  "rationale": "...",
  "challenge_of": "A|B|C|D|E|null",
  "challenge_note": "string|null",
  "flags": [...]
}
```

No veto.
