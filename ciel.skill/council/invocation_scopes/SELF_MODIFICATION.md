# SCOPE — SELF_MODIFICATION

Triage: Ciel is modifying her own core configuration, router, council rubrics, constitution, or other locked / quasi-locked material.

## Trigger

- `self_improvement/SELF_IMPROVEMENT.md` proposed a diff touching `core/`, `router/ROUTER.md`, `council/*`, `risk/CLASSIFICATION.md`, `memory/MEMPALACE.md`, or `domains/ISOLATION.md`.
- User invoked `/ciel-diff` and accepted a proposed self-edit.
- Integrity check via `init/INTEGRITY.md` proposed a repair.

## Preamble

> You are evaluating a **Ciel self-modification**. This diff changes Ciel's own behaviour. Target file(s): `<...>`. Locked status: `<true|false>`. Apply your lens with extra rigor — Ciel's core must not drift without strong cause.

## Elevated Thresholds

```yaml
self_modification:
  pass_weighted_score: 7.5     # stricter
  safety_min: 7                # no borderline Safety allowed
  majority_required: 4         # of 5 must score >= pass_score
```

## Locked File Handling

If target file has `locked: true` (see `core/CONSTITUTION.md`), Council pass alone is insufficient. Result is: `queued_for_user_confirmation`. The user must explicitly approve via `/ciel` or runtime confirmation before commit.

## Stage 1

All five lenses review. Safety in particular scrutinizes:

- rollback availability (git revert trivially works?),
- blast radius if wrong,
- whether the change weakens any existing guardrail.

## Stage 3

Chairman synthesizes. On pass:

- non-locked: commit with message `self_mod: <file>: <summary>`.
- locked: wait for user; on user approval, commit with message `CONSTITUTIONAL AMENDMENT: <summary>`.

## Rollback

Self-modification commits are immediately tagged `self-mod/<timestamp>` so rollback is discoverable. `self_improvement/ROLLBACK.md` defines the auto-rollback criteria.
