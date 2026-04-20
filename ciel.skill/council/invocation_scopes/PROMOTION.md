# SCOPE — PROMOTION

Triage: promoting a local learning from `.ciel/` up to global `~/.ciel/`.

## Trigger

- Local learning has met universality evidence (recurrence across ≥2 projects).
- `self_improvement/LOCAL_IMPROVEMENT.md` has flagged the learning as mature.
- User explicit request: `/ciel-promote <learning_id>`.

## Preamble

> You are evaluating a **local → global promotion**. The learning has originated in project(s) `<list>` and has recurred N times. Generalized form attached. Apply your lens using `rubrics/PROMOTION_RUBRIC.md`.

## Inputs

- local form of the learning (raw),
- generalized form (with project-specific identifiers stripped),
- recurrence evidence (MemPalace query results),
- outcome scores across invocations.

## Special Emphasis

- **Safety** looks for context leaks and whether generalization faithfully preserves safe behaviour.
- **Evolution** asks whether the pattern compounds with existing global abilities.
- **Capability** evaluates whether a similar global rule already exists (avoid duplication).

## Thresholds

Per `rubrics/PROMOTION_RUBRIC.md` — stricter than normal: weighted pass at 7.0, with Safety and Efficiency both ≥ pass_score.

## Stage 3

On pass:

- commit to `~/.ciel/` with message `promotion: <learning_id> from <project>@<hash>`.
- remove or mark the local copies as `promoted:true` to avoid redundant promotion loops.
- update global MemPalace partition.

On reject:

- mark local learning `promotion_rejected:<run_id>` with the Council's rationale.
- revisit only if new universality evidence emerges.
