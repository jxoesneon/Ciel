# ACQUISITION_PATH

Gap detected. The registry cannot satisfy this request. Acquire.

## Entry

Invoked when:

- fast path < floor AND reasoning path < floor, OR
- reasoning path identified explicit `gaps`, OR
- user explicitly requests acquisition ("find me a skill for X").

## Pipeline

Defers to `acquisition/ACQUISITION.md`:

```text
gap_detection (prompts/router/gap_detection.md)
      ↓
tier 1 — curated registry  (acquisition/TIER_1_REGISTRY.md)
      ↓ (miss)
tier 2 — MCP discovery     (acquisition/TIER_2_MCP.md)
      ↓ (miss)
tier 3 — web extraction    (acquisition/TIER_3_WEB.md)
      ↓
composition                 (acquisition/COMPOSITION.md)
      ↓
harmonization               (acquisition/HARMONIZATION.md)
      ↓
trust gate / sandbox        (acquisition/TRUST_MODEL.md, SANDBOX.md)
      ↓
Council of Five             (council/invocation_scopes/SKILL_INTEGRATION.md)
      ↓  (pass)
register                    (registry/REGISTRY.md)
      ↓
route and execute           (back to router/ROUTER.md)
```

## Hand-off

The router packages:

- the gap description,
- the failed fast-path and reasoning-path diagnostics,
- project context and runtime,
- risk classification,

and hands the bundle to `acquisition/ACQUISITION.md`.

## Partial Success

If acquisition succeeds for some gaps but not others, the router:

- executes the satisfied sub-plan,
- escalates the remaining gaps to the user with full research record,
- commits the new skill(s) regardless — they are still valuable future captures.

## Failure

If all three tiers miss:

- the router generates a **research report** (`seed_skills/research/SKILL.md`),
- the report is attached to the user escalation,
- no Council is invoked for a non-acquisition.

## Budget

`acquisition.config.md` sets per-tier time and token budgets. Exceeding budget degrades to the next tier or escalates.

## Telemetry

```json
{ "path": "acquisition", "tier_hit": 3, "council_pass": true, "new_skill": "<id>", "ms": 22040 }
```
