---
name: convoy-orchestration
version: 1.0.0
format: skill/1.0
description: CIEL's framework for dependency-graph-based task distribution across parallel agent crews. Manages inter-task dependencies, checkpoints, and synthesis.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
side_effects: ["shell", "state_mutation"]
triggers:
  - pattern: "(convoy|distribute tasks|parallel work streams|dependency graph).*(agent|task|work)"
    confidence: 0.9
  - pattern: "agent coordination at scale"
    confidence: 0.8
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Convoy Orchestration (Dependency-Graph Distribution)

This skill provides dependency-graph-based task distribution for agent crews. While `ciel-swarm-orchestration` focuses on automatic task decomposition and parallel fan-out, convoy-orchestration adds explicit dependency modeling — you define which tasks block which, and the system routes work accordingly. Use this when tasks have complex interdependencies that automatic decomposition cannot capture.

## When to Activate

- A project is too large for one agent session and tasks have explicit dependencies.
- You need coordinated effort across multiple parallel work streams with ordering constraints.
- Work needs tracking across distributed agents with checkpoint-based coordination.
- You are managing a critical path where some tasks block everything downstream.

## Orchestration Patterns

- **Parallel Map**: Same task on different inputs — fan-out → process → fan-in.
- **Pipeline**: Sequential stages — Stage 1 → Stage 2 → Stage 3.
- **Reduce**: Aggregate parallel results — distribute → process → collect → combine.
- **Tree**: Hierarchical decomposition — parent → children → grandchildren.
- **Mesh**: Interdependent tasks — dependency graph with partial ordering.

## Workflow

1. **Decompose**: Break project scope into parallelizable work units with clear completion criteria.
2. **Define Dependencies**: Map which tasks need what — identify the critical path.
3. **Assign**: Distribute tasks across available agent capacity, respecting dependency order.
4. **Execute & Monitor**: Track progress, handle blockers, checkpoint frequently.
5. **Synthesize**: Combine outputs into a coherent deliverable; run Council coherence check.

## Dependency Management

- Minimize dependencies to maximize parallelization.
- Identify the critical path early — these tasks block everything.
- Handle failures gracefully: retry, reassign, or find workaround.
- Checkpoint frequently — don't lose progress on agent failure.
- Version intermediate results for rollback to last known good state.

## Task Design

- Clear completion criteria — never open-ended tasks.
- Independent where possible — avoid tight coupling.
- Single responsibility — never multi-concern tasks.
- Estimated duration — never unknown scope.

## Scaling

- **2-5 agents**: Direct coordination.
- **5-20 agents**: Structured workflows.
- **20+ agents**: Hierarchical decomposition.

## Risk Classification

- **LOW**: Parallel independent tasks with clear scope.
- **MID**: Interdependent tasks requiring coordination.
- **HIGH**: Complex dependency graphs, shared resources.
- **CRITICAL**: Production systems, irreversible changes.

## Anti-Patterns

- **Tight Coupling**: Creating tasks that cannot proceed without constant synchronization.
- **No Checkpoints**: Running long parallel streams without saving intermediate state.
- **Blind Synthesis**: Merging parallel outputs without a final coherence check via Council.
- **Ignored Critical Path**: Failing to prioritize the single blocking chain that determines total duration.
