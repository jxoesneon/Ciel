---
name: convoy-orchestration
version: 1.0.0
description: Multi-agent work coordination for complex projects. Distribute tasks across parallel agents, track progress, and manage dependencies at scale.
format: skill/1.0
author: Ciel Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "convoy"
  - "multi-agent"
  - "parallel work"
  - "distribute tasks"
  - "agent coordination"
  - "sling work"
  - "work tracking"
  - "scale up"
entrypoint: SKILL.md
---

# Convoy Orchestration

Coordinate multiple AI agents working in parallel on complex projects. Distribute tasks, track progress, and maintain oversight as your team of agents scales.

## When to Activate

Invoke convoy-orchestration when:

- A project is too large for one agent session
- Tasks can be parallelized across multiple work streams
- You need coordinated effort on multiple fronts simultaneously
- Work needs tracking across distributed agents
- You're managing dependencies between parallel tasks

## Core Concepts

### Work Units

The atomic unit of work in convoy orchestration:

- **Task** — A discrete piece of work with clear completion criteria
- **Dependency** — Tasks that must complete before others start  
- **Assignment** — Routing tasks to appropriate agents
- **Checkpoint** — Progress milestones for coordination

### Orchestration Patterns

| Pattern | Use Case | Structure |
|---------|----------|-----------|
| **Parallel Map** | Same task on different inputs | Fan-out → Process → Fan-in |
| **Pipeline** | Sequential stages | Stage 1 → Stage 2 → Stage 3 |
| **Reduce** | Aggregate parallel results | Distribute → Process → Collect → Combine |
| **Tree** | Hierarchical decomposition | Parent → Children → Grandchildren |
| **Mesh** | Interdependent tasks | Dependency graph with partial ordering |

## Workflow

```
Project Scope
    │
    ▼
┌─────────────────────┐
│ Decompose into      │  Break into parallelizable work units
│ work units          │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Define dependencies │  Map which tasks need what
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Assign to agents    │  Distribute across available capacity
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Execute & monitor   │  Track progress, handle blockers
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Synthesize results  │  Combine outputs into coherent deliverable
└─────────────────────┘
```

## Commands

### Setup

Initialize orchestration for a project:

```bash
# Define project scope
# Create work unit registry
# Set up tracking infrastructure
```

### Task Management

```bash
# Create work unit
# Assign to agent
# Set dependencies
# Update status
# Mark complete
```

### Monitoring

```bash
# View all active work
# Check blocked tasks
# View completion status
# Review agent assignments
# Identify bottlenecks
```

## Best Practices

### Task Design

| Good | Bad |
|------|-----|
| Clear completion criteria | Open-ended tasks |
| Independent where possible | Tight coupling |
| Estimated duration | Unknown scope |
| Single responsibility | Multi-concern tasks |

### Dependency Management

- Minimize dependencies — more parallelization
- Identify critical path — these block everything
- Handle failures gracefully — retry or reassign
- Checkpoint frequently — don't lose progress

### Scaling Considerations

| Scale | Approach |
|-------|----------|
| 2-5 agents | Direct coordination |
| 5-20 agents | Structured workflows |
| 20+ agents | Hierarchical decomposition |
| 100+ agents | Automated load balancing |

## Integration with Ciel

Convoy orchestration works with other Ciel skills:

- **blueprint** — Project planning before orchestration
- **backend-patterns / frontend-patterns** — Technical implementation
- **testing skills** — Validation of parallel work
- **security-review** — Audit distributed changes
- **council** — Deliberation on complex tradeoffs

## Example Scenarios

### Scenario 1: Multi-Service Refactor

```
Refactor a monolith into 5 microservices
    │
    ├─→ Service A (auth) ─ Agent 1
    ├─→ Service B (billing) ─ Agent 2
    ├─→ Service C (inventory) ─ Agent 3
    ├─→ Service D (notifications) ─ Agent 4
    └─→ Service E (analytics) ─ Agent 5
         │
         └─→ Shared library updates (blocks all)
```

**Orchestration**: Shared lib first, then 5 parallel service extractions, integration testing last.

### Scenario 2: Documentation Sprint

```
Document entire API surface
    │
    ├─→ Core endpoints (Agent 1)
    ├─→ Authentication (Agent 2)
    ├─→ Error handling (Agent 3)
    ├─→ SDK examples (Agent 4)
    └─→ Webhook integration (Agent 5)
         │
         └─→ Style guide (blocks all)
```

**Orchestration**: Style guide → parallel documentation → editorial review → publish.

### Scenario 3: Research Deep Dive

```
Research competitive landscape
    │
    ├─→ Product A analysis (Agent 1)
    ├─→ Product B analysis (Agent 2)
    ├─→ Product C analysis (Agent 3)
    └─→ Market trends (Agent 4)
         │
         └─→ Synthesis report (depends on all)
```

**Orchestration**: Parallel research → synthesis → executive summary.

## Error Handling

### Common Issues

| Issue | Response |
|-------|----------|
| Agent failure | Reassign to another agent |
| Dependency blocked | Escalate blocker or find workaround |
| Scope creep | Re-decompose, adjust assignments |
| Result conflict | Council deliberation on resolution |
| Timeout | Retry with modified approach |

### Recovery

- **Checkpoint** work frequently
- **Log** all agent activities
- **Version** intermediate results
- **Rollback** to last known good state

## Risk Classification

- **LOW** — Parallel independent tasks with clear scope
- **MID** — Interdependent tasks requiring coordination
- **HIGH** — Complex dependency graphs, shared resources
- **CRITICAL** — Production systems, irreversible changes

## Provenance

This skill was inspired by concepts from the ECC ecosystem's gastown skill, rewritten fresh for Ciel. See `PROVENANCE.md`.

## Version History

- 1.0.0 — Initial adaptation for Ciel ecosystem
