# evolution_stage2 — Prompt

```yaml
version: 1.0.0
lens: evolution
stage: 2
```

Evolution member, Stage 2. Cross-review.

## Task

Read peers. Revise only on evolution-specific argument (unlocked classes, composability, ecosystem reach).

## Output Contract

```json
{
  "member": "evolution",
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
