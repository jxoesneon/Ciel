# ROLLBACK

Reverting a self-modification that regressed.

## Triggers

- `REGRESSION_DETECTION.md` confirmed regression.
- Safety member raised post-change flag.
- User explicitly requested `/ciel-rollback <commit>`.
- Integrity check discovered silent corruption introduced by a self-mod.

## Procedure

1. **Identify commits** — all commits from `self-mod/<tag>` to HEAD for the affected component.
2. **Assess dependents** — any downstream skills relying on the mutated one; list them.
3. **Generate revert plan** — `git revert --no-commit <commits>` from most-recent to oldest.
4. **Dry-run** — integrity check on the would-be state.
5. **Council-gate** — run `council/invocation_scopes/SELF_MODIFICATION.md` on the revert (reverting a self-mod is itself a self-mod).
6. **Apply** — `git revert` with message `rollback: <original_subject> (regression observed)`.
7. **Post-rollback integrity** — full integrity check.
8. **Observe** — new watch window (half the normal length) to confirm stability.

## Data Preservation

No hard resets; only reverts. The original problematic state is preserved in history for inspection.

## Rollback of Rollback

If the rollback itself regresses (rare), escalate to user — we have a pathological condition and human judgment is required.

## Memory Side Effects

Memory operations that accompanied the mutation (new MemPalace entries) are tagged `tentative: true` on creation. On rollback, tentative entries are either:

- promoted to permanent if they carried general value,
- cold-archived if they were only useful under the reverted state.

Council chooses on rollback runs.
