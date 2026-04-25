---
name: autonomous-loops
version: 1.0.0
format: skill/1.0
description: Architectural patterns for continuous, autonomous agent loops within CIEL. Defines acceptable patterns from simple sequential pipelines to DAG-driven orchestration.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(setup|run|create).*(autonomous|continuous).*(loop|pipeline|workflow)"
    confidence: 0.95
  - pattern: "orchestrate (multiple|parallel) (agents|tasks)"
    confidence: 0.85

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Autonomous-Loops (Loop Architecture)

This skill dictates the approved architectural patterns for running autonomous agent loops within the CIEL ecosystem. It prevents chaotic, unbounded agent executions by defining structured, verifiable pipeline patterns.

## Integration Context

Adapted from `~/.agents/skills/autonomous-loops/`. This skill provides the high-level orchestration architectures that utilize foundational CIEL skills like `subagent-driven-development`, `verification-loop`, and `safety-guard`.

## CIEL Approved Loop Patterns

### 1. The Sequential Pipeline (Standard Workflow)
The default pattern for automated tasks. Breaks work into isolated, sequential steps to prevent context degradation.
- **Implement**: Dispatch sub-agent for code generation.
- **De-Sloppify**: Dispatch a cleanup sub-agent (removes unnecessary tests, over-defensive code, console logs). *Crucial: Do not use negative instructions in the Implement step; use a separate De-Sloppify pass.*
- **Verify**: Run `verification-loop` (lint, test, build).
- **Commit**: Auto-commit with conventional messages if verified.

### 2. The Continuous Iteration Loop (Mid-Tier)
Used for multi-turn tasks driven by a persistent state file.
- Requires a `SHARED_TASK_NOTES.md` file to bridge context between iterations.
- Must include hard limits: `--max-runs`, `--max-cost`, or a completion signal.
- Involves an automated "Fix pass" if CI/verification fails.

### 3. RFC-Driven DAG Orchestration (High-Tier)
Reserved for massive, multi-file refactors or epic-level features.
- **Decomposition**: The Orchestrator breaks the RFC into a Dependency Graph (DAG) of discrete work units.
- **Tiered Pipelines**: Work units are assigned a complexity tier (trivial, small, medium, large), determining how many review/planning stages they undergo.
- **Isolated Contexts**: The Author sub-agent and Reviewer sub-agent MUST be different invocations with separate context windows to eliminate author-bias.

## Anti-Patterns (Enforced by Orchestrator)

- **Infinite Loops**: Running continuous agents without a hard exit condition (cost, time, or max-runs).
- **Context Bleed**: Re-using the same context window for implementation and review.
- **Blind Retries**: Simply re-running a failed iteration without appending the failure context to the prompt.
- **Negative Prompting for Quality**: Telling an implementer "don't write bad tests" instead of using a dedicated De-Sloppify cleanup pass.
