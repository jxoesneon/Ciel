---
name: council
version: 1.0.0
format: skill/1.0
description: Ciel's primary decision-making and evaluation body. Convene the Council of Five for evaluation, triage, and high-stakes decisions.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "convene.*council"

    confidence: 0.9

  - pattern: "council.*deliberation"

    confidence: 0.9

  - pattern: "evaluate.*(decision|choice|path)"

    confidence: 0.9

  - pattern: "second opinion"

    confidence: 0.8

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Council of Five

This skill implements CIEL's core reasoning and evaluation mechanism. It convenes five specialized sub-agents to provide multi-perspective analysis on complex decisions.

## Integration Context

Adapted from `~/.agents/skills/council/`. This version is specifically optimized for CIEL's **Council of Five** architecture used in skill ingestion, risk assessment, and strategic orchestration.

## The Council of Five

| Member | Lens | Focus |
| --- | --- | --- |
| **Coherence** | Alignment | Does this align with CIEL's orchestration philosophy? |
| **Capability** | Utility | Does this fill a genuine gap? Is it documented well? |
| **Safety** | Integrity | Any risky operations? Are sandboxes or guards required? |
| **Efficiency** | Composition | Is it composable? Does it duplicate existing skills? |
| **Evolution** | Growth | Can it grow? Is it self-improving ready? |

## Workflow

### 1. Define the Case

Identify the decision, constraints, and success criteria.

### 2. Form initial position (Chairman)

CIEL's current voice acts as the Chairman, forming an initial position to avoid simple mirroring.

### 3. Parallel Deliberation

Invoke five sub-agents in parallel with the Case and their specific Role.

### 4. Synthesis & Verdict

Chairman synthesizes the votes, highlighting the strongest dissent and reaching a final verdict.

## Usage

Use this skill for:

- **Skill Ingestion**: Evaluation of newly discovered capabilities.
- **Strategic Triage**: Deciding between multiple architectural or operational paths.
- **High-Risk Operations**: Extra verification for destructive or system-level changes.

---
*Original Documentation preserved at source.*
