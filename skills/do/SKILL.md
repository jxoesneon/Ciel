---
name: do
version: 1.0.0
format: skill/1.0
description: Orchestrated, multi-agent execution of phased implementation plans.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "execute.*plan"

    confidence: 0.9

  - pattern: "do.*(implementation|tasks)"

    confidence: 0.9

  - pattern: "start.*(phase|execution)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Do (The Execution Layer)

This skill serves as CIEL's "Execution Layer," responsible for the systematic, high-fidelity implementation of approved plans. It prioritizes delegation, verification, and adherence to evidence over raw speed.

## Integration Context

Adapted from `~/.agents/skills/do/`. This skill is the final stage of the CIEL lifecycle, transforming a **make-plan** (Strategy) into a verified reality. It mandates "Delegated Verification" after every atomic change.

## Orchestration Logic

### 1. Plan Ingestion

The execution MUST start by reading the active implementation plan from `docs/ciel/plans/`. Each turn must focus on a single, well-defined phase.

### 2. Multi-Agent Delegation

CIEL acts as the Orchestrator, delegating discrete tasks to specialized sub-agents:

- **Implementer**: Carries out the code changes, following the plan's documentation references exactly.
- **Verifier**: Runs the phase's verification checklist (tests, grep checks) in a separate context.
- **Auditor**: Checks for anti-patterns and ensures alignment with **ai-first-engineering** principles.

### 3. Checkpoint Persistence

Each completed phase must be recorded in the session log and, if requested by the plan, committed to git ONLY after verification passes.

### 4. Memory Sync

Successful implementations must be indexed in **MemPalace** as `ImplementedFeature` nodes, linking back to the original `ProposedPlan` and `ProposedDesign`.

### 5. Transition

The terminal state of this skill is the invocation of the **verification-loop** for final E2E validation.

---
*Original Execution Protocol and Failure Mode prevention preserved at source.*
