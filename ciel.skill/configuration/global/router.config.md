# router.config — Global Router

```yaml

# <anchor:start>

router:
  fast_path_floor: 0.80
  reasoning_floor: 0.70
  cache_ttl_minutes: 60
  cache_version: 1
  context_budget:
    total_max_tokens: 32000
    registry_l0_max: 4000
    candidate_l1_k: 5
    candidate_l1_max: 3000
    council_stage1_max: 8000
    acquisition_l2_max: 16000
  prompt_cache:
    floor: 0.50
  plan_mode:
    budget_tokens: 8000

# <anchor:end>

```

## Notes

- `fast_path_floor` — confidence required to take fast path. Raise to be pickier. Self-tunable in ±0.02 range.
- `reasoning_floor` — confidence required to execute an LLM-planned route without falling to acquisition. Raise for stricter.
- `context_budget.*` — enforced via eviction + compression. See `router/CONTEXT_BUDGET.md`.
- `cache_version` — bumped by Ciel to force prompt cache refresh after major mutations.

## Self-Tuning Triggers

- fast-path miss rate > 20% → consider lowering floor; self-improvement evaluates.
- prompt_cache hit ratio < 0.50 → inspect cache layout / invalidation cadence.
- context evictions > 10/session → raise context budgets or improve compression.
