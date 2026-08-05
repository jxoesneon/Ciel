# Ciel Self-Improvement Guide

## Overview

Ciel's evolution loop treats every meaningful interaction as a candidate growth signal. The loop detects signals, scores outcomes, proposes improvements, gates non-trivial changes through the Council, applies them, and watches for regressions.

## Loop

```
meaningful interaction
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
Council-gate if non-trivial (SELF_MODIFICATION or SKILL_INTEGRATION scope)
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

## Growth Signal Detection

Triggers fire from `TRIGGERS.md`:

| Trigger | Source | Threshold |
|---|---|---|
| `route_miss` | fast path misses in an area | rate > 20% over last 50 routings |
| `confidence_floor_breach` | reasoning path confidence | avg < 0.65 over last 20 |
| `outcome_regression` | post-execution score drop | > 10% drop vs baseline |
| `execution_error` | tool invocation fails | 3 consecutive or 5 in an hour |
| `safety_flag` | Safety member raised concern post-hoc | any |
| `council_flag` | any member flagged a systemic issue | any |
| `overlap_detected` | registry conflict | any |
| `orphan_route` | route not hit in 30 days | any |
| `novel_context` | project signature differs > threshold | any |
| `user_correction` | user rejected a proposal | any |
| `user_escalation` | user escalated because Ciel missed | any |
| `scheduled_sweep` | periodic | weekly |
| `capability_drift` | runtime or MCP behaviour changed | probe diff |
| `model_fallback` | repeated fallback to weaker model | > 5 in a session |
| `context_pressure` | frequent eviction events | > 10/session |

### Signal Aggregation
Triggers accumulate in MemPalace partition `ciel/improvements/signals/`. Aggregation groups related signals and produces proposal candidates.

### De-duplication
Identical signals (same file, same symptom) within `trigger_dedup_window` (default 6h) are merged.

### Suppression
Recently rejected proposals mark their signals `suppressed:<run_id>` for `suppression_days` (default 7). New evidence clears suppression.

### Priority
1. Safety flags
2. Outcome regressions
3. Conflict resolutions
4. Efficiency opportunities
5. Capability expansions
6. Hygiene (orphan pruning, coherence touch-ups)

## Improvement Proposals

Generated via `prompts/self_improvement/improvement_proposal.md`. Each proposal includes:
- Subject (one-line description)
- Trigger (what growth signal fired)
- Proposed change (detailed diff + rationale)
- Risk classification (trivial | standard | structural | constitutional)
- Council gate (required | not_required)
- Rollback plan
- Outcome scoring criteria

## Outcome Scoring

Post-execution scoring per `OUTCOME_SCORING.md`:

| Dimension | Weight | Measurement |
|---|---|---|
| success | 0.35 | exit code 0 + no error markers |
| correctness | 0.25 | LLM-judged match to intent (sampled) |
| side_effects | 0.15 | declared vs observed side effects |
| efficiency | 0.10 | ms vs skill's avg; tokens vs budget |
| user_satisfaction | 0.10 | implicit (no retry/correction) or explicit |
| safety_observed | 0.05 | no Safety flags triggered post-hoc |

`outcome_score = Σ weight_i * dim_i` in 0..1.

Stored per-invocation in MemPalace partition `ciel/traces/outcomes/<run_id>`. Baseline is a rolling EMA (α=0.1) across last 100 invocations.

Score drops beyond thresholds fire `outcome_regression` trigger.

## Regression Detection

After every Ciel-applied change, per `REGRESSION_DETECTION.md`:

### Watch Window
- N = `watch_invocations` (default 20) invocations of the affected skill, OR
- T = `watch_hours` (default 48 hours),
- Whichever is longer.

### Regression Criteria
- `success_rate` drops > 10%
- `avg_ms` grows > 50% without corresponding capability gain
- `safety_observed` drops below 0.95
- Any new Safety flag observed on the component

### Action on Regression
1. Commit `regression_detected:` marker
2. Non-locked component → auto-propose rollback
3. Locked component → escalate with evidence bundle
4. Safety flag post-change → immediate rollback regardless of LLM judgment

### Positive Cases
Clean watch window → `improvement:confirmed` commit marker + baseline update + Evolution member logs positive signal.

## Rollback

Per `ROLLBACK.md`:

### Triggers
- Confirmed regression
- Safety member post-change flag
- User `/ciel-rollback <commit>` request
- Integrity check discovered silent corruption

### Procedure
1. Identify commits from `self-mod/<tag>` to HEAD
2. Assess dependents (downstream skills)
3. Generate revert plan (`git revert --no-commit`)
4. Dry-run integrity check
5. Council-gate (reverting a self-mod is itself a self-mod)
6. Apply: `git revert` with `rollback: <subject> (regression observed)`
7. Post-rollback integrity check
8. New watch window (half normal length)

No hard resets; only reverts. Original problematic state preserved in history.

## Council Gate

Non-trivial improvements require Council approval under `SELF_MODIFICATION` or `SKILL_INTEGRATION` invocation scopes.

### Proposal Categories
| Category | Gating | Example |
|---|---|---|
| `trivial` | Auto-apply with log | Config fine-tuning within defaults (±10% = `auto_tune_range`) |
| `standard` | Council-gate (SELF_MODIFICATION) | Functional changes to non-locked files |
| `structural` | Full Council + wider watch window | Changes touching multiple components |
| `constitutional` | Council + explicit user confirmation | Touches locked files |

## Global vs Local Improvements

### Global (`GLOBAL_IMPROVEMENT.md` — `~/.ciel/`)
- Core identity files (Constitutional amendment)
- Router weights, path floors
- Council rubrics (non-locked portions)
- Global registry skills: add, update, deprecate, remove
- Adapters: update existing, build new
- Seed skills: tuning
- Acquisition sources: trust adjustments
- Rate limit: `global_max_per_day` (default 20)

### Local (`LOCAL_IMPROVEMENT.md` — `.ciel/`)
- Project rules (codified conventions)
- Project-scoped config overrides
- Project-scoped skills (custom, local-only)
- Learnings in local partition (candidates for promotion)
- Escalation override adjustments
- Rate limit: `local_max_per_day` (default 10)
- Isolation: changes do not propagate unless promoted via `PROMOTION.md`

### Domain Decision
- Learning is project-specific → Local
- Learning is universal → Global directly OR via `PROMOTION.md`
- Learning touches a locked file → Council + user (Constitutional amendment)

## Improvement Queue

The improvement queue holds pending proposals awaiting Council runs. Proposals are processed by priority (see above). When Council runs are rate-limited (`global_max_per_day` / `local_max_per_day`), excess proposals queue and batch.

## Cadence
- **Event-driven** — triggers fire improvements
- **Periodic** — weekly sweep proposals from accumulated signals
- **Manual** — `/ciel-diff` and `/ciel-improve` user invocations

## Observability

Every improvement has:
- trigger event
- proposal diff
- Council run record
- apply/reject record
- post-change outcome score delta

All stored in `~/.ciel/improvements/<id>/`.
