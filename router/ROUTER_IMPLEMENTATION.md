# ROUTER_IMPLEMENTATION — Devin Runtime Guide

How the Ciel router works in practice for the `devin-for-terminal` runtime. This document is the operational reference for Phase 6 (Route) of the invocation contract.

## Overview

The router is the spine of Ciel. Every invocation passes through it. It uses a hybrid three-path architecture with early exit:

1. **Fast path** — deterministic registry lookup (trigger + tag matching).
2. **Reasoning path** — LLM-driven composition over the registry.
3. **Acquisition path** — tiered search for a new skill when the registry cannot satisfy the request.

Paths are tried in order. The first path that clears its confidence floor wins. A miss is never an error — it is a normal fall-through signal.

## Runtime Artifacts

| Artifact | Path | Purpose |
| --- | --- | --- |
| Route Registry | `~/.ciel/registry/ROUTE_REGISTRY.json` | All skill routes with triggers, tags, domain, confidence, hit stats |
| Trigger Registry | `~/.ciel/registry/TRIGGER_REGISTRY.json` | All activation trigger patterns with match type and confidence |
| Router State | `~/.ciel/router_state.json` | Current runtime, floor thresholds, context budget, statistics |
| Activity Log | `~/.ciel/activity.log` | Append-only telemetry for every router event |
| Router Config | `ciel.skill/configuration/global/router.config.md` | Source of truth for floor values and budgets |

## 1. Request Entry

A request enters the router as an envelope:

```yaml
request:
  text: "..."                  # user or agent input
  host_runtime: devin-for-terminal  # from RUNTIME_DETECTION.md
  project_ctx:
    language: ...
    framework: ...
    rules: [...]
  history_ptr: <mempalace-key>
  risk_hint: optional
```

### Entry Sequence

1. **Runtime detection** — if `router_state.current_runtime` is unset or the session is new, run `RUNTIME_DETECTION.md`. For Devin, the probe checks `DEVIN_VERSION` / `DEVIN_CLI=1` / `DEVIN_API_KEY` env vars, `.devin/` directory fingerprints, and `devin --version` binary probe. Result is cached in `router_state.json`.
2. **Risk classification** — classify the request per `risk/CLASSIFICATION.md`. High-risk requests get additional gating.
3. **Normalization** — lowercase and tokenize `request.text` for trigger matching.
4. **Load registries** — load `ROUTE_REGISTRY.json` and `TRIGGER_REGISTRY.json` into memory (L0 metadata only per `CONTEXT_BUDGET.md`).

## 2. Trigger Matching

Trigger matching is the first and fastest routing mechanism. It uses the `TRIGGER_REGISTRY.json` patterns.

### Match Order

Patterns are matched in priority order (highest confidence first):

1. **Direct** (confidence 0.95–1.0) — exact skill names and aliases. Example: `"git"`, `"blueprint"`.
2. **Functional** (confidence 0.7–0.9) — capability descriptions. Example: `"(route|orchestrate).*(skill|agent)"`, `"search web"`.
3. **Domain** (confidence 0.6–0.8) — subject matter. Example: `"docker"`, `"git"`, `"markdown"`.
4. **Intent** (confidence 0.5–0.7) — user goals. Example: `"(self.?improve|evolve).*(skill|system)"`.

### Matching Algorithm

```text
for each trigger in TRIGGER_REGISTRY (sorted by confidence desc):
    if trigger.pattern matches normalized request.text:
        candidate = trigger.skill_name
        confidence = trigger.confidence
        apply scoring modifiers:
            + word_order_match bonus (+0.05)
            + context_alignment bonus (+0.15 if project_ctx matches skill domain)
            + history_bonus (+0.10 if skill recently used successfully)
            - frequency_penalty (-0.05 if pattern is overused generic)
            - ambiguity_penalty (-0.15 if multiple skills match)
        if adjusted_confidence >= fast_path_floor (0.80):
            → route to candidate skill (FAST PATH HIT)
            break
```

### Pattern Types

- **Exact keywords** — plain string match against tokenized request (e.g., `"git"`, `"research"`).
- **Regex patterns** — compiled regex for complex patterns (e.g., `"(route|orchestrate).*(skill|agent)"`).
- **Composite patterns** — multi-pattern workflows (e.g., `["find.*skill", "then.*use.*it"]`).

### Conflict Resolution

When multiple triggers match with confidence within 0.05:

1. Prefer the skill with higher historical success on this project context.
2. If still tied, prefer the leaner skill (smaller unpacked byte size).
3. If still tied, hand to the reasoning path as a composition problem.

## 3. Fast Path Lookup (Tag-Based)

If trigger matching does not clear the floor, the fast path falls back to tag-based lookup using `ROUTE_REGISTRY.json`.

### Lookup Order

1. **Exact trigger match** — check each route's `triggers` array against normalized request tokens.
2. **Tag intersection** — rank candidates by tag-match cardinality × usage-recency weight. Tags come from each route's `tags` field.
3. **Input-contract match** — filter candidates by I/O contract compatibility with the request shape.
4. **Confidence score** — compute:

```text
confidence = 0.4 * trigger_match
           + 0.3 * tag_intersection
           + 0.2 * contract_compat
           + 0.1 * recency_bias
```

### Floor

`router_state.fast_path_floor` = **0.80** (from `router.config.md`). Below floor → fall through to reasoning path.

### Cache

- In-memory LRU of last 64 `(request_hash → skill_id)` pairs for the current session.
- Cache is process-local, not persisted.
- TTL: 60 minutes (from `router.config.md` `cache_ttl_minutes`).

### Telemetry

Every fast-path attempt logs:

```json
{ "path": "fast", "confidence": 0.87, "candidate": "git/SKILL.md", "hit": true, "ms": 12 }
```

## 4. Reasoning Path

Invoked when the fast path misses (confidence < 0.80). The reasoning path uses the host LLM to compose a plan from existing registry skills.

### Inputs

- Request envelope.
- Registry metadata slice (L0 for all skills + L1 for top-K candidates, K=5) — skill name, triggers, tags, domain, one-line description.
- Recent execution trace from MemPalace for continuity.

### Process

1. The LLM is prompted (via `prompts/router/reasoning_path.md`) to:
   - Identify the atomic sub-tasks the request decomposes into.
   - For each sub-task, match the best registered skill OR mark as **gap**.
   - Produce a plan: ordered list of `(skill_id, input_shape)`.
   - Return a self-reported confidence 0..1.
2. The output contract:

```json
{
  "plan": [ { "skill": "git/SKILL.md", "input": {} } ],
  "gaps": [ { "subtask": "...", "reason": "..." } ],
  "confidence": 0.72
}
```

### Confidence Floor

`router_state.reasoning_floor` = **0.70** (from `router.config.md`).

- **≥ floor, no gaps** → execute the plan sequentially.
- **≥ floor, with gaps** → partial-execute known steps, route gaps to acquisition path.
- **< floor** → route the whole request to acquisition path.

### Composition

Multi-step plans execute sequentially by default. Steps marked independent can be parallelized — the Devin runtime adapter supports up to 5 concurrent subagents.

### Cost Awareness

Reasoning path consumes LLM tokens. Budgeted by `router_state.context_budget_tokens` = **32000**. Exceeding budget escalates to user with an explanation.

### Telemetry

```json
{ "path": "reasoning", "confidence": 0.78, "steps": 3, "gaps": 0, "ms": 940 }
```

## 5. Acquisition Path

Invoked when both fast path and reasoning path miss, or when the reasoning path identifies explicit gaps.

### Pipeline (Tiered)

```text
gap_detection
      ↓
tier 1 — curated registry  (check existing skills with broader search)
      ↓ (miss)
tier 2 — MCP discovery     (search MCP servers for matching tools)
      ↓ (miss)
tier 3 — web extraction    (search the web, extract skill from docs)
      ↓
composition                 (assemble skill from found components)
      ↓
harmonization               (standardize to Ciel skill/1.0 format)
      ↓
trust gate / sandbox        (security review, sandbox evaluation)
      ↓
Council of Five             (gated approval for new skill registration)
      ↓ (pass)
register                    (add to ~/.ciel/skills/, update registries)
      ↓
route and execute           (back to router with the new skill)
```

### Hand-off Package

The router packages and hands to the acquisition pipeline:
- The gap description.
- Failed fast-path and reasoning-path diagnostics.
- Project context and runtime.
- Risk classification.

### Partial Success

If acquisition succeeds for some gaps but not others:
- Execute the satisfied sub-plan.
- Escalate remaining gaps to the user with full research record.
- Commit new skills regardless — they are valuable future captures.

### Failure (All Tiers Miss)

- Generate a research report (`seed_skills/research/SKILL.md`).
- Attach to user escalation.
- No Council invocation for a non-acquisition.

### Telemetry

```json
{ "path": "acquisition", "tier_hit": 3, "council_pass": true, "new_skill": "<id>", "ms": 22040 }
```

## 6. Post-Execution Scoring

After any path executes a skill, the router scores the outcome per `self_improvement/OUTCOME_SCORING.md`.

### Scoring Process

1. **Capture outcome** — success/failure, execution time, side effects, user feedback (if any).
2. **Compute outcome score** — 0..1 based on success, correctness, efficiency.
3. **Update ROUTE_REGISTRY** — for the matched route:
   - Increment `hit_count`.
   - Set `last_hit` to current timestamp.
   - Update `avg_confidence` as exponential moving average (α = 0.2).
   - Update `success_rate` based on outcome score.
4. **Update TRIGGER_REGISTRY** — for the matched trigger:
   - Increment `hit_count`.
   - Set `last_hit` to current timestamp.
5. **Growth signal** — if the outcome reveals a new capability gap, enqueue a growth-signal for `core/AWARENESS.md`.
6. **Self-update check** — `self_improvement/TRIGGERS.md` watches for:
   - `success_rate` drop > 10% over last 20 invocations.
   - `avg_confidence` drop > 15%.
   - A tag whose route distribution drifts heavily toward reasoning/acquisition (suggests missing fast-path entry).
   - Orphan routes (hits = 0 for > 30 days → pruning candidate).

## 7. Activity Log Entries

Every router event appends a line to `~/.ciel/activity.log`. Format:

```
<ISO-8601 timestamp>|<EVENT_TYPE>|<key=value;...>|<context>
```

### Event Types

| Event | Format |
| --- | --- |
| Router init | `<ts>\|ROUTER_INIT\|route_registry=<N>,trigger_registry=<N>\|runtime=<id>` |
| Fast path hit | `<ts>\|FAST_PATH_HIT\|skill=<name>,confidence=<f>,ms=<n>\|runtime=<id>` |
| Fast path miss | `<ts>\|FAST_PATH_MISS\|confidence=<f>,ms=<n>\|runtime=<id>` |
| Reasoning path hit | `<ts>\|REASONING_HIT\|skills=<list>,confidence=<f>,steps=<n>,ms=<n>\|runtime=<id>` |
| Reasoning path miss | `<ts>\|REASONING_MISS\|confidence=<f>,ms=<n>\|runtime=<id>` |
| Acquisition path | `<ts>\|ACQUISITION\|tier=<n>,new_skill=<name>,council_pass=<bool>,ms=<n>\|runtime=<id>` |
| Execution error | `<ts>\|EXEC_ERROR\|skill=<name>,error=<msg>\|runtime=<id>` |
| Outcome score | `<ts>\|OUTCOME\|skill=<name>,score=<f>,success=<bool>\|runtime=<id>` |

### Statistics Update

After each event, `router_state.json` statistics are updated:
- `total_requests` incremented.
- The matching path counter (`fast_path_hits` / `reasoning_path_hits` / `acquisition_path_hits`) incremented on a hit.
- `cache_misses` incremented when the LRU cache misses.

## Context Budget Management

The router respects `CONTEXT_BUDGET.md` — progressive disclosure with three load levels:

| Level | Size | Contents |
| --- | --- | --- |
| L0 — metadata | ~150 tokens | Skill id, triggers, tags, domain, one-line description |
| L1 — description | ~500 tokens | Frontmatter + intro prose + capability list |
| L2 — full | full size | Entire SKILL.md + sub-files |

### Load Policy

| Path | Load Level |
| --- | --- |
| Fast path | L0 for whole registry; L1 for top-5 candidates; L2 only for chosen skill |
| Reasoning path | L0 for full registry + L1 for model-flagged candidates |
| Acquisition path | L2 for the acquired skill under evaluation only |

Budget: `router_state.context_budget_tokens` = **32000** total. Segment budgets:
- Registry L0: 4000 tokens
- Candidate L1 (K=5): 3000 tokens
- Council Stage 1: 8000 tokens
- Acquisition L2: 16000 tokens

Exceeding a segment triggers compression via `seed_skills/context_summarizer/SKILL.md`.

## Devin Runtime Specifics

The `devin-for-terminal` runtime has a dedicated adapter at `adapters/devin/ADAPTER.md` with:

- **Subagents**: up to 5 concurrent (maps to reasoning path parallel execution).
- **MCP**: supported (maps to acquisition tier 2).
- **Hooks**: supported (lifecycle events).
- **Plan mode**: supported (budget: 8000 tokens).
- **Browser automation**: supported (maps to `seed_skills/browser-qa/SKILL.md`).
- **Web research**: supported (maps to acquisition tier 3).
- **Sandbox**: supported (trust gate for acquired skills).

## Quick Reference — Floor Values

| Threshold | Value | Source |
| --- | --- | --- |
| Fast path floor | 0.80 | `router.config.md` |
| Reasoning floor | 0.70 | `router.config.md` |
| Acquisition threshold | 0.60 | derived (below reasoning floor) |
| Prompt cache floor | 0.50 | `router.config.md` |
| Context budget | 32000 tokens | `router.config.md` |
| Cache TTL | 60 minutes | `router.config.md` |

## Related Specs

- `ciel.skill/router/ROUTER.md` — master router architecture
- `ciel.skill/router/FAST_PATH.md` — fast path details
- `ciel.skill/router/REASONING_PATH.md` — reasoning path details
- `ciel.skill/router/ACQUISITION_PATH.md` — acquisition path details
- `ciel.skill/router/TRIGGER_SYSTEM.md` — comprehensive trigger system
- `ciel.skill/router/TRIGGER_GENERATOR.md` — dynamic trigger pipeline
- `ciel.skill/router/CONTEXT_BUDGET.md` — progressive disclosure
- `ciel.skill/router/RUNTIME_DETECTION.md` — runtime fingerprinting
- `ciel.skill/configuration/global/router.config.md` — config source of truth
