---
name: agentic-engineering
version: 1.0.0
format: skill/1.0
description: Ciel's operational framework for agent-led engineering and cost-aware routing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(agentic|autonomous|agent-led).*engineering"
    confidence: 1.0
  - pattern: "route.*model.*tier"
    confidence: 0.95
  - pattern: "eval-first.*"
    confidence: 1.0
  - pattern: "task.*decomposition"
    confidence: 0.9
---

# CIEL ADAPTATION: Agentic Engineering

This skill provides CIEL's operational framework for high-efficiency, agent-led engineering. It coordinates task decomposition, model tier routing, and eval-driven verification.

## Integration Context

Adapted from `~/.agents/skills/agentic-engineering/`. This skill transforms CIEL into an active "Engineering Factory" by standardizing how sub-tasks are delegated and verified.

## Operating Principles

### 1. Eval-First Loop
Define capability and regression evals **before** implementation. CIEL must capture failure signatures as the baseline for all engineering tasks.

### 2. Task Decomposition (The 15-Minute Rule)
Decompose work into independent, verifiable units. Each unit must have a single dominant risk and a clear "done" condition.

### 3. Model Tier Routing
- **Efficiency Tier (Flash)**: Classification, boilerplate, narrow edits.
- **Implementation Tier (Pro/Default)**: Standard feature implementation and refactoring.
- **Reasoning Tier (Experimental/Large)**: Architecture, root-cause analysis, complex invariants.

## Orchestration Logic

### 1. Planning
The **orchestration** skill must apply the 15-minute unit rule when breaking down complex roadmaps.

### 2. Review Gate
AI-generated code for high-risk components must be reviewed by the **Council of Five**, focusing on invariants, security, and auth assumptions.

### 3. Cost & Performance Tracking
Use **MemPalace** to record token estimates, retries, and success rates per task to refine future routing strategies.

---
*Original Documentation preserved at source.*
