# ACQUISITION — Pipeline Master

Tiered skill acquisition pipeline. Invoked by `router/ACQUISITION_PATH.md` on gap detection.

## Pipeline

```text
gap_description  →  tier 1 (curated)  →  tier 2 (MCP)  →  tier 3 (web)
                                           │              │
                                           v              v
                                      composition    composition
                                           │              │
                                           v              v
                                      harmonization  harmonization
                                           │              │
                                           v              v
                                      trust model / sandbox
                                           │
                                           v
                                      Council of Five (SKILL_INTEGRATION)
                                           │
                                     pass  v  fail
                                           v      └─→ discard + log
                                      register
                                           │
                                           v
                                      route & execute
```

## Tier Order

0. **Tier 0** — local skill discovery (`LOCAL_DISCOVERY.md`). Ingest existing skills from runtimes.
1. **Tier 1** — curated registry (`TIER_1_REGISTRY.md`). Fastest, highest trust.
2. **Tier 2** — MCP server discovery (`TIER_2_MCP.md`). Medium trust if server is known.
3. **Tier 3** — web extraction + synthesis (`TIER_3_WEB.md`). Lowest trust; must pass sandbox.

## Composition vs Acquire-Whole

Ciel prefers composing new skills from existing fragments + one or two new primitives over importing a monolithic blob. See `COMPOSITION.md`. Whole-skill acquisition is allowed but scored lower by Efficiency member in Council unless justified.

## Harmonization

Every acquired artifact passes `HARMONIZATION.md` before Council. Shape, docs, triggers, tags aligned to Ciel conventions.

## Trust Gate

`TRUST_MODEL.md` + `SANDBOX.md`. Tier-dependent enforcement:

- Tier 1: metadata validation + light sandbox.
- Tier 2: full schema validation + sandbox with declared side-effects.
- Tier 3: full sandbox, network denied by default, isolated fs.

## Budgets

```yaml
acquisition:
  tier1_timeout_s: 10
  tier2_timeout_s: 30
  tier3_timeout_s: 120
  total_wall_budget_s: 300
  token_budget: 80000
```

Exceeded budgets degrade to the next tier or escalate to user.

## Output

On successful Council pass:

- New `.skill` file installed in `~/.ciel/skills/<id>/`
- Trigger generation via `TRIGGER_GENERATOR.md`
- Registry updated (`ROUTE_REGISTRY.md`, `TRIGGER_REGISTRY.md`)
- MemPalace embedding computed
- Git committed

On Council reject: the artifact is retained at `~/.ciel/.attic/acquired_rejected/<run_id>/` for audit, and `SOURCES.md` lowers the origin's trust.
