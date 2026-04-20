# LOCAL_IMPROVEMENT

Improvement loop targeting `<project>/.ciel/`.

## What Changes Here

- Project rules (codified conventions).
- Project-scoped config overrides.
- Project-scoped skills (custom, local-only).
- Learnings in the local partition (candidates for promotion).
- Escalation override adjustments.

## Isolation

Changes here do not propagate to other projects unless promoted via `council/invocation_scopes/PROMOTION.md`.

## Flow

1. Trigger fires from local activity.
2. Proposal drafted targeting `.ciel/` file(s).
3. Category determines gating:
    - `trivial` — additive project rule (e.g., new forbidden-op pattern). Auto-apply with log.
    - `standard` — changes to local config or overrides. Council-gate with reduced rigour (3-member Stage 1 acceptable for pure-local non-safety changes, per `council.config.local_quorum_min`).
    - `promotion-candidate` — looks universal. Mark for promotion evaluation rather than apply locally.
4. Apply → `.ciel/` file update → local activity log entry.

## Promotion Candidates

When `LOCAL_IMPROVEMENT` detects:

- same pattern seen in other projects' histories (cross-partition semantic match),
- no project-specific identifier required in the generalized form,

it queues a `promotion-candidate` which is picked up by the Council through `PROMOTION.md`.

## Local Skills

Projects can have their own skills (e.g., project-specific build scripts wrapped as skills). They live at `<project>/.ciel/skills/` with scope `local` and are never routed outside the project.

## Rate Limiting

`improvement.config.local_max_per_day` (default 10) per project.

## Privacy

Local learnings may reference project paths, file names, and repo-specific terminology. This is fine within the local domain; it never leaks until promotion stripping.
