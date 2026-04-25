---
name: orchestration
version: 1.0.0
format: skill/1.0
description: High-level task decomposition, planning, and multi-agent coordination.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(blueprint|plan|roadmap).*for"
    confidence: 1.0
  - pattern: "orchestrate.*"
    confidence: 0.95
  - pattern: "break down.*into steps"
    confidence: 0.9
  - pattern: "complex.*task"
    confidence: 0.7
---

# CIEL ADAPTATION: Orchestration

This skill provides CIEL's high-level planning and coordination engine. It transforms complex objectives into actionable, step-by-step construction plans.

## Integration Context

Adapted from `~/.agents/skills/blueprint/`. This version integrates CIEL's **Council of Five** for plan review, ensuring architectural alignment and risk mitigation.

## The Orchestration Pipeline

### 1. Research
Analyze the workspace, existing infrastructure, and constraints. CIEL uses its memory graph to ensure context continuity.

### 2. Design
Decompose the objective into discrete, "PR-sized" steps. Assign dependencies, parallel workflows, and model tiers.

### 3. Draft
Generate a self-contained Markdown plan. Each step must be executable "cold" by any sub-agent.

### 4. Council Review (Ciel Core)
The draft plan is submitted to the **Council of Five** for evaluation:
- **Coherence**: Does the plan follow our architectural standards?
- **Capability**: Are the steps technically sound and sufficient?
- **Safety**: Are there risks to production or system integrity?
- **Efficiency**: Is the parallelism optimized?
- **Evolution**: Is there a clear path for verification and rollback?

### 5. Execution & Mutation
Orchestrate the execution of steps. If obstacles arise, use the Mutation Protocol to adapt the plan mid-stream.

## Usage

Use this skill when:
- A task requires more than 3 tool calls or spans multiple sessions.
- Coordinating work across multiple parallel sub-agents.
- Building a roadmap for large-scale refactors or new features.

---
*Original Documentation preserved at source.*
