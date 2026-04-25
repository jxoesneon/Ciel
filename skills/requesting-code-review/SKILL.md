---
name: requesting-code-review
version: 1.0.0
format: skill/1.0
description: The formal protocol for requesting and processing peer code reviews from the Auditor sub-agent during CIEL execution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(request|get|perform).*(code review|audit|peer review)"
    confidence: 0.95
  - pattern: "review (my|the) code"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Requesting Code Review (The Auditor Protocol)

This skill formalizes the "Code Quality Review" stage of CIEL's `subagent-driven-development` cycle. It dictates how the Orchestrator requests a peer review from the Auditor sub-agent, ensuring that reviews are objective, context-isolated, and strictly aligned with CIEL standards.

## Integration Context

Adapted from `~/.agents/skills/requesting-code-review/`. This skill is the mechanism by which the Orchestrator enforces the `coding-standards` skill. It guarantees that the agent writing the code is never the same agent reviewing the code (eliminating author-bias).

## The Code Review Protocol

### 1. Context Isolation (Mandatory)
When dispatching the Auditor sub-agent, the Orchestrator MUST NOT pass its own entire session history. The reviewer must judge the work product, not the thought process.

**Required Context for the Auditor:**
- `BASE_SHA` and `HEAD_SHA` (or the specific git diff).
- The original Task Specification / Requirements.
- A brief summary of what was implemented.

### 2. Evaluation Rubric
The Auditor sub-agent must evaluate the diff against the following rubric:
1. **Spec Compliance**: Does the code fulfill the exact requirements without adding unauthorized features (YAGNI)?
2. **Structural Integrity**: Does it adhere to CIEL's `coding-standards` (e.g., Immutability, no magic numbers, no deep nesting)?
3. **Test Coverage**: Are there meaningful tests included for the new business logic?

### 3. Feedback Categorization
The Auditor must categorize issues into three levels:
- **CRITICAL**: Security flaws, infinite loops, broken builds. *Must be fixed immediately.*
- **IMPORTANT**: Violations of `coding-standards`, missing tests, logic errors. *Must be fixed before the task is marked COMPLETED.*
- **MINOR**: Nitpicks, styling, optional refactors. *Can be noted for later or ignored.*

### 4. Orchestrator Action
Upon receiving the Auditor's report:
- **If CRITICAL or IMPORTANT issues exist**: The Orchestrator MUST dispatch a fix sub-agent (or the original implementer) with the review feedback to resolve the issues. Proceeding without fixing these is a violation of the protocol.
- **Evolutionary Feedback (Meta-Learning)**: The Orchestrator MUST log recurring structural or logic failures to the MemPalace Knowledge Graph (`mempalace_kg_add`). This allows future `make-plan` and `coding-standards` iterations to adapt to systemic implementer weaknesses.
- **If only MINOR issues or no issues exist**: The task is marked `COMPLETED` and the Orchestrator proceeds.

## Anti-Patterns (Enforced by Orchestrator)
- **Self-Review**: The Orchestrator or Implementer sub-agent attempting to declare its own code "ready" without an isolated peer review.
- **Ignoring Feedback**: Proceeding to merge or to the next task while IMPORTANT or CRITICAL issues remain unaddressed.
- **Context Pollution**: Sending the Auditor the entire debugging history instead of just the final diff and spec.
