# TRIGGERS

When a meaningful interaction graduates to an improvement signal.

## Trigger Table

| Trigger | Source | Threshold |
| --- | --- | --- |
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
| `capability_drift` | runtime or MCP behaviour changed | detected via probe diff |
| `model_fallback` | repeated fallback to weaker model | > 5 in a session |
| `context_pressure` | frequent eviction events | > 10/session |

## Signal Aggregation

Triggers accumulate in MemPalace partition `ciel/improvements/signals/`. Aggregation job groups related signals and produces a proposal candidate.

## De-duplication

Identical signals (same file, same symptom) within `trigger_dedup_window` (default 6h) are merged into one candidate.

## Suppression

Recently rejected proposals mark their signals `suppressed:<run_id>` for `suppression_days` (default 7). New evidence clears suppression.

## Priority

Proposals are prioritised by:

1. Safety flags.
2. Outcome regressions.
3. Conflict resolutions.
4. Efficiency opportunities.
5. Capability expansions.
6. Hygiene (orphan pruning, coherence touch-ups).
