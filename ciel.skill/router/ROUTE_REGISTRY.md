# ROUTE_REGISTRY

Live map of routes, their conditions, hit rates, and freshness. Feeds self-update decisions.

## Schema

Each route entry:

```yaml
route_id: <hash>
matcher:
  triggers: [...]           # References TRIGGER_REGISTRY
  compiled_pattern: <regex>  # Pre-compiled for fast matching
  tags: [...]
  contract: {...}
target_skill: <skill_id>
trigger_confidence: 0.85    # From TRIGGER_REGISTRY scoring
path_used: fast | reasoning | acquisition
hits: 142
last_hit: 2026-01-15T09:24:00Z
last_miss: 2026-01-10T12:01:00Z
avg_confidence: 0.86
avg_ms: 18
success_rate: 0.97
notes: "..."
```

Stored in MemPalace partition `ciel/route_registry/` keyed by `route_id`. Indexed by `target_skill` and by tag.

## Hit Rate Tracking

Every router invocation updates:

- `hits` on a hit,
- `last_hit` / `last_miss`,
- `avg_confidence` as exponential moving average (α = 0.2),
- `avg_ms`,
- `success_rate` based on post-execution outcome score (`self_improvement/OUTCOME_SCORING.md`).

## Self-Update Signals

`self_improvement/TRIGGERS.md` watches for:

- `success_rate` drop > 10% over last 20 invocations,
- `avg_confidence` drop > 15%,
- a tag whose route distribution drifts heavily toward reasoning/acquisition (suggests missing fast-path entry),
- orphan routes (hits = 0 for > 30 days → candidate for pruning by Efficiency member).

## Pruning

Orphan pruning is proposed to the Council of Five via `council/invocation_scopes/SKILL_INTEGRATION.md` (reverse direction: de-registration). Safety veto on anything that references a skill still appearing in reasoning plans.

## Query Interface

Seed skill `registry/REGISTRY.md` exposes:

- `route.find(request_hash)`,
- `route.stats(skill_id)`,
- `route.drift(tag)`,
- `route.orphans(days=30)`.

These are called by the router and by the self-improvement loop.
