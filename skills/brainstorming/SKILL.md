---
name: brainstorming
version: 1.0.0
format: skill/1.0
description: Pre-implementation exploration of intent, requirements, and architectural design.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(brainstorm|design|spec|requirements).*(for|new|feature)"

    confidence: 0.9

  - pattern: "how should we.*(build|implement|architecture)"

    confidence: 0.9

  - pattern: "let's explore.*(idea|concept|approach)"

    confidence: 0.9

  - pattern: "I want to.*(add|create).*(but|not sure)"

    confidence: 0.8

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Brainstorming (The Creative Layer)

This skill serves as CIEL's "Creative Layer," enforcing a strict design-before-action mandate. It ensures that intent is fully understood and architectural trade-offs are weighed before a single line of code is written.

## Integration Context

Adapted from `~/.agents/skills/brainstorming/`. This skill is the primary entry point for all non-trivial tasks. It feeds directly into the **Council of Five** for architectural deliberation and **writing-plans** for execution.

## The Design-Before-Action Mandate

<HARD-GATE>
CIEL MUST NOT invoke implementation skills (writing code, scaffolding, refactoring) until a design document has been approved by the user. "Simple" projects are NOT exempt; unexamined assumptions are the primary source of wasted context and tokens.
</HARD-GATE>

## Orchestration Logic

### 1. Context Mining

Before asking the first question, CIEL must use `grep_search` and `mempalace-rs` to understand the existing project DNA (conventions, patterns, prior ADRs).

### 2. The Socratic Loop

Ask clarifying questions **one at a time**. Use multiple-choice options to reduce user friction while maintaining precision.

### 3. Council Consultation

Once 2-3 approaches are formulated, CIEL must internally consult the **Council of Five** (Coherence, Capability, Safety, Efficiency, Evolution) to weight the trade-offs before presenting them to the user.

### 4. Spec Persistence

Approved designs must be written to `docs/ciel/specs/YYYY-MM-DD-<topic>.md` and indexed in **MemPalace** as a `ProposedDesign` node.

### 5. Transition

The terminal state of this skill is the invocation of **writing-plans**. No implementation occurs here.

---
*Original Process Flow and Visual Companion logic preserved at source.*
