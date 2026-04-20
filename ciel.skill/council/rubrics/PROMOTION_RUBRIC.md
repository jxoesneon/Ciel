# PROMOTION_RUBRIC

Criteria and scoring weights for local → global promotion.

## Triggering Scope

Invoked via `invocation_scopes/PROMOTION.md`.

## Criteria

A local learning is considered for promotion when:

1. **Universality evidence** — the learning has recurred in ≥ 2 distinct projects (tracked via MemPalace analogous-pattern search), OR matches a domain-agnostic pattern.
2. **Stability evidence** — the learning has produced improved outcome scores over ≥ N=10 invocations locally (`self_improvement/OUTCOME_SCORING.md`).
3. **Non-leaking** — promoting the learning would not expose project-specific info (names, secrets, internal URLs).
4. **Non-conflicting** — does not conflict with an existing global rule without Coherence member's approval.

## Per-Lens Scoring (Council synthesis)

| Lens | Weight | Asks |
| --- | --- | --- |
| Coherence | 0.20 | Does the global repo have a natural slot for this? |
| Capability | 0.20 | Is the learning really globally applicable or only coincidentally? |
| Safety | 0.25 | Does promoting risk leaking project context or introducing drift? |
| Efficiency | 0.15 | Is the promoted form lean, not a copy of the whole local blob? |
| Evolution | 0.20 | Does this compound Ciel's general ability or is it project-idiosyncratic? |

## Pass Threshold

```yaml
promotion:
  pass_weighted_score: 7.0     # stricter than normal 6.5
  require_2_of_2_safety_efficiency: true
  universality_recurrences_min: 2
```

## Output

On pass, the learning is:

- **generalized** — stripped of project-specific identifiers,
- committed to `~/.ciel/` under an appropriate subtree,
- annotated with provenance (projects it came from, hashed),
- replicated into the global MemPalace partition.

## Reject Path

Rejected promotions remain local and are tagged `promotion_rejected:<run_id>` so the improvement loop won't re-propose them without new evidence.
