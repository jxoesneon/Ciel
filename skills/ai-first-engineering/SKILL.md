---
name: ai-first-engineering
version: 1.0.0
format: skill/1.0
description: Ciel's operating model for AI-native engineering and agent-friendly architecture.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "ai-first.*engineering"

    confidence: 0.9

  - pattern: "agent-friendly.*architecture"

    confidence: 0.9

  - pattern: "raise.*testing.*bar"

    confidence: 0.8

  - pattern: "process.*shift"

    confidence: 0.7

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: AI-First Engineering

This skill provides CIEL's core philosophy for engineering in an AI-native environment. It prioritizes planning quality, deterministic verification, and agent-friendly architectures.

## Integration Context

Adapted from `~/.agents/skills/ai-first-engineering/`. This is CIEL's "Cultural Directive" for how it designs its own processes and reviews.

## Core Mandates

### 1. Planning Over Typing

The **orchestration** skill must prioritize the "Research -> Strategy" phase. Implementation is a derivative of a well-defined plan.

### 2. Agent-Friendly Architecture

When proposing refactors or new features, prioritize:

- **Explicit Boundaries**: Decoupled services and clear API contracts.
- **Typed Interfaces**: Use TypeScript, Zod, or formal schemas for all data flow.
- **Deterministic Tests**: No flaky tests; every failure must be reproducible.

### 3. Elevated Testing Standard

For all CIEL-orchestrated work:

- Regression coverage is mandatory for touched domains.
- Explicit assertions for edge cases and failure modes.
- Integration checks for all interface boundaries.

## Orchestration Logic

### 1. Architectural Review

The **Council of Five** must evaluate all major architectural proposals against the "Agent-Friendly" criteria.

### 2. Review Focus

CIEL's internal code review (via **agentic-engineering**) must focus on system behavior, security, and integrity, ignoring style issues covered by automation.

### 3. Hiring & Evolution

Use this skill to refine CIEL's own "Self-Improvement" strategies, focusing on the quality of its own generated plans and evals.

---
*Original Documentation preserved at source.*
