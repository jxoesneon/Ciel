# Ciel Improvement Proposal: Rate Limit & Resource Awareness

**Trigger:** `RESOURCE_EXHAUSTED (code 429)` across multiple parallel subagent invocations.
**Date:** 2026-08-16
**Domain:** Global Orchestration (`~/.ciel/`)

## Diagnosis
The `router` and `invoke_subagent` execution paths were unconstrained regarding parallel agent fan-out. Deploying 4 Domain Coordinators which simultaneously spawned internal workers resulted in >8 concurrent requests against the `pro` model tier, triggering hard capacity limits and stalling the workflow.

## Resolution
Implement a hard cap on concurrent subagent deployment and enforce progressive degradation logic:
1. **Concurrency Cap:** The orchestrator must not exceed spawning 3 simultaneous `pro` tier subagents, or 5 `flash` tier subagents in a single turn.
2. **Batching:** Workloads demanding higher fan-out (e.g., 100+ files) must be batched sequentially or executed via deterministic scripts (Python/Bash) via the `run_command` tool to preserve API tokens.
3. **Model Degradation:** If a 429 error is detected, automatically throttle the loop, pause for 10 seconds, and degrade the next fallback batch to the `flash` model.

## Action Taken
Created `config/rate_limits.config.md` to permanently codify these resource constraints into the Ciel orchestration layer.
