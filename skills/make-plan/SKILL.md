---
name: make-plan
version: 1.0.0
format: skill/1.0
description: Structured, LLM-friendly implementation planning based on approved design specs.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "make.*plan.*(for|implement|build)"
    confidence: 1.0
  - pattern: "how should we.*(execute|start).*(implementation|this)"
    confidence: 0.95
  - pattern: "create.*implementation.*steps"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Make-Plan (The Strategic Layer)

This skill serves as CIEL's "Strategic Layer," transforming high-level design specs into detailed, phased implementation plans. It ensures that every action is grounded in evidence and documentation.

## Integration Context

Adapted from `~/.agents/skills/make-plan/`. This skill is the bridge between **brainstorming** (Design) and **do** (Execution). It mandates "Documentation Discovery" as the first step of any implementation.

## Orchestration Logic

### 1. Spec Ingestion
The plan MUST start by reading the approved design spec from `docs/ciel/specs/`. Every phase of the plan must map back to a requirement or architectural choice in the spec.

### 2. Phase 0: Documentation Discovery (MANDATORY)
Before any implementation tasks are defined, CIEL must:
- Use `grep_search` and `mempalace-rs` to find relevant internal examples.
- Use `documentation-lookup` or `web_fetch` for external API signatures.
- Output a "Source of Truth" summary: exact file paths, API signatures, and copy-ready patterns.

### 3. Phased Execution Design (Blueprint)
Break the implementation into **Task Atomic Units** (TAUs):
- Each task should take 5-15 minutes of agent time.
- Tasks must be sequentially independent where possible.
- Each task MUST specify:
  - **Files**: Exact paths to create, modify, or read.
  - **Test Plan**: Specific test commands and expected output.
  - **Commit Mandate**: Explicit commit message for the task.

### 🚫 The "No Placeholders" Mandate
The following patterns are **Plan Failures** and are strictly prohibited:
- "Refine implementation logic to align with Ciel 1.0 standards.", "TBD", or "Implement later."
- "Add appropriate validation" (must specify the exact validation logic).
- "Similar to Task N" (must repeat the instructions; tasks may be executed out of order).
- "Write tests" (must include the actual test code snippet or specific test criteria).

### 4. Plan Persistence
The finalized plan must be written to `docs/ciel/plans/YYYY-MM-DD-<topic>-plan.md` and indexed in **MemPalace** as a `ProposedPlan` node.

### 5. Transition
The terminal state of this skill is the invocation of the **do** skill.

---
*Original Delegation Model and Reporting Contract preserved at source.*
