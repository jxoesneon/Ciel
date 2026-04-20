# SELF_IMPROVEMENT — Master

Ciel's evolution loop. Every meaningful interaction is a candidate growth signal.

## Domains

- **Global** — `GLOBAL_IMPROVEMENT.md` — `~/.ciel/`.
- **Local** — `LOCAL_IMPROVEMENT.md` — `.ciel/`.

## Loop

```text
meaningful_interaction (core/AWARENESS.md)
      │
      v
growth_signal detected?  (TRIGGERS.md)
      │ yes
      v
outcome_scoring          (OUTCOME_SCORING.md)
      │
      v
diagnose → improvement_proposal
      │
      v
Council-gate if non-trivial (invocation_scopes/SELF_MODIFICATION.md or SKILL_INTEGRATION.md)
      │ pass
      v
apply (git commit)
      │
      v
observe post-change outcomes (REGRESSION_DETECTION.md)
      │
      v
rollback? (ROLLBACK.md) — only on regression
```

## Domain Decision

Where does the improvement land?

- Learning is project-specific → `LOCAL_IMPROVEMENT.md`.
- Learning is universal → `GLOBAL_IMPROVEMENT.md` directly OR via `council/invocation_scopes/PROMOTION.md`.
- Learning touches a locked file → Council + user (Constitutional amendment).

## Cadence

- **Event-driven** — triggers fire improvements.
- **Periodic** — weekly sweep proposals from accumulated signals.
- **Manual** — `/ciel-diff` and `/ciel-improve` user invocations.

## Cost Guardrails

`improvement.config.max_proposals_per_day` caps the number of Council runs from improvement alone. Beyond that, proposals queue and batch.

## Observability

Every improvement has:

- trigger event,
- proposal diff,
- Council run record,
- apply/reject record,
- post-change outcome score delta.

All in `~/.ciel/improvements/<id>/`.
