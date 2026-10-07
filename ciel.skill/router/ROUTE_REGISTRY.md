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

# Active Registered Route: taste
route_id: route_taste_domain_v1
matcher:
  triggers: ["taste", "anti-slop", "frontend taste", "ui taste", "redesign ui"]
  compiled_pattern: "^(taste|taste.?skill|anti.?slop|clean.?ui|design.?dials|redesign.?ui)$"
  tags: ["frontend", "design", "ui", "ux", "anti-slop", "accessibility"]
target_skill: taste
trigger_confidence: 0.95
path_used: fast
hits: 1
last_hit: 2026-10-01T22:30:00Z
avg_confidence: 0.95
avg_ms: 5
success_rate: 1.00
notes: "Anti-slop frontend engineering directive registered via dual council pass"

# Active Registered Route: impeccable
route_id: route_impeccable_domain_v1
matcher:
  triggers: ["impeccable", "design audit", "design critique", "polish ui", "harden ui", "design review"]
  compiled_pattern: "^(impeccable|design.?audit|design.?critique|polish.?ui|harden.?ui|design.?review)$"
  tags: ["design", "frontend", "ui", "ux", "anti-slop", "audit"]
target_skill: impeccable
trigger_confidence: 0.95
path_used: fast
hits: 0
avg_confidence: 0.95
avg_ms: 0
success_rate: 1.00
notes: "Anti-slop design-quality layer acquired from pbakaus/impeccable (Apache-2.0); trust state untrusted pending paired eval"

# Active Registered Route: frontend-design
route_id: route_frontend_design_domain_v1
matcher:
  triggers: ["frontend design", "visual design", "design direction", "aesthetic direction", "ui aesthetics"]
  compiled_pattern: "^(frontend.?design|visual.?design|design.?direction|aesthetic.?direction|ui.?aesthetics)$"
  tags: ["design", "frontend", "ui", "aesthetics"]
target_skill: frontend-design
trigger_confidence: 0.95
path_used: fast
hits: 0
avg_confidence: 0.95
avg_ms: 0
success_rate: 1.00
notes: "Anthropic official frontend-design skill (Apache-2.0)"

# Active Registered Route: ui-ux-pro-max
route_id: route_ui_ux_pro_max_domain_v1
matcher:
  triggers: ["ui ux", "design intelligence", "color palette", "font pairing", "ux guidelines", "chart type", "ui style"]
  compiled_pattern: "^(ui.?ux.?pro.?max|uipro|design.?intelligence|color.?palette|font.?pairing|ux.?guidelines|chart.?type|ui.?style)$"
  tags: ["design", "frontend", "ui", "ux", "reference-data"]
target_skill: ui-ux-pro-max
trigger_confidence: 0.95
path_used: fast
hits: 0
avg_confidence: 0.95
avg_ms: 0
success_rate: 1.00
notes: "UI/UX design intelligence with local searchable data (MIT); trust state untrusted pending paired eval"

# Active Registered Route: emil-design-eng (motion family hub)
route_id: route_emil_motion_family_v1
matcher:
  triggers: ["animate", "animation", "motion", "micro-interaction", "spring", "gesture", "easing", "review animations", "improve animations", "find animation opportunities", "animation vocabulary", "apple design", "emil design eng"]
  compiled_pattern: "^(animate|animate.?expo|animation.?vocabulary|apple.?design|emil.?design.?eng|find.?animation.?opportunities|improve.?animations|review.?animations|motion|spring|gesture|easing)$"
  tags: ["motion", "animation", "design-engineering", "frontend", "ui"]
target_skill: emil-design-eng
trigger_confidence: 0.90
path_used: fast
hits: 0
avg_confidence: 0.90
avg_ms: 0
success_rate: 1.00
notes: "Emil Kowalski motion/interaction family hub; sub-skills dispatch by name (animate, review-animations, improve-animations, find-animation-opportunities, animation-vocabulary, apple-design, animate-expo)"

# Active Registered Route: e2e (tester-army agentic e2e runner)
route_id: route_e2e_agentic_v1
matcher:
  triggers: ["e2e", "tester-army", "e2e.config", "agent.act", "agentic e2e", "natural language e2e", "bug bash", "e2e explore", "e2e mcp", "e2e replay"]
  compiled_pattern: "^(e2e|tester.?army|tester.?army.?e2e)$"
  tags: ["testing", "e2e", "agentic", "browser", "mobile", "bug-bash", "qa"]
target_skill: e2e
trigger_confidence: 0.90
path_used: fast
hits: 0
avg_confidence: 0.90
avg_ms: 0
success_rate: 1.00
notes: "tester-army/e2e agentic e2e framework skill (Apache-2.0); Tier-3 acquisition, council pass 8.05 weighted; trust state untrusted pending paired eval"
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
