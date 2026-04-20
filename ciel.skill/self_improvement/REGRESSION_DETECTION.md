# REGRESSION_DETECTION

After every Ciel-applied change, watch for regression. Roll back on confirmation.

## Watch Window

After a mutation commit, Ciel observes for:

- **N = `regression.watch_invocations`** (default 20) invocations of the affected skill/component, OR
- **T = `regression.watch_hours`** (default 48 hours),

whichever is longer.

## Comparison

- Baseline: rolling average pre-change.
- Post: rolling average across the watch window.

Computed per dimension (`OUTCOME_SCORING.md`).

## Regression Criteria

- `success_rate` drops > 10%.
- `avg_ms` grows > 50% without corresponding capability gain.
- `safety_observed` drops below 0.95.
- Any new Safety flag observed on the component.

## LLM Judgment

For ambiguous cases, `prompts/self_improvement/regression_judgment.md` is run against a git diff + the before/after metric set. Produces pass/fail + rationale.

## Action on Regression

1. Commit a `regression_detected:` marker (no file changes).
2. If affected component is non-locked → auto-propose rollback via `ROLLBACK.md`.
3. If locked → escalate with the evidence bundle.
4. Safety flag post-change → immediate rollback regardless of LLM judgment.

## Positive Cases

A clean watch window results in:

- `improvement:confirmed` commit marker,
- baseline update with the new averages,
- Evolution member logged a positive signal for future decisions.
