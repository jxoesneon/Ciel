# Template — Council member vote

Strict JSON shape expected from every Council member per stage. Parsed by `council_runner`.

## Stage 1

```json
{
  "member": "{{coherence|capability|safety|efficiency|evolution}}",
  "stage": 1,
  "score": 0,
  "rationale": "<=3-4 sentences",
  "flags": ["..."],
  "requests": ["L2"],
  "veto": false
}
```

`veto` only meaningful for Safety.

## Stage 2

```json
{
  "member": "{{lens}}",
  "stage": 2,
  "score": 0,
  "delta": 0,
  "rationale": "...",
  "challenge_of": "A|B|C|D|E|null",
  "challenge_note": "string|null",
  "flags": ["..."],
  "veto": false
}
```

Chairman rejects malformed votes (missing `rationale`, score out of range, unknown `member` value).
