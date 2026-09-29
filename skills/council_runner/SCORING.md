# SCORING

Canonical 0–10 scale and pass/reject thresholds for the Council of Five.

## Scale

| Score | Meaning |
| --- | --- |
| 10 | exemplary, lens-perfect |
| 8–9 | strong pass for this lens |
| 6–7 | acceptable with noted mitigations |
| 5 | borderline |
| 4 | subpar; requires rework |
| 2–3 | unacceptable; Safety ≤ 3 = veto |
| 0–1 | malformed or actively harmful |

## Pass Thresholds

```yaml
pass_score: 6          # individual member threshold
weighted_pass: 6.5     # chairman weighted-score threshold
reject_threshold: 4.5  # chairman weighted-score reject threshold
majority_required: 3   # of 5 members scoring >= pass_score
```

## Accuracy-as-Ceiling Rule

Safety's score is a ceiling on the Chairman's verdict authority:

- Safety ≤ 3 → reject, no override.
- Safety 4–5 → weighted pass if average threshold met (refer to COUNCIL.md).
- Safety ≥ 6 → normal synthesis applies.

Inspired by `llm-council`'s ADR-016.

## Weights (Chairman synthesis)

```yaml
weights:
  coherence:  0.20
  capability: 0.20
  safety:     0.25
  efficiency: 0.15
  evolution:  0.20
```

## Rationale Requirements

Every score must be accompanied by rationale. Scores without rationale are discarded (treated as abstention by Chairman).

## Flag Taxonomy

See each member's file for lens-specific flags. Flags are structured signals the Chairman and self-improvement loop can regex-match on, distinct from prose rationale.
