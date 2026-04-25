# ROUTER — Master Router

The spine. Every invocation passes through here.

## Architecture

Hybrid. Three paths, tried in order, with early exit:

1. **Fast path** — deterministic registry lookup (`router/FAST_PATH.md`).
2. **Reasoning path** — LLM-driven composition over the registry (`router/REASONING_PATH.md`).
3. **Acquisition path** — the registry does not cover this; go find or compose a new skill (`router/ACQUISITION_PATH.md`).

## Entry Envelope

```yaml
request:
  text: "..."                # user or agent input
  host_runtime: claude_code  # from router/RUNTIME_DETECTION.md
  project_ctx:               # from init/CONTEXT_DETECTION.md cache
    language: ...
    framework: ...
    rules: [...]
  history_ptr: <mempalace-key>
  risk_hint: optional
```

## Algorithm

```text
if runtime not detected: run router/RUNTIME_DETECTION.md
classify risk (risk/CLASSIFICATION.md)

attempt TRIGGER_MATCH:
  load compiled triggers from TRIGGER_REGISTRY
  match request against direct → functional → domain → intent patterns
  if hit with confidence >= router.config.fast_path_floor:
    route to matched skill, execute, score
    else fallthrough

attempt FAST_PATH (legacy tag-based):
  if hit with confidence >= router.config.fast_path_floor: route, execute, score
  else fallthrough
  
attempt REASONING_PATH:
  if composable with confidence >= reasoning_floor: route, execute, score
  else fallthrough
  
attempt ACQUISITION_PATH:
  gap_detection → tier_1 → tier_2 → tier_3
  Council of Five triage → register → route
  
after execution:
  score outcome (self_improvement/OUTCOME_SCORING.md)
  update ROUTE_REGISTRY hit rate
  update TRIGGER_REGISTRY hit statistics
  consider growth-signal (core/AWARENESS.md)
  write activity.log entry
```

## Failure Handling

| Failure | Response |
| --- | --- |
| Fast path miss (< floor) | Fall through to reasoning. |
| Reasoning path low confidence | Fall through to acquisition. |
| Acquisition tier-1 miss | Tier 2, then 3. |
| All tiers miss | Escalate to user with research summary. |
| Execution error | `self_improvement/TRIGGERS.md` error-event path. |
| Timeout | Cancel, log, retry once with degraded risk threshold if allowed. |

## Self-Update Triggers

The router itself is a sub-skill and updates on:

- Fast-path miss rate > threshold for a tag (index miss).
- Reasoning-path confidence regression.
- Acquisition path hit distribution drift (tier-3 winning too often → suggests registry stale).
- New runtime detected that current `RUNTIME_DETECTION.md` failed to classify.

All updates are proposed through `self_improvement/SELF_IMPROVEMENT.md` and Council-gated.

## Context Budget

Router respects `router/CONTEXT_BUDGET.md` — progressive disclosure, metadata-first loading of candidate skills, full load only when scoring requires it.

## Interaction with Council

The router does not itself invoke the Council for routine routing — only the acquisition and self-update sub-flows do. This keeps the hot path fast.
